# Como a Memory funciona com os Agents

A "Memory" é um sistema de **RAG (Retrieval-Augmented Generation) semântico**: o
usuário guarda trechos de texto que viram vetores (embeddings); na hora em que um
agente executa dentro de um flow, os trechos mais relevantes são buscados por
similaridade e injetados no prompt antes da chamada ao LLM.

## As 3 peças de configuração

| Peça | Onde mora | O que faz |
|------|-----------|-----------|
| **Modelo de embedding global** (`user_embedding_settings`) | Settings → "Embedding" | 1 modelo por usuário, usado para vetorizar tudo |
| **Memory records** (`memory_records`) | Settings → "Memory" | Os textos guardados + seu vetor + `agent_id` (null = global, ou preso a um agente) |
| **Perfil semântico por agente** (`agent_semantic_profiles`) | endpoint `/agents/{id}/semantic-profile` | Override opcional: modelo de embedding e `memory_scope` (`all`/`own`) específicos do agente |

E cada **Agent** carrega dois campos numéricos que governam o contexto
(`backend/src/agents/model.rs`):

- **`top_k`** (0–50, default 5) — quantos memory records buscar.
  **`top_k = 0` desliga a retrieval.**
- **`recent_n`** (0–100, default 10) — quantos turnos de conversa recente injetar
  (complementa a memória).

## Fluxo de gravação (save → embed → store)

`store::create()` em `backend/src/memory/store.rs`:

1. Valida o texto (1–8000 chars).
2. Valida que o agente (se houver `agent_id`) é do usuário.
3. Busca o **modelo de embedding global** do usuário — se não houver, erro `MEM002`.
4. Chama o **gateway LiteLLM** para gerar o vetor (`gateway.embed(...)`).
5. Só persiste em `memory_records` **se o embed deu certo** — falha de embed nunca
   grava lixo.

A coluna `embedding` é `VECTOR` (pgvector, dimensão livre), e cada linha guarda o
nome do `embedding_model` que a gerou — crucial para o passo seguinte.

## Fluxo de recuperação (retrieve → inject) — em tempo de execução do flow

Dentro de `backend/src/runs/engine.rs`, para **cada nó** do DAG:

1. Monta o `forwarded_input` (prompt do run, no nó raiz; ou a saída concatenada dos
   nós upstream).
2. Chama `retrieval::retrieve(db, gateway, user_id, agent.id, agent.top_k, forwarded)`.
3. A retrieval (`backend/src/memory/retrieval.rs`):
   - Resolve qual modelo + escopo usar — o **perfil semântico do agente** se existir,
     senão o global com escopo `"all"`.
   - Embeda o próprio prompt com **o mesmo modelo**.
   - Roda a busca por similaridade de cosseno:
     ```sql
     SELECT id, text, 1 - (embedding <=> $query) AS score
     FROM memory_records
     WHERE user_id = $u AND embedding_model = $m  -- nunca compara entre modelos
       AND (<filtro de escopo>)
     ORDER BY embedding <=> $query
     LIMIT top_k
     ```
   - **Escopo `all`**: pega records globais (agent_id NULL) + os do próprio agente.
   - **Escopo `own`**: só os records com `agent_id` igual ao do agente.
4. Os textos recuperados são unidos por linhas em branco e **prepended à mensagem
   `user`**, em `assemble_messages()`:
   ```
   [system: preamble + system_prompt]
   [...recent_n turnos de histórico]
   [user: <contexto recuperado>\n\n<forwarded input>]
   ```
5. Essa pilha de mensagens vai pro gateway → LLM.

## Dois detalhes de design importantes

- **Retrieval é infalível**: qualquer erro (sem modelo, embed falhou, DB caiu) vira
  um resultado `Skipped` com motivo, e o run continua. A memória nunca derruba a
  execução. O evento SSE `node.completed` carrega um `RetrievalSummary`
  (`retrieved_count`, `excluded_mismatched`, `status`).
- **Filtro por modelo**: records gerados com um modelo de embedding diferente do
  atual são **excluídos** da busca (e contados em `excluded_mismatched`) — vetores de
  modelos distintos não são comparáveis. Por isso o frontend avisa quando há "modelos
  em uso" misturados.

## Importante: a memória é só-leitura durante o run

Os outputs dos agentes **não são gravados de volta** na memória automaticamente. A
memória é uma base de conhecimento curada manualmente pelo usuário (via Settings →
Memory / `MemoryRecordModal`), e durante a execução ela é apenas consultada.

## Constantes e limites

| Constante | Valor | Local |
|-----------|-------|-------|
| `MEMORY_TEXT_MAX` | 8.000 chars | `memory/types.rs` |
| `top_k` | 0–50 (default 5) | `agents/model.rs` |
| `recent_n` | 0–100 (default 10) | `agents/model.rs` |
| `memory_scope` | `all` \| `own` | `memory/types.rs` |

## Códigos de erro relevantes

| Código | Cenário |
|--------|---------|
| `MEM001` | Texto do record excede 8000 chars |
| `MEM002` | Nenhum modelo de embedding global configurado |
| `MEM_VALIDATION` | Agente não pertence ao usuário; escopo inválido |
| `GW001` | Gateway indisponível |
| `GW002` | Modelo não encontrado ou não é de embedding |
| `RUN003` | Agente ausente durante a execução |

---

**Resumo do ciclo de vida:** o usuário escolhe um **modelo de embedding**, cadastra
**memory records** (globais ou presos a um agente), e ajusta o **`top_k`** de cada
agente. Na execução do flow, cada nó embeda seu prompt, busca por cosseno os `top_k`
records mais parecidos respeitando o escopo, e injeta esses textos no prompt do LLM —
RAG por agente, isolado por usuário e por modelo.

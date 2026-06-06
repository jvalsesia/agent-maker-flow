# Project Intent: Multi-Agent Flow Orchestrator & Chat System

This document outlines the architectural blueprint and operational requirements for a high-performance, node-based multi-agent layout framework. The application allows developers and engineers to visually string together autonomous language model chains, handle stateful context passing, and interact with the workflow via a real-time conversational interface.

---

## Technical Stack Overview

| Layer | Component | Implementation Details |
| --- | --- | --- |
| **Frontend Core** | ReactJS | Component architecture, state management, and async streaming hooks. |
| **Graph Interface** | React Flow | Node canvas, drag-and-drop edges, validation, and layout routing. |
| **Authentication** | Clerk | JWT session management, user validation, and edge middleware protection. |
| **Backend API** | Rust Axum v0.8.9 | Blazing-fast async HTTP routing, type-safe extractors, and Tokio execution runtime. |
| **LLM Gateway** | LiteLLM Proxy | Unified API interface wrapper running on local Docker and deployed via Railway.com. |
| **State & Cache** | Redis | Active token usage counters, prompt caching orchestration, and quick memory storage. |
| **Vector Storage** | PostgreSQL (`pgvector`) | Storage for agent embeddings, system memory blocks, and semantic lookup indexes. |

---

## 1. Agents Dashboard (Registry & Lifecycle)

The **Agents Dashboard** acts as the primary registry interface. It handles creating, editing, updating, and viewing granular configuration profiles for each agent.

### Agent Configuration Parameters

When creating or modifying an agent, the UI exposes a structured form containing:

* **Name:** `Text Field` — Unique semantic identifier for the agent node (e.g., *CodeAnalyzer*, *Copywriter*).
* **Preamble:** `Text Field` — Pre-context injected before system execution parameters to prime behavioral styles.
* **System Prompt:** `Large Text Area` — Detailed instructional criteria governing the behavior, constraints, and outputs of the agent.
* **LLM Provider:** `Dropdown Selector` — Populated dynamically via LiteLLM endpoints (e.g., `OpenAI`, `Anthropic`, `Groq`, `Ollama`).
* **LLM Model:** `Dropdown Selector` — Contextually filtered models mapped to the selected provider (e.g., `gpt-4o`, `claude-3-5-sonnet`).
* **Recent-N Override:** `Integer Text Field` — Limits the maximum historical conversational turns passed down in memory.
* **Top-K Override:** `Integer Text Field` — Custom boundaries restricting token selection probability distribution for retrieval context.

---

## 2. Agent Flow & Execution Dashboard

A split-pane canvas environment provides a structural playground alongside real-time output verification.

### Left Split: Visual Canvas Management

Using **React Flow**, this panel renders a directed graph environment allowing complex routing schemas.

* **Node Lifecycle:** Users drag agents instantiated in the Agents Dashboard onto the canvas. Nodes can be deleted, duplicated, or detached smoothly.
* **Reactive Edges:** Input and output ports enable connecting the output of an upstream agent to the input socket of a downstream agent.
* **Root Assignment:** A graphical toggle allows marking a specific agent as the **Root Agent**. The initialization payload from the user chat maps automatically here.
* **Pipeline Controls:** A floating global execution toolbar contains a prominent **"Run Flow"** button to dispatch execution payloads down the pipeline.

### Right Split: Conversational Monitor & Execution View

A classic, split vertical panel handling user prompts and monitoring back-and-forth system operations.

* **User Input Panel:** Main prompt submission bar to supply data structures, prompt requests, or engineering scripts.
* **Turn-Based Conversational Stream:** Displays individual chat turns.
* **Real-time Streaming Updates:** As the pipeline executes, each active agent block lights up, printing intermediate reasoning chains, token metrics, or final formatted message responses directly into the conversational feed.

---

## 3. Data Management, Memory & Embeddings

The backend data topology ensures that processing elements maintain context awareness across separate processing iterations.

### Core Architecture & Integration Specifications

```
               +-------------------------------------------------+
               |                React Frontend                   |
               |         (React Flow Canvas + Chat UI)           |
               +-----------------------+-------------------------+
                                       |
                                  REST / SSE
                                       |
                                       v
               +-------------------------------------------------+
               |              Rust Axum Backend                  |
               |                   (v0.8.9)                      |
               +-------+-----------------------+-----------------+
                       |                       |
                  SQL / Vector            OpenAI API
                       |                       |
                       v                       v
        +--------------+---------------+ +-----+-----------------+
        |          PostgreSQL          | |   LiteLLM Proxy       |
        |         (pgvector)           | | (Docker / Railway.com)|
        +------------------------------+ +-----+-----------+-----+
                                               |           |
                                             Redis       LLMs
                                            (Cache)   (Providers)

```

* **Vector Search (`pgvector`):** The Rust Axum backend uses `pgvector` to run cosine similarity match strategies. This enables retrieval-augmented generation (RAG) by fetching contextually accurate records matching the user's initial prompt state.
* **Configuration Plane:** Users define embedding generation models (e.g., `text-embedding-3-small`) globally or attach separate semantic profiles per node.
* **LiteLLM & Redis Cache Layer:** The LiteLLM layer relies on Redis instances to preserve context history, trace usage logs, handle fallback exceptions, and avoid repeating identical generation requests.

---

## 4. Execution Workflow

1. **Initialization:** The user formats a prompt on the right-side chat screen and clicks **Run Flow**.
2. **Ingestion Layer:** The entry context maps directly to the designated canvas **Root Agent**.
3. **Graph Evaluation:** The Axum backend translates the React Flow graph state into a directed acyclic graph (DAG) topology map.
4. **Agent Processing & Forwarding:** The Root Agent completes its execution turn via LiteLLM. Its generated response text automatically feeds forward into any connected downstream target nodes as entry context parameters.
5. **Final Aggregation:** The final edge node passes its finished payload back to the UI chat stream handler, resolving the cycle.
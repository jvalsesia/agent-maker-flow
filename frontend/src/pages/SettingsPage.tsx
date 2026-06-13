import { useEffect, useState } from "react";

import { EmbeddingModelSelect } from "../components/EmbeddingModelSelect";
import { MemoryRecordForm } from "../components/MemoryRecordForm";
import { MemoryRecordList } from "../components/MemoryRecordList";
import {
  useEmbeddingSetting,
  useSetEmbeddingSetting,
} from "../lib/embeddingSettings";
import { useAgents } from "../lib/agents";
import { useDeleteMemoryRecord, useMemoryRecords, type MemoryRecord } from "../lib/memory";
import { useModels, useProviders } from "../lib/models";
import { Alert, Card, Select, SkeletonRows } from "../components/ui";
import styles from "./SettingsPage.module.css";

/**
 * Settings panel (F05): pick the global embedding model (from the F03 catalog,
 * embedding-mode only) and manage the memory-record store (add/edit/delete with
 * embed-on-save). The embedding-model list is provider-scoped, mirroring the
 * F04 agent form's provider→model cascade.
 */
export function SettingsPage() {
  const setting = useEmbeddingSetting();
  const providers = useProviders();
  const [provider, setProvider] = useState("");
  const models = useModels(provider.length > 0 ? provider : null);
  const setEmbedding = useSetEmbeddingSetting();

  const memory = useMemoryRecords();
  const agents = useAgents();
  const deleteRecord = useDeleteMemoryRecord();
  const [editing, setEditing] = useState<MemoryRecord | null>(null);

  // Agent id → name, so the records list can label agent-scoped records (F06).
  const agentNames = Object.fromEntries(
    (agents.data ?? []).map((agent) => [agent.id, agent.name]),
  );

  // Default the provider picker to the first discovered provider.
  useEffect(() => {
    if (provider.length === 0 && providers.data && providers.data.length > 0) {
      setProvider(providers.data[0].id);
    }
  }, [providers.data, provider]);

  return (
    <section className={styles.page} aria-label="Settings">
      <h2 className={styles.title}>Settings</h2>

      <Card title="Embedding" as="h3">
        <div className={styles.section} aria-label="Embedding model">
          <p className={styles.subhead}>
            The global embedding model used to store and search memory records.
          </p>
          <Select
            label="Provider"
            id="embedding-provider"
            value={provider}
            onChange={(e) => setProvider(e.target.value)}
          >
            <option value="">Select a provider…</option>
            {(providers.data ?? []).map((p) => (
              <option key={p.id} value={p.id}>
                {p.id}
              </option>
            ))}
          </Select>

          <EmbeddingModelSelect
            models={models.data ?? []}
            value={setting.data?.embedding_model ?? null}
            onChange={(model) => setEmbedding.mutate(model)}
            modelsInUse={memory.data?.models_in_use ?? []}
            disabled={setEmbedding.isPending}
          />
          {setEmbedding.isError && (
            <Alert variant="danger" role="alert">
              Could not update the embedding model.
            </Alert>
          )}
        </div>
      </Card>

      <Card title="Memory" as="h3">
        <div className={styles.section} aria-label="Memory records">
          <div className={styles.formBlock}>
            {editing ? (
              <MemoryRecordForm
                record={editing}
                onSaved={() => setEditing(null)}
                onCancel={() => setEditing(null)}
              />
            ) : (
              <MemoryRecordForm />
            )}
          </div>

          <div className={styles.records}>
            <span className={styles.recordsLabel}>Records</span>
            {memory.isLoading && <SkeletonRows count={3} />}
            {memory.isError && (
              <Alert variant="danger" role="alert">
                Could not load memory records. Please retry.
              </Alert>
            )}
            {memory.data && (
              <MemoryRecordList
                records={memory.data.records}
                agentNames={agentNames}
                onEdit={(record) => setEditing(record)}
                onDelete={(record) => deleteRecord.mutate(record.id)}
              />
            )}
          </div>
        </div>
      </Card>
    </section>
  );
}

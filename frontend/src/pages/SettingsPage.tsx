import { useEffect, useState } from "react";

import { EmbeddingModelSelect } from "../components/EmbeddingModelSelect";
import { MemoryRecordForm } from "../components/MemoryRecordForm";
import { MemoryRecordList } from "../components/MemoryRecordList";
import {
  useEmbeddingSetting,
  useSetEmbeddingSetting,
} from "../lib/embeddingSettings";
import { useDeleteMemoryRecord, useMemoryRecords, type MemoryRecord } from "../lib/memory";
import { useModels, useProviders } from "../lib/models";

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
  const deleteRecord = useDeleteMemoryRecord();
  const [editing, setEditing] = useState<MemoryRecord | null>(null);

  // Default the provider picker to the first discovered provider.
  useEffect(() => {
    if (provider.length === 0 && providers.data && providers.data.length > 0) {
      setProvider(providers.data[0].id);
    }
  }, [providers.data, provider]);

  return (
    <section aria-label="Settings">
      <h2>Settings</h2>

      <section aria-label="Embedding model">
        <h3>Embedding</h3>
        <label htmlFor="embedding-provider">Provider</label>
        <br />
        <select
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
        </select>

        <EmbeddingModelSelect
          models={models.data ?? []}
          value={setting.data?.embedding_model ?? null}
          onChange={(model) => setEmbedding.mutate(model)}
          modelsInUse={memory.data?.models_in_use ?? []}
          disabled={setEmbedding.isPending}
        />
        {setEmbedding.isError && <p role="alert">Could not update the embedding model.</p>}
      </section>

      <section aria-label="Memory records">
        <h3>Memory</h3>

        {editing ? (
          <MemoryRecordForm
            record={editing}
            onSaved={() => setEditing(null)}
            onCancel={() => setEditing(null)}
          />
        ) : (
          <MemoryRecordForm />
        )}

        {memory.isLoading && <p>Loading records…</p>}
        {memory.isError && <p role="alert">Could not load memory records. Please retry.</p>}
        {memory.data && (
          <MemoryRecordList
            records={memory.data.records}
            onEdit={(record) => setEditing(record)}
            onDelete={(record) => deleteRecord.mutate(record.id)}
          />
        )}
      </section>
    </section>
  );
}

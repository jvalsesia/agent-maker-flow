import type { Model } from "../lib/models";
import { Alert, Select } from "./ui";

interface EmbeddingModelSelectProps {
  /** Catalog models for the chosen provider; only embedding-mode ones show. */
  models: Model[];
  /** The currently selected global embedding model (`null` if unset). */
  value: string | null;
  onChange: (model: string) => void;
  /** Distinct models present across the user's existing records. */
  modelsInUse?: string[];
  disabled?: boolean;
}

/**
 * Global embedding-model dropdown. Lists only `mode === "embedding"` models
 * from the F03 catalog and surfaces an advisory warning when existing records
 * were embedded with a different model (changing the model does not re-embed
 * them automatically — that's a manual per-record edit).
 */
export function EmbeddingModelSelect({
  models,
  value,
  onChange,
  modelsInUse = [],
  disabled = false,
}: EmbeddingModelSelectProps) {
  const embeddingModels = models.filter((m) => m.mode === "embedding");

  // If the saved model isn't in the current provider's list, still show it.
  const hasValue = value != null && value.length > 0;
  const valueMissing = hasValue && !embeddingModels.some((m) => m.id === value);

  const staleModels = modelsInUse.filter((m) => m !== value);
  const showWarning = hasValue && staleModels.length > 0;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}>
      <Select
        label="Global embedding model"
        value={value ?? ""}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
      >
        <option value="">Select an embedding model…</option>
        {valueMissing && <option value={value as string}>{value}</option>}
        {embeddingModels.map((m) => (
          <option key={m.id} value={m.id}>
            {m.label}
          </option>
        ))}
      </Select>

      {showWarning && (
        <Alert variant="warning" role="alert" title="Existing records use a different model">
          Some records were embedded with {staleModels.join(", ")}. They are not re-embedded
          automatically — edit a record to re-embed it with the current model.
        </Alert>
      )}
    </div>
  );
}

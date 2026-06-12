import { useEffect, useId, useState } from "react";

import { ApiClientError } from "../../lib/apiClient";
import type { Agent, AgentInput } from "../../lib/agents";
import {
  PREAMBLE_MAX,
  SYSTEM_PROMPT_MAX,
  validateAgentForm,
  type AgentFieldErrors,
  type AgentFormValues,
} from "../../lib/agentValidation";
import { useModels, useProviders } from "../../lib/models";
import { Alert, Button, Input, Modal, Select, Textarea } from "../ui";
import styles from "./AgentForm.module.css";

export type AgentFormMode = "create" | "edit" | "duplicate";

interface AgentFormProps {
  mode: AgentFormMode;
  /** Source agent for edit/duplicate; ignored in create mode. */
  initial?: Agent;
  /** Persist the agent. Should reject with `ApiClientError` on failure so the
   *  form can map server error codes back to the relevant fields. */
  onSubmit: (input: AgentInput) => Promise<void>;
  onCancel: () => void;
}

function initialValues(mode: AgentFormMode, initial?: Agent): AgentFormValues {
  if (mode === "create" || !initial) {
    return {
      name: "",
      preamble: "",
      system_prompt: "",
      provider: "",
      model: "",
      recent_n: 10,
      top_k: 5,
    };
  }
  return {
    name: mode === "duplicate" ? `${initial.name} (copy)` : initial.name,
    preamble: initial.preamble ?? "",
    system_prompt: initial.system_prompt,
    provider: initial.provider,
    model: initial.model,
    recent_n: initial.recent_n,
    top_k: initial.top_k,
  };
}

/** Map a server error envelope code to the field it concerns. */
function mapApiError(error: ApiClientError, name: string): AgentFieldErrors {
  switch (error.code) {
    case "AGENT_NAME_TAKEN":
      return { name: `An agent named '${name}' already exists.` };
    case "GW002":
      return { model: error.message };
    default:
      return {};
  }
}

/**
 * Controlled create/edit/duplicate agent form (now a modal). The provider and
 * model dropdowns come from the F03 catalog; the model select stays disabled
 * until a provider is chosen and is cleared if the chosen provider no longer
 * offers the selected model. Fields are validated inline against the shared
 * rules; server error codes are mapped back onto the relevant fields on submit.
 */
export function AgentForm({ mode, initial, onSubmit, onCancel }: AgentFormProps) {
  const [values, setValues] = useState<AgentFormValues>(() => initialValues(mode, initial));
  const [errors, setErrors] = useState<AgentFieldErrors>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const formId = useId();

  const providersQuery = useProviders();
  const modelsQuery = useModels(values.provider.length > 0 ? values.provider : null);
  const providers = providersQuery.data ?? [];
  const models = (modelsQuery.data ?? []).filter((m) => m.mode === "chat");

  // Clear a selected model the current provider no longer offers.
  useEffect(() => {
    if (values.model && models.length > 0 && !models.some((m) => m.id === values.model)) {
      setValues((prev) => ({ ...prev, model: "" }));
    }
  }, [models, values.model]);

  function update<K extends keyof AgentFormValues>(field: K, value: AgentFormValues[K]) {
    setValues((prev) => ({ ...prev, [field]: value }));
    // Re-validate the touched field so inline messages track edits.
    setErrors((prev) => {
      const next = { ...prev };
      delete next[field];
      const single = validateAgentForm({ ...values, [field]: value })[field];
      if (single) next[field] = single;
      return next;
    });
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    setFormError(null);

    const fieldErrors = validateAgentForm(values);
    if (Object.keys(fieldErrors).length > 0) {
      setErrors(fieldErrors);
      return;
    }

    const input: AgentInput = {
      name: values.name.trim(),
      preamble: values.preamble.trim().length > 0 ? values.preamble : null,
      system_prompt: values.system_prompt,
      provider: values.provider,
      model: values.model,
      recent_n: values.recent_n,
      top_k: values.top_k,
    };

    setSubmitting(true);
    try {
      await onSubmit(input);
    } catch (error) {
      if (error instanceof ApiClientError) {
        const mapped = mapApiError(error, input.name);
        if (Object.keys(mapped).length > 0) {
          setErrors((prev) => ({ ...prev, ...mapped }));
        } else {
          setFormError(error.message);
        }
      } else {
        setFormError("Something went wrong. Please retry.");
      }
    } finally {
      setSubmitting(false);
    }
  }

  const title =
    mode === "edit" ? "Edit agent" : mode === "duplicate" ? "Duplicate agent" : "New agent";
  const submitLabel = mode === "edit" ? "Save changes" : "Create agent";
  const preambleLen = [...values.preamble].length;
  const systemLen = [...values.system_prompt].length;

  return (
    <Modal
      open
      onClose={onCancel}
      title={title}
      size="lg"
      busy={submitting}
      footer={
        <>
          <Button variant="secondary" onClick={onCancel} disabled={submitting}>
            Cancel
          </Button>
          <Button
            type="submit"
            form={formId}
            variant="primary"
            loading={submitting}
            loadingLabel="Saving…"
          >
            {submitLabel}
          </Button>
        </>
      }
    >
      {mode === "duplicate" && initial && (
        <p className={styles.caption}>Duplicating from {initial.name}</p>
      )}

      <form id={formId} onSubmit={handleSubmit} aria-label={title}>
        <div className={styles.grid}>
          <div className={styles.column}>
            <Input
              label="Name"
              value={values.name}
              onChange={(e) => update("name", e.target.value)}
              error={errors.name}
            />
            <Select
              label="Provider"
              value={values.provider}
              onChange={(e) => update("provider", e.target.value)}
              error={errors.provider}
            >
              <option value="">Select a provider…</option>
              {providers.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.id}
                </option>
              ))}
            </Select>
            <Select
              label="Model"
              value={values.model}
              onChange={(e) => update("model", e.target.value)}
              disabled={values.provider.length === 0}
              error={errors.model}
              hint={values.provider.length === 0 ? "Select a provider first" : undefined}
            >
              <option value="">Select a model…</option>
              {models.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.label}
                </option>
              ))}
            </Select>
            <div className={styles.row}>
              <Input
                label="Recent messages (recent-N)"
                type="number"
                value={Number.isNaN(values.recent_n) ? "" : values.recent_n}
                onChange={(e) => update("recent_n", e.target.valueAsNumber)}
                error={errors.recent_n}
              />
              <Input
                label="Retrieval breadth (top-K)"
                type="number"
                value={Number.isNaN(values.top_k) ? "" : values.top_k}
                onChange={(e) => update("top_k", e.target.valueAsNumber)}
                error={errors.top_k}
              />
            </div>
          </div>

          <div className={styles.column}>
            <Textarea
              label="Preamble (optional)"
              mono
              rows={4}
              value={values.preamble}
              onChange={(e) => update("preamble", e.target.value)}
              error={errors.preamble}
              counter={`${preambleLen} / ${PREAMBLE_MAX}`}
              counterOver={preambleLen > PREAMBLE_MAX}
            />
            <Textarea
              label="System prompt"
              mono
              rows={8}
              value={values.system_prompt}
              onChange={(e) => update("system_prompt", e.target.value)}
              error={errors.system_prompt}
              counter={`${systemLen} / ${SYSTEM_PROMPT_MAX}`}
              counterOver={systemLen > SYSTEM_PROMPT_MAX}
            />
          </div>
        </div>

        {formError && (
          <div className={styles.formError}>
            <Alert variant="danger" role="alert">
              {formError}
            </Alert>
          </div>
        )}
      </form>
    </Modal>
  );
}

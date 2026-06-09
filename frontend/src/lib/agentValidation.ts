/**
 * Pure client-side mirror of the backend agent field rules (F04). Used for
 * inline form validation; the server remains the source of truth and may still
 * reject a write (e.g. name-uniqueness, provider/model). Messages match the
 * server's so the UX is consistent whichever side reports the error.
 */

export const NAME_MAX = 64;
export const PREAMBLE_MAX = 2000;
export const SYSTEM_PROMPT_MAX = 32000;
export const RECENT_N_MIN = 0;
export const RECENT_N_MAX = 100;
export const TOP_K_MIN = 0;
export const TOP_K_MAX = 50;

/** Count Unicode code points to mirror the server's character-length checks. */
function charLength(value: string): number {
  return [...value].length;
}

export function validateName(name: string): string | null {
  const trimmed = name.trim();
  if (trimmed.length === 0) return "Name is required.";
  if (charLength(trimmed) > NAME_MAX) return `Name must be ${NAME_MAX} characters or fewer.`;
  return null;
}

export function validatePreamble(preamble: string): string | null {
  if (charLength(preamble) > PREAMBLE_MAX) {
    return `Preamble must be ${PREAMBLE_MAX} characters or fewer.`;
  }
  return null;
}

export function validateSystemPrompt(systemPrompt: string): string | null {
  if (systemPrompt.trim().length === 0) return "System prompt is required.";
  if (charLength(systemPrompt) > SYSTEM_PROMPT_MAX) {
    return `System prompt must be ${SYSTEM_PROMPT_MAX} characters or fewer.`;
  }
  return null;
}

function validateRange(value: number, min: number, max: number): string | null {
  if (!Number.isInteger(value) || value < min || value > max) {
    return `Value must be between ${min} and ${max}.`;
  }
  return null;
}

export function validateRecentN(recentN: number): string | null {
  return validateRange(recentN, RECENT_N_MIN, RECENT_N_MAX);
}

export function validateTopK(topK: number): string | null {
  return validateRange(topK, TOP_K_MIN, TOP_K_MAX);
}

export function validateProvider(provider: string): string | null {
  return provider.length === 0 ? "Select a provider." : null;
}

export function validateModel(model: string): string | null {
  return model.length === 0 ? "Select a model." : null;
}

/** Field-keyed validation messages; only failing fields are present. */
export interface AgentFieldErrors {
  name?: string;
  preamble?: string;
  system_prompt?: string;
  provider?: string;
  model?: string;
  recent_n?: string;
  top_k?: string;
}

export interface AgentFormValues {
  name: string;
  preamble: string;
  system_prompt: string;
  provider: string;
  model: string;
  recent_n: number;
  top_k: number;
}

/** Validate a whole form, returning only the fields that failed. */
export function validateAgentForm(values: AgentFormValues): AgentFieldErrors {
  const errors: AgentFieldErrors = {};
  const checks: Array<[keyof AgentFieldErrors, string | null]> = [
    ["name", validateName(values.name)],
    ["preamble", validatePreamble(values.preamble)],
    ["system_prompt", validateSystemPrompt(values.system_prompt)],
    ["provider", validateProvider(values.provider)],
    ["model", validateModel(values.model)],
    ["recent_n", validateRecentN(values.recent_n)],
    ["top_k", validateTopK(values.top_k)],
  ];
  for (const [field, message] of checks) {
    if (message) errors[field] = message;
  }
  return errors;
}

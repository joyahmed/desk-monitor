/** One <option> of a Select. */
export type SelectOption = { value: string; label: string };

/** Plain strings to options; `labels` gives a display name per value (else the value). */
export const toOptions = (
  values: readonly string[],
  labels: Readonly<Record<string, string>> = {}
): SelectOption[] => values.map((v) => ({ value: v, label: labels[v] ?? v }));

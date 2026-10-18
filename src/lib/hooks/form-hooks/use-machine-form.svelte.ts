import {
  saveConfig,
  type Config,
  type Machine,
  type Os,
  type Role,
} from "$lib/api";
import {
  ACTION_FIELDS,
  machineFields,
  roleOptionsFor,
  type FieldKey,
  type MachineField,
} from "$lib/machines/machine-fields";

const slug = (s: string) =>
  s
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

// Same rule as the Rust validator in src-tauri/src/config.rs.
const aliasError = (a: string) =>
  a.startsWith("-") || !/^[A-Za-z0-9._@-]+$/.test(a)
    ? "SSH alias: use letters, digits, . _ @ - and do not start with -."
    : "";

export const useMachineForm = (
  getConfig: () => Config,
  getMachine: () => Machine | null,
  getRole: () => Role,
  onsaved: (c: Config) => void,
) => {
  // Deliberate one-time read: the parent re-mounts this panel (keyed) per machine,
  // so the initial values are a snapshot, not something to keep in sync.
  const machine = getMachine();
  const editing = machine !== null;
  const origId = machine?.id ?? "";
  const lockedRole = machine?.role === "deskflow-server";
  const roleOptions = roleOptionsFor(machine?.role ?? getRole(), lockedRole);

  const values = $state<Record<FieldKey, string>>({
    name: machine?.name ?? "",
    label: machine?.label ?? "",
    id: machine?.id ?? "",
    os: machine?.os ?? "windows",
    role: machine?.role ?? getRole(),
    host: machine?.host ?? "",
    ssh_alias: machine?.ssh_alias ?? "",
    deskflow_name: machine?.deskflow_name ?? "",
    wake_script: machine?.wake_script ?? "",
    login_command: machine?.login_command ?? "",
    client_restart_command: machine?.client_restart_command ?? "",
  });
  let idEdited = $state(false);
  let error = $state("");
  let saving = $state(false);
  let confirmingDelete = $state(false);

  const id = $derived(
    editing ? origId : idEdited ? values.id : slug(values.name),
  );
  const role = $derived(values.role as Role);
  const visibleFields = $derived(machineFields.filter((f) => f.showFor(role)));
  const hasDeskflowFields = $derived(
    visibleFields.some((f) => f.key === "ssh_alias"),
  );
  const canDelete = editing && !lockedRole;

  const optionsFor = (f: MachineField) =>
    f.key === "role" ? roleOptions : (f.options ?? []);
  const hintFor = (f: MachineField) => f.hint?.(role);
  const valueOf = (key: FieldKey) => (key === "id" ? id : values[key]);
  const setValue = (key: FieldKey, v: string) => {
    if (key === "id") {
      idEdited = true;
      values.id = slug(v);
    } else values[key] = v;
  };
  const isReadonly = (key: FieldKey) =>
    (key === "id" && editing) || (key === "role" && lockedRole);

  const validate = () => {
    if (!values.name.trim()) return "Name is required.";
    if (!values.host.trim()) return "Host is required.";
    if (!id) return "Id is empty. Use letters or digits in the name.";
    if (!editing && getConfig().machines.some((m) => m.id === id))
      return `Id "${id}" is already used.`;
    const alias = values.ssh_alias.trim();
    if (hasDeskflowFields && alias) return aliasError(alias);
    return "";
  };

  const persist = async (next: Config) => {
    saving = true;
    error = "";
    try {
      await saveConfig(next);
      onsaved(next);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  };

  const save = () => {
    error = validate();
    if (error) return;
    const config = getConfig();
    const m = {
      ...(machine ?? {}),
      id,
      name: values.name.trim(),
      os: values.os as Os,
      role,
      host: values.host.trim(),
      label: values.label.trim() || undefined,
    } as Machine;
    if (hasDeskflowFields) {
      m.ssh_alias = values.ssh_alias.trim() || undefined;
      m.deskflow_name = values.deskflow_name.trim() || undefined;
    }
    for (const k of ACTION_FIELDS) {
      m[k] = role === "deskflow-client" ? values[k].trim() || undefined : undefined;
    }
    const machines = editing
      ? config.machines.map((x) => (x.id === origId ? m : x))
      : [...config.machines, m];
    void persist({ ...config, machines });
  };

  const remove = () => {
    const config = getConfig();
    void persist({
      ...config,
      machines: config.machines.filter((x) => x.id !== origId),
    });
  };

  const submit = (e: Event) => {
    e.preventDefault();
    save();
  };
  const askDelete = () => {
    confirmingDelete = true;
  };
  const cancelDelete = () => {
    confirmingDelete = false;
  };

  return {
    editing,
    canDelete,
    machineName: machine?.name ?? "",
    get visibleFields() {
      return visibleFields;
    },
    get error() {
      return error;
    },
    get saving() {
      return saving;
    },
    get confirmingDelete() {
      return confirmingDelete;
    },
    submit,
    askDelete,
    cancelDelete,
    valueOf,
    optionsFor,
    hintFor,
    setValue,
    isReadonly,
    save,
    remove,
  };
};

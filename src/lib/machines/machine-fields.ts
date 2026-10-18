import type { Role } from "$lib/api";

export type FieldKey =
  | "name"
  | "label"
  | "id"
  | "os"
  | "role"
  | "host"
  | "ssh_alias"
  | "deskflow_name"
  | "wake_script"
  | "login_command"
  | "client_restart_command";

export interface MachineField {
  key: FieldKey;
  label: string;
  type: "text" | "select";
  options?: string[];
  showFor: (role: Role) => boolean;
  hint?: (role: Role) => string | undefined;
}

/** Role choices for the form: the tab it was opened from decides. */
/** What the OS and Role selects show; the stored values stay lowercase ids. */
export const OPTION_LABEL: Readonly<Record<string, string>> = {
  windows: "Windows",
  linux: "Linux",
  mac: "macOS",
  rdp: "Remote Desktop",
  "deskflow-server": "Deskflow server",
  "deskflow-client": "Deskflow client",
};

export const roleOptionsFor = (
  context: Role,
  editingServer: boolean,
): Role[] =>
  context === "rdp"
    ? ["rdp"]
    : editingServer
      ? ["deskflow-server", "deskflow-client"]
      : ["deskflow-client"];

export const hostHint = (role: Role) =>
  role === "rdp"
    ? "Store the PC name, not the IP, because addresses move."
    : "Raw LAN IP. Deskflow clients can't resolve .local names.";

const always = () => true;
const deskflowOnly = (role: Role) =>
  role === "deskflow-server" || role === "deskflow-client";
const clientOnly = (role: Role) => role === "deskflow-client";

/** Optional per-client actions: each button shows only when its field has text. */
export const ACTION_FIELDS = [
  "wake_script",
  "login_command",
  "client_restart_command",
] as const satisfies readonly FieldKey[];

export const machineFields: MachineField[] = [
  { key: "name", label: "Name", type: "text", showFor: always },
  { key: "label", label: "Label", type: "text", showFor: always },
  { key: "id", label: "Id", type: "text", showFor: always },
  {
    key: "os",
    label: "OS",
    type: "select",
    options: ["windows", "linux", "mac"],
    showFor: always,
  },
  {
    key: "role",
    label: "Role",
    type: "select",
    showFor: always,
  },
  {
    key: "host",
    label: "Host",
    type: "text",
    showFor: always,
    hint: hostHint,
  },
  { key: "ssh_alias", label: "SSH alias", type: "text", showFor: deskflowOnly },
  {
    key: "deskflow_name",
    label: "Deskflow name",
    type: "text",
    showFor: deskflowOnly,
  },
  {
    key: "wake_script",
    label: "Wake script",
    type: "text",
    showFor: clientOnly,
    hint: () => "Optional. Full path to a .ps1 on this PC. Adds a Wake button.",
  },
  {
    key: "login_command",
    label: "Log in command",
    type: "text",
    showFor: clientOnly,
    hint: () =>
      "Optional. Run over ssh; gets the password on stdin, exits 0 ok, 3 wrong, 4 busy. Adds Log in.",
  },
  {
    key: "client_restart_command",
    label: "Client restart command",
    type: "text",
    showFor: clientOnly,
    hint: () =>
      "Optional. Run over ssh to restart Deskflow there. Adds Restart client.",
  },
];

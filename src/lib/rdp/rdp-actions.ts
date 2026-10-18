import type { Machine } from "$lib/api";

export type RdpActionKey = "connect" | "sessions" | "paste";

export interface RdpAction {
  key: RdpActionKey;
  label: string;
  variant: "primary" | "ghost" | "danger";
  title: string;
  showFor: (m: Machine) => boolean;
}

export const rdpActions: RdpAction[] = [
  {
    key: "connect",
    label: "Connect",
    variant: "primary",
    title: "Open a Remote Desktop connection",
    showFor: () => true,
  },
  {
    key: "sessions",
    label: "Sessions",
    variant: "ghost",
    title: "Show or hide sessions",
    showFor: () => true,
  },
  {
    key: "paste",
    label: "Fix paste",
    variant: "ghost",
    title: "Restart the clipboard helper on the PC",
    showFor: () => true,
  },
];

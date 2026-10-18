import { rdpConnect, rdpFixPaste, type Machine } from "$lib/api";
import { rdpActions, type RdpActionKey } from "$lib/rdp/rdp-actions";

export const useRdpActions = (
  getMachine: () => Machine,
  onedit: (m: Machine | null, role: "rdp") => void,
) => {
  let busy = $state<RdpActionKey | "">("");
  let message = $state("");
  let messageErr = $state(false);
  let sessionsOpen = $state(false);

  const run = async (key: RdpActionKey) => {
    if (key === "sessions") {
      sessionsOpen = !sessionsOpen;
      return;
    }
    const id = getMachine().id;
    busy = key;
    message = "";
    messageErr = false;
    try {
      if (key === "connect") {
        await rdpConnect(id);
        message = "Connecting…";
      } else {
        message = await rdpFixPaste(id);
      }
    } catch (e) {
      message = String(e);
      messageErr = true;
    } finally {
      busy = "";
    }
  };

  const shown = $derived(rdpActions.filter((a) => a.showFor(getMachine())));
  const isBusy = (key: RdpActionKey) => busy === key;
  const isDisabled = (key: RdpActionKey) =>
    busy !== "" && (key === "connect" || key === "paste");

  return {
    get shown() {
      return shown;
    },
    isBusy,
    isDisabled,
    get busy() {
      return busy;
    },
    get message() {
      return message;
    },
    get messageErr() {
      return messageErr;
    },
    get sessionsOpen() {
      return sessionsOpen;
    },
    run,
    edit: () => onedit(getMachine(), "rdp"),
  };
};

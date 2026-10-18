import { saveConfig, type Config } from '$lib/api';

export const DEFAULT_DESK_NAME = 'Your desk';

/** The desk's name (config.desk_name, "Your desk" when unset/blank) and its inline rename form. */
export const useDeskName = (getConfig: () => Config, onsaved: (c: Config) => void) => {
  const name = $derived(getConfig().desk_name?.trim() || DEFAULT_DESK_NAME);

  let renaming = $state(false);
  let draft = $state('');
  let saving = $state(false);
  let error = $state('');

  const start = () => {
    draft = getConfig().desk_name?.trim() ?? '';
    error = '';
    renaming = true;
  };
  const cancel = () => {
    renaming = false;
    error = '';
  };
  const save = async () => {
    if (saving) return;
    saving = true;
    error = '';
    try {
      // "" clears it: the backend stores a blank name as unset
      const next: Config = { ...getConfig(), desk_name: draft.trim() };
      await saveConfig(next);
      onsaved(next);
      renaming = false;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  };
  const onkeydown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      void save();
    } else if (e.key === 'Escape') cancel();
  };

  return {
    get name() {
      return name;
    },
    get renaming() {
      return renaming;
    },
    get draft() {
      return draft;
    },
    set draft(v: string) {
      draft = v;
    },
    get saving() {
      return saving;
    },
    get error() {
      return error;
    },
    start,
    cancel,
    save,
    onkeydown
  };
};

export type DeskNameHook = ReturnType<typeof useDeskName>;

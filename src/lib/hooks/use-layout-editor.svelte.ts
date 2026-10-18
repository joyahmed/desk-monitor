import { tick } from "svelte";
import { layoutGet, layoutSave } from "$lib/api";

export const useLayoutEditor = () => {
  let order = $state<string[]>([]);
  let loadedOrder = $state<string[]>([]);
  let loaded = $state(false);
  let loadError = $state("");
  let saving = $state(false);
  let confirming = $state(false);
  let result = $state<{ ok: boolean; text: string } | null>(null);
  let dragName = $state<string | null>(null);
  let dragDx = $state(0);
  let dragDy = $state(0);

  // Plain map (not reactive on purpose); filled by the `tile` action.
  const els = new Map<string, HTMLElement>();
  let grabX = 0;
  let pointerId = -1;

  const dirty = $derived(
    order.length !== loadedOrder.length ||
      order.some((n, i) => n !== loadedOrder[i]),
  );

  const load = async () => {
    try {
      const o = await layoutGet();
      order = o;
      loadedOrder = o.slice();
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  };
  void load();

  const tile = (node: HTMLElement, name: string) => {
    els.set(name, node);
    return { destroy: () => void els.delete(name) };
  };

  const move = (from: number, to: number) => {
    if (to < 0 || to >= order.length || from === to) return;
    const next = order.slice();
    const [x] = next.splice(from, 1);
    next.splice(to, 0, x);
    order = next;
    result = null;
    confirming = false;
  };

  const down = (e: PointerEvent, name: string) => {
    if (e.button !== 0 || saving) return;
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    pointerId = e.pointerId;
    dragName = name;
    const r = el.getBoundingClientRect();
    grabX = e.clientX - r.left;
    dragDx = 0;
    dragDy = 0;
  };

  // Horizontal only: the desk is one row. r includes the current transform; remove it to get
  // the slot position, so the block stays under the pointer after a reorder moves its slot.
  const follow = (e: PointerEvent) => {
    if (dragName === null) return;
    const r = els.get(dragName)?.getBoundingClientRect();
    if (!r) return;
    dragDx = e.clientX - grabX - (r.left - dragDx);
    dragDy = 0;
  };

  const centre = (r: DOMRect) => r.left + r.width / 2;

  // Target slot = how many other blocks have their centre left of the dragged block's centre.
  // Comparing centres (not "pointer inside a rect") keeps a narrow block from flipping back and
  // forth over a wide one (the server block is three monitors wide).
  const onmove = async (e: PointerEvent) => {
    if (dragName === null || e.pointerId !== pointerId) return;
    const name = dragName;
    follow(e);
    const me = els.get(name)?.getBoundingClientRect();
    if (!me) return;
    const c = centre(me);
    const target = order.filter((n) => {
      if (n === name) return false;
      const r = els.get(n)?.getBoundingClientRect();
      return !!r && centre(r) < c;
    }).length;
    const idx = order.indexOf(name);
    if (target !== idx) {
      move(idx, target);
      await tick();
      follow(e);
    }
  };

  const onup = (e: PointerEvent) => {
    if (e.pointerId !== pointerId) return;
    dragName = null;
    dragDx = 0;
    dragDy = 0;
    pointerId = -1;
  };

  const onkey = async (e: KeyboardEvent, name: string) => {
    if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
    e.preventDefault();
    const i = order.indexOf(name);
    move(i, i + (e.key === "ArrowLeft" ? -1 : 1));
    await tick();
    els.get(name)?.focus();
  };

  const askSave = () => {
    if (dirty) confirming = true;
  };
  const cancelSave = () => {
    confirming = false;
  };

  /** Throws away an unsaved reorder (leaving edit mode without saving). */
  const revert = () => {
    if (saving) return;
    order = loadedOrder.slice();
    confirming = false;
  };

  const save = async () => {
    confirming = false;
    saving = true;
    result = null;
    try {
      const text = await layoutSave(order);
      result = { ok: true, text: text || "Saved." };
      loadedOrder = order.slice();
    } catch (e) {
      result = { ok: false, text: String(e) };
    } finally {
      saving = false;
    }
  };

  return {
    get order() {
      return order;
    },
    get loaded() {
      return loaded;
    },
    get loadError() {
      return loadError;
    },
    get saving() {
      return saving;
    },
    get confirming() {
      return confirming;
    },
    get result() {
      return result;
    },
    get dirty() {
      return dirty;
    },
    get dragName() {
      return dragName;
    },
    get dragDx() {
      return dragDx;
    },
    get dragDy() {
      return dragDy;
    },
    tile,
    down,
    onmove,
    onup,
    onkey,
    askSave,
    cancelSave,
    save,
    revert,
  };
};

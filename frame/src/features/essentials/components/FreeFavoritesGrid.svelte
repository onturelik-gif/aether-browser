<script lang="ts">
  import { tabs } from "$domain/tabs";
  import { favicons } from "$domain/favicons";
  import * as tabDrag from "$session/tab-drag.svelte";
  import DragMark from "$shared/ui/DragMark";
  import EssentialTile from "./EssentialTile.svelte";
  import type { SidebarEntry } from "$features/tabs";
  import type { TabView } from "$shared/ipc/bindings";

  let {
    entries,
    columns = 3,
    onSelect,
  }: {
    entries: SidebarEntry[];
    columns?: number;
    onSelect: (id: string) => void;
  } = $props();

  const STORAGE_KEY = "aether.dock.freegrid.v1";

  let favoriteTabs = $derived(
    entries.flatMap((e) => (e.kind === "tab" ? [e.tab] : [])),
  );

  function tabKey(tab: TabView): string {
    return tab.url ? `u:${tab.url}` : `id:${tab.id}`;
  }

  function loadMap(): Record<string, number> {
    if (typeof localStorage === "undefined") return {};
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) return {};
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === "object") return parsed as Record<string, number>;
    } catch {}
    return {};
  }

  let slotMap = $state<Record<string, number>>(loadMap());
  let dragState = $state<{
    key: string;
    tab: TabView;
    fromSlot: number;
    startX: number;
    startY: number;
    x: number;
    y: number;
    active: boolean;
    overSlot: number | null;
  } | null>(null);
  let swallowClick = false;

  function saveMap(next: Record<string, number>) {
    slotMap = next;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {}
  }

  // Keep enough rows for all tiles + saved slots, and always at least 1 empty row while dragging
  // so the user can freely move any tile out and leave empty spaces anywhere in the grid.
  let totalSlots = $derived.by(() => {
    if (favoriteTabs.length === 0) return 0;
    const cols = Math.max(1, columns);
    const maxAssigned = Object.values(slotMap).reduce(
      (m, v) => (Number.isInteger(v) && v > m ? v : m),
      -1,
    );
    const minRows = Math.max(3, Math.ceil(favoriteTabs.length / cols));
    const neededByPos = maxAssigned >= 0 ? Math.ceil((maxAssigned + 1) / cols) : 0;
    let rows = Math.max(minRows, neededByPos);
    // While dragging, ensure there's always at least one empty row at the top/bottom to drop into
    if (dragState?.active && rows * cols <= favoriteTabs.length) {
      rows += 1;
    }
    return rows * cols;
  });

  // Resolve each slot index -> TabView | null
  let cells = $derived.by(() => {
    const out: (TabView | null)[] = Array.from({ length: totalSlots }, () => null);
    const placed = new Set<string>();

    for (const tab of favoriteTabs) {
      const key = tabKey(tab);
      const idx = slotMap[key];
      if (Number.isInteger(idx) && idx >= 0 && idx < out.length && out[idx] === null) {
        out[idx] = tab;
        placed.add(tab.id);
      }
    }

    for (const tab of favoriteTabs) {
      if (placed.has(tab.id)) continue;
      let chosen = -1;
      for (let i = out.length - 1; i >= 0; i--) {
        if (out[i] === null) {
          chosen = i;
          break;
        }
      }
      if (chosen !== -1) {
        out[chosen] = tab;
        placed.add(tab.id);
      }
    }
    return out;
  });

  function handleContextMenu(event: MouseEvent, tab: TabView) {
    event.preventDefault();
    dragState = null;
    tabs.openTabMenu(tab.id, event.clientX, event.clientY);
  }

  function slotAtPoint(x: number, y: number): number | null {
    const hit = document.elementFromPoint(x, y);
    const cell = hit?.closest<HTMLElement>("[data-free-slot]");
    if (!cell) return null;
    const n = Number(cell.dataset.freeSlot);
    return Number.isInteger(n) && n >= 0 ? n : null;
  }

  function onTilePointerDown(e: PointerEvent, tab: TabView, slotIdx: number) {
    if (e.button !== 0) return;
    (e.currentTarget as HTMLElement)?.setPointerCapture?.(e.pointerId);
    dragState = {
      key: tabKey(tab),
      tab,
      fromSlot: slotIdx,
      startX: e.clientX,
      startY: e.clientY,
      x: e.clientX,
      y: e.clientY,
      active: false,
      overSlot: slotIdx,
    };
  }

  function onTilePointerMove(e: PointerEvent) {
    if (!dragState) return;
    const dx = e.clientX - dragState.startX;
    const dy = e.clientY - dragState.startY;
    if (!dragState.active && Math.hypot(dx, dy) < 6) return;
    const over = slotAtPoint(e.clientX, e.clientY);
    dragState = {
      ...dragState,
      active: true,
      x: e.clientX,
      y: e.clientY,
      overSlot: over,
    };
  }

  function onTilePointerUp(e: PointerEvent) {
    const cur = dragState;
    dragState = null;
    if (!cur || !cur.active) return;

    swallowClick = true;
    setTimeout(() => {
      swallowClick = false;
    }, 50);

    const targetSlot = slotAtPoint(e.clientX, e.clientY);
    if (targetSlot === null || targetSlot === cur.fromSlot) return;

    // Snapshot every currently visible tile's exact slot so leaving a slot empty stays empty!
    const next: Record<string, number> = {};
    for (let i = 0; i < cells.length; i++) {
      const t = cells[i];
      if (!t) continue;
      next[tabKey(t)] = i;
    }

    const occupant = cells[targetSlot];
    if (occupant && tabKey(occupant) !== cur.key) {
      // Swap occupant into the dragged tile's previous slot
      next[tabKey(occupant)] = cur.fromSlot;
    }
    next[cur.key] = targetSlot;
    saveMap(next);
  }

  function onTilePointerCancel() {
    dragState = null;
  }

  function handleSelect(id: string) {
    if (swallowClick) return;
    onSelect(id);
  }

  let activeId = $derived(tabs.activeId());
  let anyDragging = $derived((dragState?.active ?? false) || tabDrag.draggedId() !== null);
</script>

{#if totalSlots > 0}
  <ul
    class="aether-free-grid"
    style:--fg-cols={Math.max(1, columns)}
    data-dragging={anyDragging || undefined}
    data-essentials-drop
    aria-label="Sık Kullanılanlar"
  >
    {#each cells as tab, idx (idx)}
      <li
        class="aether-free-cell"
        data-free-slot={idx}
        data-empty={tab === null || undefined}
        data-over={(dragState?.active && dragState.overSlot === idx) || undefined}
      >
        {#if tab}
          <div
            class="aether-free-item"
            role="presentation"
            data-dragging-self={(dragState?.active && dragState.key === tabKey(tab)) || undefined}
            onpointerdown={(e) => onTilePointerDown(e, tab, idx)}
            onpointermove={onTilePointerMove}
            onpointerup={onTilePointerUp}
            onpointercancel={onTilePointerCancel}
          >
            <EssentialTile
              {tab}
              active={tab.id === activeId}
              splitCandidate={false}
              onSelect={handleSelect}
              onContextMenu={handleContextMenu}
              onPointerDown={() => {}}
              onPointerMove={() => {}}
              onPointerUp={() => {}}
              onPointerCancel={() => {}}
            />
          </div>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

{#if dragState?.active}
  <DragMark
    x={dragState.x}
    y={dragState.y}
    image={favicons.image(dragState.tab.icon)}
    tone={favicons.tone(dragState.tab.icon)}
    title=""
  />
{/if}

<style>
  .aether-free-grid {
    display: grid;
    grid-template-columns: repeat(var(--fg-cols, 3), minmax(0, 1fr));
    gap: var(--dock-gap, 6px);
    list-style: none;
    margin: 0;
    padding: 0;
    width: 100%;
    user-select: none;
  }

  .aether-free-cell {
    min-width: 0;
    height: 40px;
    border-radius: 11px;
    transition: background-color 120ms ease, box-shadow 120ms ease;
  }

  .aether-free-grid[data-dragging] .aether-free-cell[data-empty] {
    background: rgba(255, 255, 255, 0.04);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.14);
  }

  .aether-free-cell[data-over] {
    background: rgba(56, 189, 248, 0.16) !important;
    box-shadow: inset 0 0 0 1.5px rgba(56, 189, 248, 0.78) !important;
  }

  .aether-free-item {
    width: 100%;
    height: 100%;
    touch-action: none;
  }

  .aether-free-item[data-dragging-self] {
    opacity: 0.28;
  }

  .aether-free-item :global(> li.essential) {
    list-style: none;
    width: 100%;
    height: 100%;
  }
</style>

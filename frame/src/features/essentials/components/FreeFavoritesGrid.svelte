<script lang="ts">
  import { tabs } from "$domain/tabs";
  import * as tabDrag from "$session/tab-drag.svelte";
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
  let draggingKey = $state<string | null>(null);
  let overSlot = $state<number | null>(null);

  function saveMap(next: Record<string, number>) {
    slotMap = next;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {}
  }

  // Always keep at least 3 rows when favorites exist so user can freely place tiles in top/middle/bottom rows.
  let totalSlots = $derived.by(() => {
    const cols = Math.max(1, columns);
    const maxAssigned = Object.values(slotMap).reduce((m, v) => (Number.isInteger(v) && v > m ? v : m), -1);
    const neededByCount = Math.max(cols * 3, Math.ceil(favoriteTabs.length / cols) * cols);
    const neededByPos = maxAssigned >= 0 ? Math.ceil((maxAssigned + 1) / cols) * cols : 0;
    const base = Math.max(neededByCount, neededByPos);
    return favoriteTabs.length === 0 ? 0 : base;
  });

  // Resolve each slot index -> TabView | null
  let cells = $derived.by(() => {
    const out: (TabView | null)[] = Array.from({ length: totalSlots }, () => null);
    const placed = new Set<string>();

    // 1. Place tabs that already have a valid saved slot
    for (const tab of favoriteTabs) {
      const key = tabKey(tab);
      const idx = slotMap[key];
      if (Number.isInteger(idx) && idx >= 0 && idx < out.length && out[idx] === null) {
        out[idx] = tab;
        placed.add(tab.id);
      }
    }

    // 2. Place remaining tabs into open slots from the bottom row up (nearest the dock)
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
    tabs.openTabMenu(tab.id, event.clientX, event.clientY);
  }

  function onDragStart(e: DragEvent, tab: TabView) {
    draggingKey = tabKey(tab);
    e.dataTransfer?.setData("text/plain", draggingKey);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }

  function onDragOver(e: DragEvent, slotIdx: number) {
    if (!draggingKey) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    overSlot = slotIdx;
  }

  function onDrop(e: DragEvent, targetSlot: number) {
    e.preventDefault();
    const key = draggingKey;
    draggingKey = null;
    overSlot = null;
    if (!key) return;

    // Build snapshot of current resolved positions so unpinned slots stay put
    const next: Record<string, number> = {};
    let fromSlot = -1;
    for (let i = 0; i < cells.length; i++) {
      const t = cells[i];
      if (!t) continue;
      const k = tabKey(t);
      next[k] = i;
      if (k === key) fromSlot = i;
    }

    const occupant = cells[targetSlot];
    if (occupant && tabKey(occupant) !== key && fromSlot !== -1) {
      // Swap
      next[tabKey(occupant)] = fromSlot;
    }
    next[key] = targetSlot;
    saveMap(next);
  }

  function onDragEnd() {
    draggingKey = null;
    overSlot = null;
  }

  let activeId = $derived(tabs.activeId());
  let anyDragging = $derived(draggingKey !== null || tabDrag.draggedId() !== null);
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
        data-empty={tab === null || undefined}
        data-over={overSlot === idx || undefined}
        ondragover={(e) => onDragOver(e, idx)}
        ondragleave={() => {
          if (overSlot === idx) overSlot = null;
        }}
        ondrop={(e) => onDrop(e, idx)}
      >
        {#if tab}
          <div
            class="aether-free-item"
            draggable="true"
            role="presentation"
            data-dragging-self={draggingKey === tabKey(tab) || undefined}
            ondragstart={(e) => onDragStart(e, tab)}
            ondragend={onDragEnd}
          >
            <EssentialTile
              {tab}
              active={tab.id === activeId}
              splitCandidate={false}
              {onSelect}
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

<style>
  .aether-free-grid {
    display: grid;
    grid-template-columns: repeat(var(--fg-cols, 3), minmax(0, 1fr));
    gap: var(--dock-gap, 6px);
    list-style: none;
    margin: 0;
    padding: 0;
    width: 100%;
  }

  .aether-free-cell {
    min-width: 0;
    height: var(--dock-tile, 40px);
    border-radius: 11px;
    transition: background-color 120ms ease, box-shadow 120ms ease;
  }

  .aether-free-grid[data-dragging] .aether-free-cell[data-empty] {
    background: rgba(255, 255, 255, 0.03);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.1);
  }

  .aether-free-cell[data-over] {
    background: rgba(56, 189, 248, 0.14) !important;
    box-shadow: inset 0 0 0 1.5px rgba(56, 189, 248, 0.7) !important;
  }

  .aether-free-item {
    width: 100%;
    height: 100%;
  }

  .aether-free-item[data-dragging-self] {
    opacity: 0.35;
  }

  .aether-free-item :global(> li.essential) {
    list-style: none;
    width: 100%;
    height: 100%;
  }
</style>

<script lang="ts">
  import {
    ArrowLeft01Icon,
    ArrowRight01Icon,
    Cancel01Icon,
    Globe02Icon,
    PinIcon,
    PinOffIcon,
    PlusSignIcon,
    PuzzleIcon,
    RefreshIcon,
    Settings02Icon,
    SidebarLeftIcon,
    VolumeHighIcon,
    VolumeOffIcon,
  } from "@hugeicons/core-free-icons";
  import { SvelteSet } from "svelte/reactivity";
  import { tabs } from "$domain/tabs";
  import { favicons } from "$domain/favicons";
  import { preferences } from "$domain/preferences";
  import { updates } from "$domain/updates";
  import { commands } from "$shared/ipc/bindings";
  import { AddressField, SiteDetails, SiteHead } from "$features/address";
  import { BlockerShield } from "$features/blocker";
  import { ExtensionsSection } from "$features/extensions";
  import { UtilityTray } from "$features/sidebar";
  import { sidebarTree } from "$features/tabs";
  import {
    sidebarDisplayUnits,
    collapsedSidebarUnits,
    type SidebarDisplayUnit,
  } from "$features/tabs/lib/sidebar-model";
  import { RowDrag } from "$session/row-drag.svelte";
  import * as tabDrag from "$session/tab-drag.svelte";
  import Disclosure from "$shared/ui/Disclosure";
  import DragMark from "$shared/ui/DragMark";
  import FavIcon from "$shared/ui/FavIcon";
  import Icon from "$shared/ui/Icon";
  import { IS_WINDOWS } from "$shared/platform";
  import * as m from "$shared/i18n/messages";
  import type { Snippet } from "svelte";
  import type { TabView } from "$shared/ipc/bindings";

  let {
    onSelect,
    onSettings,
  }: {
    onSelect: (id: string) => void;
    onSettings: () => void;
  } = $props();

  let tree = $derived(sidebarTree(tabs.sidebarNodes(), tabs.tabs()));
  let pinnedTabs = $derived(
    tree.favorites.flatMap((entry) => (entry.kind === "tab" ? [entry.tab] : [])),
  );
  let displayUnits = $derived(sidebarDisplayUnits("today", tree.today, tabs.splitGroup()));
  const folded = new SvelteSet<string>();
  let hiddenUnits = $derived(collapsedSidebarUnits(displayUnits, folded, tabs.activeId()));

  let activeId = $derived(tabs.activeId());
  let activeTab = $derived(tabs.activeTab());
  let updateState = $derived(updates.status());
  let updateReady = $derived(updateState.state === "ready");
  let updateBusy = $derived(
    updateState.state === "installing" || updates.pendingRelaunch(),
  );
  let headerEl = $state<HTMLElement>();
  let tabStripEl = $state<HTMLElement>();
  let bookmarksEl = $state<HTMLElement>();
  let tabDropBefore = $state<string | null>(null);
  let tabDropAtEnd = $state(false);
  let groupDragKey = $state<string | null>(null);

  function computeTabDrop(clientX: number, excludeIds: Set<string>): { before: string | null; atEnd: boolean } {
    if (!tabStripEl) return { before: null, atEnd: false };
    const els = Array.from(tabStripEl.querySelectorAll<HTMLElement>("[data-topbar-tab]"));
    for (const el of els) {
      const id = el.dataset.topbarTab;
      if (!id || excludeIds.has(id)) continue;
      const r = el.getBoundingClientRect();
      if (clientX < r.left + r.width / 2) {
        return { before: id, atEnd: false };
      }
    }
    return { before: null, atEnd: true };
  }

  function groupTabIds(groupKey: string): string[] {
    const out: string[] = [];
    let inside = false;
    for (const unit of displayUnits) {
      if (unit.kind === "folder") {
        if (unit.key === groupKey) inside = true;
        else if (inside) break;
      } else if (inside) {
        for (const t of unitTabs(unit)) out.push(t.id);
      }
    }
    return out;
  }

  // Horizontal drag-to-reorder for bookmarks in the TopBar bookmarks row.
  let bmDrag = $state<{
    id: string;
    startX: number;
    startY: number;
    x: number;
    y: number;
    active: boolean;
    dropBefore: string | null;
    dropAtEnd: boolean;
  } | null>(null);
  let bmSwallowClick = false;

  function computeBookmarkDrop(clientX: number, draggedId: string): { before: string | null; atEnd: boolean } {
    if (!bookmarksEl) return { before: null, atEnd: false };
    const items = Array.from(
      bookmarksEl.querySelectorAll<HTMLElement>("[data-bookmark-item]"),
    );
    for (const el of items) {
      const id = el.dataset.bookmarkItem;
      if (!id || id === draggedId) continue;
      const r = el.getBoundingClientRect();
      if (clientX < r.left + r.width / 2) {
        return { before: id, atEnd: false };
      }
    }
    return { before: null, atEnd: true };
  }

  function handleBookmarkPointerDown(e: PointerEvent, id: string) {
    if (e.button !== 0) return;
    (e.currentTarget as HTMLElement)?.setPointerCapture?.(e.pointerId);
    bmDrag = {
      id,
      startX: e.clientX,
      startY: e.clientY,
      x: e.clientX,
      y: e.clientY,
      active: false,
      dropBefore: null,
      dropAtEnd: false,
    };
  }

  function handleBookmarkPointerMove(e: PointerEvent) {
    if (!bmDrag) return;
    const dx = e.clientX - bmDrag.startX;
    const dy = e.clientY - bmDrag.startY;
    if (!bmDrag.active && Math.hypot(dx, dy) < 6) return;
    const target = computeBookmarkDrop(e.clientX, bmDrag.id);
    bmDrag = {
      ...bmDrag,
      active: true,
      x: e.clientX,
      y: e.clientY,
      dropBefore: target.before,
      dropAtEnd: target.atEnd,
    };
  }

  function handleBookmarkPointerUp(e: PointerEvent) {
    if (!bmDrag) return;
    const cur = bmDrag;
    bmDrag = null;
    if (!cur.active) return;
    bmSwallowClick = true;
    setTimeout(() => {
      bmSwallowClick = false;
    }, 0);
    const barBottom = headerEl?.getBoundingClientRect().bottom ?? 108;
    if (e.clientY > barBottom + 12) {
      tabs.dropTab(cur.id, e.clientX, e.clientY);
      return;
    }
    const target = computeBookmarkDrop(e.clientX, cur.id);
    void tabDrag.move(cur.id, true, target.before);
  }

  function handleBookmarkPointerCancel() {
    bmDrag = null;
  }

  const drag = new RowDrag({
    list: () => undefined,
    reorder: () => null,
    drop: ({ id, x, y, over, before }) => {
      tabDropBefore = null;
      tabDropAtEnd = false;
      if (over?.closest("[data-essentials-drop]")) {
        if (tabDrag.keepable(id)) {
          void tabDrag.move(id, true, before);
        }
        return;
      }
      const barBottom = headerEl?.getBoundingClientRect().bottom ?? 108;
      if (y > barBottom) {
        tabs.dropTab(id, x, y);
        return;
      }
      const target = computeTabDrop(x, new Set([id]));
      void tabDrag.move(id, false, target.before);
    },
  });

  $effect(() => {
    const g = drag.ghost;
    if (!g) {
      tabDropBefore = null;
      tabDropAtEnd = false;
      return;
    }
    const stripRect = tabStripEl?.getBoundingClientRect();
    if (stripRect && g.y >= stripRect.top - 12 && g.y <= stripRect.bottom + 12) {
      const target = computeTabDrop(g.x, new Set([g.id]));
      tabDropBefore = target.before;
      tabDropAtEnd = target.atEnd;
    } else {
      tabDropBefore = null;
      tabDropAtEnd = false;
    }
  });

  $effect(() => () => drag.abandon());

  const STORAGE_PREFIX = "aether.group.name:";

  function groupLabel(key: string, fallback: string): string {
    if (typeof localStorage === "undefined") return fallback;
    try {
      return localStorage.getItem(STORAGE_PREFIX + key)?.trim() || fallback;
    } catch {
      return fallback;
    }
  }

  function toggleGroup(key: string) {
    if (folded.has(key)) folded.delete(key);
    else folded.add(key);
  }

  const PIN_STORAGE_KEY = "aether.topbar.pinned";
  let topPinned = $state<boolean>(
    typeof localStorage === "undefined"
      ? true
      : localStorage.getItem(PIN_STORAGE_KEY) !== "false",
  );
  let topCollapsed = $state(false);
  let extOpen = $state(false);
  let utilityOpen = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | null = null;

  function cancelHide() {
    if (hideTimer) {
      clearTimeout(hideTimer);
      hideTimer = null;
    }
  }

  function revealTopBar() {
    cancelHide();
    if (topCollapsed) {
      topCollapsed = false;
      void commands.runCommand("topbar.expand").catch(() => {});
    }
  }

  function scheduleHideTopBar() {
    if (topPinned) return;
    if (extOpen || utilityOpen || drag.ghost !== null || bmDrag?.active) return;
    if (
      document.activeElement instanceof HTMLInputElement &&
      headerEl?.contains(document.activeElement)
    ) {
      return;
    }
    // Never auto-hide on an empty New Tab page (there's no webpage beneath).
    if (!activeTab?.url) return;
    cancelHide();
    hideTimer = setTimeout(() => {
      hideTimer = null;
      if (topPinned || extOpen || utilityOpen || drag.ghost !== null || bmDrag?.active) return;
      topCollapsed = true;
      void commands.runCommand("topbar.collapse").catch(() => {});
    }, 240);
  }

  function toggleTopPin() {
    topPinned = !topPinned;
    try {
      localStorage.setItem(PIN_STORAGE_KEY, topPinned ? "true" : "false");
    } catch {}
    if (topPinned) {
      revealTopBar();
    } else {
      scheduleHideTopBar();
    }
  }

  $effect(() => {
    if (!activeTab?.url && topCollapsed) {
      revealTopBar();
    }
  });

  $effect(() => () => {
    cancelHide();
    if (topCollapsed) {
      void commands.runCommand("topbar.expand").catch(() => {});
    }
  });

  function switchToSidebar() {
    cancelHide();
    topCollapsed = false;
    void commands.runCommand("topbar.expand").catch(() => {});
    void preferences.set("sidebar.mode", "default");
  }

  function selectTab(id: string) {
    if (drag.swallowClick()) return;
    onSelect(id);
  }

  function handleTabMouseUp(event: MouseEvent, id: string, closable: boolean) {
    if (event.button === 1 && closable) {
      event.preventDefault();
      tabs.close(id);
    }
  }

  let pinnedIds = $derived(new Set(pinnedTabs.map((t) => t.id)));

  function openBookmarkInCurrent(tab: TabView) {
    if (bmSwallowClick) return;
    const url = tab.url;
    if (!url) {
      onSelect(tab.id);
      return;
    }
    if (activeId && !pinnedIds.has(activeId)) {
      tabs.navigate(activeId, url);
    } else {
      void commands.runCommand(`bookmark.openTab=${url}`).catch(() => {});
    }
  }

  function handleBookmarkMouseDown(event: MouseEvent) {
    if (event.button === 1) {
      event.preventDefault();
    }
  }

  function handleBookmarkMouseUp(event: MouseEvent, tab: TabView) {
    if (event.button === 1) {
      event.preventDefault();
      if (tab.url) {
        void commands.runCommand(`bookmark.newTab=${tab.url}`).catch(() => {});
      } else {
        onSelect(tab.id);
      }
    }
  }

  function bookmarkName(tab: TabView): string {
    if (tab.title && tab.title.trim()) return tab.title.trim();
    if (tab.url) {
      try {
        const host = new URL(tab.url).hostname.replace(/^www\./, "");
        const base = host.split(".")[0] ?? host;
        return base.charAt(0).toUpperCase() + base.slice(1);
      } catch {}
    }
    return m.untitled_tab();
  }

  function unitTabs(unit: SidebarDisplayUnit): TabView[] {
    if (unit.kind === "tab") return [unit.tab];
    if (unit.kind === "split") return unit.tabs.map((t) => t.tab);
    return [];
  }
</script>

<header
  bind:this={headerEl}
  class="aether-topbar"
  data-windows={IS_WINDOWS || undefined}
  data-collapsed={topCollapsed || undefined}
  onpointerenter={revealTopBar}
  onpointerleave={scheduleHideTopBar}
>
  {#if topCollapsed}
    <div class="topbar-hover-strip" aria-label="Üst çubuğu göster">
      <span class="topbar-hover-pill"></span>
    </div>
  {:else}
  <!-- Row 1: Classic Chrome/Edge horizontal tab strip with horizontal tab groups & drag-to-split -->
  <div class="topbar-tabs-row" data-tauri-drag-region>
    <div
      bind:this={tabStripEl}
      class="topbar-tab-strip"
      role="tablist"
      aria-label="Sekmeler"
      data-drop-end={tabDropAtEnd || undefined}
    >
      {#each displayUnits as unit (unit.key)}
        {#if unit.kind === "folder"}
          {@const collapsed = folded.has(unit.key)}
          {@const colorIdx = unit.groupMeta?.colorIndex ?? 0}
          <button
            type="button"
            class={`topbar-group-chip aether-tint-${colorIdx}`}
            draggable="true"
            data-collapsed={collapsed || undefined}
            title={collapsed ? "Grubu aç · Sürükleyerek taşı" : "Grubu daralt · Sürükleyerek taşı"}
            onclick={() => toggleGroup(unit.key)}
            ondragstart={() => {
              groupDragKey = unit.key;
            }}
            ondragover={(e) => {
              if (!groupDragKey) return;
              e.preventDefault();
            }}
            ondrop={(e) => {
              e.preventDefault();
              if (!groupDragKey) return;
              const ids = groupTabIds(groupDragKey);
              const target = computeTabDrop(e.clientX, new Set(ids));
              for (const tid of ids) {
                void tabDrag.move(tid, false, target.before);
              }
              groupDragKey = null;
            }}
            ondragend={() => {
              groupDragKey = null;
            }}
          >
            {#if unit.groupMeta?.icon}
              <FavIcon
                image={favicons.image(unit.groupMeta.icon)}
                tone={favicons.tone(unit.groupMeta.icon)}
                size={13}
                lit
                fallback={Globe02Icon}
              />
            {/if}
            <span class="topbar-group-label">{groupLabel(unit.key, unit.node.kind.name)}</span>
            {#if collapsed && unit.groupMeta}
              <span class="topbar-group-count">{unit.groupMeta.count}</span>
            {/if}
          </button>
        {:else if !hiddenUnits.has(unit.key)}
          {#each unitTabs(unit) as tab (tab.id)}
            {@const active = tab.id === activeId}
            {@const closable = !tabs.loneNewTab(tab.id)}
            {@const colorIdx = unit.kind === "tab" ? unit.groupColorIndex : undefined}
            <div
              class={[
                "topbar-tab",
                colorIdx !== undefined ? `topbar-tab-grouped aether-tint-${colorIdx}` : "",
              ].join(" ")}
              role="tab"
              tabindex={active ? 0 : -1}
              aria-selected={active}
              data-topbar-tab={tab.id}
              data-drop-before={tabDropBefore === tab.id || undefined}
              data-active={active || undefined}
              ondragover={(e) => {
                if (!groupDragKey) return;
                e.preventDefault();
              }}
              ondrop={(e) => {
                e.preventDefault();
                if (!groupDragKey) return;
                const ids = groupTabIds(groupDragKey);
                if (!ids.includes(tab.id)) {
                  for (const tid of ids) {
                    void tabDrag.move(tid, false, tab.id);
                  }
                }
                groupDragKey = null;
              }}
              data-zephium-tab-id={tab.id}
              data-zephium-tab-url={tab.url ?? ""}
              data-zephium-projection-revision={tab.projection_revision}
              title={tab.title || m.untitled_tab()}
              onpointerdown={(e) => drag.down(e, tab.id)}
              onpointermove={(e) => drag.move(e)}
              onpointerup={(e) => drag.up(e)}
              onpointercancel={(e) => drag.cancel(e)}
              onclick={() => selectTab(tab.id)}
              onmouseup={(e) => handleTabMouseUp(e, tab.id, closable)}
              oncontextmenu={(e) => {
                e.preventDefault();
                drag.abandon();
                tabs.openTabMenu(tab.id, e.clientX, e.clientY);
              }}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  onSelect(tab.id);
                }
              }}
            >
              <span class="topbar-tab-icon">
                <FavIcon
                  image={favicons.image(tab.icon)}
                  tone={favicons.tone(tab.icon)}
                  loading={tab.loading}
                  size={15}
                  lit={active}
                  fallback={Globe02Icon}
                />
              </span>
              <span class="topbar-tab-title" data-zephium-tab-label>
                {tab.title || m.untitled_tab()}
              </span>
              {#if tab.audio}
                <button
                  type="button"
                  class="topbar-tab-audio"
                  title={tab.audio.muted ? m.tab_unmute() : m.tab_mute()}
                  aria-label={tab.audio.muted ? m.tab_unmute() : m.tab_mute()}
                  onpointerdown={(e) => e.stopPropagation()}
                  onclick={(e) => {
                    e.stopPropagation();
                    void tabs.setMuted(tab.id, !tab.audio?.muted);
                  }}
                >
                  <Icon icon={tab.audio.muted ? VolumeOffIcon : VolumeHighIcon} size={12} />
                </button>
              {/if}
              {#if closable}
                <button
                  type="button"
                  class="topbar-tab-close"
                  aria-label={m.close_named_tab({ title: tab.title || m.untitled_tab() })}
                  onpointerdown={(e) => e.stopPropagation()}
                  onclick={(e) => {
                    e.stopPropagation();
                    tabs.close(tab.id);
                  }}
                >
                  <Icon icon={Cancel01Icon} size={11} strokeWidth={2} />
                </button>
              {/if}
            </div>
          {/each}
        {/if}
      {/each}
    </div>
    <button
      type="button"
      class="topbar-newtab"
      title="Yeni Sekme"
      aria-label="Yeni Sekme"
      onclick={() => tabs.open()}
    >
      <Icon icon={PlusSignIcon} size={15} />
    </button>
    <div class="topbar-drag-spacer" data-tauri-drag-region></div>
  </div>

  <!-- Row 2: Navigation + Address Bar + Extensions + Sidebar Toggle + Settings -->
  <div class="topbar-nav-row">
    <div class="topbar-nav-group">
      <button
        type="button"
        class="topbar-icon-btn"
        disabled={!activeTab?.can_go_back}
        title="Geri"
        aria-label="Geri"
        onclick={() => tabs.backActive()}
      >
        <Icon icon={ArrowLeft01Icon} size={16} />
      </button>
      <button
        type="button"
        class="topbar-icon-btn"
        disabled={!activeTab?.can_go_forward}
        title="İleri"
        aria-label="İleri"
        onclick={() => tabs.forwardActive()}
      >
        <Icon icon={ArrowRight01Icon} size={16} />
      </button>
      <button
        type="button"
        class="topbar-icon-btn"
        title="Yenile"
        aria-label="Yenile"
        onclick={() => tabs.reloadActive()}
      >
        <Icon icon={RefreshIcon} size={15} />
      </button>
    </div>

    <div class="topbar-address">
      <AddressField compact={false}>
        {#snippet trailing(page)}
          <UtilityTray
            label={page === null
              ? m.utilities()
              : page.warning
                ? m.site_info_label_status({ status: page.warning })
                : m.site_info_label()}
            warning={page?.warning}
            onopen={() => {
              utilityOpen = true;
              cancelHide();
            }}
            onclose={() => {
              utilityOpen = false;
            }}
          >
            {#snippet children(open)}
              {#snippet siteHead(name: string | null, control?: Snippet)}
                <SiteHead {page} label={name}>{@render control?.()}</SiteHead>
              {/snippet}
              <BlockerShield {open} head={page ? siteHead : undefined} />
              <ExtensionsSection />
              <SiteDetails {page} {open} />
            {/snippet}
          </UtilityTray>
        {/snippet}
      </AddressField>
    </div>

    <div class="topbar-nav-group">
      {#if updateReady || updateBusy}
        <button
          type="button"
          class="topbar-update-pill"
          disabled={updateBusy}
          title="Yeni Aether sürümünü kur ve yeniden başlat"
          onclick={() => void updates.relaunch()}
        >
          <Icon icon={RefreshIcon} size={13} />
          <span>{updateBusy ? "Güncelleniyor…" : "Güncelle"}</span>
        </button>
      {/if}
      <Disclosure
        label="Eklentiler"
        triggerTitle="Eklentiler"
        triggerClass="topbar-icon-btn"
        panelClass="utilities-panel sidebar-softens"
        align="end"
        onopen={() => {
          extOpen = true;
          cancelHide();
        }}
        onclose={() => {
          extOpen = false;
        }}
      >
        {#snippet trigger()}
          <Icon icon={PuzzleIcon} size={15} />
        {/snippet}
        <div class="topbar-ext-popup">
          <ExtensionsSection />
        </div>
      </Disclosure>
      <button
        type="button"
        class="topbar-icon-btn"
        data-pinned={topPinned || undefined}
        title={topPinned ? "Üst çubuğu otomatik gizle" : "Üst çubuğu sabitle"}
        aria-label={topPinned ? "Üst çubuğu otomatik gizle" : "Üst çubuğu sabitle"}
        onclick={toggleTopPin}
      >
        <Icon icon={topPinned ? PinIcon : PinOffIcon} size={15} />
      </button>
      <button
        type="button"
        class="topbar-icon-btn"
        title="Sol Kenar Çubuğuna Geç"
        aria-label="Sol Kenar Çubuğuna Geç"
        onclick={switchToSidebar}
      >
        <Icon icon={SidebarLeftIcon} size={16} />
      </button>
      <button
        type="button"
        class="topbar-icon-btn"
        title="Ayarlar"
        aria-label="Ayarlar"
        onclick={onSettings}
      >
        <Icon icon={Settings02Icon} size={16} />
      </button>
    </div>
  </div>

  <!-- Row 3: Classic Bookmarks / Favorites Bar (drop tabs here to pin; drag to reorder; click=current tab, middle-click=new tab) -->
  <div
    bind:this={bookmarksEl}
    class="topbar-bookmarks-row"
    aria-label="Sık Kullanılanlar"
    data-essentials-drop
    data-over={tabDrag.overEssentials()}
    data-drop-end={(bmDrag?.active && bmDrag.dropAtEnd) || undefined}
  >
    {#if pinnedTabs.length === 0}
      <span class="topbar-bookmarks-empty">Sık kullanılanlara eklemek için bir sekmeyi buraya sürükle</span>
    {:else}
      {#each pinnedTabs as tab (tab.id)}
        {@const active = tab.id === activeId}
        {@const draggingThis = bmDrag?.active && bmDrag.id === tab.id}
        {@const dropBeforeThis = bmDrag?.active && bmDrag.dropBefore === tab.id}
        <button
          type="button"
          class="topbar-bookmark-item"
          data-bookmark-item={tab.id}
          data-active={active || undefined}
          data-dragging={draggingThis || undefined}
          data-drop-before={dropBeforeThis || undefined}
          data-zephium-tab-id={tab.id}
          data-zephium-tab-url={tab.url ?? ""}
          data-zephium-projection-revision={tab.projection_revision}
          title={tab.title || tab.url || m.untitled_tab()}
          onclick={() => openBookmarkInCurrent(tab)}
          onpointerdown={(e) => handleBookmarkPointerDown(e, tab.id)}
          onpointermove={handleBookmarkPointerMove}
          onpointerup={handleBookmarkPointerUp}
          onpointercancel={handleBookmarkPointerCancel}
          onmousedown={handleBookmarkMouseDown}
          onmouseup={(e) => handleBookmarkMouseUp(e, tab)}
          oncontextmenu={(e) => {
            e.preventDefault();
            bmDrag = null;
            tabs.openTabMenu(tab.id, e.clientX, e.clientY);
          }}
        >
          <FavIcon
            image={favicons.image(tab.icon)}
            tone={favicons.tone(tab.icon)}
            loading={tab.loading}
            size={14}
            lit
            fallback={Globe02Icon}
          />
          <span class="topbar-bookmark-label" data-zephium-tab-label>{bookmarkName(tab)}</span>
        </button>
      {/each}
    {/if}
  </div>
  {/if}
</header>

{#if drag.ghost !== null}
  {@const held = tabs.tabs().find((tab) => tab.id === drag.ghost?.id)}
  <DragMark
    x={drag.ghost.x}
    y={drag.ghost.y}
    image={favicons.image(held?.icon)}
    tone={favicons.tone(held?.icon)}
    title={held?.title}
  />
{:else if bmDrag?.active}
  {@const held = pinnedTabs.find((tab) => tab.id === bmDrag?.id)}
  <DragMark
    x={bmDrag.x}
    y={bmDrag.y}
    image={favicons.image(held?.icon)}
    tone={favicons.tone(held?.icon)}
    title=""
  />
{/if}

<style>
  .aether-topbar {
    display: flex;
    flex-direction: column;
    flex: none;
    height: 108px;
    gap: 3px;
    padding: 0 2px 3px;
    user-select: none;
  }

  .aether-topbar[data-collapsed] {
    height: 8px;
    padding: 0;
    gap: 0;
  }

  .topbar-hover-strip {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 8px;
    cursor: default;
  }

  .topbar-hover-pill {
    width: 48px;
    height: 3px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.22);
    transition: background-color 140ms ease, width 140ms ease;
  }

  .topbar-hover-strip:hover .topbar-hover-pill {
    width: 72px;
    background: rgba(56, 189, 248, 0.7);
  }

  .topbar-ext-popup {
    min-width: 240px;
    padding: 4px 0;
  }

  :global(.topbar-icon-btn[data-pinned]) {
    color: #38bdf8;
  }

  .topbar-update-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 26px;
    padding: 0 10px;
    border: 1px solid rgba(52, 211, 153, 0.45);
    border-radius: 999px;
    background: rgba(16, 185, 129, 0.18);
    color: #6ee7b7;
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: background-color 120ms ease, border-color 120ms ease;
  }

  .topbar-update-pill:hover:not(:disabled) {
    background: rgba(16, 185, 129, 0.3);
    border-color: rgba(52, 211, 153, 0.75);
    color: #ecfdf5;
  }

  .topbar-update-pill:disabled {
    opacity: 0.65;
    cursor: default;
  }

  .topbar-tabs-row {
    display: flex;
    align-items: center;
    height: 36px;
    gap: 4px;
    min-width: 0;
  }

  .aether-topbar[data-windows] .topbar-tabs-row {
    padding-inline-end: 138px;
  }

  .topbar-tab-strip {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    max-width: 100%;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .topbar-tab-strip::-webkit-scrollbar {
    display: none;
  }

  .topbar-tab[data-drop-before] {
    box-shadow: inset 2px 0 0 #38bdf8;
  }

  .topbar-tab-strip[data-drop-end]::after {
    content: "";
    flex: none;
    width: 2px;
    height: 18px;
    border-radius: 2px;
    background: #38bdf8;
  }

  .topbar-group-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    flex: none;
    height: 26px;
    padding: 0 9px;
    margin-inline-start: 2px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 8px;
    background: var(--group-bg, rgba(110, 145, 190, 0.22));
    color: var(--color-text);
    font-size: 12px;
    font-weight: 640;
    cursor: pointer;
    transition: background-color 120ms ease, opacity 120ms ease;
  }

  .topbar-group-chip:hover {
    background: var(--group-bg-hover, rgba(110, 145, 190, 0.3));
  }

  .topbar-group-label {
    max-width: 110px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .topbar-group-count {
    padding: 0 5px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
  }

  .topbar-tab {
    position: relative;
    display: flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    min-width: 96px;
    max-width: 215px;
    flex: 0 1 180px;
    padding: 0 8px 0 10px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--color-label-secondary);
    font-size: 12.5px;
    cursor: default;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      box-shadow 120ms ease;
  }

  .topbar-tab-grouped {
    background: var(--group-bg, rgba(110, 145, 190, 0.12));
    box-shadow: inset 0 -2px 0 rgba(255, 255, 255, 0.14);
  }

  .topbar-tab:hover {
    background: var(--row-hover, rgba(255, 255, 255, 0.08));
    color: var(--color-text);
  }

  .topbar-tab[data-active] {
    background: var(--row-active, rgba(255, 255, 255, 0.13));
    color: var(--color-text);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.14),
      0 2px 10px rgba(56, 189, 248, 0.12);
  }

  .topbar-tab[data-active]::after {
    content: "";
    position: absolute;
    inset-inline: 10px;
    bottom: 0;
    height: 2px;
    border-radius: 999px;
    background: linear-gradient(90deg, #38bdf8, #818cf8);
  }

  .topbar-tab-icon {
    display: grid;
    place-items: center;
    flex: none;
  }

  .topbar-tab-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .topbar-tab-audio {
    display: grid;
    place-items: center;
    flex: none;
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--color-accent, #38bdf8);
    cursor: pointer;
  }

  .topbar-tab-audio:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .topbar-tab-close {
    display: grid;
    place-items: center;
    flex: none;
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--color-muted);
    opacity: 0;
    cursor: default;
    transition:
      opacity 100ms ease,
      background-color 100ms ease,
      color 100ms ease;
  }

  .topbar-tab:hover .topbar-tab-close,
  .topbar-tab[data-active] .topbar-tab-close {
    opacity: 1;
  }

  .topbar-tab-close:hover {
    background: rgba(255, 255, 255, 0.15);
    color: var(--color-text);
  }

  .topbar-newtab {
    display: grid;
    place-items: center;
    flex: none;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--color-label-secondary);
    cursor: default;
    transition: background-color 120ms ease, color 120ms ease;
  }

  .topbar-newtab:hover {
    background: var(--row-hover, rgba(255, 255, 255, 0.08));
    color: var(--color-text);
  }

  .topbar-drag-spacer {
    flex: 1;
    min-width: 24px;
    height: 100%;
  }

  .topbar-nav-row {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
  }

  .topbar-nav-group {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: none;
  }

  .topbar-icon-btn {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--color-label-secondary);
    cursor: default;
    transition: background-color 120ms ease, color 120ms ease;
  }

  .topbar-icon-btn:hover:not(:disabled) {
    background: var(--row-hover, rgba(255, 255, 255, 0.08));
    color: var(--color-text);
  }

  .topbar-icon-btn:disabled {
    opacity: 0.35;
  }

  .topbar-address {
    flex: 1;
    min-width: 0;
  }

  .topbar-address :global(.field) {
    margin: 0 !important;
  }

  /* Row 3: Classic Bookmarks Bar */
  .topbar-bookmarks-row {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 28px;
    padding-inline: 4px;
    border-radius: 8px;
    overflow-x: auto;
    scrollbar-width: none;
    transition: box-shadow 140ms ease, background-color 140ms ease;
  }

  .topbar-bookmarks-row[data-over="true"] {
    background: rgba(56, 189, 248, 0.1);
    box-shadow: inset 0 0 0 1.5px rgba(56, 189, 248, 0.5);
  }

  .topbar-bookmarks-row::-webkit-scrollbar {
    display: none;
  }

  .topbar-bookmarks-empty {
    font-size: 11.5px;
    color: var(--color-muted);
    padding-inline: 6px;
    opacity: 0.7;
  }

  .topbar-bookmark-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: none;
    height: 24px;
    max-width: 150px;
    padding: 0 8px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--color-label-secondary);
    font-size: 12px;
    cursor: default;
    transition: background-color 110ms ease, color 110ms ease;
  }

  .topbar-bookmark-item:hover {
    background: var(--row-hover, rgba(255, 255, 255, 0.08));
    color: var(--color-text);
  }

  .topbar-bookmark-item[data-active] {
    background: var(--row-active, rgba(255, 255, 255, 0.11));
    color: var(--color-text);
  }

  .topbar-bookmark-item[data-dragging] {
    opacity: 0.35;
  }

  .topbar-bookmark-item[data-drop-before] {
    box-shadow: inset 2px 0 0 #38bdf8;
  }

  .topbar-bookmarks-row[data-drop-end]::after {
    content: "";
    flex: none;
    width: 2px;
    height: 16px;
    border-radius: 2px;
    background: #38bdf8;
  }

  .topbar-bookmark-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>

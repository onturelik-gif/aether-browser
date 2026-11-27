<script lang="ts">
  import { Cancel01Icon, PlusSignIcon } from "@hugeicons/core-free-icons";
  import { tabs } from "$domain/tabs";
  import { commands } from "$shared/ipc/bindings";
  import Icon from "$shared/ui/Icon";

  interface Shortcut {
    id: string;
    name: string;
    url: string;
  }

  const STORAGE_KEY = "aether.newtab.shortcuts.v1";

  const DEFAULTS: Shortcut[] = [
    { id: "yt", name: "YouTube", url: "https://www.youtube.com" },
    { id: "gg", name: "Google", url: "https://www.google.com" },
    { id: "x", name: "X", url: "https://x.com" },
    { id: "ig", name: "Instagram", url: "https://www.instagram.com" },
    { id: "gh", name: "GitHub", url: "https://github.com" },
    { id: "cg", name: "ChatGPT", url: "https://chatgpt.com" },
    { id: "gm", name: "Gemini", url: "https://gemini.google.com" },
    { id: "wa", name: "WhatsApp", url: "https://web.whatsapp.com" },
  ];

  function loadShortcuts(): Shortcut[] {
    if (typeof localStorage === "undefined") return DEFAULTS;
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) return DEFAULTS;
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed)) return parsed;
    } catch {}
    return DEFAULTS;
  }

  let items = $state<Shortcut[]>(loadShortcuts());
  let adding = $state(false);
  let draftName = $state("");
  let draftUrl = $state("");
  let dragId = $state<string | null>(null);
  let overId = $state<string | null>(null);

  function save(next: Shortcut[]) {
    items = next;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {}
  }

  function normalizeUrl(input: string): string | null {
    const trimmed = input.trim();
    if (!trimmed) return null;
    const withScheme = /^https?:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`;
    try {
      return new URL(withScheme).toString();
    } catch {
      return null;
    }
  }

  function hostOf(url: string): string {
    try {
      return new URL(url).hostname;
    } catch {
      return "";
    }
  }

  function faviconUrl(url: string): string {
    const host = hostOf(url);
    return host ? `https://www.google.com/s2/favicons?domain=${encodeURIComponent(host)}&sz=64` : "";
  }

  function openShortcut(item: Shortcut) {
    const active = tabs.activeId();
    if (active) {
      tabs.navigate(active, item.url);
    } else {
      void commands.runCommand(`bookmark.openTab=${item.url}`).catch(() => {});
    }
  }

  function handleMouseUp(e: MouseEvent, item: Shortcut) {
    if (e.button === 1) {
      e.preventDefault();
      void commands.runCommand(`bookmark.newTab=${item.url}`).catch(() => {});
    }
  }

  function removeItem(e: MouseEvent, id: string) {
    e.stopPropagation();
    e.preventDefault();
    save(items.filter((i) => i.id !== id));
  }

  function submitAdd(e: Event) {
    e.preventDefault();
    const url = normalizeUrl(draftUrl);
    if (!url) return;
    const host = hostOf(url).replace(/^www\./, "");
    const name = draftName.trim() || host.split(".")[0] || host;
    const id = `sc_${Date.now().toString(36)}`;
    save([...items, { id, name, url }]);
    draftName = "";
    draftUrl = "";
    adding = false;
  }

  function onDragStart(id: string) {
    dragId = id;
  }

  function onDragOver(e: DragEvent, targetId: string) {
    if (!dragId || dragId === targetId) return;
    e.preventDefault();
    overId = targetId;
  }

  function onDrop(e: DragEvent, targetId: string) {
    e.preventDefault();
    if (!dragId || dragId === targetId) {
      dragId = null;
      overId = null;
      return;
    }
    const from = items.findIndex((i) => i.id === dragId);
    const to = items.findIndex((i) => i.id === targetId);
    if (from !== -1 && to !== -1) {
      const next = [...items];
      const [moved] = next.splice(from, 1);
      next.splice(to, 0, moved);
      save(next);
    }
    dragId = null;
    overId = null;
  }

  function onDragEnd() {
    dragId = null;
    overId = null;
  }
</script>

<div class="ntp-shortcuts-wrap" aria-label="Hızlı Kısayollar">
  <div class="ntp-shortcuts-grid">
    {#each items as item (item.id)}
      <div
        class="ntp-tile"
        role="button"
        tabindex="0"
        draggable="true"
        data-dragging={dragId === item.id || undefined}
        data-over={overId === item.id || undefined}
        title={item.url}
        onclick={() => openShortcut(item)}
        onmousedown={(e) => {
          if (e.button === 1) e.preventDefault();
        }}
        onmouseup={(e) => handleMouseUp(e, item)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            openShortcut(item);
          }
        }}
        ondragstart={() => onDragStart(item.id)}
        ondragover={(e) => onDragOver(e, item.id)}
        ondrop={(e) => onDrop(e, item.id)}
        ondragend={onDragEnd}
      >
        <button
          type="button"
          class="ntp-tile-remove"
          title="Kısayolu kaldır"
          aria-label="Kısayolu kaldır"
          onclick={(e) => removeItem(e, item.id)}
        >
          <Icon icon={Cancel01Icon} size={10} strokeWidth={2.2} />
        </button>
        <div class="ntp-tile-icon">
          <img
            src={faviconUrl(item.url)}
            alt=""
            width="24"
            height="24"
            loading="lazy"
            onerror={(e) => {
              (e.currentTarget as HTMLImageElement).style.display = "none";
            }}
          />
          <span class="ntp-tile-fallback">{item.name.charAt(0).toUpperCase()}</span>
        </div>
        <span class="ntp-tile-label">{item.name}</span>
      </div>
    {/each}

    <button
      type="button"
      class="ntp-tile ntp-tile-add"
      onclick={() => {
        adding = true;
      }}
    >
      <div class="ntp-tile-icon ntp-tile-icon-add">
        <Icon icon={PlusSignIcon} size={20} strokeWidth={1.8} />
      </div>
      <span class="ntp-tile-label">Kısayol ekle</span>
    </button>
  </div>

  {#if adding}
    <div
      class="ntp-modal-backdrop"
      role="presentation"
      onclick={() => {
        adding = false;
      }}
    >
      <form
        class="ntp-modal"
        onsubmit={submitAdd}
        onclick={(e) => e.stopPropagation()}
      >
        <h3 class="ntp-modal-title">Kısayol Ekle</h3>
        <label class="ntp-field">
          <span>Ad</span>
          <input
            type="text"
            bind:value={draftName}
            placeholder="Örn: YouTube"
            autocomplete="off"
          />
        </label>
        <label class="ntp-field">
          <span>URL</span>
          <input
            type="text"
            bind:value={draftUrl}
            placeholder="https://youtube.com"
            required
            autocomplete="off"
          />
        </label>
        <div class="ntp-modal-actions">
          <button
            type="button"
            class="ntp-btn ntp-btn-ghost"
            onclick={() => {
              adding = false;
            }}
          >
            İptal
          </button>
          <button type="submit" class="ntp-btn ntp-btn-primary">Ekle</button>
        </div>
      </form>
    </div>
  {/if}
</div>

<style>
  .ntp-shortcuts-wrap {
    position: absolute;
    left: 50%;
    top: 56%;
    transform: translate(-50%, -20%);
    z-index: 4;
    width: min(760px, calc(100% - 64px));
    display: flex;
    justify-content: center;
    pointer-events: auto;
  }

  .ntp-shortcuts-grid {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 14px 16px;
    max-width: 740px;
  }

  .ntp-tile {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
    width: 74px;
    padding: 8px 4px 6px;
    border: 0;
    border-radius: 14px;
    background: transparent;
    color: rgba(248, 250, 252, 0.94);
    cursor: pointer;
    user-select: none;
    transition:
      transform 140ms cubic-bezier(0.22, 1, 0.36, 1),
      background-color 140ms ease,
      opacity 120ms ease;
  }

  .ntp-tile:hover {
    background: rgba(15, 23, 42, 0.36);
    transform: translateY(-2px);
  }

  .ntp-tile[data-dragging] {
    opacity: 0.35;
  }

  .ntp-tile[data-over] {
    box-shadow: inset 0 0 0 1.5px rgba(56, 189, 248, 0.7);
    background: rgba(56, 189, 248, 0.12);
  }

  .ntp-tile-icon {
    position: relative;
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    border-radius: 15px;
    background: rgba(15, 20, 34, 0.62);
    border: 1px solid rgba(255, 255, 255, 0.14);
    box-shadow:
      0 8px 22px rgba(0, 0, 0, 0.32),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(18px);
    -webkit-backdrop-filter: blur(18px);
    transition:
      border-color 140ms ease,
      box-shadow 140ms ease,
      background-color 140ms ease;
  }

  .ntp-tile:hover .ntp-tile-icon {
    border-color: rgba(56, 189, 248, 0.45);
    background: rgba(18, 25, 42, 0.76);
    box-shadow:
      0 10px 26px rgba(0, 0, 0, 0.42),
      0 0 0 1px rgba(56, 189, 248, 0.22);
  }

  .ntp-tile-icon img {
    position: relative;
    z-index: 2;
    border-radius: 6px;
    object-fit: contain;
  }

  .ntp-tile-fallback {
    position: absolute;
    z-index: 1;
    font-size: 16px;
    font-weight: 600;
    color: rgba(226, 232, 240, 0.78);
  }

  .ntp-tile-icon-add {
    background: rgba(15, 20, 34, 0.42);
    border-style: dashed;
    border-color: rgba(255, 255, 255, 0.22);
    color: rgba(226, 232, 240, 0.85);
  }

  .ntp-tile-label {
    max-width: 72px;
    font-size: 11.5px;
    font-weight: 500;
    line-height: 1.2;
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-shadow: 0 1px 6px rgba(0, 0, 0, 0.65);
  }

  .ntp-tile-remove {
    position: absolute;
    top: 3px;
    right: 6px;
    z-index: 3;
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    padding: 0;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 999px;
    background: rgba(15, 20, 34, 0.9);
    color: rgba(241, 245, 249, 0.85);
    opacity: 0;
    pointer-events: none;
    cursor: pointer;
    transition: opacity 120ms ease, background-color 120ms ease;
  }

  .ntp-tile:hover .ntp-tile-remove {
    opacity: 1;
    pointer-events: auto;
  }

  .ntp-tile-remove:hover {
    background: rgba(239, 68, 68, 0.85);
    color: #fff;
  }

  .ntp-modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    background: rgba(5, 8, 16, 0.52);
    backdrop-filter: blur(8px);
  }

  .ntp-modal {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 320px;
    padding: 18px;
    border-radius: 16px;
    background: rgba(15, 20, 32, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.14);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.55);
    color: #f1f5f9;
  }

  .ntp-modal-title {
    margin: 0;
    font-size: 14.5px;
    font-weight: 600;
  }

  .ntp-field {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 12px;
    color: #cbd5e1;
  }

  .ntp-field input {
    height: 34px;
    padding: 0 10px;
    border-radius: 9px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.06);
    color: #f8fafc;
    font-size: 12.5px;
    outline: none;
  }

  .ntp-field input:focus {
    border-color: #38bdf8;
  }

  .ntp-modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  .ntp-btn {
    height: 32px;
    padding: 0 14px;
    border-radius: 9px;
    font-size: 12.5px;
    font-weight: 550;
    cursor: pointer;
  }

  .ntp-btn-ghost {
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: transparent;
    color: #e2e8f0;
  }

  .ntp-btn-primary {
    border: 0;
    background: linear-gradient(135deg, #0ea5e9, #6366f1);
    color: #fff;
  }
</style>

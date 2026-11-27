<script lang="ts">
  import {
    Folder01Icon,
    ArrowRight01Icon,
    Edit02Icon,
    Globe02Icon,
  } from "@hugeicons/core-free-icons";
  import Icon from "$shared/ui/Icon";
  import FavIcon from "$shared/ui/FavIcon";
  import { favicons } from "$domain/favicons";
  import type { AutoGroupMeta } from "../lib/sidebar-model";

  let {
    name,
    depth,
    expanded = true,
    ontoggle,
    motionKey,
    groupMeta,
    class: className = "",
  }: {
    motionKey?: string;
    name: string;
    depth: number;
    expanded?: boolean;
    ontoggle?: () => void;
    groupMeta?: AutoGroupMeta;
    class?: string;
  } = $props();

  const STORAGE_PREFIX = "aether.group.name:";

  function readSavedName(host: string | undefined): string {
    if (!host || typeof localStorage === "undefined") return "";
    try {
      return localStorage.getItem(STORAGE_PREFIX + host) ?? "";
    } catch {
      return "";
    }
  }

  let customName = $state("");
  let editing = $state(false);
  let draft = $state("");
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    customName = readSavedName(motionKey ?? groupMeta?.host);
  });

  let displayName = $derived(customName.trim() || name);

  let brand = $derived.by(() => {
    const host = groupMeta?.host ?? "";
    if (!host) return null;
    if (host === "x.com" || host === "twitter.com" || host.endsWith(".x.com")) return "x";
    if (host === "google.com" || host.startsWith("google.")) return "google";
    if (host === "youtube.com" || host === "youtu.be" || host.endsWith(".youtube.com")) return "youtube";
    if (host === "instagram.com" || host.endsWith(".instagram.com")) return "instagram";
    if (host === "facebook.com" || host === "fb.com" || host.endsWith(".facebook.com")) return "facebook";
    if (host === "github.com" || host.endsWith(".github.com")) return "github";
    if (host === "reddit.com" || host.endsWith(".reddit.com")) return "reddit";
    if (host === "spotify.com" || host.endsWith(".spotify.com")) return "spotify";
    if (host === "discord.com" || host.endsWith(".discord.com")) return "discord";
    if (host === "twitch.tv" || host.endsWith(".twitch.tv")) return "twitch";
    return null;
  });

  function startRename(event?: Event) {
    event?.stopPropagation();
    if (!groupMeta) return;
    draft = displayName;
    editing = true;
    queueMicrotask(() => {
      inputEl?.focus();
      inputEl?.select();
    });
  }

  function commitRename() {
    if (!editing || !groupMeta) return;
    const cleaned = draft.trim();
    editing = false;
    const storageKey = motionKey ?? groupMeta.host;
    customName = cleaned === name ? "" : cleaned;
    try {
      if (customName) localStorage.setItem(STORAGE_PREFIX + storageKey, customName);
      else localStorage.removeItem(STORAGE_PREFIX + storageKey);
    } catch {}
  }

  function cancelRename() {
    editing = false;
  }
</script>

<li
  class={[
    className,
    groupMeta
      ? `aether-group-header aether-tint-${groupMeta.colorIndex}${groupMeta.isSubgroup ? " aether-subgroup" : ""}`
      : "",
  ].join(" ")}
  style:--tree-level={groupMeta?.treeLevel ?? 0}
  data-motion-key={motionKey}
  data-group-expanded={expanded || undefined}
>
  <div
    class="folder-row aether-folder-bar"
    role="button"
    tabindex="0"
    aria-expanded={expanded}
    onclick={() => {
      if (!editing) ontoggle?.();
    }}
    onkeydown={(e) => {
      if (!editing && (e.key === "Enter" || e.key === " ")) {
        e.preventDefault();
        ontoggle?.();
      }
    }}
  >
    <span class="group-logo" aria-hidden="true">
      {#if brand === "x"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <path fill="currentColor" d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231 5.45-6.231Zm-1.161 17.52h1.833L7.084 4.126H5.117L17.083 19.77Z"/>
        </svg>
      {:else if brand === "google"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <path fill="#4285F4" d="M23.49 12.27c0-.79-.07-1.54-.2-2.27H12v4.3h6.45a5.52 5.52 0 0 1-2.39 3.62v3h3.87c2.26-2.08 3.56-5.15 3.56-8.65Z"/>
          <path fill="#34A853" d="M12 24c3.24 0 5.95-1.07 7.93-2.91l-3.87-3a7.18 7.18 0 0 1-10.68-3.77H1.39v3.09A12 12 0 0 0 12 24Z"/>
          <path fill="#FBBC05" d="M5.38 14.32a7.2 7.2 0 0 1 0-4.64V6.59H1.39a12 12 0 0 0 0 10.82l3.99-3.09Z"/>
          <path fill="#EA4335" d="M12 4.75c1.76 0 3.34.61 4.58 1.8l3.44-3.44C17.95 1.19 15.24 0 12 0A12 12 0 0 0 1.39 6.59l3.99 3.09A7.17 7.17 0 0 1 12 4.75Z"/>
        </svg>
      {:else if brand === "youtube"}
        <svg width="15" height="15" viewBox="0 0 24 24">
          <path fill="#FF0033" d="M23.5 6.19a3.02 3.02 0 0 0-2.12-2.14C19.5 3.54 12 3.54 12 3.54s-7.5 0-9.38.51A3.02 3.02 0 0 0 .5 6.19 31.6 31.6 0 0 0 0 12a31.6 31.6 0 0 0 .5 5.81 3.02 3.02 0 0 0 2.12 2.14c1.88.51 9.38.51 9.38.51s7.5 0 9.38-.51a3.02 3.02 0 0 0 2.12-2.14A31.6 31.6 0 0 0 24 12a31.6 31.6 0 0 0-.5-5.81Z"/>
          <path fill="#FFFFFF" d="M9.6 15.6V8.4L15.85 12 9.6 15.6Z"/>
        </svg>
      {:else if brand === "instagram"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <rect width="24" height="24" rx="6" fill="#E1306C"/>
          <rect x="5" y="5" width="14" height="14" rx="4" fill="none" stroke="#fff" stroke-width="2"/>
          <circle cx="12" cy="12" r="3.4" fill="none" stroke="#fff" stroke-width="2"/>
        </svg>
      {:else if brand === "facebook"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="12" fill="#1877F2"/>
          <path fill="#fff" d="M16.67 15.47 17.2 12h-3.33V9.75c0-.95.46-1.87 1.95-1.87h1.51V4.93s-1.37-.23-2.68-.23c-2.73 0-4.52 1.66-4.52 4.66V12H7.08v3.47h3.05v8.39a12.1 12.1 0 0 0 3.74 0v-8.39h2.8Z"/>
        </svg>
      {:else if brand === "github"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <path fill="currentColor" fill-rule="evenodd" clip-rule="evenodd" d="M12 1.5C6.2 1.5 1.5 6.2 1.5 12c0 4.64 3.01 8.58 7.18 9.97.53.1.72-.23.72-.51v-1.78c-2.92.64-3.54-1.41-3.54-1.41-.48-1.22-1.17-1.54-1.17-1.54-.95-.65.07-.64.07-.64 1.05.07 1.61 1.08 1.61 1.08.94 1.61 2.46 1.14 3.06.87.09-.68.37-1.14.67-1.41-2.33-.27-4.79-1.17-4.79-5.19 0-1.15.41-2.08 1.08-2.82-.11-.27-.47-1.33.1-2.78 0 0 .88-.28 2.89 1.08a10.1 10.1 0 0 1 5.26 0c2.01-1.36 2.89-1.08 2.89-1.08.57 1.45.21 2.51.1 2.78.67.74 1.08 1.67 1.08 2.82 0 4.03-2.46 4.92-4.8 5.18.38.33.71.97.71 1.95v2.89c0 .28.19.61.73.51C19.49 20.58 22.5 16.64 22.5 12c0-5.8-4.7-10.5-10.5-10.5Z"/>
        </svg>
      {:else if brand === "reddit"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="12" fill="#FF4500"/>
          <circle cx="12" cy="13.5" r="5.5" fill="#fff"/>
        </svg>
      {:else if brand === "spotify"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="12" fill="#1DB954"/>
        </svg>
      {:else if brand === "discord"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <rect width="24" height="24" rx="6" fill="#5865F2"/>
        </svg>
      {:else if brand === "twitch"}
        <svg width="14" height="14" viewBox="0 0 24 24">
          <path fill="#9146FF" d="M4.3 2 2 5.8v14.4h5.1V23l3.1-2.8h4.4l6.4-6.4V2H4.3Z"/>
        </svg>
      {:else if groupMeta}
        <FavIcon
          image={favicons.image(groupMeta.icon)}
          tone={favicons.tone(groupMeta.icon)}
          size={15}
          lit
          fallback={Globe02Icon}
        />
      {:else}
        <Icon icon={Folder01Icon} size={15} />
      {/if}
    </span>

    {#if editing}
      <input
        bind:this={inputEl}
        class="group-rename-input"
        type="text"
        bind:value={draft}
        aria-label="Grup adını düzenle"
        onclick={(e) => e.stopPropagation()}
        onblur={commitRename}
        onkeydown={(e) => {
          e.stopPropagation();
          if (e.key === "Enter") commitRename();
          else if (e.key === "Escape") cancelRename();
        }}
      />
    {:else}
      <span
        class="group-name"
        role="presentation"
        ondblclick={startRename}
        title={groupMeta ? "Adı değiştirmek için çift tıkla" : displayName}
      >{displayName}</span>
    {/if}

    <button
      type="button"
      class="folder-chevron group-chevron-btn"
      data-expanded={expanded}
      aria-label={expanded ? "Grubu gizle" : "Grubu göster"}
      title={expanded ? "Grubu gizle" : "Grubu göster"}
      onclick={(e) => {
        e.stopPropagation();
        ontoggle?.();
      }}
    >
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path
          d="M2.5 4.25L6 7.75L9.5 4.25"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <span class="group-spacer"></span>

    {#if groupMeta && !editing}
      <button
        type="button"
        class="group-edit-btn"
        title="Grup adını değiştir"
        aria-label="Grup adını değiştir"
        onclick={startRename}
      >
        <Icon icon={Edit02Icon} size={12} />
      </button>
      {#if !expanded}
        <span class="group-count">{groupMeta.count}</span>
      {/if}
    {/if}
  </div>
</li>

<style>
  .aether-folder-bar {
    display: flex !important;
    align-items: center !important;
    justify-content: flex-start !important;
    gap: 7px !important;
    width: 100%;
    min-height: 36px;
    padding: 5px 8px 5px 10px !important;
    cursor: pointer;
    user-select: none;
  }

  .aether-folder-bar > .group-logo {
    display: grid;
    place-items: center;
    flex: none !important;
    width: 18px;
    height: 18px;
  }

  .aether-folder-bar > .group-name {
    flex: 0 1 auto !important;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14.5px;
    font-weight: 680;
    color: var(--color-text);
    letter-spacing: -0.012em;
  }

  .aether-folder-bar > .group-spacer {
    flex: 1 1 auto !important;
  }

  .group-rename-input {
    flex: 1;
    min-width: 0;
    height: 22px;
    padding: 0 6px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.22);
    background: rgba(0, 0, 0, 0.28);
    color: var(--color-text);
    font-size: 12.5px;
    outline: none;
  }

  .group-edit-btn {
    display: grid;
    place-items: center;
    flex: none;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: var(--color-muted);
    opacity: 0;
    cursor: pointer;
    transition: opacity 110ms ease, background-color 110ms ease, color 110ms ease;
  }

  .aether-folder-bar:hover .group-edit-btn {
    opacity: 0.85;
  }

  .group-edit-btn:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.12);
    color: var(--color-text);
  }

  .group-count {
    flex: none;
    min-width: 18px;
    padding: 1px 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.09);
    color: var(--color-label-secondary);
    font-size: 10.5px;
    font-weight: 600;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .group-chevron-btn {
    display: grid;
    place-items: center;
    flex: none;
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--color-label-secondary);
    cursor: pointer;
    transform: rotate(-90deg);
    transition: transform 160ms ease, color 110ms ease;
  }

  .group-chevron-btn[data-expanded="true"] {
    transform: rotate(0deg);
  }

  .group-chevron-btn:hover {
    color: var(--color-text);
  }
</style>

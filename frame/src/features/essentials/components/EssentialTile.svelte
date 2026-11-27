<script lang="ts">
  import * as m from "$shared/i18n/messages";
  import { Cancel01Icon, Globe02Icon } from "@hugeicons/core-free-icons";
  import type { TabView } from "$shared/ipc/bindings";
  import FavIcon from "$shared/ui/FavIcon";
  import Icon from "$shared/ui/Icon";
  import { favicons } from "$domain/favicons";
  import CaptureControl from "$shared/ui/CaptureControl";
  import AudioControl from "$shared/ui/AudioControl";
  import { tabs } from "$domain/tabs";

  let {
    tab,
    active,
    splitCandidate,
    class: className = "",
    onSelect,
    onContextMenu,
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onPointerCancel,
  }: {
    tab: TabView;
    active: boolean;
    splitCandidate: boolean;
    class?: string;
    onSelect: (id: string) => void;
    onContextMenu: (event: MouseEvent, tab: TabView) => void;
    onPointerDown: (event: PointerEvent, tab: TabView) => void;
    onPointerMove: (event: PointerEvent) => void;
    onPointerUp: (event: PointerEvent) => void;
    onPointerCancel: (event: PointerEvent) => void;
  } = $props();

  // An Essential with no page behind it only keeps its site; one that has a
  // page can be closed without being let go.
  let open = $derived(tab.availability?.state !== "sleeping");
  let name = $derived(tab.title || m.untitled_tab());

  let brand = $derived.by(() => {
    const raw = tab.url ?? "";
    if (!raw) return null;
    try {
      const host = new URL(raw).hostname.toLowerCase().replace(/^www\./, "");
      if (host === "google.com" || host.startsWith("google.")) return "google";
      if (host === "youtube.com" || host === "youtu.be" || host.endsWith(".youtube.com")) return "youtube";
      if (host === "instagram.com" || host.endsWith(".instagram.com")) return "instagram";
      if (host === "facebook.com" || host === "fb.com" || host.endsWith(".facebook.com")) return "facebook";
      if (host === "x.com" || host === "twitter.com" || host.endsWith(".x.com")) return "x";
      if (host === "github.com" || host.endsWith(".github.com")) return "github";
      if (host === "reddit.com" || host.endsWith(".reddit.com")) return "reddit";
      if (host === "spotify.com" || host.endsWith(".spotify.com")) return "spotify";
      if (host === "discord.com" || host.endsWith(".discord.com")) return "discord";
      if (host === "twitch.tv" || host.endsWith(".twitch.tv")) return "twitch";
      if (host === "chatgpt.com" || host === "chat.openai.com") return "chatgpt";
      return null;
    } catch {
      return null;
    }
  });

  function closePage(event: MouseEvent) {
    event.stopPropagation();
    tabs.close(tab.id);
  }
</script>

<!--
  Essentials share the row's width evenly, so one kept site fills the row and
  four divide it. The tile is wide rather than square: a wide plate leaves the
  mark plenty of quiet ground around it, which is what keeps a handful of
  unrelated brand colours from reading as a block of noise.
-->
<!--
  The presentation sentinels belong to the row, not to the tile's styling.
  Native resolves the tab by id and verifies its url and revision through
  this element before it will reveal page content, so a tile without them
  activates nothing at all.
-->
<li
  data-motion-key={`tab:${tab.id}`}
  data-zephium-tab-id={tab.id}
  data-zephium-tab-url={tab.url ?? ""}
  data-zephium-projection-revision={tab.projection_revision}
  data-audio-host
  class={["essential", className]}
  data-open={open}
  data-active={active}
  data-split-candidate={splitCandidate}
>
  <button
    type="button"
    data-plate
    aria-current={active ? "page" : undefined}
    aria-label={name}
    title={name}
    oncontextmenu={(event) => onContextMenu(event, tab)}
    onpointerdown={(event) => onPointerDown(event, tab)}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerCancel}
    onclick={() => onSelect(tab.id)}
  >
    {#if brand === "google"}
      <svg class="brand-svg" width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="#4285F4" d="M23.49 12.27c0-.79-.07-1.54-.2-2.27H12v4.3h6.45a5.52 5.52 0 0 1-2.39 3.62v3h3.87c2.26-2.08 3.56-5.15 3.56-8.65Z"/>
        <path fill="#34A853" d="M12 24c3.24 0 5.95-1.07 7.93-2.91l-3.87-3a7.18 7.18 0 0 1-10.68-3.77H1.39v3.09A12 12 0 0 0 12 24Z"/>
        <path fill="#FBBC05" d="M5.38 14.32a7.2 7.2 0 0 1 0-4.64V6.59H1.39a12 12 0 0 0 0 10.82l3.99-3.09Z"/>
        <path fill="#EA4335" d="M12 4.75c1.76 0 3.34.61 4.58 1.8l3.44-3.44C17.95 1.19 15.24 0 12 0A12 12 0 0 0 1.39 6.59l3.99 3.09A7.17 7.17 0 0 1 12 4.75Z"/>
      </svg>
    {:else if brand === "youtube"}
      <svg class="brand-svg" width="23" height="23" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="#FF0033" d="M23.5 6.19a3.02 3.02 0 0 0-2.12-2.14C19.5 3.54 12 3.54 12 3.54s-7.5 0-9.38.51A3.02 3.02 0 0 0 .5 6.19 31.6 31.6 0 0 0 0 12a31.6 31.6 0 0 0 .5 5.81 3.02 3.02 0 0 0 2.12 2.14c1.88.51 9.38.51 9.38.51s7.5 0 9.38-.51a3.02 3.02 0 0 0 2.12-2.14A31.6 31.6 0 0 0 24 12a31.6 31.6 0 0 0-.5-5.81Z"/>
        <path fill="#FFFFFF" d="M9.6 15.6V8.4L15.85 12 9.6 15.6Z"/>
      </svg>
    {:else if brand === "instagram"}
      <svg class="brand-svg" width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
        <defs>
          <radialGradient id="ig-g" cx="30%" cy="107%" r="150%">
            <stop offset="0%" stop-color="#fdf497"/>
            <stop offset="5%" stop-color="#fdf497"/>
            <stop offset="45%" stop-color="#fd5949"/>
            <stop offset="60%" stop-color="#d6249f"/>
            <stop offset="90%" stop-color="#285AEB"/>
          </radialGradient>
        </defs>
        <rect width="24" height="24" rx="6" fill="url(#ig-g)"/>
        <rect x="4.5" y="4.5" width="15" height="15" rx="4.2" fill="none" stroke="#fff" stroke-width="1.8"/>
        <circle cx="12" cy="12" r="3.6" fill="none" stroke="#fff" stroke-width="1.8"/>
        <circle cx="16.4" cy="7.6" r="1.15" fill="#fff"/>
      </svg>
    {:else if brand === "facebook"}
      <svg class="brand-svg" width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="12" fill="#1877F2"/>
        <path fill="#FFFFFF" d="M16.67 15.47 17.2 12h-3.33V9.75c0-.95.46-1.87 1.95-1.87h1.51V4.93s-1.37-.23-2.68-.23c-2.73 0-4.52 1.66-4.52 4.66V12H7.08v3.47h3.05v8.39a12.1 12.1 0 0 0 3.74 0v-8.39h2.8Z"/>
      </svg>
    {:else if brand === "x"}
      <svg class="brand-svg" width="19" height="19" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="currentColor" d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231 5.45-6.231Zm-1.161 17.52h1.833L7.084 4.126H5.117L17.083 19.77Z"/>
      </svg>
    {:else if brand === "github"}
      <svg class="brand-svg" width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="currentColor" fill-rule="evenodd" clip-rule="evenodd" d="M12 1.5C6.2 1.5 1.5 6.2 1.5 12c0 4.64 3.01 8.58 7.18 9.97.53.1.72-.23.72-.51v-1.78c-2.92.64-3.54-1.41-3.54-1.41-.48-1.22-1.17-1.54-1.17-1.54-.95-.65.07-.64.07-.64 1.05.07 1.61 1.08 1.61 1.08.94 1.61 2.46 1.14 3.06.87.09-.68.37-1.14.67-1.41-2.33-.27-4.79-1.17-4.79-5.19 0-1.15.41-2.08 1.08-2.82-.11-.27-.47-1.33.1-2.78 0 0 .88-.28 2.89 1.08a10.1 10.1 0 0 1 5.26 0c2.01-1.36 2.89-1.08 2.89-1.08.57 1.45.21 2.51.1 2.78.67.74 1.08 1.67 1.08 2.82 0 4.03-2.46 4.92-4.8 5.18.38.33.71.97.71 1.95v2.89c0 .28.19.61.73.51C19.49 20.58 22.5 16.64 22.5 12c0-5.8-4.7-10.5-10.5-10.5Z"/>
      </svg>
    {:else if brand === "reddit"}
      <svg class="brand-svg" width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="12" fill="#FF4500"/>
        <circle cx="12" cy="13.5" r="5.5" fill="#fff"/>
        <circle cx="9.8" cy="12.8" r="1" fill="#FF4500"/>
        <circle cx="14.2" cy="12.8" r="1" fill="#FF4500"/>
        <path d="M9.5 15.5c.7.7 1.6 1 2.5 1s1.8-.3 2.5-1" stroke="#FF4500" stroke-width="1.1" stroke-linecap="round" fill="none"/>
        <circle cx="17.2" cy="7.2" r="1.3" fill="#fff"/>
        <path d="M12 8l1.2-3.2 2.8.8" stroke="#fff" stroke-width="1.2" stroke-linecap="round" fill="none"/>
      </svg>
    {:else if brand === "spotify"}
      <svg class="brand-svg" width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="12" fill="#1DB954"/>
        <path d="M6.8 9.6c3.4-1 7.6-.8 10.6.9M7.6 12.8c2.8-.8 6.2-.6 8.8.8M8.4 15.8c2.2-.6 4.9-.4 7 .7" stroke="#111" stroke-width="1.7" stroke-linecap="round" fill="none"/>
      </svg>
    {:else if brand === "discord"}
      <svg class="brand-svg" width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
        <rect width="24" height="24" rx="6" fill="#5865F2"/>
        <path fill="#fff" d="M16.9 7.6a11.3 11.3 0 0 0-2.8-.9l-.4.8a10.4 10.4 0 0 0-3.4 0l-.4-.8a11.3 11.3 0 0 0-2.8.9C5.3 10.3 4.8 13 5 15.6a11.4 11.4 0 0 0 3.5 1.8l.7-1.1a7.4 7.4 0 0 1-1.1-.5l.3-.2a8.1 8.1 0 0 0 7.2 0l.3.2c-.4.2-.7.4-1.1.5l.7 1.1a11.4 11.4 0 0 0 3.5-1.8c.3-3-.5-5.7-2.1-8ZM9.6 14c-.7 0-1.2-.6-1.2-1.4 0-.7.5-1.4 1.2-1.4.7 0 1.2.6 1.2 1.4 0 .8-.5 1.4-1.2 1.4Zm4.8 0c-.7 0-1.2-.6-1.2-1.4 0-.7.5-1.4 1.2-1.4.7 0 1.2.6 1.2 1.4 0 .8-.5 1.4-1.2 1.4Z"/>
      </svg>
    {:else if brand === "twitch"}
      <svg class="brand-svg" width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="#9146FF" d="M4.3 2 2 5.8v14.4h5.1V23l3.1-2.8h4.4l6.4-6.4V2H4.3Zm14.7 10.9-3.2 3.2h-4.5l-2.8 2.5v-2.5H5.2V4.1H19v8.8Z"/>
        <path fill="#FFFFFF" d="M15 6.8h1.8v5.1H15V6.8Zm-4.3 0h1.8v5.1h-1.8V6.8Z"/>
      </svg>
    {:else if brand === "chatgpt"}
      <svg class="brand-svg" width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
        <rect width="24" height="24" rx="6" fill="#10A37F"/>
        <circle cx="12" cy="12" r="5.5" fill="none" stroke="#fff" stroke-width="1.8"/>
        <circle cx="12" cy="12" r="2" fill="#fff"/>
      </svg>
    {:else}
      <FavIcon
        image={favicons.image(tab.icon)}
        tone={favicons.tone(tab.icon)}
        loading={tab.loading}
        size={22}
        lit
        fallback={Globe02Icon}
        notched={!!tab.audio}
      />
    {/if}
    <span data-zephium-tab-label class="sr-only">{tab.title}</span>
  </button>
  {#if tab.audio}
    <AudioControl
      audio={tab.audio}
      size={18}
      onToggle={(muted) => void tabs.setMuted(tab.id, muted)}
    />
  {/if}
  {#if tab.capture}
    <span class="capture-badge">
      <CaptureControl
        badge
        onOpen={(x, y) => tabs.openMediaMenu(tab.id, x, y)}
        site={tab.url ?? tab.title}
        capture={tab.capture}
      />
    </span>
  {/if}
  {#if open}
    <button
      type="button"
      class="essential-close"
      aria-label={m.close_named_tab({ title: name })}
      onclick={closePage}
    >
      <Icon icon={Cancel01Icon} size={10} strokeWidth={2.2} />
    </button>
  {/if}
</li>

<style>
  .essential {
    position: relative;
    min-width: 0;
  }

  .capture-badge {
    position: absolute;
    inset-inline-end: 2px;
    inset-block-end: 2px;
  }

  /* A plate of quiet ground and nothing else. A drawn ring made every tile
     read as a button; the fill alone already separates it from the column,
     and a row of unrelated brand marks stays calm on it. */
  .essential > button[data-plate] {
    display: grid;
    place-items: center;
    width: 100%;
    height: var(--dock-tile);
    border: 0;
    border-radius: var(--radius-card);
    background: var(--color-tile);
    box-shadow: var(--shine), var(--shadow-tile);
    cursor: default;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      box-shadow var(--motion-fast) var(--ease-out),
      scale var(--motion-slow) var(--ease-spring);
  }

  .essential > button[data-plate]:hover {
    background: var(--color-tile-hover);
  }

  .essential > button[data-plate]:active {
    scale: 0.95;
    transition-duration: var(--motion-instant);
  }

  .essential > button[data-plate][aria-current="page"] {
    background: var(--color-fill-active);
    box-shadow: var(--row-rim);
  }

  .essential[data-split-candidate="true"] > button[data-plate] {
    box-shadow: inset 0 0 0 1px var(--color-accent);
  }

  /* A dot under the mark says a page is open behind the tile, and the plate
     lit as it is for the current tab says which one is in front. A closed
     Essential has neither: it is only the site. */
  .essential[data-open="true"]::after {
    content: "";
    position: absolute;
    inset-inline: 0;
    inset-block-end: 4px;
    width: 4px;
    height: 4px;
    margin-inline: auto;
    border-radius: var(--radius-capsule);
    background: var(--color-faint);
    pointer-events: none;
  }

  .essential[data-active="true"]::after {
    background: var(--color-label-secondary);
  }

  /* Ends the page, never the Essential. It hangs in the corner rather than
     taking room from the mark, and shows for a hand or the keyboard alike. */
  .essential-close {
    position: absolute;
    inset-block-start: 4px;
    inset-inline-end: 4px;
    z-index: 2;
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-capsule);
    background: var(--color-fill-hover);
    color: var(--color-muted);
    opacity: 0;
    cursor: default;
    transition:
      opacity var(--motion-fast) var(--ease-out),
      background-color var(--motion-fast) var(--ease-out),
      color var(--motion-fast) var(--ease-out);
  }

  .essential-close:hover {
    background: var(--color-fill-pressed);
    color: var(--color-text);
  }

  .essential-close:focus-visible,
  .essential:is(:hover, :focus-within) .essential-close {
    opacity: 1;
  }

  @media (forced-colors: active) {
    .essential[data-open="true"]::after {
      background: CanvasText;
    }

    .essential-close {
      border: 1px solid ButtonText;
      background: ButtonFace;
      color: ButtonText;
    }
  }
</style>

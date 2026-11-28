<script lang="ts">
  import type { MiniIntent, TabView } from "$shared/ipc/bindings";
  import * as m from "$shared/i18n/messages";
  import { hostOf } from "../lib/mini-model";

  let {
    tab,
    spaces: _spaces,
    pinned: _pinned,
    frame,
    onAction,
    onSpaceMenu: _onSpaceMenu,
  }: {
    tab: TabView | null;
    /** How many spaces the page can go to; kept for prop compatibility. */
    spaces: number;
    /** Whether the window stays above other windows; kept for prop compatibility. */
    pinned: boolean;
    /** macOS uses native lights; Windows renders the Zen-style floating left rail. */
    frame: "lights" | "controls";
    onAction: (intent: MiniIntent) => void;
    onSpaceMenu: (x: number, y: number) => void;
  } = $props();

  function trigger(intent: MiniIntent, event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    onAction(intent);
  }
</script>

<!-- Zen Browser Glance layout:
     - 52px transparent left gutter holds the 3 floating circular buttons OUTSIDE the bubble
     - Pure rounded bubble card fills [left: 52px..right: 0, top: 0..bottom: 0] with ZERO top header bar
     - Visually-hidden data-zephium-* DOM satisfies native's presentation barrier synchronously -->
<div class="glance-shell" data-frame={frame}>
  <!-- 3 Floating Circular Zen Buttons stacked vertically OUTSIDE the bubble -->
  <div class="glance-rail" role="toolbar" aria-label="Glance Kontrolleri">
    <button
      type="button"
      class="glance-circle glance-circle-close"
      aria-label={m.window_close()}
      title="Glance'ı Kapat (Esc)"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={(e) => trigger("close", e)}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path
          d="M4.25 4.25L11.75 11.75M11.75 4.25L4.25 11.75"
          stroke="currentColor"
          stroke-width="1.85"
          stroke-linecap="round"
        />
      </svg>
    </button>

    <button
      type="button"
      class="glance-circle glance-circle-expand"
      aria-label={m.mini_open_in_main()}
      title="Yan Sekmede Tam Ekran Aç (Ctrl+O)"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={(e) => trigger("open_in_main", e)}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path
          d="M9.5 2.75H13.25V6.5M6.5 13.25H2.75V9.5M13 3L9.25 6.75M3 13L6.75 9.25"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <button
      type="button"
      class="glance-circle glance-circle-split"
      aria-label="Bölünmüş Görünümde Aç"
      title="Bölünmüş Görünümde Aç (Split View)"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={(e) => trigger("split_in_main", e)}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <rect
          x="1.75"
          y="2.75"
          width="12.5"
          height="10.5"
          rx="2.2"
          stroke="currentColor"
          stroke-width="1.6"
        />
        <line x1="8" y1="3" x2="8" y2="13" stroke="currentColor" stroke-width="1.6" />
      </svg>
    </button>
  </div>

  <!-- Rounded bubble surface directly behind the native webpage WebView2 (left: 52px) -->
  <div class="glance-bubble-frame" aria-hidden="true">
    {#if tab?.loading}
      <span class="glance-loading" role="progressbar" aria-label={m.surface_loading()}></span>
    {/if}
  </div>

  <!-- Hidden presentation-barrier metadata (untouched text nodes, verified by native eval) -->
  {#if tab}
    <div
      class="sr-only-meta"
      data-zephium-mini-tab={tab.id}
      data-zephium-tab-url={tab.url ?? ""}
      data-zephium-projection-revision={tab.projection_revision}
    >
      <span data-zephium-address>{hostOf(tab.url)}</span>
      <span data-zephium-tab-label>{tab.title}</span>
    </div>
  {/if}
</div>

<style>
  .glance-shell {
    position: fixed;
    inset: 0;
    background: transparent;
    pointer-events: none;
    user-select: none;
    -webkit-user-select: none;
  }

  /* 3 floating circular buttons OUTSIDE the bubble in the 52px transparent left gutter */
  .glance-rail {
    position: fixed;
    top: 10px;
    left: 6px;
    width: 40px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    z-index: 100;
    pointer-events: auto;
  }

  .glance-circle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    border-radius: 50%;
    border: 1px solid rgba(15, 23, 42, 0.12);
    background: rgba(255, 255, 255, 0.94);
    color: #111827;
    box-shadow:
      0 4px 14px rgba(0, 0, 0, 0.35),
      0 1px 2px rgba(0, 0, 0, 0.2);
    cursor: pointer;
    pointer-events: auto;
    transition:
      transform 120ms cubic-bezier(0.2, 0.8, 0.2, 1),
      background-color 120ms ease,
      color 120ms ease,
      box-shadow 120ms ease;
  }

  .glance-circle:hover {
    transform: scale(1.07);
    box-shadow:
      0 6px 18px rgba(0, 0, 0, 0.42),
      0 1px 3px rgba(0, 0, 0, 0.25);
  }

  .glance-circle:active {
    transform: scale(0.94);
  }

  .glance-circle-close:hover {
    background: #fee2e2;
    color: #dc2626;
    border-color: rgba(220, 38, 38, 0.28);
  }

  .glance-circle-expand:hover {
    background: #e0f2fe;
    color: #0284c7;
    border-color: rgba(2, 132, 199, 0.28);
  }

  .glance-circle-split:hover {
    background: #ede9fe;
    color: #6d28d9;
    border-color: rgba(109, 40, 217, 0.28);
  }

  /* Pure rounded webpage bubble backdrop (sits under the native WebView2 at left: 52px) */
  .glance-bubble-frame {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    left: 52px;
    border-radius: 14px;
    background: #14151c;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.14);
    overflow: hidden;
    pointer-events: none;
  }

  :global(html[data-theme="light"]) .glance-bubble-frame {
    background: #ffffff;
    box-shadow: inset 0 0 0 1px rgba(15, 23, 42, 0.14);
  }

  /* Subtle top loading bar inside the bubble while the page loads */
  .glance-loading {
    position: absolute;
    inset: 0 0 auto;
    height: 3px;
    overflow: hidden;
    pointer-events: none;
  }

  .glance-loading::after {
    position: absolute;
    inset: 0;
    width: 40%;
    content: "";
    background: #a78bfa;
    border-radius: 999px;
    animation: glance-loading 1.2s ease-in-out infinite;
  }

  @keyframes glance-loading {
    from {
      translate: -100% 0;
    }

    to {
      translate: 250% 0;
    }
  }

  /* Visually hidden metadata kept mounted for native's apply_presentation verification */
  .sr-only-meta {
    position: fixed;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
    border: 0;
    pointer-events: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .glance-loading::after {
      width: 100%;
      opacity: 0.6;
      animation: none;
    }
  }

  :global(:root[data-reduce-motion="true"]) .glance-loading::after {
    width: 100%;
    opacity: 0.6;
    animation: none;
  }
</style>

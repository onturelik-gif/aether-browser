<script lang="ts">
  import {
    ArrowDown01Icon,
    ArrowLeft02Icon,
    ArrowRight02Icon,
    PinIcon,
    Refresh01Icon,
  } from "@hugeicons/core-free-icons";
  import type { IconSvgElement } from "@hugeicons/svelte";
  import type { MiniIntent, TabView } from "$shared/ipc/bindings";
  import * as m from "$shared/i18n/messages";
  import Icon from "$shared/ui/Icon";
  import { hostOf } from "../lib/mini-model";

  let {
    tab,
    spaces,
    pinned,
    frame,
    onAction,
    onSpaceMenu,
  }: {
    tab: TabView | null;
    /** How many spaces the page can go to; a choice is offered past one. */
    spaces: number;
    /** Whether the window stays above other windows. */
    pinned: boolean;
    /** macOS draws the lights at the start of the row; Windows draws no
     *  frame, so Glance renders its own top-left floating controls. */
    frame: "lights" | "controls";
    onAction: (intent: MiniIntent) => void;
    onSpaceMenu: (x: number, y: number) => void;
  } = $props();

  function chooseSpace(event: MouseEvent) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    onSpaceMenu(rect.left, rect.bottom + 4);
  }
</script>

<!-- The page's title and host are untrusted; they are only ever text.
     NEVER put data-tauri-drag-region on <header> or buttons — only on the
     empty drag spacers — so Tauri's native window drag never swallows clicks! -->
<header class="mini-toolbar" data-frame={frame}>
  <!-- Zen Glance signature top-left 3-button control pill: Close, Expand to Tab, Split View -->
  <div class="glance-actions" role="group" aria-label="Glance Kontrolleri">
    <button
      type="button"
      class="glance-btn glance-btn-close"
      aria-label={m.window_close()}
      title="Glance'ı Kapat (Esc)"
      onclick={() => onAction("close")}
    >
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path
          d="M4 4L12 12M12 4L4 12"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
        />
      </svg>
    </button>
    <button
      type="button"
      class="glance-btn glance-btn-expand"
      disabled={!tab}
      aria-label={m.mini_open_in_main()}
      title="Sekmeye Genişlet (Ctrl+O)"
      onclick={() => onAction("open_in_main")}
    >
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path
          d="M9.5 2.5H13.5V6.5M6.5 13.5H2.5V9.5M13.25 2.75L9.25 6.75M2.75 13.25L6.75 9.25"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <button
      type="button"
      class="glance-btn glance-btn-split"
      disabled={!tab}
      aria-label="Bölünmüş Görünümde Aç"
      title="Bölünmüş Görünümde Aç"
      onclick={() => onAction("split_in_main")}
    >
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <rect
          x="1.75"
          y="2.75"
          width="12.5"
          height="10.5"
          rx="2.2"
          stroke="currentColor"
          stroke-width="1.5"
        />
        <line x1="8" y1="3" x2="8" y2="13" stroke="currentColor" stroke-width="1.5" />
      </svg>
    </button>
    {#if tab && spaces > 1}
      <button
        type="button"
        class="glance-btn glance-btn-space"
        aria-label={m.mini_choose_space()}
        title={m.mini_choose_space()}
        aria-haspopup="menu"
        onclick={chooseSpace}
      >
        <Icon icon={ArrowDown01Icon} size={13} />
      </button>
    {/if}
  </div>

  <span class="glance-divider" aria-hidden="true"></span>

  <!-- Compact Back / Forward / Reload navigation buttons -->
  {#snippet navControl(icon: IconSvgElement, label: string, intent: MiniIntent, disabled: boolean)}
    <button
      type="button"
      class="glance-nav-btn"
      aria-label={label}
      title={label}
      {disabled}
      onclick={() => onAction(intent)}
    >
      <Icon {icon} size={15} />
    </button>
  {/snippet}
  <div class="glance-nav-group" role="group" aria-label={m.ui_navigation()}>
    {@render navControl(ArrowLeft02Icon, m.ui_back(), "back", tab?.can_go_back !== true)}
    {@render navControl(ArrowRight02Icon, m.ui_forward(), "forward", tab?.can_go_forward !== true)}
    {@render navControl(Refresh01Icon, m.ui_reload(), "reload", !tab)}
  </div>

  <!-- Left empty drag spacer -->
  <div class="glance-drag-spacer" data-tauri-drag-region aria-hidden="true"></div>

  <!-- Sleek centered host pill with subtle "Glance" badge.
       Preserves exact data-zephium-* attributes for native's presentation barrier. -->
  {#if tab}
    <button
      type="button"
      class="mini-page"
      title={tab.url ?? undefined}
      data-zephium-mini-tab={tab.id}
      data-zephium-tab-url={tab.url ?? ""}
      data-zephium-projection-revision={tab.projection_revision}
      onclick={() => onAction("edit_address")}
    >
      <span class="glance-badge">Glance</span>
      <span class="mini-host" data-zephium-address>{hostOf(tab.url)}</span>
      <span class="mini-title" data-zephium-tab-label>{tab.title}</span>
    </button>
  {:else}
    <div class="mini-page-placeholder" data-tauri-drag-region>
      <span class="glance-badge">Glance</span>
    </div>
  {/if}

  <!-- Right empty drag spacer -->
  <div class="glance-drag-spacer" data-tauri-drag-region aria-hidden="true"></div>

  <!-- Subtle pin-on-top toggle on the far right -->
  <button
    type="button"
    class={["glance-nav-btn", pinned && "glance-pin-active"]}
    aria-label={m.mini_keep_on_top()}
    title={m.mini_keep_on_top()}
    aria-pressed={pinned}
    onclick={() => onAction("keep_on_top")}
  >
    <Icon icon={PinIcon} size={14} />
  </button>

  {#if tab?.loading}
    <span class="mini-loading" role="progressbar" aria-label={m.surface_loading()}></span>
  {/if}
</header>

<style>
  /* CRITICAL: Explicit 40px height matches MINI_TOOLBAR_HEIGHT on Windows, where
     the toolbar WebView spans the full window height behind the content WebView2. */
  .mini-toolbar {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 40px;
    width: 100%;
    box-sizing: border-box;
    padding-inline: 10px;
    background: linear-gradient(180deg, #1b1c24 0%, #14151c 100%);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    color: #f8fafc;
    user-select: none;
    -webkit-user-select: none;
  }

  :global(html[data-theme="light"]) .mini-toolbar {
    background: linear-gradient(180deg, #f8fafc 0%, #eef2f7 100%);
    border-bottom: 1px solid rgba(15, 23, 42, 0.1);
    color: #0f172a;
  }

  /* macOS traffic lights occupy the leading edge. */
  .mini-toolbar[data-frame="lights"] {
    padding-inline-start: 76px;
  }

  /* Signature Zen Glance Top-Left 3-Button Floating Control Pill */
  .glance-actions {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    flex: none;
  }

  :global(html[data-theme="light"]) .glance-actions {
    background: rgba(15, 23, 42, 0.05);
    border: 1px solid rgba(15, 23, 42, 0.1);
  }

  .glance-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    color: #f8fafc;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      transform 100ms ease;
  }

  :global(html[data-theme="light"]) .glance-btn {
    background: rgba(15, 23, 42, 0.06);
    color: #1e293b;
  }

  .glance-btn:active:not(:disabled) {
    transform: scale(0.92);
  }

  .glance-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .glance-btn-close:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.28);
    color: #fecaca;
  }

  :global(html[data-theme="light"]) .glance-btn-close:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.18);
    color: #dc2626;
  }

  .glance-btn-expand:hover:not(:disabled) {
    background: rgba(56, 189, 248, 0.28);
    color: #bae6fd;
  }

  :global(html[data-theme="light"]) .glance-btn-expand:hover:not(:disabled) {
    background: rgba(2, 132, 199, 0.16);
    color: #0284c7;
  }

  .glance-btn-split:hover:not(:disabled) {
    background: rgba(139, 92, 246, 0.3);
    color: #ddd6fe;
  }

  :global(html[data-theme="light"]) .glance-btn-split:hover:not(:disabled) {
    background: rgba(124, 58, 237, 0.16);
    color: #7c3aed;
  }

  .glance-btn-space:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
  }

  .glance-divider {
    width: 1px;
    height: 16px;
    background: rgba(255, 255, 255, 0.12);
    flex: none;
    margin-inline: 2px;
  }

  :global(html[data-theme="light"]) .glance-divider {
    background: rgba(15, 23, 42, 0.12);
  }

  /* Compact Navigation Buttons */
  .glance-nav-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    flex: none;
  }

  .glance-nav-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: #cbd5e1;
    cursor: pointer;
    flex: none;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      transform 100ms ease;
  }

  :global(html[data-theme="light"]) .glance-nav-btn {
    color: #475569;
  }

  .glance-nav-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
  }

  :global(html[data-theme="light"]) .glance-nav-btn:hover:not(:disabled) {
    background: rgba(15, 23, 42, 0.08);
    color: #0f172a;
  }

  .glance-nav-btn:active:not(:disabled) {
    transform: scale(0.93);
  }

  .glance-nav-btn:disabled {
    opacity: 0.32;
    cursor: default;
  }

  .glance-pin-active {
    background: rgba(139, 92, 246, 0.22);
    color: #c4b5fd;
  }

  :global(html[data-theme="light"]) .glance-pin-active {
    background: rgba(124, 58, 237, 0.14);
    color: #6d28d9;
  }

  /* Drag spacers — ONLY these carry data-tauri-drag-region */
  .glance-drag-spacer {
    flex: 1 1 16px;
    height: 100%;
    min-width: 12px;
  }

  /* Centered Sleek Host Pill with Glance Badge */
  .mini-page {
    display: inline-flex;
    flex: 0 1 420px;
    min-width: 0;
    max-width: 460px;
    align-items: center;
    gap: 7px;
    height: 26px;
    padding: 0 10px 0 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.09);
    color: #e2e8f0;
    text-align: start;
    cursor: pointer;
    font-size: 12px;
    line-height: 1;
    transition:
      background-color 120ms ease,
      border-color 120ms ease;
  }

  :global(html[data-theme="light"]) .mini-page {
    background: rgba(15, 23, 42, 0.05);
    border: 1px solid rgba(15, 23, 42, 0.09);
    color: #1e293b;
  }

  .mini-page:hover {
    background: rgba(255, 255, 255, 0.11);
    border-color: rgba(255, 255, 255, 0.16);
  }

  :global(html[data-theme="light"]) .mini-page:hover {
    background: rgba(15, 23, 42, 0.09);
    border-color: rgba(15, 23, 42, 0.15);
  }

  .mini-page-placeholder {
    display: inline-flex;
    align-items: center;
    height: 26px;
    padding: 0 8px;
  }

  .glance-badge {
    flex: none;
    display: inline-flex;
    align-items: center;
    height: 18px;
    padding: 0 6px;
    border-radius: 999px;
    background: rgba(139, 92, 246, 0.22);
    border: 1px solid rgba(167, 139, 250, 0.32);
    color: #ddd6fe;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  :global(html[data-theme="light"]) .glance-badge {
    background: rgba(124, 58, 237, 0.12);
    border: 1px solid rgba(124, 58, 237, 0.25);
    color: #6d28d9;
  }

  .mini-host {
    flex: none;
    max-width: 180px;
    overflow: hidden;
    font-weight: 500;
    color: #f8fafc;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(html[data-theme="light"]) .mini-host {
    color: #0f172a;
  }

  .mini-title {
    flex: 1 1 auto;
    min-width: 0;
    overflow: hidden;
    color: #94a3b8;
    font-size: 11.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(html[data-theme="light"]) .mini-title {
    color: #64748b;
  }

  /* Thin loading indicator along the bottom edge of the 40px toolbar */
  .mini-loading {
    position: absolute;
    inset: auto 0 0;
    height: 2px;
    overflow: hidden;
    pointer-events: none;
  }

  .mini-loading::after {
    position: absolute;
    inset: 0;
    width: 40%;
    content: "";
    background: #a78bfa;
    border-radius: 999px;
    animation: mini-loading 1.2s ease-in-out infinite;
  }

  @keyframes mini-loading {
    from {
      translate: -100% 0;
    }

    to {
      translate: 250% 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .mini-loading::after {
      width: 100%;
      opacity: 0.6;
      animation: none;
    }
  }

  :global(:root[data-reduce-motion="true"]) .mini-loading::after {
    width: 100%;
    opacity: 0.6;
    animation: none;
  }

  @media (forced-colors: active) {
    .glance-actions,
    .mini-page {
      border: 1px solid ButtonText;
    }

    .mini-loading::after {
      background: Highlight;
    }
  }
</style>

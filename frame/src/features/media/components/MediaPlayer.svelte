<script lang="ts">
  import {
    Cancel01Icon,
    PictureInPictureOnIcon,
  } from "@hugeicons/core-free-icons";
  import * as m from "$shared/i18n/messages";
  import Icon from "$shared/ui/Icon";
  import FavIcon from "$shared/ui/FavIcon";
  import { favicons } from "$domain/favicons";
  import { mediaPlayer } from "$domain/media-player";
  import { tabs } from "$domain/tabs";
  import { clock, fraction } from "../lib/time";
  import { everyWhileVisible } from "$shared/lib/visible-interval";

  let { compact = false }: { compact?: boolean } = $props();

  // The disc is 36px and its ring a 1.5px stroke along its edge.
  const RING = 17.25;

  let player = $derived(mediaPlayer.view());
  // Native sends the position as of each change; between changes it is
  // arithmetic, redrawn once a second only while something plays.
  let now = $state(performance.now());
  $effect(() => {
    if (!player?.playing || player.waiting) return;
    now = performance.now();
    return everyWhileVisible(1000, () => (now = performance.now()));
  });
  // A seek holds its place until native reports the next state.
  let sought = $state.raw<{ for: object; seconds: number } | null>(null);
  let position = $derived(
    sought && sought.for === player ? sought.seconds : mediaPlayer.position(now),
  );
  let duration = $derived(player?.duration ?? null);
  let progress = $derived(fraction(position, duration));

  let hovered = $state(false);
  let focused = $state(false);
  let expanded = $derived(hovered || focused);
  let volume = $state(100);
  let volHover = $state(false);
  let volDragging = $state(false);
  let volTimer: ReturnType<typeof setTimeout> | null = null;
  let volOpen = $derived(volHover || volDragging);
  let volTrackEl: HTMLDivElement | null = $state(null);

  function openVol() {
    if (volTimer) {
      clearTimeout(volTimer);
      volTimer = null;
    }
    volHover = true;
  }

  function closeVol() {
    if (volTimer) clearTimeout(volTimer);
    volTimer = setTimeout(() => {
      volHover = false;
      volTimer = null;
    }, 180);
  }

  function toggle() {
    if (player?.playing) mediaPlayer.pause();
    else mediaPlayer.play();
  }

  function seek(event: Event) {
    const seconds = Number((event.currentTarget as HTMLInputElement).value);
    if (!player || !Number.isFinite(seconds)) return;
    sought = { for: player, seconds };
    mediaPlayer.seek(seconds);
  }

  function applyVolume(pct: number) {
    if (!player) return;
    const clamped = Math.max(0, Math.min(100, Math.round(pct)));
    volume = clamped;
    mediaPlayer.setVolume(clamped / 100);
    if (clamped > 0 && player.muted) {
      void tabs.setMuted(player.tab_id, false);
    }
  }

  function updateVolFromPointer(clientY: number) {
    if (!volTrackEl) return;
    const rect = volTrackEl.getBoundingClientRect();
    const ratio = 1 - (clientY - rect.top) / rect.height;
    applyVolume(ratio * 100);
  }

  function onVolPointerDown(event: PointerEvent) {
    event.preventDefault();
    volDragging = true;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    updateVolFromPointer(event.clientY);
  }

  function onVolPointerMove(event: PointerEvent) {
    if (!volDragging) return;
    updateVolFromPointer(event.clientY);
  }

  function onVolPointerUp(event: PointerEvent) {
    if (!volDragging) return;
    volDragging = false;
    try {
      (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    } catch {}
  }

  function onVolWheel(event: WheelEvent) {
    event.preventDefault();
    const current = player?.muted ? 0 : volume;
    const delta = event.deltaY < 0 ? 5 : -5;
    applyVolume(current + delta);
  }
</script>

{#snippet playGlyph(playing: boolean, size: number)}
  <svg width={size} height={size} viewBox="0 0 16 16" aria-hidden="true" class="media-glyph">
    {#if playing}
      <rect x="3.5" y="2.5" width="3" height="11" rx="1.1" />
      <rect x="9.5" y="2.5" width="3" height="11" rx="1.1" />
    {:else}
      <path
        d="M5 3.1v9.8a.9.9 0 0 0 1.37.77l7.6-4.9a.9.9 0 0 0 0-1.54l-7.6-4.9A.9.9 0 0 0 5 3.1Z"
      />
    {/if}
  </svg>
{/snippet}

{#if player}
  {#if compact}
    <!-- The rail has room for the site and what plays it: a plate like its
         other buttons, ringed by how far it has played from the top, which
         plays or pauses. -->
    <button
      type="button"
      class="media-disc"
      data-playing={player.playing || undefined}
      title={player.title}
      aria-label={`${player.playing ? m.media_pause() : m.media_play()}: ${player.title}`}
      onclick={toggle}
    >
      <svg class="media-ring" viewBox="0 0 36 36" aria-hidden="true">
        <circle class="media-ring-track" cx="18" cy="18" r={RING} />
        {#if duration !== null && progress > 0}<circle
            class="media-ring-fill"
            cx="18"
            cy="18"
            r={RING}
            pathLength="100"
            stroke-dasharray={`${progress * 100} 100`}
            transform="rotate(-90 18 18)"
          />{/if}
      </svg>
      <span class="media-disc-icon">
        <FavIcon
          image={favicons.image(player.icon)}
          tone={favicons.tone(player.icon)}
          size={18}
          lit
        />
      </span>
      <span class="media-disc-glyph">{@render playGlyph(player.playing, 14)}</span>
    </button>
  {:else}
    <section
      class="media-player"
      aria-label={m.media_player()}
      data-expanded={expanded || undefined}
      data-playing={player.playing || undefined}
      onpointerenter={() => (hovered = true)}
      onpointerleave={() => (hovered = false)}
      onfocusin={() => (focused = true)}
      onfocusout={(event) => {
        const next = event.relatedTarget as Node | null;
        if (!next || !(event.currentTarget as HTMLElement).contains(next)) focused = false;
      }}
    >
      <!-- The site's own colour, as an ambient glow behind the card. -->
      <span class="media-ambient" aria-hidden="true">
        <FavIcon image={favicons.image(player.icon)} size={56} lit />
      </span>
      <div class="media-head">
        <button
          type="button"
          class="media-jump"
          aria-label={m.media_show_tab({ title: player.title })}
          title={m.media_show_tab({ title: player.title })}
          onclick={() => mediaPlayer.show()}
        >
          <span class="media-site">
            <FavIcon
              image={favicons.image(player.icon)}
              tone={favicons.tone(player.icon)}
              size={20}
              lit
            />
          </span>
          <span class="media-text">
            <span class="media-title" data-media-title>{player.title}</span>
            {#if player.artist}<span class="media-artist">{player.artist}</span>{/if}
          </span>
        </button>
        <button
          type="button"
          class="media-control media-toggle"
          aria-label={player.playing ? m.media_pause() : m.media_play()}
          title={player.playing ? m.media_pause() : m.media_play()}
          onclick={toggle}
        >
          {@render playGlyph(player.playing, 14)}
        </button>
        {#if player.can_mute}
          {@const displayVol = player.muted ? 0 : volume}
          <div
            class="media-vol-wrap"
            data-open={volOpen || undefined}
            onpointerenter={openVol}
            onpointerleave={closeVol}
            onwheel={onVolWheel}
          >
            <div class="media-vol-popover" role="group" aria-label="Ses seviyesi">
              <div class="media-vol-card">
                <span class="media-vol-pct">{displayVol}</span>
                <div
                  class="media-vol-hit"
                  role="slider"
                  tabindex="0"
                  aria-label="Ses seviyesi"
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-valuenow={displayVol}
                  onpointerdown={onVolPointerDown}
                  onpointermove={onVolPointerMove}
                  onpointerup={onVolPointerUp}
                  onpointercancel={onVolPointerUp}
                  onkeydown={(e) => {
                    if (e.key === "ArrowUp" || e.key === "ArrowRight") {
                      e.preventDefault();
                      applyVolume(displayVol + 5);
                    } else if (e.key === "ArrowDown" || e.key === "ArrowLeft") {
                      e.preventDefault();
                      applyVolume(displayVol - 5);
                    }
                  }}
                >
                  <div class="media-vol-track" bind:this={volTrackEl}>
                    <div class="media-vol-fill" style:height={`${displayVol}%`}></div>
                    <div class="media-vol-thumb" style:bottom={`${displayVol}%`}></div>
                  </div>
                </div>
              </div>
            </div>
            <button
              type="button"
              class="media-control media-vol-btn"
              data-muted={displayVol === 0 || undefined}
              aria-label={player.muted ? m.tab_unmute() : m.tab_mute()}
              aria-pressed={player.muted}
              title={player.muted ? m.tab_audio_muted() : m.tab_mute()}
              onclick={() => void tabs.setMuted(player!.tab_id, !player!.muted)}
            >
              <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                <path
                  d="M2.5 6.2H5L8.4 3.3C8.8 2.95 9.4 3.25 9.4 3.78V12.22C9.4 12.75 8.8 13.05 8.4 12.7L5 9.8H2.5C1.95 9.8 1.5 9.35 1.5 8.8V7.2C1.5 6.65 1.95 6.2 2.5 6.2Z"
                  fill="currentColor"
                />
                {#if displayVol === 0}
                  <path
                    d="M11.4 6.1L14.4 9.9M14.4 6.1L11.4 9.9"
                    stroke="currentColor"
                    stroke-width="1.45"
                    stroke-linecap="round"
                  />
                {:else if displayVol < 50}
                  <path
                    d="M11.3 5.8C12.1 6.5 12.5 7.2 12.5 8C12.5 8.8 12.1 9.5 11.3 10.2"
                    stroke="currentColor"
                    stroke-width="1.4"
                    stroke-linecap="round"
                  />
                {:else}
                  <path
                    d="M11.2 5.9C11.9 6.5 12.25 7.2 12.25 8C12.25 8.8 11.9 9.5 11.2 10.1"
                    stroke="currentColor"
                    stroke-width="1.35"
                    stroke-linecap="round"
                  />
                  <path
                    d="M13 4.1C14.1 5.1 14.7 6.5 14.7 8C14.7 9.5 14.1 10.9 13 11.9"
                    stroke="currentColor"
                    stroke-width="1.35"
                    stroke-linecap="round"
                    opacity="0.72"
                  />
                {/if}
              </svg>
            </button>
          </div>{/if}
      </div>
      <div class="media-more" inert={!expanded}>
        <div class="media-more-inner">
          {#if duration !== null}
            <input
              class="media-progress"
              type="range"
              min="0"
              max={duration}
              step="any"
              value={position}
              style:--progress={progress}
              aria-label={m.media_position()}
              aria-valuetext={m.media_position_text({
                position: clock(position),
                duration: clock(duration),
              })}
              onchange={seek}
            />
          {/if}
          <div class="media-foot">
            {#if duration !== null}
              <span class="media-time">{clock(position)}</span>
            {:else}
              <span class="media-live">{m.media_live()}</span>
            {/if}
            <span class="media-actions">
              {#if player.can_pip}<button
                  type="button"
                  class="media-control media-small"
                  aria-label={m.media_pip()}
                  aria-pressed={player.in_pip}
                  title={m.media_pip()}
                  onclick={() => mediaPlayer.pictureInPicture()}
                >
                  <Icon icon={PictureInPictureOnIcon} size={14} />
                </button>{/if}
              <button
                type="button"
                class="media-control media-small"
                aria-label={m.media_close()}
                title={m.media_close()}
                onclick={() => mediaPlayer.close()}
              >
                <Icon icon={Cancel01Icon} size={12} />
              </button>
            </span>
            {#if duration !== null}<span class="media-time"
                >-{clock(Math.max(0, duration - position))}</span
              >{/if}
          </div>
        </div>
      </div>
    </section>
  {/if}
{/if}

<style>
  .media-player {
    position: relative;
    isolation: isolate;
    flex: none;
    margin: 6px var(--sidebar-inset);
    padding: 8px 6px 8px 8px;
    overflow: visible;
    border-radius: var(--radius-card);
    background: var(--row-active);
    box-shadow: var(--row-rim);
    animation: player-in var(--motion-base) var(--ease-emphasized) both;
  }

  /* The site's mark, blown up and blurred, tints the card with its colour
     the way album art tints a car's dashboard: drawn once, never animated. */
  .media-ambient {
    position: absolute;
    inset-block-start: -18px;
    inset-inline-start: -14px;
    z-index: -1;
    opacity: 0.32;
    filter: blur(22px) saturate(1.6);
    pointer-events: none;
    transition: opacity var(--motion-base) var(--ease-out);
  }

  .media-player:not([data-playing]) .media-ambient {
    opacity: 0.16;
  }

  .media-head {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .media-jump {
    display: flex;
    flex: 1;
    min-width: 0;
    align-items: center;
    gap: 8px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-inset);
    background: transparent;
    text-align: start;
    cursor: pointer;
    transition: opacity var(--motion-fast) var(--ease-out);
  }

  .media-jump:hover .media-site {
    background: var(--row-pressed);
  }

  .media-jump:hover .media-title {
    opacity: 0.92;
  }

  .media-vol-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  /* Zero-gap hover bridge: padding-bottom extends flush to the button so
     moving the pointer straight up never drops hover. */
  .media-vol-popover {
    position: absolute;
    bottom: 100%;
    left: 50%;
    padding-bottom: 6px;
    transform: translateX(-50%) translateY(4px) scale(0.96);
    opacity: 0;
    pointer-events: none;
    transition:
      opacity 130ms var(--ease-out),
      transform 160ms var(--ease-emphasized);
    z-index: 60;
  }

  .media-vol-wrap[data-open] .media-vol-popover,
  .media-vol-wrap:focus-within .media-vol-popover {
    opacity: 1;
    pointer-events: auto;
    transform: translateX(-50%) translateY(0) scale(1);
  }

  .media-vol-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
    width: 30px;
    padding: 8px 0 9px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--color-surface, #191c22) 88%, transparent);
    backdrop-filter: blur(18px) saturate(1.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow:
      0 8px 24px rgba(0, 0, 0, 0.42),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
  }

  .media-vol-pct {
    font-size: 9.5px;
    font-weight: 600;
    line-height: 1;
    color: var(--color-label-secondary);
    font-variant-numeric: tabular-nums;
    user-select: none;
  }

  .media-vol-hit {
    display: grid;
    place-items: center;
    width: 24px;
    height: 76px;
    cursor: pointer;
    touch-action: none;
    outline: none;
  }

  .media-vol-track {
    position: relative;
    width: 4px;
    height: 72px;
    border-radius: 999px;
    background: var(--color-fill-strong, rgba(255, 255, 255, 0.16));
  }

  .media-vol-fill {
    position: absolute;
    inset-inline: 0;
    bottom: 0;
    border-radius: 999px;
    background: var(--color-text, #f3f4f6);
    pointer-events: none;
  }

  .media-vol-thumb {
    position: absolute;
    left: 50%;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #ffffff;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.45);
    transform: translate(-50%, 50%);
    pointer-events: none;
    transition: scale 110ms var(--ease-out);
  }

  .media-vol-hit:hover .media-vol-thumb,
  .media-vol-hit:active .media-vol-thumb {
    scale: 1.18;
  }

  .media-vol-btn {
    color: var(--color-label-secondary);
  }

  .media-vol-wrap[data-open] .media-vol-btn {
    background: var(--row-pressed);
    color: var(--color-text);
  }

  .media-site {
    display: grid;
    flex: none;
    place-items: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-inset);
    background: var(--color-fill);
    cursor: default;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      scale var(--motion-slow) var(--ease-spring);
  }

  .media-site:hover {
    background: var(--row-pressed);
  }

  .media-site:active {
    scale: 0.92;
    transition-duration: var(--motion-instant);
  }

  .media-text {
    display: grid;
    flex: 1;
    min-width: 0;
  }

  .media-title,
  .media-artist {
    overflow: hidden;
    white-space: nowrap;
    mask-image: var(--mask-fade-end);
  }

  .media-title {
    color: var(--color-text);
    font-size: var(--text-label);
    font-weight: 550;
  }

  .media-artist {
    color: var(--color-muted);
    font-size: var(--text-caption);
  }

  .media-title:dir(rtl),
  .media-artist:dir(rtl) {
    mask-image: var(--mask-fade-end-rtl);
  }

  .media-control {
    display: grid;
    flex: none;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-capsule);
    background: transparent;
    color: var(--color-label-secondary);
    cursor: default;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      color var(--motion-fast) var(--ease-out),
      scale var(--motion-slow) var(--ease-spring);
  }

  .media-control:hover {
    background: var(--row-pressed);
    color: var(--color-text);
  }

  .media-control:active {
    scale: 0.88;
    transition-duration: var(--motion-instant);
  }

  .media-control:focus-visible,
  .media-site:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .media-toggle {
    color: var(--color-text);
  }

  .media-control[data-muted],
  .media-control[aria-pressed="true"]:not(.media-toggle) {
    color: var(--color-accent);
  }

  .media-small {
    width: 24px;
    height: 24px;
  }

  .media-glyph {
    fill: currentcolor;
  }

  /* The rest opens under the head while the pointer or focus is in the
     card, and folds away again: rows, not height, so nothing is measured. */
  .media-more {
    display: grid;
    grid-template-rows: 0fr;
    opacity: 0;
    transition:
      grid-template-rows var(--motion-base) var(--ease-emphasized),
      opacity var(--motion-fast) var(--ease-out);
  }

  .media-player[data-expanded] .media-more {
    grid-template-rows: 1fr;
    opacity: 1;
  }

  .media-more-inner {
    display: grid;
    gap: 2px;
    min-height: 0;
    overflow: hidden;
    padding-inline-end: 2px;
  }

  .media-progress {
    --track: var(--color-fill-strong);
    --fill: var(--color-text);

    appearance: none;
    width: 100%;
    height: 14px;
    margin: 8px 0 0;
    padding: 0;
    background: transparent;
    cursor: default;
  }

  .media-progress::-webkit-slider-runnable-track {
    height: 3px;
    border-radius: var(--radius-capsule);
    background: linear-gradient(
      to right,
      var(--fill) calc(var(--progress) * 100%),
      var(--track) calc(var(--progress) * 100%)
    );
    transition: height var(--motion-fast) var(--ease-out);
  }

  .media-progress:dir(rtl)::-webkit-slider-runnable-track {
    background: linear-gradient(
      to left,
      var(--fill) calc(var(--progress) * 100%),
      var(--track) calc(var(--progress) * 100%)
    );
  }

  .media-progress::-webkit-slider-thumb {
    appearance: none;
    width: 10px;
    height: 10px;
    margin-top: -3.5px;
    border-radius: var(--radius-capsule);
    background: var(--fill);
    box-shadow: var(--shadow-badge);
    scale: 0;
    transition: scale var(--motion-fast) var(--ease-out);
  }

  .media-progress:is(:hover, :focus-visible)::-webkit-slider-thumb {
    scale: 1;
  }

  .media-progress:focus-visible {
    outline: none;
  }

  .media-progress:focus-visible::-webkit-slider-runnable-track {
    box-shadow: 0 0 0 2px var(--color-accent);
  }

  .media-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 24px;
  }

  .media-time,
  .media-live {
    min-width: 38px;
    color: var(--color-muted);
    font-size: var(--text-caption);
    font-variant-numeric: tabular-nums;
  }

  .media-time:last-child {
    text-align: end;
  }

  .media-live {
    color: var(--color-accent);
    font-weight: 600;
  }

  .media-actions {
    display: flex;
    gap: 2px;
  }

  /* The rail: the site on the plate of the profile button below it, the same
     size, centred in the column and a rail step above the rule, ringed by how
     far it has played. */
  .media-disc {
    position: relative;
    display: grid;
    flex: none;
    place-items: center;
    align-self: center;
    width: 36px;
    height: 36px;
    margin-block-start: 8px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-capsule);
    background: var(--row-active);
    box-shadow: var(--shine-top);
    color: var(--color-text);
    cursor: default;
    animation: player-in var(--motion-base) var(--ease-emphasized) both;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      scale var(--motion-slow) var(--ease-spring);
  }

  .media-disc:hover {
    background: var(--row-pressed);
  }

  .media-disc:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .media-disc:active {
    scale: 0.92;
    transition-duration: var(--motion-instant);
  }

  .media-ring {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    fill: none;
    stroke: var(--color-fill-strong);
    stroke-width: 1.5;
    pointer-events: none;
  }

  .media-ring-fill {
    stroke: var(--color-text);
    stroke-linecap: round;
  }

  .media-disc-icon,
  .media-disc-glyph {
    display: grid;
    place-items: center;
    grid-area: 1 / 1;
    transition:
      opacity var(--motion-fast) var(--ease-out),
      scale var(--motion-base) var(--ease-spring);
  }

  .media-disc-glyph {
    opacity: 0;
    scale: 0.6;
  }

  .media-disc:is(:hover, :focus-visible) .media-disc-icon {
    opacity: 0;
    scale: 0.6;
  }

  .media-disc:is(:hover, :focus-visible) .media-disc-glyph {
    opacity: 1;
    scale: 1;
  }

  @keyframes player-in {
    from {
      opacity: 0;
      translate: 0 6px;
    }
  }

  @media (forced-colors: active) {
    .media-title,
    .media-artist {
      mask-image: none;
    }
  }
</style>

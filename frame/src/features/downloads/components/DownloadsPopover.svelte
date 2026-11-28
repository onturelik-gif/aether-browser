<script lang="ts">
  import { untrack } from "svelte";
  import {
    Cancel01Icon,
    Delete02Icon,
    Download04Icon,
    FolderOpenIcon,
    PauseIcon,
    PlayIcon,
  } from "@hugeicons/core-free-icons";
  import {
    DownloadSession,
    downloadProgress,
    finished,
    formatDownloadBytes,
  } from "$domain/downloads";
  import type { DownloadView } from "$shared/ipc/bindings";
  import Icon from "$shared/ui/Icon";
  import FileGlyph from "./FileGlyph.svelte";

  let {
    profile,
    onShowAll,
  }: {
    profile: string;
    onShowAll?: () => void;
  } = $props();

  let session = $state.raw(untrack(() => new DownloadSession(profile)));
  $effect(() => {
    if (!profile) return;
    const next = new DownloadSession(profile);
    session = next;
    void next.start();
    return () => next.stop();
  });

  let recent = $derived(
    session.entries
      .toSorted((a, b) => Number(b.created_at) - Number(a.created_at))
      .slice(0, 12),
  );
  let clearable = $derived(session.entries.some(finished));

  function statusLine(entry: DownloadView): string {
    if (entry.state === "receiving") {
      const got = formatDownloadBytes(entry.received);
      const tot = entry.total ? ` / ${formatDownloadBytes(entry.total)}` : "";
      return `İndiriliyor · ${got}${tot}`;
    }
    if (entry.state === "pending") return "Başlatılıyor…";
    if (entry.state === "paused") return "Duraklatıldı";
    if (entry.state === "completed") return formatDownloadBytes(entry.total ?? entry.received);
    if (entry.state === "cancelled") return "İptal edildi";
    if (entry.state === "failed" || entry.state === "interrupted") return "Başarısız";
    return entry.state;
  }
</script>

<div class="dl-pop">
  <div class="dl-pop-head">
    <span class="dl-pop-title">İndirilenler</span>
    <div class="dl-pop-head-actions">
      {#if clearable}
        <button
          type="button"
          class="dl-pop-link"
          disabled={session.busy}
          onclick={() => void session.perform({ kind: "clear" })}
        >
          Temizle
        </button>
      {/if}
      {#if onShowAll}
        <button type="button" class="dl-pop-link" data-dismiss onclick={onShowAll}>
          Tümü
        </button>
      {/if}
    </div>
  </div>

  {#if recent.length === 0}
    <div class="dl-pop-empty">
      <Icon icon={Download04Icon} size={20} />
      <span>Henüz indirilen dosya yok</span>
    </div>
  {:else}
    <ul class="dl-pop-list">
      {#each recent as entry (entry.id)}
        {@const progress = downloadProgress(entry)}
        <li class="dl-pop-row" data-state={entry.state}>
          {#if entry.state === "completed"}
            <button
              type="button"
              class="dl-pop-main"
              title="Dosyayı aç"
              disabled={session.busy}
              onclick={() => void session.perform({ kind: "open", id: entry.id })}
            >
              <FileGlyph filename={entry.filename} />
              <span class="dl-pop-meta">
                <span class="dl-pop-name">{entry.filename}</span>
                <span class="dl-pop-sub">{statusLine(entry)}</span>
              </span>
            </button>
          {:else}
            <div class="dl-pop-main">
              <FileGlyph filename={entry.filename} />
              <span class="dl-pop-meta">
                <span class="dl-pop-name">{entry.filename}</span>
                <span class="dl-pop-sub">{statusLine(entry)}</span>
                {#if entry.state === "receiving" || entry.state === "pending"}
                  <span class="dl-pop-bar">
                    <i
                      style:width={progress === undefined
                        ? "45%"
                        : `${Math.round(progress * 100)}%`}
                    ></i>
                  </span>
                {/if}
              </span>
            </div>
          {/if}

          <div class="dl-pop-actions">
            {#if entry.state === "completed"}
              <button
                type="button"
                class="dl-pop-icon-btn"
                title="Klasörde göster"
                aria-label="Klasörde göster"
                disabled={session.busy}
                onclick={() => void session.perform({ kind: "reveal", id: entry.id })}
              >
                <Icon icon={FolderOpenIcon} size={14} />
              </button>
            {/if}
            {#if entry.state === "receiving"}
              <button
                type="button"
                class="dl-pop-icon-btn"
                title="Duraklat"
                aria-label="Duraklat"
                disabled={session.busy}
                onclick={() => void session.perform({ kind: "pause", id: entry.id })}
              >
                <Icon icon={PauseIcon} size={13} />
              </button>
            {/if}
            {#if entry.state === "paused"}
              <button
                type="button"
                class="dl-pop-icon-btn"
                title="Devam et"
                aria-label="Devam et"
                disabled={session.busy}
                onclick={() => void session.perform({ kind: "resume", id: entry.id })}
              >
                <Icon icon={PlayIcon} size={13} />
              </button>
            {/if}
            {#if entry.state === "receiving" || entry.state === "pending" || entry.state === "paused"}
              <button
                type="button"
                class="dl-pop-icon-btn"
                title="İptal"
                aria-label="İptal"
                disabled={session.busy}
                onclick={() => void session.perform({ kind: "cancel", id: entry.id })}
              >
                <Icon icon={Cancel01Icon} size={13} />
              </button>
            {/if}
            {#if finished(entry)}
              <button
                type="button"
                class="dl-pop-icon-btn"
                title="Listeden kaldır"
                aria-label="Listeden kaldır"
                disabled={session.busy}
                onclick={() => void session.perform({ kind: "forget", id: entry.id })}
              >
                <Icon icon={Delete02Icon} size={13} />
              </button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .dl-pop {
    width: 310px;
    max-width: min(310px, calc(100vw - 24px));
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 2px 0;
  }

  .dl-pop-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 6px 4px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .dl-pop-title {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--color-text);
  }

  .dl-pop-head-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dl-pop-link {
    border: 0;
    background: none;
    padding: 2px 4px;
    border-radius: 6px;
    color: var(--color-accent, #38bdf8);
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
  }

  .dl-pop-link:hover {
    background: rgba(56, 189, 248, 0.12);
  }

  .dl-pop-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 22px 12px;
    color: var(--color-muted);
    font-size: 12px;
  }

  .dl-pop-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 310px;
    overflow-y: auto;
  }

  .dl-pop-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 4px;
    border-radius: 9px;
    transition: background-color 110ms ease;
  }

  .dl-pop-row:hover {
    background: var(--row-hover, rgba(255, 255, 255, 0.07));
  }

  .dl-pop-main {
    display: flex;
    align-items: center;
    gap: 9px;
    flex: 1;
    min-width: 0;
    padding: 3px 4px;
    border: 0;
    background: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
  }

  .dl-pop-meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    gap: 1px;
  }

  .dl-pop-name {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dl-pop-sub {
    font-size: 11px;
    color: var(--color-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dl-pop-bar {
    display: block;
    width: 100%;
    height: 3px;
    margin-top: 3px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }

  .dl-pop-bar i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: #38bdf8;
    transition: width 160ms ease;
  }

  .dl-pop-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: none;
  }

  .dl-pop-icon-btn {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--color-label-secondary);
    cursor: pointer;
    transition: background-color 100ms ease, color 100ms ease;
  }

  .dl-pop-icon-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: var(--color-text);
  }
</style>

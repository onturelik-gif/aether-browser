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

  function brandOf(url: string): string | null {
    const host = hostOf(url).toLowerCase().replace(/^www\./, "");
    if (!host) return null;
    if (host === "youtube.com" || host === "youtu.be" || host.endsWith(".youtube.com")) return "youtube";
    if (host === "google.com" || host.startsWith("google.") || host === "mail.google.com") {
      return host === "mail.google.com" ? "gmail" : "google";
    }
    if (host === "gemini.google.com") return "gemini";
    if (host === "x.com" || host === "twitter.com" || host.endsWith(".x.com")) return "x";
    if (host === "instagram.com" || host.endsWith(".instagram.com")) return "instagram";
    if (host === "github.com" || host.endsWith(".github.com")) return "github";
    if (host === "chatgpt.com" || host === "chat.openai.com") return "chatgpt";
    if (host === "web.whatsapp.com" || host === "whatsapp.com" || host.endsWith(".whatsapp.com")) return "whatsapp";
    if (host === "discord.com" || host.endsWith(".discord.com")) return "discord";
    if (host === "twitch.tv" || host.endsWith(".twitch.tv")) return "twitch";
    if (host === "spotify.com" || host.endsWith(".spotify.com")) return "spotify";
    if (host === "reddit.com" || host.endsWith(".reddit.com")) return "reddit";
    if (host === "facebook.com" || host === "fb.com" || host.endsWith(".facebook.com")) return "facebook";
    if (host === "netflix.com" || host.endsWith(".netflix.com")) return "netflix";
    return null;
  }

  function accentFor(name: string): string {
    const palette = [
      "linear-gradient(135deg,#0ea5e9,#6366f1)",
      "linear-gradient(135deg,#10b981,#06b6d4)",
      "linear-gradient(135deg,#f59e0b,#ef4444)",
      "linear-gradient(135deg,#a855f7,#ec4899)",
      "linear-gradient(135deg,#3b82f6,#8b5cf6)",
    ];
    let h = 0;
    for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0;
    return palette[h % palette.length];
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
      {@const brand = brandOf(item.url)}
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
          {#if brand === "youtube"}
            <svg viewBox="0 0 24 24" width="26" height="26" aria-hidden="true">
              <path fill="#FF0033" d="M23.5 6.2a3 3 0 0 0-2.1-2.1C19.5 3.5 12 3.5 12 3.5s-7.5 0-9.4.6A3 3 0 0 0 .5 6.2 31.4 31.4 0 0 0 0 12a31.4 31.4 0 0 0 .5 5.8 3 3 0 0 0 2.1 2.1c1.9.6 9.4.6 9.4.6s7.5 0 9.4-.6a3 3 0 0 0 2.1-2.1A31.4 31.4 0 0 0 24 12a31.4 31.4 0 0 0-.5-5.8z"/>
              <path fill="#FFF" d="M9.6 15.6V8.4L15.9 12l-6.3 3.6z"/>
            </svg>
          {:else if brand === "google"}
            <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
              <path fill="#EA4335" d="M12 5c1.6 0 3 .6 4.1 1.6l3.1-3.1A11.8 11.8 0 0 0 12 0C7.3 0 3.3 2.7 1.3 6.6l3.7 2.9C5.9 6.9 8.7 5 12 5z"/>
              <path fill="#4285F4" d="M23.5 12.3c0-.8-.1-1.6-.2-2.3H12v4.5h6.5c-.3 1.5-1.1 2.8-2.4 3.6l3.7 2.9c2.2-2 3.7-5 3.7-8.7z"/>
              <path fill="#FBBC05" d="M5 14.5A7.2 7.2 0 0 1 4.6 12c0-.9.2-1.7.4-2.5L1.3 6.6A12 12 0 0 0 0 12c0 1.9.5 3.8 1.3 5.4L5 14.5z"/>
              <path fill="#34A853" d="M12 24c3.2 0 6-1.1 8-2.9l-3.7-2.9c-1.1.7-2.5 1.2-4.3 1.2-3.3 0-6.1-2.2-7-5.2l-3.7 2.9C3.3 21.3 7.3 24 12 24z"/>
            </svg>
          {:else if brand === "x"}
            <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
              <path fill="#F8FAFC" d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
            </svg>
          {:else if brand === "instagram"}
            <svg viewBox="0 0 24 24" width="25" height="25" aria-hidden="true">
              <defs>
                <radialGradient id="ig-ntp" cx="30%" cy="107%" r="150%">
                  <stop offset="0%" stop-color="#fdf497"/>
                  <stop offset="5%" stop-color="#fdf497"/>
                  <stop offset="45%" stop-color="#fd5949"/>
                  <stop offset="60%" stop-color="#d6249f"/>
                  <stop offset="90%" stop-color="#285AEB"/>
                </radialGradient>
              </defs>
              <rect width="22" height="22" x="1" y="1" rx="6" fill="url(#ig-ntp)"/>
              <rect width="12" height="12" x="6" y="6" rx="3.4" fill="none" stroke="#fff" stroke-width="1.8"/>
              <circle cx="12" cy="12" r="2.8" fill="none" stroke="#fff" stroke-width="1.8"/>
              <circle cx="15.8" cy="8.2" r="1" fill="#fff"/>
            </svg>
          {:else if brand === "github"}
            <svg viewBox="0 0 24 24" width="25" height="25" aria-hidden="true">
              <path fill="#F8FAFC" d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.56v-2.16c-3.2.7-3.87-1.36-3.87-1.36-.52-1.33-1.28-1.68-1.28-1.68-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.69 0-1.26.45-2.29 1.19-3.09-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.18 1.18a11 11 0 0 1 5.8 0c2.21-1.49 3.18-1.18 3.18-1.18.63 1.59.23 2.76.11 3.05.74.8 1.19 1.83 1.19 3.09 0 4.42-2.69 5.39-5.26 5.68.41.36.78 1.07.78 2.16v3.2c0 .31.21.67.8.56A11.51 11.51 0 0 0 23.5 12C23.5 5.65 18.35.5 12 .5z"/>
            </svg>
          {:else if brand === "chatgpt"}
            <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
              <path fill="#10A37F" d="M22.28 9.82a5.98 5.98 0 0 0-.52-4.91 6.05 6.05 0 0 0-6.51-2.9A6.07 6.07 0 0 0 4.98 4.18a5.98 5.98 0 0 0-4 2.9 6.05 6.05 0 0 0 .74 7.1 5.98 5.98 0 0 0 .51 4.91 6.05 6.05 0 0 0 6.52 2.9 5.98 5.98 0 0 0 4.5 2.01 6.06 6.06 0 0 0 5.77-4.2 5.98 5.98 0 0 0 4-2.9 6.06 6.06 0 0 0-.74-7.08zm-9.02 12.6a4.48 4.48 0 0 1-2.88-1.04l.14-.08 4.78-2.76a.8.8 0 0 0 .39-.68v-6.74l2.02 1.17a.07.07 0 0 1 .04.05v5.58a4.5 4.5 0 0 1-4.49 4.5zm-9.66-4.13a4.47 4.47 0 0 1-.54-3.01l.14.08 4.78 2.76a.77.77 0 0 0 .78 0l5.84-3.37v2.33a.08.08 0 0 1-.03.06L9.74 19.95a4.5 4.5 0 0 1-6.14-1.66zM2.34 7.9a4.48 4.48 0 0 1 2.37-1.97V11.6a.77.77 0 0 0 .39.68l5.81 3.35-2.02 1.17a.08.08 0 0 1-.07 0L3.99 14A4.5 4.5 0 0 1 2.34 7.9zm16.6 3.86-5.84-3.37 2.02-1.16a.08.08 0 0 1 .07 0l4.83 2.79a4.5 4.5 0 0 1-.68 8.11v-5.68a.79.79 0 0 0-.4-.69zm2.01-3.02-.14-.09-4.77-2.78a.78.78 0 0 0-.79 0L9.41 9.23V6.9a.07.07 0 0 1 .03-.06l4.83-2.79a4.5 4.5 0 0 1 6.68 4.66zM8.31 12.86 6.29 11.7a.08.08 0 0 1-.04-.06V6.07a4.5 4.5 0 0 1 7.38-3.45l-.14.08-4.78 2.76a.8.8 0 0 0-.39.68zm1.1-2.37L12 8.99l2.6 1.5v3L12 15l-2.6-1.5z"/>
            </svg>
          {:else if brand === "gemini"}
            <svg viewBox="0 0 24 24" width="25" height="25" aria-hidden="true">
              <defs>
                <linearGradient id="gem-ntp" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stop-color="#4285F4"/>
                  <stop offset="50%" stop-color="#9B72CB"/>
                  <stop offset="100%" stop-color="#D96570"/>
                </linearGradient>
              </defs>
              <path fill="url(#gem-ntp)" d="M12 1C12 7.2 7.2 12 1 12c6.2 0 11 4.8 11 11 0-6.2 4.8-11 11-11-6.2 0-11-4.8-11-11z"/>
            </svg>
          {:else if brand === "whatsapp"}
            <svg viewBox="0 0 24 24" width="25" height="25" aria-hidden="true">
              <path fill="#25D366" d="M12.04 2C6.58 2 2.13 6.45 2.13 11.91c0 1.75.46 3.45 1.32 4.95L2.05 22l5.25-1.38a9.87 9.87 0 0 0 4.74 1.2h.01c5.46 0 9.91-4.45 9.91-9.91A9.85 9.85 0 0 0 19.06 4.9 9.85 9.85 0 0 0 12.04 2zm5.79 14.04c-.24.68-1.41 1.3-1.95 1.38-.5.07-1.13.1-1.82-.12-.42-.14-.96-.31-1.65-.61-2.9-1.25-4.8-4.18-4.95-4.38-.14-.2-1.18-1.57-1.18-2.99 0-1.42.74-2.12 1.01-2.41.26-.29.58-.36.77-.36h.56c.18 0 .42-.07.65.5.24.58.82 2 .89 2.15.07.14.12.31.02.5-.1.2-.15.32-.29.49-.15.17-.31.38-.44.51-.14.15-.3.31-.13.6.17.29.76 1.25 1.63 2.02 1.12 1 2.06 1.31 2.35 1.46.29.14.46.12.63-.07.17-.2.72-.84.91-1.13.19-.29.39-.24.65-.14.27.1 1.69.8 1.98.94.29.15.48.22.55.34.07.12.07.7-.17 1.38z"/>
            </svg>
          {:else if brand === "discord"}
            <svg viewBox="0 0 24 24" width="25" height="25" aria-hidden="true">
              <path fill="#5865F2" d="M20.32 4.37a19.8 19.8 0 0 0-4.89-1.52.07.07 0 0 0-.08.04c-.21.38-.44.87-.61 1.25a18.27 18.27 0 0 0-5.49 0 12.6 12.6 0 0 0-.62-1.25.08.08 0 0 0-.08-.04A19.74 19.74 0 0 0 3.67 4.37a.07.07 0 0 0-.03.03C.53 9.05-.32 13.58.1 18.06a.08.08 0 0 0 .03.06 19.9 19.9 0 0 0 5.99 3.03.08.08 0 0 0 .08-.03c.46-.63.87-1.3 1.22-2a.08.08 0 0 0-.04-.11 13.1 13.1 0 0 1-1.87-.9.08.08 0 0 1-.01-.13c.13-.09.25-.19.37-.29a.07.07 0 0 1 .08-.01c3.93 1.79 8.18 1.79 12.06 0a.07.07 0 0 1 .08.01c.12.1.24.2.37.29a.08.08 0 0 1-.01.13c-.6.35-1.22.65-1.87.9a.08.08 0 0 0-.04.11c.36.7.77 1.37 1.22 2a.08.08 0 0 0 .08.03 19.83 19.83 0 0 0 6-3.03.08.08 0 0 0 .03-.06c.5-5.18-.84-9.68-3.55-13.66a.06.06 0 0 0-.03-.03zM8.02 15.33c-1.18 0-2.16-1.08-2.16-2.42 0-1.33.96-2.42 2.16-2.42 1.21 0 2.18 1.1 2.16 2.42 0 1.34-.96 2.42-2.16 2.42zm7.97 0c-1.18 0-2.16-1.08-2.16-2.42 0-1.33.96-2.42 2.16-2.42 1.21 0 2.18 1.1 2.16 2.42 0 1.34-.95 2.42-2.16 2.42z"/>
            </svg>
          {:else if brand === "twitch"}
            <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
              <path fill="#9146FF" d="M4.3 2 2 6v14h5v3h3l3-3h4l5-5V2H4.3zm15.7 12-3 3h-5l-3 3v-3H5V4h15v10zm-4-7h-2v5h2V7zm-5 0H9v5h2V7z"/>
            </svg>
          {:else if brand === "spotify"}
            <svg viewBox="0 0 24 24" width="25" height="25" aria-hidden="true">
              <path fill="#1DB954" d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm4.59 14.42a.62.62 0 0 1-.86.21c-2.35-1.44-5.31-1.76-8.79-.97a.62.62 0 1 1-.28-1.21c3.81-.87 7.08-.5 9.72 1.11.29.18.38.56.21.86zm1.22-2.72a.78.78 0 0 1-1.07.26c-2.69-1.65-6.79-2.13-9.97-1.16a.78.78 0 1 1-.45-1.49c3.63-1.1 8.15-.57 11.23 1.32.37.23.48.71.26 1.07zm.1-2.83c-3.22-1.91-8.54-2.09-11.62-1.15a.94.94 0 1 1-.54-1.8c3.54-1.07 9.41-.86 13.12 1.34a.94.94 0 0 1-.96 1.61z"/>
            </svg>
          {:else if brand === "reddit"}
            <svg viewBox="0 0 24 24" width="25" height="25" aria-hidden="true">
              <circle cx="12" cy="12" r="10" fill="#FF4500"/>
              <path fill="#fff" d="M18.67 12c0-.73-.59-1.33-1.33-1.33-.36 0-.68.14-.92.38-.91-.65-2.16-1.07-3.55-1.12l.6-2.84 1.97.42a.94.94 0 1 0 .14-.62l-2.21-.47a.32.32 0 0 0-.38.24l-.68 3.2c-1.42.04-2.7.46-3.63 1.12a1.32 1.32 0 0 0-.92-.38 1.33 1.33 0 0 0-.74 2.43c-.03.16-.04.33-.04.5 0 2.55 2.97 4.62 6.64 4.62s6.64-2.07 6.64-4.62c0-.17-.01-.34-.04-.5.45-.23.75-.69.75-1.03zM9.33 13a.93.93 0 1 1 1.87 0 .93.93 0 0 1-1.87 0zm5.2 2.47c-.64.64-1.87.69-2.23.69-.36 0-1.59-.05-2.23-.69a.24.24 0 0 1 .34-.34c.4.4 1.26.55 1.89.55.63 0 1.49-.15 1.89-.55a.24.24 0 0 1 .34.34zm-.17-1.54a.93.93 0 1 1 0-1.87.93.93 0 0 1 0 1.87z"/>
            </svg>
          {:else if brand === "facebook"}
            <svg viewBox="0 0 24 24" width="25" height="25" aria-hidden="true">
              <circle cx="12" cy="12" r="10" fill="#1877F2"/>
              <path fill="#fff" d="M15.6 12.8l.4-2.7h-2.6V8.4c0-.7.4-1.5 1.5-1.5h1.2V4.6s-1.1-.2-2.1-.2c-2.2 0-3.6 1.3-3.6 3.7v2H8v2.7h2.4v6.6a10 10 0 0 0 3 0v-6.6h2.2z"/>
            </svg>
          {:else if brand === "netflix"}
            <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
              <path fill="#E50914" d="M6 2h3.5l5 13.8V2H18v20h-3.4L9.5 8.1V22H6V2z"/>
            </svg>
          {:else if brand === "gmail"}
            <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
              <path fill="#EA4335" d="M20 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2zm0 4.2-8 5.3-8-5.3V6l8 5.3L20 6v2.2z"/>
            </svg>
          {:else}
            <span class="ntp-tile-monogram" style:background={accentFor(item.name)}>
              {item.name.charAt(0).toUpperCase()}
            </span>
          {/if}
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

  .ntp-tile-monogram {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 9px;
    font-size: 14px;
    font-weight: 700;
    color: #fff;
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

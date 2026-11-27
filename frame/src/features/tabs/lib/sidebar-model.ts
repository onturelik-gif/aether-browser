import type {
  SidebarNodeView,
  SidebarSectionView,
  SplitGroupView,
  TabView,
} from "$shared/ipc/bindings";
import { getTabParent } from "./tab-lineage.svelte";

export type AutoGroupMeta = {
  host: string;
  count: number;
  colorIndex: number;
  icon: TabView["icon"];
  isSubgroup?: boolean;
  treeLevel?: number;
};

export type SidebarFolderEntry = {
  kind: "folder";
  key: string;
  node: SidebarNodeView & { kind: { type: "folder"; name: string } };
  depth: number;
  groupMeta?: AutoGroupMeta;
};

export type SidebarTabEntry = {
  kind: "tab";
  key: string;
  node: SidebarNodeView & { kind: { type: "tab"; tab_id: string } };
  tab: TabView;
  depth: number;
  groupColorIndex?: number;
  groupLast?: boolean;
  isSubgroup?: boolean;
  treeLevel?: number;
  treeHasChildren?: boolean;
  treeChildCount?: number;
};

export type SidebarEntry = SidebarFolderEntry | SidebarTabEntry;

export type SidebarDisplayUnit =
  | SidebarFolderEntry
  | SidebarTabEntry
  | {
      kind: "split";
      key: string;
      tabs: SidebarTabEntry[];
      depth: number;
    };

export type SidebarTree = Record<SidebarSectionView, SidebarEntry[]>;

const EMPTY_TREE = (): SidebarTree => ({
  favorites: [],
  pinned: [],
  today: [],
});

/**
 * Admits Rust's bounded pre-order sidebar records into a renderable tree.
 *
 * Broken parent links and duplicate references are ignored. Any authoritative
 * TabView left without a valid node is appended to Today so the native
 * presentation barrier can still find exactly one row and fail independently
 * on its exact revision checks.
 */
// The last tree's tab entries, so a row whose tab, node and depth are all as
// they were keeps its entry object and the list leaves it alone.
let previousTabEntries = new Map<string, SidebarTabEntry>();

function tabEntry(
  node: SidebarTabEntry["node"],
  tab: TabView,
  depth: number,
  next: Map<string, SidebarTabEntry>,
): SidebarTabEntry {
  const key = `tab:${tab.id}`;
  const previous = previousTabEntries.get(key);
  const entry =
    previous !== undefined &&
    previous.tab === tab &&
    previous.depth === depth &&
    previous.node.id === node.id &&
    previous.node.parent_id === node.parent_id &&
    previous.node.section === node.section
      ? previous
      : { kind: "tab" as const, key, node, tab, depth };
  next.set(key, entry);
  return entry;
}

export function sidebarTree(
  nodes: readonly SidebarNodeView[],
  tabs: readonly TabView[],
): SidebarTree {
  const tree = EMPTY_TREE();
  const entries = new Map<string, SidebarTabEntry>();
  const tabsById = new Map(tabs.map((tab) => [tab.id, tab] as const));
  const admittedTabs = new Set<string>();
  const admittedNodes = new Map<
    string,
    { depth: number; section: SidebarSectionView; folder: boolean }
  >();

  for (const node of nodes) {
    if (admittedNodes.has(node.id)) continue;

    let depth = 0;
    if (node.parent_id !== null) {
      const parent = admittedNodes.get(node.parent_id);
      if (parent === undefined || !parent.folder || parent.section !== node.section) continue;
      depth = parent.depth + 1;
      if (depth > 64) continue;
    }

    if (node.kind.type === "folder") {
      const entry: SidebarFolderEntry = {
        kind: "folder",
        key: `folder:${node.id}`,
        node: node as SidebarFolderEntry["node"],
        depth,
      };
      admittedNodes.set(node.id, { depth, section: node.section, folder: true });
      tree[node.section].push(entry);
      continue;
    }

    const tab = tabsById.get(node.kind.tab_id);
    if (tab === undefined || admittedTabs.has(tab.id)) continue;
    const entry = tabEntry(node as SidebarTabEntry["node"], tab, depth, entries);
    admittedNodes.set(node.id, { depth, section: node.section, folder: false });
    admittedTabs.add(tab.id);
    tree[node.section].push(entry);
  }

  for (const tab of tabs) {
    if (admittedTabs.has(tab.id)) continue;
    const node: SidebarTabEntry["node"] = {
      id: tab.id,
      parent_id: null,
      section: "today",
      kind: { type: "tab", tab_id: tab.id },
    };
    tree.today.push(tabEntry(node, tab, 0, entries));
  }

  previousTabEntries = entries;
  return tree;
}

/**
 * Groups one section only when every authoritative split member is present.
 * Cross-section and malformed groups stay as independent rows; this never
 * clones or removes a presentation sentinel.
 */
function siteGroupHost(url: string | null | undefined): string | null {
  if (!url) return null;
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
    const host = parsed.hostname.toLowerCase().replace(/^www\./, "");
    return host || null;
  } catch {
    return null;
  }
}

function hashHostColor(host: string): number {
  let h = 2166136261;
  for (let i = 0; i < host.length; i++) {
    h ^= host.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return Math.abs(h) % 6;
}

function friendlyGroupName(host: string): string {
  const known: Record<string, string> = {
    "youtube.com": "YouTube",
    "youtu.be": "YouTube",
    "google.com": "Google",
    "x.com": "X",
    "twitter.com": "X",
    "instagram.com": "Instagram",
    "facebook.com": "Facebook",
    "github.com": "GitHub",
    "reddit.com": "Reddit",
    "spotify.com": "Spotify",
    "discord.com": "Discord",
    "twitch.tv": "Twitch",
    "chatgpt.com": "ChatGPT",
    "claude.ai": "Claude",
    "linkedin.com": "LinkedIn",
    "wikipedia.org": "Wikipedia",
  };
  if (known[host]) return known[host]!;
  for (const [domain, label] of Object.entries(known)) {
    if (host.endsWith(`.${domain}`)) return label;
  }
  const base = host.split(".")[0] ?? host;
  return base.charAt(0).toUpperCase() + base.slice(1);
}

/**
 * Organizes Today tabs as a clean direct parent-child tree:
 *   YouTube        ⌄
 *     YouTube
 *     YouTube      ⌄
 *       GitHub
 * Every tab opened from another tab (middle-click / link) nests directly
 * beneath its opener tab, indented right per level — with no extra synthetic
 * folder rows in between.
 */
function autoGroupBySite(units: SidebarDisplayUnit[]): SidebarDisplayUnit[] {
  const tabById = new Map<string, SidebarTabEntry>();
  for (const unit of units) {
    if (unit.kind === "tab" && unit.depth === 0) tabById.set(unit.tab.id, unit);
  }

  // Build direct parent -> children map.
  // If a tab's opener is not in Today (e.g. opened from a pinned Essential),
  // group siblings opened from that same pinned opener under the first sibling.
  const childrenByParent = new Map<string, SidebarTabEntry[]>();
  const claimedChildIds = new Set<string>();
  const externalOpenerLead = new Map<string, string>();

  for (const unit of units) {
    if (unit.kind !== "tab" || unit.depth !== 0) continue;
    const rawParent = getTabParent(unit.tab.id);
    if (!rawParent || rawParent === unit.tab.id) continue;

    let effectiveParent: string | null = null;
    if (tabById.has(rawParent)) {
      effectiveParent = rawParent;
    } else {
      const lead = externalOpenerLead.get(rawParent);
      if (lead && lead !== unit.tab.id && tabById.has(lead)) {
        effectiveParent = lead;
      } else {
        externalOpenerLead.set(rawParent, unit.tab.id);
      }
    }

    if (effectiveParent) {
      let list = childrenByParent.get(effectiveParent);
      if (!list) {
        list = [];
        childrenByParent.set(effectiveParent, list);
      }
      list.push(unit);
      claimedChildIds.add(unit.tab.id);
    }
  }

  function countDescendants(id: string, seen = new Set<string>()): number {
    if (seen.has(id)) return 0;
    seen.add(id);
    const kids = childrenByParent.get(id);
    if (!kids || kids.length === 0) return 0;
    let total = 0;
    for (const kid of kids) {
      total += 1 + countDescendants(kid.tab.id, seen);
    }
    return total;
  }

  const visited = new Set<string>();
  const output: SidebarDisplayUnit[] = [];

  function emitTree(entry: SidebarTabEntry, level: number, colorIndex: number) {
    if (visited.has(entry.tab.id)) return;
    visited.add(entry.tab.id);
    const clampedLevel = Math.min(level, 4);
    const kids = (childrenByParent.get(entry.tab.id) ?? []).filter((k) => !visited.has(k.tab.id));
    const descendants = countDescendants(entry.tab.id);

    output.push({
      ...entry,
      depth: clampedLevel,
      treeLevel: clampedLevel,
      treeHasChildren: kids.length > 0,
      treeChildCount: descendants,
      groupColorIndex: colorIndex,
      isSubgroup: clampedLevel > 0,
    });

    for (const kid of kids) {
      emitTree(kid, clampedLevel + 1, colorIndex);
    }
  }

  for (const unit of units) {
    if (unit.kind === "tab" && unit.depth === 0) {
      if (claimedChildIds.has(unit.tab.id) || visited.has(unit.tab.id)) continue;
      if (childrenByParent.has(unit.tab.id)) {
        const host = siteGroupHost(unit.tab.url) ?? unit.tab.id;
        const colorIndex = hashHostColor(host);
        const totalCount = 1 + countDescendants(unit.tab.id);
        output.push({
          kind: "folder",
          key: `autogroup:tree:${unit.tab.id}`,
          node: {
            id: `autogroup:tree:${unit.tab.id}`,
            parent_id: null,
            section: "today",
            kind: { type: "folder", name: friendlyGroupName(host) },
          },
          depth: 0,
          groupMeta: {
            host,
            count: totalCount,
            colorIndex,
            icon: unit.tab.icon,
            treeLevel: 0,
          },
        });
        const startIdx = output.length;
        emitTree(unit, 1, colorIndex);
        // Mark the very last emitted row in this tree family so its bottom corners round cleanly.
        const last = output[output.length - 1];
        if (output.length > startIdx && last && last.kind === "tab") {
          output[output.length - 1] = { ...last, groupLast: true };
        }
        continue;
      }
      visited.add(unit.tab.id);
    }
    output.push(unit);
  }

  return output;
}

export function sidebarDisplayUnits(
  section: SidebarSectionView,
  entries: readonly SidebarEntry[],
  splitGroup: SplitGroupView | null,
): SidebarDisplayUnit[] {
  let base: SidebarDisplayUnit[];
  if (splitGroup === null || splitGroup.members.length < 2) {
    base = [...entries];
  } else {
    const positions = new Map<string, { index: number; entry: SidebarTabEntry }>();
    entries.forEach((entry, index) => {
      if (entry.kind === "tab") positions.set(entry.tab.id, { index, entry });
    });

    const ids = new Set<string>();
    const members: SidebarTabEntry[] = [];
    let insertionIndex = entries.length;
    let valid = true;
    for (const id of splitGroup.members) {
      const member = positions.get(id);
      if (member === undefined || ids.has(id)) {
        valid = false;
        break;
      }
      ids.add(id);
      members.push(member.entry);
      insertionIndex = Math.min(insertionIndex, member.index);
    }

    if (!valid) {
      base = [...entries];
    } else {
      const output: SidebarDisplayUnit[] = [];
      entries.forEach((entry, index) => {
        if (index === insertionIndex) {
          output.push({
            kind: "split",
            key: `split:${section}`,
            tabs: members,
            depth: Math.min(...members.map((member) => member.depth)),
          });
        }
        if (entry.kind !== "tab" || !ids.has(entry.tab.id)) output.push(entry);
      });
      base = output;
    }
  }

  return section === "today" ? autoGroupBySite(base) : base;
}

/** Keep the real active tab visible even when its parent was folded. */
export function collapsedSidebarUnits(
  units: readonly SidebarDisplayUnit[],
  folded: ReadonlySet<string>,
  activeId: string | null,
): Set<string> {
  const hidden = new Set<string>();
  const ancestors: number[] = [];
  for (let index = 0; index < units.length; index++) {
    const unit = units[index]!;
    while (ancestors.length && ancestors[ancestors.length - 1]! >= unit.depth) ancestors.pop();
    if (ancestors.length) hidden.add(unit.key);
    if (unit.kind === "tab" && unit.treeHasChildren && folded.has(unit.key)) {
      ancestors.push(unit.depth);
      continue;
    }
    if (unit.kind !== "folder" || !folded.has(unit.key)) continue;
    // Auto-groups always collapse cleanly when clicked so the user can hide
    // and reveal the group at will, even if a tab inside it is active.
    if (unit.groupMeta) {
      ancestors.push(unit.depth);
      continue;
    }
    let hasActive = false;
    for (let childIndex = index + 1; childIndex < units.length; childIndex++) {
      const child = units[childIndex]!;
      if (child.depth <= unit.depth) break;
      if (
        (child.kind === "tab" && child.tab.id === activeId) ||
        (child.kind === "split" && child.tabs.some((entry) => entry.tab.id === activeId))
      ) {
        hasActive = true;
        break;
      }
    }
    if (!hasActive) ancestors.push(unit.depth);
  }
  return hidden;
}

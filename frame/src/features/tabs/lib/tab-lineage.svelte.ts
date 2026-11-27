import { SvelteMap } from "svelte/reactivity";

/** Tracks direct opener lineage (`childId -> parentId`) so nested groups form a true tree. */
const parentByChild = new SvelteMap<string, string>();

export function recordTabLineage(parentId: string, childId: string) {
  if (!parentId || !childId || parentId === childId) return;
  // Guard against cycles while preserving direct parent-child depth.
  let cursor: string | undefined = parentId;
  const seen = new Set<string>([childId]);
  while (cursor) {
    if (seen.has(cursor)) return;
    seen.add(cursor);
    cursor = parentByChild.get(cursor);
  }
  parentByChild.set(childId, parentId);
}

export function getTabParent(childId: string): string | undefined {
  return parentByChild.get(childId);
}

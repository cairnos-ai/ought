import { writable, derived } from "svelte/store";
import type { ApiResponse, Spec, Section, Clause } from "./types";

export const data = writable<ApiResponse | null>(null);
export const activeSpecIndex = writable<number>(0);
export const searchQuery = writable<string>("");
export const activeFilter = writable<string | null>(null);

export const activeSpec = derived(
  [data, activeSpecIndex],
  ([$data, $idx]) => $data?.specs[$idx] ?? null
);

export async function loadData() {
  const res = await fetch("/api/specs");
  const json: ApiResponse = await res.json();
  data.set(json);
}

/** Count total clauses (including otherwise) in a section tree */
export function countClauses(sections: Section[]): number {
  let n = 0;
  for (const s of sections) {
    n += s.clauses.length;
    for (const c of s.clauses) {
      n += c.otherwise.length;
    }
    n += countClauses(s.subsections);
  }
  return n;
}

/** Filter clauses by search query and keyword filter */
export function filterClauses(
  clauses: Clause[],
  query: string,
  filter: string | null
): Clause[] {
  return clauses.filter((c) => {
    if (filter && c.keyword !== filter) return false;
    if (query) {
      const q = query.toLowerCase();
      return (
        c.text.toLowerCase().includes(q) ||
        c.id.toLowerCase().includes(q) ||
        (c.condition ?? "").toLowerCase().includes(q)
      );
    }
    return true;
  });
}

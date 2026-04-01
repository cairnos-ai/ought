<script lang="ts">
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import ChevronRight from "lucide-svelte/icons/chevron-right";
  import FileText from "lucide-svelte/icons/file-text";
  import { data, activeSpecIndex, countClauses } from "$lib/stores.js";
  import type { Section } from "$lib/types.js";

  let expandedSpecs = $state<Set<number>>(new Set([0]));

  function selectSpec(idx: number) {
    activeSpecIndex.set(idx);
    expandedSpecs.add(idx);
    expandedSpecs = new Set(expandedSpecs);
  }

  function toggleExpand(idx: number) {
    if (expandedSpecs.has(idx)) {
      expandedSpecs.delete(idx);
    } else {
      expandedSpecs.add(idx);
    }
    expandedSpecs = new Set(expandedSpecs);
  }

  function scrollToSection(title: string) {
    const id = "sec-" + title.replace(/\s+/g, "-");
    const el = document.getElementById(id);
    if (el) el.scrollIntoView({ behavior: "smooth" });
  }
</script>

<ScrollArea
  class="w-[260px] min-w-[220px] bg-[var(--card)] border-r shrink-0 py-2 hidden md:block"
>
  {#if $data}
    {#each $data.specs as spec, si (spec.name)}
      {@const isActive = $activeSpecIndex === si}
      {@const isExpanded = expandedSpecs.has(si)}
      {@const cc = countClauses(spec.sections)}

      <button
        class="w-full flex items-center gap-1.5 py-1.5 px-3 mx-2 cursor-pointer text-sm rounded-md border-0 bg-transparent text-inherit text-left transition-colors
          {isActive
          ? 'bg-[var(--accent)] font-medium'
          : 'hover:bg-[var(--accent)]'}"
        style="width: calc(100% - 16px);"
        onclick={() => {
          selectSpec(si);
          toggleExpand(si);
        }}
      >
        <span
          class="inline-flex transition-transform shrink-0 {isExpanded
            ? 'rotate-90'
            : ''}"
        >
          <ChevronRight class="h-3 w-3 text-[var(--muted-foreground)]" />
        </span>
        <FileText class="h-3.5 w-3.5 shrink-0 opacity-50" />
        <span class="truncate flex-1">{spec.name}</span>
        <span
          class="text-[11px] text-[var(--muted-foreground)] tabular-nums shrink-0"
          >{cc}</span
        >
      </button>

      {#if isExpanded}
        <div class="pl-8">
          {#each spec.sections as section (section.title)}
            <button
              class="w-full text-left text-xs text-[var(--muted-foreground)] py-1 px-3 mx-2 cursor-pointer rounded-md border-0 bg-transparent hover:bg-[var(--accent)] transition-colors truncate"
              style="width: calc(100% - 16px);"
              onclick={() => {
                selectSpec(si);
                scrollToSection(section.title);
              }}
            >
              {section.title}
            </button>
          {/each}
        </div>
      {/if}
    {/each}
  {/if}
</ScrollArea>

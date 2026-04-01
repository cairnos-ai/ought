<script lang="ts">
  import KeywordBadge from "./KeywordBadge.svelte";
  import type { Clause } from "$lib/types.js";

  interface Props {
    clause: Clause;
  }

  let { clause }: Props = $props();
</script>

<div class="flex gap-3 py-2 border-b border-[var(--border)]/50 items-baseline">
  <KeywordBadge keyword={clause.keyword} />
  <div class="flex-1">
    {#if clause.condition}
      <div class="text-xs text-sky-400/80 font-mono mb-1">
        GIVEN: {clause.condition}
      </div>
    {/if}
    <p class="font-serif text-[16px] leading-relaxed">
      {clause.text}
    </p>
    {#if clause.temporal}
      <span
        class="inline-block mt-1 text-[10px] font-semibold px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 border border-emerald-500/20"
      >
        {clause.temporal.kind === "invariant"
          ? "INVARIANT"
          : clause.temporal.duration}
      </span>
    {/if}
    {#if clause.hints.length > 0}
      {#each clause.hints as hint}
        <pre
          class="mt-2 p-3 text-xs bg-[var(--muted)] rounded-md border font-mono overflow-x-auto">{hint}</pre>
      {/each}
    {/if}
  </div>
</div>

{#if clause.otherwise.length > 0}
  <div class="ml-8 border-l border-dashed border-[var(--border)] pl-3">
    {#each clause.otherwise as ow}
      <div class="flex gap-3 py-2 items-baseline opacity-80">
        <KeywordBadge keyword={ow.keyword} />
        <p class="font-serif text-[16px] leading-relaxed">{ow.text}</p>
      </div>
    {/each}
  </div>
{/if}

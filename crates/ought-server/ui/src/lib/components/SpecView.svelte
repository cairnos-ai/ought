<script lang="ts">
  import SectionCard from "./SectionCard.svelte";
  import { activeSpec } from "$lib/stores.js";
</script>

{#if $activeSpec}
  <div>
    <h2 class="font-display text-[22px] font-bold mb-0.5 tracking-wide">
      {$activeSpec.name}
    </h2>
    <p class="text-[11px] text-[var(--muted-foreground)] mb-5 font-mono">
      {$activeSpec.source_path}
    </p>

    <!-- Metadata block -->
    {#if $activeSpec.metadata.context || $activeSpec.metadata.sources.length > 0 || $activeSpec.metadata.schemas.length > 0 || $activeSpec.metadata.requires.length > 0}
      <div
        class="bg-[var(--muted)] border rounded-lg p-3 px-4 mb-6 text-[13px] text-[var(--muted-foreground)]"
      >
        {#if $activeSpec.metadata.context}
          <div class="mb-1">
            <span class="font-medium text-[var(--foreground)]">Context:</span>
            {$activeSpec.metadata.context}
          </div>
        {/if}
        {#if $activeSpec.metadata.sources.length > 0}
          <div class="mb-1">
            <span class="font-medium text-[var(--foreground)]">Sources:</span>
            {$activeSpec.metadata.sources.join(", ")}
          </div>
        {/if}
        {#if $activeSpec.metadata.schemas.length > 0}
          <div class="mb-1">
            <span class="font-medium text-[var(--foreground)]">Schemas:</span>
            {$activeSpec.metadata.schemas.join(", ")}
          </div>
        {/if}
        {#if $activeSpec.metadata.requires.length > 0}
          <div>
            <span class="font-medium text-[var(--foreground)]">Requires:</span>
            {$activeSpec.metadata.requires
              .map((r) => r.label || r.path)
              .join(", ")}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Sections -->
    {#each $activeSpec.sections as section (section.title)}
      <div id="sec-{section.title.replace(/\s+/g, '-')}">
        <SectionCard {section} />
      </div>
    {/each}
  </div>
{:else}
  <p class="text-[var(--muted-foreground)] p-10">Select a spec</p>
{/if}

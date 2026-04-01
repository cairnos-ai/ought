<script lang="ts">
  import { cn } from "$lib/utils/index.js";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  interface Props extends HTMLAttributes<HTMLParagraphElement> {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className, ...rest }: Props = $props();
</script>

<p class={cn("text-sm text-[var(--muted-foreground)]", className)} {...rest}>
  {#if children}{@render children()}{/if}
</p>

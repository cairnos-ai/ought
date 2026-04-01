<script lang="ts">
  import { cn } from "$lib/utils/index.js";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className, ...rest }: Props = $props();
</script>

<div class={cn("flex flex-col space-y-1.5 p-6", className)} {...rest}>
  {#if children}{@render children()}{/if}
</div>

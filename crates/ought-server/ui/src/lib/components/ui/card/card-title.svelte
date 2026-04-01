<script lang="ts">
  import { cn } from "$lib/utils/index.js";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  interface Props extends HTMLAttributes<HTMLHeadingElement> {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className, ...rest }: Props = $props();
</script>

<h3 class={cn("text-2xl font-semibold leading-none tracking-tight", className)} {...rest}>
  {#if children}{@render children()}{/if}
</h3>

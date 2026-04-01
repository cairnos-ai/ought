<script lang="ts">
  import { cn } from "$lib/utils/index.js";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    variant?: "default" | "secondary" | "destructive" | "outline";
    children?: Snippet;
    class?: string;
  }

  let { variant = "default", children, class: className, ...rest }: Props = $props();

  const variantClasses: Record<string, string> = {
    default: "border-transparent bg-[var(--primary)] text-[var(--primary-foreground)]",
    secondary: "border-transparent bg-[var(--secondary)] text-[var(--secondary-foreground)]",
    destructive: "border-transparent bg-[var(--destructive)] text-[var(--destructive-foreground)]",
    outline: "text-[var(--foreground)]",
  };
</script>

<div
  class={cn(
    "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors",
    variantClasses[variant],
    className
  )}
  {...rest}
>
  {#if children}{@render children()}{/if}
</div>

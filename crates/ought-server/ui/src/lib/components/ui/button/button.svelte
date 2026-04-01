<script lang="ts">
  import { cn } from "$lib/utils/index.js";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  interface Props extends HTMLButtonAttributes {
    variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
    size?: "default" | "sm" | "lg" | "icon";
    children?: Snippet;
    class?: string;
  }

  let { variant = "default", size = "default", children, class: className, ...rest }: Props = $props();

  const variantClasses: Record<string, string> = {
    default: "bg-[var(--primary)] text-[var(--primary-foreground)] hover:bg-[var(--primary)]/90",
    destructive: "bg-[var(--destructive)] text-[var(--destructive-foreground)] hover:bg-[var(--destructive)]/90",
    outline: "border border-[var(--input)] bg-transparent hover:bg-[var(--accent)] hover:text-[var(--accent-foreground)]",
    secondary: "bg-[var(--secondary)] text-[var(--secondary-foreground)] hover:bg-[var(--secondary)]/80",
    ghost: "hover:bg-[var(--accent)] hover:text-[var(--accent-foreground)]",
    link: "text-[var(--primary)] underline-offset-4 hover:underline",
  };

  const sizeClasses: Record<string, string> = {
    default: "h-10 px-4 py-2",
    sm: "h-9 rounded-md px-3",
    lg: "h-11 rounded-md px-8",
    icon: "h-10 w-10",
  };
</script>

<button
  class={cn(
    "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-[var(--background)] transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ring)] focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 cursor-pointer",
    variantClasses[variant],
    sizeClasses[size],
    className
  )}
  {...rest}
>
  {#if children}{@render children()}{/if}
</button>

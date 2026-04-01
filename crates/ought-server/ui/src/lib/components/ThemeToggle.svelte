<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import Sun from "lucide-svelte/icons/sun";
  import Moon from "lucide-svelte/icons/moon";

  let dark = $state(true);

  function toggle() {
    dark = !dark;
    document.documentElement.classList.toggle("dark", dark);
    localStorage.setItem("ought-theme", dark ? "dark" : "light");
  }

  $effect(() => {
    const saved = localStorage.getItem("ought-theme");
    dark = saved !== "light";
    document.documentElement.classList.toggle("dark", dark);
  });
</script>

<Button variant="outline" size="icon" onclick={toggle} class="h-8 w-8">
  {#if dark}
    <Sun class="h-4 w-4" />
  {:else}
    <Moon class="h-4 w-4" />
  {/if}
</Button>

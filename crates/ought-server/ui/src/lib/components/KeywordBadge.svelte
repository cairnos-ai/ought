<script lang="ts">
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { KW_LABELS } from "$lib/types.js";

  interface Props {
    keyword: string;
  }

  let { keyword }: Props = $props();

  const label = $derived(KW_LABELS[keyword] ?? keyword);

  const colorClass: Record<string, string> = {
    Must: "bg-red-500/15 text-red-500 border-red-500/20",
    MustNot: "bg-red-500/15 text-red-500 border-red-500/20",
    MustAlways: "bg-red-500/15 text-red-500 border-red-500/20",
    MustBy: "bg-red-500/15 text-red-500 border-red-500/20",
    Should: "bg-amber-500/15 text-amber-500 border-amber-500/20",
    ShouldNot: "bg-amber-500/15 text-amber-500 border-amber-500/20",
    May: "bg-zinc-500/15 text-zinc-400 border-zinc-500/20",
    Wont: "bg-violet-500/15 text-violet-400 border-violet-500/20",
    Given: "bg-sky-500/15 text-sky-400 border-sky-500/20",
    Otherwise: "bg-orange-500/15 text-orange-400 border-orange-500/20",
  };
</script>

<Badge
  variant="outline"
  class="text-[10px] font-semibold uppercase tracking-wider rounded-md min-w-[48px] justify-center {colorClass[keyword] ?? ''}"
>
  {label}
</Badge>

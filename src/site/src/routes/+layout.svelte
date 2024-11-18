<script lang="ts">
    import "../app.css";
    import type {Snippet} from "svelte";
    import NavBar from "$lib/NavBar.svelte";

    interface Props {
        children?: Snippet;
    }

    let {children}: Props = $props();
</script>

<NavBar/>

{@render children?.()}
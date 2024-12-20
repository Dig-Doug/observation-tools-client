<script lang="ts">
    import type {PageData} from './$houdini';

    export let data: PageData;
</script>


<main>
    <div class="p-4">
        <h1 class="font-bold text-3xl">
            {data.name}
        </h1>
        <p>
            {data.node.id}
        </p>
    </div>
    <div class="grid grid-cols-[300px_1fr]">
        <ul class="border-r">
            {#each data.children as child}
                <li>
                    <a class="hover:bg-base-200 p-4 grid grid-cols-2" href={`/artifact/${child.id}`}>
                        <p class="font-bold text-sm">
                            {child.name}
                        </p>
                    </a>
                </li>
            {/each}
        </ul>

        <pre class="text-wrap">
                    {data.payload || "<no payload>"}
                </pre>
    </div>
</main>
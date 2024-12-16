<script lang="ts">
    import type {PageData} from './$houdini';

    interface Props {
        data: PageData;
    }

    let {data}: Props = $props();
</script>

<main class="min-h-screen">
    <div class="hero min-h-96">
        <div class="hero-content">
            <div class="max-w-5xl">
                <div class="py-4">
                    <h1 class="font-bold text-3xl">
                        {data.name || "<unnamed-project>"}
                    </h1>
                    <h2>
                        {data.id}
                    </h2>
                </div>

                <ul class="border rounded-lg">
                    {#each data.runs as run}
                        <li>
                            <a class="hover:bg-base-200 p-4 grid grid-cols-2" href={`/artifact/${run.id}`}>
                                <p class="font-bold text-sm">
                                    {run.name}
                                </p>
                                <p>
                                    {run.clientCreationTime}
                                </p>
                                <p class="text-xs col-span-2 pt-1">
                                    {run.id}
                                </p>
                            </a>
                        </li>
                    {/each}
                </ul>
            </div>
        </div>
    </div>
</main>

<pre>
{JSON.stringify(data, null, 2)}
</pre>

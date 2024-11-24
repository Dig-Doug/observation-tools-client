<script lang="ts">
    import type {PageData} from './$houdini';
    import {QueryParam} from "$lib";
    import {page} from '$app/stores';
    import Pagination from "$lib/Pagination.svelte";
    import type {ActionData} from "./$types";
    import {enhance} from '$app/forms';
    import MdiError from '~icons/mdi/error';

    interface Props {
        data: PageData;
        form: ActionData;
    }

    let {data, form}: Props = $props();

    function buildUrl(cursor: string | null, paramToSet: string, paramToDelete: string): URL {
        let url = new URL($page.url);
        url.searchParams.delete(paramToDelete);
        if (cursor) {
            url.searchParams.set(paramToSet, cursor);
        }
        return url;
    }

    let pageInfo = $derived(data.pageInfo);
    let previousPageUrl = $derived(
        pageInfo?.hasPreviousPage
            ? buildUrl(pageInfo.startCursor, QueryParam.Before, QueryParam.After)
            : null
    );
    let nextPageUrl = $derived(
        pageInfo?.hasNextPage ? buildUrl(pageInfo.endCursor, QueryParam.After, QueryParam.Before) : null
    );

    let creating = $state(false);
</script>

<h1>
    Projects
</h1>

<form method="POST" use:enhance={() => {
		creating = true;

		return async ({ update }) => {
			await update();
			creating = false;
		};
	}}>
    <button type="submit" class="btn btn-outline" class:btn-error={!!form?.error} disabled={creating}>
        {#if creating}
            <span class="loading loading-spinner"></span>
        {:else if form?.error}
            <MdiError/>
        {/if}
        Create Project
    </button>
</form>


<ul>
    {#each data.edges as edge}
        <li>
            <a class="link" href={`/project/${edge.node.id}`}>
                Project  {edge.node.id}
            </a>
        </li>
    {/each}
</ul>

{#if previousPageUrl || nextPageUrl}
    <div class="pagination p-4">
        <Pagination {previousPageUrl} {nextPageUrl}/>
    </div>
{/if}

<pre>
{JSON.stringify(data, null, 2)}
</pre>


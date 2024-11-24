<script lang="ts">
    import type {PageData} from './$houdini';

    interface Props {
        data: PageData;
    }

    let {data}: Props = $props();
</script>

<h1>Project</h1>

<pre>
{JSON.stringify(data, null, 2)}
</pre>

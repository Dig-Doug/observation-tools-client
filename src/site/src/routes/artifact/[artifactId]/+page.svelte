<script lang="ts">
    import type {PageData} from './$houdini';

    export let data: PageData;
</script>


<h1>
    {data.name}
</h1>

<pre>
    {data.payload}
</pre>

<pre>
{JSON.stringify(data, null, 2)}
</pre>
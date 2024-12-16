<script lang="ts">
    import type {PageData} from './$houdini';
    import 'diff2html/bundles/css/diff2html.min.css';

    export let data: PageData;
</script>

<style>
    :global(.d2h-code-line), :global(.d2h-code-line-ctn) {
        white-space: normal;
    }
</style>

{@html data.diffHtml}

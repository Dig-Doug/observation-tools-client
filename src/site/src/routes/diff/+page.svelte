<script lang="ts">
    import type {PageData} from './$houdini';
    import 'diff2html/bundles/css/diff2html.min.css';

    export let data: PageData;
</script>

{@html data.diffHtml}

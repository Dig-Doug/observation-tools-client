import type {PageServerLoad, PageServerLoadEvent} from './$types';
import {ArtifactQueryStore} from '$houdini';
import {QueryParam} from "../../../lib";

export const load: PageServerLoad = async (event: PageServerLoadEvent) => {
    const graphqlQuery = new ArtifactQueryStore();
    const selectedId = event.url.searchParams.get(QueryParam.SelectedArtifact);
    const {data, errors} = await graphqlQuery.fetch({
        event,
        variables: {
            id: event.params.artifactId,
            fetchSelected: !!selectedId,
            selectedId : selectedId || "",
        }
    });
    if (!data) {
        throw new Error(`No data: ${JSON.stringify(errors)}`);
    }
    if (data.node.__typename !== 'ArtifactVersion') {
        throw new Error(`Invalid typename: ${data.node.__typename}`);
    }
    if (data.selectedNode && data.selectedNode.__typename !== 'ArtifactVersion') {
        throw new Error(`Invalid typename: ${data.selectedNode.__typename}`);
    }
    const children = data.node.children.edges.map((edge) => edge.node).filter((node) =>
        node.__typename === 'ArtifactVersion');
    return {
        node: data.node,
        children,
        name: data.node.name,
        payload: selectedId ? data.selectedNode?.payload : data.node.payload,
    };
};

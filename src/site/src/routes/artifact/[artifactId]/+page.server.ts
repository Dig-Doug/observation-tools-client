import type {PageServerLoad, PageServerLoadEvent} from './$types';
import {ArtifactQueryStore} from '$houdini';

export const load: PageServerLoad = async (event: PageServerLoadEvent) => {
    const graphqlQuery = new ArtifactQueryStore();
    const {data, errors} = await graphqlQuery.fetch({
        event,
        variables: {
            id: event.params.artifactId
        }
    });
    if (!data) {
        throw new Error(`No data: ${JSON.stringify(errors)}`);
    }
    if (data.node.__typename !== 'ArtifactVersion') {
        throw new Error(`Invalid typename: ${data.node.__typename}`);
    }
    const children = data.node.children.edges.map((edge) => edge.node).filter((node) =>
        node.__typename === 'ArtifactVersion');
    return {
        node: data.node,
        children,
        name: data.node.name,
        payload: data.node.payload,
    };
};

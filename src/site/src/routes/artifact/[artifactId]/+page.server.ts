import type { PageServerLoad, PageServerLoadEvent } from './$types';
import { ArtifactQueryStore } from '$houdini';

export const load: PageServerLoad = async (event: PageServerLoadEvent) => {
	const graphqlQuery = new ArtifactQueryStore();
	const { data, errors } = await graphqlQuery.fetch({
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
	return {
		node: data.node
	};
};

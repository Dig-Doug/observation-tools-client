import type { PageServerLoad, PageServerLoadEvent } from './$types';
import { RunsQueryStore } from '$houdini';
import { QueryParam } from '$lib';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async (event: PageServerLoadEvent) => {
	const { url, params } = event;
	const graphqlQuery = new RunsQueryStore();
	const after = url.searchParams.get(QueryParam.After);
	const before = url.searchParams.get(QueryParam.Before);
	const { data, errors } = await graphqlQuery.fetch({
		event,
		variables: {
			projectId: params.projectId,
			after: after,
			before: before,
			first: !before ? 10 : null,
			last: before ? 10 : null
		}
	});
	if (!data) {
		throw new Error(`No data: ${JSON.stringify(errors)}`);
	}
	return {
		...data.node
	};
};

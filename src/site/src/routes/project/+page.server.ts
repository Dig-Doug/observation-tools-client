import type { PageServerLoad, PageServerLoadEvent } from './$types';
import { ProjectsQueryStore, CreateProjectStore } from '$houdini';
import { QueryParam } from '$lib';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async (event: PageServerLoadEvent) => {
	const { url } = event;
	const graphqlQuery = new ProjectsQueryStore();
	const after = url.searchParams.get(QueryParam.After);
	const before = url.searchParams.get(QueryParam.Before);
	const { data, errors } = await graphqlQuery.fetch({
		event,
		variables: {
			after: after,
			before: before,
			first: !before ? 5 : null,
			last: before ? 5 : null
		}
	});
	if (!data) {
		throw new Error(`No data: ${JSON.stringify(errors)}`);
	}
	return {
		...data.getProjects
	};
};

export const actions = {
	default: async (event) => {
		let projectId;
		try {
			const graphqlMutation = new CreateProjectStore();
			const { data, errors } = await graphqlMutation.mutate(null, { event });
			if (!data || errors) {
				throw new Error(`No data: ${JSON.stringify(errors)}`);
			}
			projectId = data.createProject.id;
		} catch (e) {
			console.error(e);
			return fail(422, {
				error: 'Error creating project'
			});
		}
		return redirect(303, `/project/${projectId}`);
	}
};

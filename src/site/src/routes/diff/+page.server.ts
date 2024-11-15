import type { PageServerLoad, PageServerLoadEvent } from './$types';
import { DiffQueryStore } from '$houdini';
import * as Diff2html from 'diff2html';

export const load: PageServerLoad = async (event: PageServerLoadEvent) => {
	const graphqlQuery = new DiffQueryStore();
	const { data, errors } = await graphqlQuery.fetch({
		event,
		variables: {
			left: event.url.searchParams.get('left'),
			right: event.url.searchParams.get('right')
		}
	});
	if (!data) {
		throw new Error(`No data: ${JSON.stringify(errors)}`);
	}
	const diffJson = Diff2html.parse(data.diffArtifacts);
	const diffHtml = Diff2html.html(diffJson, {
		outputFormat: 'side-by-side',
		drawFileList: false
	});
	return {
		diffHtml
	};
};

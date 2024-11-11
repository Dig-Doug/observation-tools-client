import { HoudiniClient } from '$houdini';
import { env } from '$env/dynamic/public';

export default new HoudiniClient({
	url: env.PUBLIC_GRAPHQL_ENDPOINT
});

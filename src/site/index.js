import { handler } from './build/handler.js';
import express from 'express';
import compression from 'compression';

const port = process.env.PORT || 3000;
const app = express();
app.set('trust proxy', true);
app
	.use(compression({}))
	.use(handler)
	.listen(port, () => {
		console.log(`Listening on port http://0.0.0.0:${port}`);
	});

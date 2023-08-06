import {run_examples_js} from "./pkg/observation_tools_client_examples.js";
import parse from "minimist";

const argv = parse(process.argv.slice(2));
await run_examples_js(argv["project-id"], argv["auth-token"], argv["ui-host"], argv["api-host"]);
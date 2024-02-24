import {Client, UserMetadataBuilder, Point2Builder, Object2Builder, Transform2Builder} from "@observation-tools/client";
import parse from "minimist";

const argv = parse(process.argv.slice(2));

const client = new Client(undefined, undefined, argv["project-id"]);

const run_handle = client.create_run_js(new UserMetadataBuilder("examples"));

const group = run_handle.result.child_uploader_js(new UserMetadataBuilder("generic"));

const group2d = group.result.child_uploader_2d_js(new UserMetadataBuilder("2d"));

const numPoints = 12;
for (let i = 0; i < numPoints; i++) {
    const angle = (i / numPoints) * 2 * Math.PI;
    const radius = 5.0 * (i / numPoints);
    const object2 = new Object2Builder(new Point2Builder(radius * Math.cos(angle), radius * Math.sin(angle)));
    object2.add_transform(Transform2Builder.identity());
    group2d.result.create_object2_js(new UserMetadataBuilder("point2"), object2);
}

console.log(`View contents at: ${run_handle.result.viewer_url()}`);

await client.shutdown();

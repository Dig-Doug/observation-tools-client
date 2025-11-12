import {defineConfig} from "@playwright/test";

export default defineConfig({
    fullyParallel: !process.env.SERVER_URL,
    testDir: "./specs",
    use: {
        video: "retain-on-failure",
    },
    timeout: 60000,
});

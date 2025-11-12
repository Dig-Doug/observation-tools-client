import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./specs",
  use: {
    video: "retain-on-failure",
  },
  timeout: 60000,
});

import { test as base } from "@playwright/test";
import { TestServer, spawnServer } from "../helpers/server";
import { ClientBuilder, Client } from "observation-tools-client";

// Extend base test with custom fixtures
type TestFixtures = {
  server: TestServer;
};

// Create a mock TestServer for external servers
class ExternalTestServer implements TestServer {
  port: number;
  baseUrl: string;
  dataDir: string;
  process: any;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
    const url = new URL(baseUrl);
    this.port = parseInt(url.port) || (url.protocol === "https:" ? 443 : 80);
    this.dataDir = ""; // Not applicable for external servers
    this.process = null; // Not applicable for external servers
  }

  createClient(): Client {
    try {
      const builder = new ClientBuilder();
      builder.setBaseUrl(this.baseUrl);
      const client = builder.build();
      return client;
    } catch (e) {
      console.error("Failed to create client:", e);
      throw e;
    }
  }

  async stop(): Promise<void> {
    // No-op for external servers - we don't own the lifecycle
    return Promise.resolve();
  }
}

export const test = base.extend<TestFixtures>({
  server: async ({}, use) => {
    const externalServerUrl = process.env.SERVER_URL;

    if (externalServerUrl) {
      // Use external server
      console.log(`Using external server at ${externalServerUrl}`);
      const server = new ExternalTestServer(externalServerUrl);
      await use(server);
      // No cleanup needed for external server
    } else {
      // Setup: start the server before each test
      const server = await spawnServer();

      // Provide the server to the test
      await use(server);

      // Teardown: stop the server after the test
      await server.stop();
    }
  },
});

export { expect } from "@playwright/test";

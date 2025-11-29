import { spawn, ChildProcess } from "child_process";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { ClientBuilder, Client} from "observation-tools-client";

export class TestServer {
  port: number;
  baseUrl: string;
  dataDir: string;
  process: ChildProcess;

  constructor(port: number, baseUrl: string, dataDir: string, process: ChildProcess) {
    this.port = port;
    this.baseUrl = baseUrl;
    this.dataDir = dataDir;
    this.process = process;
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
    return new Promise((resolve) => {
      this.process.on("exit", () => {
        // Clean up data directory
        fs.rmSync(this.dataDir, { recursive: true, force: true });
        resolve();
      });
      this.process.kill("SIGTERM");
      // Force kill after 5 seconds
      setTimeout(() => {
        if (!this.process.killed) {
          this.process.kill("SIGKILL");
        }
      }, 5000);
    });
  }
}

/**
 * Find an available port by attempting to listen on a random port
 */
async function findAvailablePort(): Promise<number> {
  const net = await import("net");
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.listen(0, () => {
      const address = server.address();
      if (address && typeof address !== "string") {
        const port = address.port;
        server.close(() => resolve(port));
      } else {
        reject(new Error("Failed to get port"));
      }
    });
    server.on("error", reject);
  });
}

/**
 * Spawn a server instance on a random port
 */
export async function spawnServer(): Promise<TestServer> {
  const port = await findAvailablePort();
  const dataDir = fs.mkdtempSync(path.join(os.tmpdir(), "observation-tools-test-"));

  console.log(`Starting server on port ${port} with data dir ${dataDir}`);

  const serverProcess = spawn(
    "cargo",
    ["run", "--bin", "observation-tools", "--", "serve", "--data-dir", dataDir],
    {
      env: {
        ...process.env,
        PORT: port.toString(),
        RUST_LOG: "info",
      },
      cwd: path.join(__dirname, "../../"),
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  // Wait for server to be ready
  await new Promise<void>((resolve, reject) => {
    const timeout = setTimeout(() => {
      reject(new Error("Server startup timeout"));
    }, 30000);

    let output = "";

    const onData = (data: Buffer) => {
      output += data.toString();
      // Look for the "Listening on" message
      if (output.includes("Listening on") || output.includes(`0.0.0.0:${port}`)) {
        clearTimeout(timeout);
        serverProcess.stdout?.off("data", onData);
        serverProcess.stderr?.off("data", onData);
        // Give it a moment to fully initialize
        setTimeout(resolve, 500);
      }
    };

    serverProcess.stdout?.on("data", onData);
    serverProcess.stderr?.on("data", onData);

    serverProcess.on("error", (err) => {
      clearTimeout(timeout);
      reject(err);
    });

    serverProcess.on("exit", (code) => {
      clearTimeout(timeout);
      if (code !== 0 && code !== null) {
        reject(new Error(`Server exited with code ${code}`));
      }
    });
  });

  const baseUrl = `http://localhost:${port}`;

  return new TestServer(port, baseUrl, dataDir, serverProcess);
}

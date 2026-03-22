import { spawn } from "node:child_process";
import fs from "node:fs";
import net from "node:net";

const endpointFile = process.argv[2];
const fixtureAppRoot = process.argv[3];

const port = await reservePort();
const electronArgs = [
  ...(process.platform === "linux" ? ["--no-sandbox"] : []),
  `--remote-debugging-port=${port}`,
  fixtureAppRoot,
];
const child = spawn(
  `${fixtureAppRoot}/node_modules/.bin/electron`,
  electronArgs,
  {
    detached: true,
    stdio: "inherit",
  },
);

fs.writeFileSync(endpointFile, `http://127.0.0.1:${port}`);

function cleanup() {
  try {
    process.kill(-child.pid, "SIGTERM");
  } catch {
    // best-effort cleanup
  }
}

process.on("exit", cleanup);
process.on("SIGTERM", () => {
  cleanup();
  process.exit(0);
});
process.on("SIGINT", () => {
  cleanup();
  process.exit(0);
});

function reservePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("failed to reserve attach port"));
        return;
      }

      server.close((error) => {
        if (error) {
          reject(error);
        } else {
          resolve(address.port);
        }
      });
    });
  });
}

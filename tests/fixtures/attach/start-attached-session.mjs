import { spawn } from "node:child_process";
import fs from "node:fs";
import net from "node:net";

const endpointFile = process.argv[2];

const port = await reservePort();
const child = spawn(
  "./tests/fixtures/electron-app/node_modules/.bin/electron",
  ["./tests/fixtures/electron-app", `--remote-debugging-port=${port}`],
  {
    stdio: "inherit",
  },
);

fs.writeFileSync(endpointFile, `http://127.0.0.1:${port}`);
process.on("exit", () => child.kill("SIGTERM"));

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

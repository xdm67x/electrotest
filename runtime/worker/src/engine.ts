import type { Request, Response } from "./protocol.js";

export async function handleRequest(request: Request): Promise<Response> {
  switch (request.type) {
    case "ping":
      return { type: "pong" };
    default:
      return {
        type: "error",
        message: `unsupported request type: ${request.type}`,
      };
  }
}

export type Request =
  | { type: "ping" }
  | { type: "launch_app"; command: string; args: string[] }
  | { type: "attach_app"; endpoint: string }
  | { type: "click"; window_id: string; locator: LocatorPayload[] }
  | { type: "screenshot"; window_id: string };

export type Response =
  | { type: "pong" }
  | { type: "app_launched"; window_id: string }
  | { type: "app_attached"; window_id: string }
  | { type: "clicked" }
  | { type: "screenshot_taken"; path: string }
  | { type: "error"; message: string };

export type LocatorPayload =
  | { type: "explicit"; selector: string }
  | { type: "test_id"; value: string }
  | { type: "role_name"; role: string; name: string }
  | { type: "text"; value: string };

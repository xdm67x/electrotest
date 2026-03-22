import { chromium, type Browser, type BrowserContext, type Locator, type Page } from "playwright";
import type { LocatorPayload, Request, Response, WindowTarget } from "./protocol.js";

declare const process: {
  on(event: "exit", handler: () => void): void;
  kill(pid: number, signal?: string): void;
};

type SessionState = {
  browser: Browser | null;
  context: BrowserContext | null;
  pages: Page[];
  activePage: Page | null;
  launchedChild: { pid: number } | null;
};

const state: SessionState = {
  browser: null,
  context: null,
  pages: [],
  activePage: null,
  launchedChild: null,
};

export async function handleRequest(request: Request): Promise<Response> {
  try {
    switch (request.type) {
      case "ping":
        return { type: "pong" };
      case "close_app":
        return await closeApp();
      case "launch_app":
        return await launchApp(request.command, request.args);
      case "attach_app":
        return await attachApp(request.endpoint);
      case "switch_window":
        return await switchWindow(request.target);
      case "current_window_title":
        return await currentWindowTitle();
      case "click":
        return await click(request.locator);
      case "screenshot":
        return { type: "screenshot_taken", path: "" };
    }

    return assertUnreachable(request);
  } catch (error) {
    return {
      type: "error",
      message: error instanceof Error ? error.message : String(error),
    };
  }
}

async function launchApp(command: string, args: string[]): Promise<Response> {
  await closeSession();
  await stopLaunchedChild();
  const port = await reservePort();
  const childProcess = await importNodeModule("node:child_process");
  const child = childProcess.spawn(command, [...args, `--remote-debugging-port=${port}`], {
    detached: true,
    stdio: "ignore",
  });
  state.launchedChild = child;
  process.on("exit", () => {
    terminateProcessGroup(child.pid);
  });

  state.browser = await connectToEndpoint(`http://127.0.0.1:${port}`);
  const context = state.browser.contexts()[0] ?? (await state.browser.newContext());
  state.context = context;
  await waitForPages(context);
  setPages(context.pages());
  const activePage = requireActivePage();
  return { type: "app_launched", window_id: windowId(activePage) };
}

async function attachApp(endpoint: string): Promise<Response> {
  await closeSession();
  await stopLaunchedChild();
  state.browser = await connectToEndpoint(endpoint);
  const context = state.browser.contexts()[0] ?? (await state.browser.newContext());
  state.context = context;
  await waitForPages(context);
  setPages(context.pages());
  const activePage = requireActivePage();
  return { type: "app_attached", window_id: windowId(activePage) };
}

async function switchWindow(target: WindowTarget): Promise<Response> {
  const context = requireContext();
  await waitForPages(context);
  setPages(context.pages());

  const page = await selectPage(target, state.pages);
  state.activePage = page;
  const description =
    target.type === "title"
      ? `Switched to window: ${await page.title()}`
      : `Switched to window index ${target.value}`;
  return {
    type: "window_switched",
    window_id: windowId(page),
    description,
  };
}

async function currentWindowTitle(): Promise<Response> {
  const page = requireActivePage();
  return {
    type: "current_window_title",
    title: await page.title(),
  };
}

async function click(locatorPayloads: LocatorPayload[]): Promise<Response> {
  const page = requireActivePage();
  const locator = buildLocator(page, locatorPayloads);
  await page.waitForLoadState("domcontentloaded");
  await locator.first().click();
  await page.waitForTimeout(250);
  const context = requireContext();
  setPages(context.pages());
  return { type: "clicked" };
}

function buildLocator(page: Page, locatorPayloads: LocatorPayload[]): Locator {
  if (locatorPayloads.length === 0) {
    throw new Error("element not found: no locators provided");
  }

  const [first, ...rest] = locatorPayloads;
  let locator = locatorFor(page, first);
  for (const payload of rest) {
    locator = locator.or(locatorFor(page, payload));
  }
  return locator;
}

function locatorFor(page: Page, payload: LocatorPayload): Locator {
  switch (payload.type) {
    case "explicit":
      return page.locator(payload.selector);
    case "test_id":
      return page.getByTestId(payload.value);
    case "role_name":
      return page.getByRole(payload.role as any, { name: payload.name });
    case "text":
      return page.getByText(payload.value);
  }
}

async function selectPage(target: WindowTarget, pages: Page[]): Promise<Page> {
  if (target.type === "index") {
    const page = pages[target.value];
    if (!page) {
      throw new Error(`window target not found: index ${target.value}`);
    }
    return page;
  }

  const matching: Page[] = [];
  for (const page of pages) {
    if ((await page.title()) === target.value) {
      matching.push(page);
    }
  }
  if (matching.length === 0) {
    throw new Error(`window target not found: ${target.value}`);
  }
  if (matching.length > 1) {
    throw new Error(`window target is ambiguous: ${target.value}`);
  }
  return matching[0];
}

function requireContext(): BrowserContext {
  if (!state.context) {
    throw new Error("app is not connected");
  }
  return state.context;
}

function requireActivePage(): Page {
  if (!state.activePage) {
    throw new Error("window target not found: no active window");
  }
  return state.activePage;
}

function setPages(pages: Page[]): void {
  state.pages = pages;
  if (!state.activePage && pages.length > 0) {
    state.activePage = pages[0];
  } else if (state.activePage && !pages.includes(state.activePage) && pages.length > 0) {
    state.activePage = pages[0];
  }
}

async function waitForPages(context: BrowserContext): Promise<void> {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    if (context.pages().length > 0) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("timeout while waiting for application window");
}

function windowId(page: Page): string {
  return String(state.pages.indexOf(page));
}

function assertUnreachable(value: never): never {
  throw new Error(`unsupported request type: ${JSON.stringify(value)}`);
}

async function connectToEndpoint(endpoint: string): Promise<Browser> {
  await waitForEndpoint(endpoint);
  return chromium.connectOverCDP(endpoint);
}

async function waitForEndpoint(endpoint: string): Promise<void> {
  const url = `${endpoint.replace(/\/$/, "")}/json/version`;
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {
      // keep polling until ready
    }

    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  throw new Error(`timeout while waiting for CDP endpoint: ${endpoint}`);
}

async function reservePort(): Promise<number> {
  const net = await importNodeModule("node:net");
  return await new Promise<number>((resolve, reject) => {
    const server = net.createServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("failed to reserve debugging port"));
        return;
      }

      const port = address.port;
      server.close((error: Error | undefined) => {
        if (error) {
          reject(error);
        } else {
          resolve(port);
        }
      });
    });
  });
}

async function stopLaunchedChild(): Promise<void> {
  if (!state.launchedChild) {
    return;
  }

  terminateProcessGroup(state.launchedChild.pid);
  state.launchedChild = null;
}

function terminateProcessGroup(pid: number): void {
  try {
    process.kill(-pid, "SIGTERM");
  } catch {
    try {
      process.kill(pid, "SIGTERM");
    } catch {
      // best-effort cleanup
    }
  }
}

async function closeApp(): Promise<Response> {
  await closeSession();
  await stopLaunchedChild();
  return { type: "app_closed" };
}

async function closeSession(): Promise<void> {
  if (state.browser) {
    try {
      await state.browser.close();
    } catch {
      // best-effort cleanup
    }
  }

  state.browser = null;
  state.context = null;
  state.pages = [];
  state.activePage = null;
}

async function importNodeModule(specifier: string): Promise<any> {
  return new Function("specifier", "return import(specifier);")(specifier) as Promise<any>;
}

const { app, BrowserWindow } = require("electron");

function fixtureHtml(title, body) {
  return `data:text/html,<!DOCTYPE html><html><head><title>${title}</title></head><body>${body}</body></html>`;
}

function createWindow(title) {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    show: false,
    title,
  });

  const body = `
    <main>
      <h1>${title}</h1>
      <button id="launch">Launch fixture flow</button>
      <button id="open-settings">Open settings</button>
      <p id="status">ready</p>
    </main>
    <script>
      const status = document.getElementById("status");
      document.getElementById("launch").addEventListener("click", () => {
        status.textContent = "launched";
      });
      document.getElementById("open-settings").addEventListener("click", () => {
        status.textContent = "settings-requested";
        window.open("electrotest://open-settings");
      });
    </script>
  `;

  win.loadURL(fixtureHtml(title, body));
  return win;
}

function createSettingsWindow() {
  const win = new BrowserWindow({
    width: 480,
    height: 320,
    show: false,
    title: "Preferences",
  });

  win.loadURL(
    fixtureHtml(
      "Preferences",
      '<main><h1>Preferences</h1><p id="settings-status">opened</p></main>'
    )
  );

  return win;
}

app.whenReady().then(() => {
  app.on("web-contents-created", (_event, contents) => {
    contents.setWindowOpenHandler(({ url }) => {
      if (url === "electrotest://open-settings") {
        createSettingsWindow();
        return { action: "deny" };
      }

      return { action: "deny" };
    });
  });

  createWindow("Fixture App");

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow("Fixture App");
    }
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

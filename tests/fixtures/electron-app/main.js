const { app, BrowserWindow } = require("electron");

function fixtureHtml(title, body) {
  return `data:text/html,<!DOCTYPE html><html><body>${body}</body></html>`;
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
    title: "Settings Window",
  });

  win.loadURL(
    fixtureHtml(
      "Settings Window",
      '<main><h1>Settings Window</h1><p id="settings-status">opened</p></main>'
    )
  );

  return win;
}

app.whenReady().then(() => {
  createWindow("Fixture App");
  createSettingsWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow("Fixture App");
      createSettingsWindow();
    }
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

const { app, BrowserWindow } = require("electron");

function createWindow(title) {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    show: false,
    title,
  });

  win.loadURL(`data:text/html,<html><body><h1>${title}</h1></body></html>`);
  return win;
}

app.whenReady().then(() => {
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

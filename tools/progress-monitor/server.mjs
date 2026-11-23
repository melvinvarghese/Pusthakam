import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { pathToFileURL } from "node:url";
import { ProgressError, progressFile, readProgress, withBreakdowns } from "./progress.mjs";

const localHost = "127.0.0.1";
const defaultPort = 4317;
const types = new Map([
  ["/", ["index.html", "text/html; charset=utf-8"]],
  ["/styles.css", ["styles.css", "text/css; charset=utf-8"]],
  ["/app.js", ["app.js", "text/javascript; charset=utf-8"]],
  ["/hanken-grotesk.woff2", ["hanken-grotesk.woff2", "font/woff2"]],
  ["/pusthakam-logo.png", ["../../src/assets/pusthakam-logo.png", "image/png"]],
  ["/favicon.png", ["../../public/favicon.png", "image/png"]],
]);

function send(response, status, type, content) {
  response.writeHead(status, {
    "Content-Type": type,
    "Cache-Control": "no-store",
    "X-Content-Type-Options": "nosniff",
    "Content-Security-Policy": "default-src 'none'; img-src 'self'; script-src 'self'; style-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'",
  });
  response.end(content);
}

export function createMonitor(file = progressFile) {
  let lastGood = null;
  return createServer(async (request, response) => {
    const port = request.socket.localPort;
    if (![`${localHost}:${port}`, `localhost:${port}`].includes(request.headers.host)) {
      return send(response, 403, "text/plain", "Use the local dashboard address.");
    }
    if (request.method !== "GET") return send(response, 405, "text/plain", "Only GET is supported.");

    const path = request.url.split("?", 1)[0];
    if (path === "/api/progress") {
      let result;
      let status = 200;
      try {
        const { data } = await readProgress(file);
        lastGood = withBreakdowns(data);
        result = { data: lastGood, warning: null };
      } catch (error) {
        // The file may be between writes. Keep the previous valid snapshot visible until it recovers.
        let warning = "PROGRESS.json is temporarily unreadable.";
        if (error.code === "ENOENT") warning = "PROGRESS.json is missing from the repository root.";
        if (error instanceof ProgressError) warning = "PROGRESS.json is invalid. Run npm run progress:validate for details.";
        status = lastGood ? 200 : 503;
        result = { data: lastGood, warning };
      }
      return send(response, status, "application/json; charset=utf-8", JSON.stringify(result));
    }

    if (path === "/favicon.ico") return send(response, 204, "text/plain", "");
    const asset = types.get(path);
    if (!asset) return send(response, 404, "text/plain", "Page not found.");
    try {
      const body = await readFile(new URL(asset[0], import.meta.url));
      return send(response, 200, asset[1], body);
    } catch {
      return send(response, 500, "text/plain", "A dashboard file could not be opened.");
    }
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const server = createMonitor();
  server.on("error", (error) => {
    if (error.code === "EADDRINUSE") {
      console.error(`Port ${defaultPort} is already in use. If the monitor is already running, open http://${localHost}:${defaultPort}/.`);
    } else {
      console.error(`Could not start the progress monitor: ${error.message}`);
    }
    process.exitCode = 1;
  });
  server.listen(defaultPort, localHost, () => {
    console.log(`Pusthakam Progress Monitor\nOpen http://${localHost}:${defaultPort}/ in your browser.\nPress Ctrl+C to stop.`);
  });
}

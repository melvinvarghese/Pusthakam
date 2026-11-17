import assert from "node:assert/strict";
import { once } from "node:events";
import { mkdtemp, rm, rmdir, writeFile } from "node:fs/promises";
import { request } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { setTimeout as delay } from "node:timers/promises";
import { runInContext } from "node:vm";
import { JSDOM } from "jsdom";
import { calculateProgress, readProgress, validateProgress } from "./progress.mjs";
import { createMonitor } from "./server.mjs";

test("weights implementation, verification and release gates before rounding", () => {
  const data = {
    tasks: [
      { id: "small", weight: 1, status: "verified" },
      { id: "medium", weight: 5, status: "implemented" },
      { id: "large", weight: 8, status: "in-progress" },
    ],
    releaseGates: [
      { weight: 1, taskIds: ["small"] },
      { weight: 3, taskIds: ["small", "medium"] },
    ],
  };
  assert.deepEqual(calculateProgress(data), {
    overall: 30.3, implementation: 42.8, verification: 7.1, releaseReadiness: 25,
  });
  for (const task of data.tasks) task.status = "verified";
  assert.deepEqual(calculateProgress(data), {
    overall: 100, implementation: 100, verification: 100, releaseReadiness: 100,
  });
});

test("validates the baseline and rejects inconsistent data and unsupported credit", async () => {
  const { data } = await readProgress();
  const invalid = structuredClone(data);
  invalid.totals.overall = invalid.totals.overall === 100 ? 0 : 100;
  assert.throws(() => validateProgress(invalid), /totals.overall/);

  invalid.tasks.push(invalid.tasks[0]);
  assert.throws(() => validateProgress(invalid), /Duplicate tasks ID/);
  invalid.tasks.pop();

  const verified = invalid.tasks.find((task) => task.status === "verified");
  verified.verificationEvidence = [];
  assert.throws(() => validateProgress(invalid), /needs passing evidence/);
});

async function startMonitor(t) {
  const folder = await mkdtemp(join(tmpdir(), "pusthakam-monitor-test-"));
  const file = join(folder, "PROGRESS.json");
  const server = createMonitor(file);
  t.after(async () => {
    await new Promise((resolve, reject) => server.close((error) => error ? reject(error) : resolve()));
    await rm(file, { force: true });
    await rmdir(folder);
  });
  await once(server.listen(0, "127.0.0.1"), "listening");
  const { address, port } = server.address();
  assert.equal(address, "127.0.0.1");
  return { base: `http://${address}:${port}`, file };
}

test("serves local assets and recovers from bad or missing progress without restarting", async (t) => {
  const { base, file } = await startMonitor(t);
  const read = async (expectedStatus) => {
    const response = await fetch(base + "/api/progress");
    assert.equal(response.status, expectedStatus);
    assert.equal(response.headers.get("cache-control"), "no-store");
    return response.json();
  };

  assert.match((await read(503)).warning, /missing/);
  await writeFile(file, "{");
  assert.match((await read(503)).warning, /invalid/);
  const { data } = await readProgress();
  await writeFile(file, JSON.stringify(data));
  const first = await read(200);
  assert.equal(first.warning, null);
  assert.deepEqual(first.data.totals, data.totals);

  await writeFile(file, "{");
  assert.deepEqual((await read(200)).data, first.data);
  await rm(file);
  const missing = await read(200);
  assert.match(missing.warning, /missing/);
  assert.deepEqual(missing.data, first.data);

  data.lastUpdated = "2030-01-01T00:00:00Z";
  await writeFile(file, JSON.stringify(data));
  const recovered = await read(200);
  assert.equal(recovered.warning, null);
  assert.equal(recovered.data.lastUpdated, data.lastUpdated);

  for (const path of ["/", "/styles.css", "/app.js", "/hanken-grotesk.woff2"]) {
    const response = await fetch(base + path);
    assert.equal(response.status, 200);
    assert.ok((await response.arrayBuffer()).byteLength > 0);
  }
  assert.equal((await fetch(base + "/package.json")).status, 404);
  const invalidHost = request(base, { headers: { host: "outside.invalid" } });
  const responseEvent = once(invalidHost, "response");
  invalidHost.end();
  const [invalidHostResponse] = await responseEvent;
  invalidHostResponse.resume();
  assert.equal(invalidHostResponse.statusCode, 403);
});

test("refreshes the dashboard every three seconds and preserves the roadmap filters", { timeout: 15000 }, async (t) => {
  const { base, file } = await startMonitor(t);
  const { data } = await readProgress();
  await writeFile(file, JSON.stringify(data));
  const html = await fetch(base).then((response) => response.text());
  const script = await fetch(base + "/app.js").then((response) => response.text());
  const dom = new JSDOM(html, { url: base, runScripts: "outside-only" });
  t.after(() => dom.window.close());
  const { window } = dom;
  const { document } = window;
  window.AbortSignal = AbortSignal;
  const intervals = [];
  const setInterval = window.setInterval.bind(window);
  window.setInterval = (callback, interval) => {
    intervals.push(interval);
    return setInterval(callback, interval);
  };
  window.fetch = (resource, options) => {
    const url = new URL(resource, base);
    assert.equal(url.origin, base);
    return fetch(url, options);
  };
  const find = (id) => document.getElementById(id);
  async function until(predicate, description) {
    const deadline = Date.now() + 4500;
    while (Date.now() < deadline) {
      if (predicate()) return;
      await delay(25);
    }
    assert.fail(description);
  }

  runInContext(script, dom.getInternalVMContext());
  await until(() => !find("dashboard").hidden, "Dashboard did not load local data.");
  assert.deepEqual(intervals, [3000]);
  assert.equal(find("overall").textContent, data.totals.overall + "%");
  assert.equal(document.querySelectorAll(".phase").length, data.phases.length);

  for (const key of ["phase", "status"]) {
    const value = data.tasks[0][key];
    const filter = find(key + "-filter");
    filter.value = value;
    filter.dispatchEvent(new window.Event("change"));
  }
  const expected = data.tasks.filter((task) => task.phase === data.tasks[0].phase && task.status === data.tasks[0].status);
  assert.equal(document.querySelectorAll(".task").length, expected.length);
  find("task-" + expected[0].id).open = true;

  await writeFile(file, "{");
  await until(() => find("connection").dataset.state === "stale", "Invalid JSON was not detected during refresh.");
  assert.equal(find("overall").textContent, data.totals.overall + "%");

  data.currentTask = { title: "Monitor refresh check", status: "verified", detail: "Temporary fixture only." };
  await writeFile(file, JSON.stringify(data));
  await until(() => find("current-task").textContent.includes(data.currentTask.title), "New work did not appear automatically.");
  assert.equal(find("connection").dataset.state, "live");
  assert.equal(document.querySelectorAll(".task").length, expected.length);
  assert.ok(find("task-" + expected[0].id).open);
});

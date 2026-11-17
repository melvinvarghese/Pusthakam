const $ = (id) => document.getElementById(id);
const labels = {
  "not-started": "Not started",
  "in-progress": "In progress",
  blocked: "Blocked",
  implemented: "Implemented",
  verified: "Verified",
};
const activityLabels = { completed: "Completed", verification: "Verification", progress: "Progress" };
const dateFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" });
const filters = ["phase", "status", "module"];
const expandedLists = { pending: false, gaps: false };
const intervalMs = 3000;
let data;
let snapshot;

function element(tag, content, className) {
  const result = document.createElement(tag);
  if (content !== undefined) result.textContent = content;
  if (className) result.className = className;
  return result;
}

function badge(status) {
  return element("span", labels[status], "badge " + status);
}

function time(iso) {
  const item = element("time", dateFormat.format(new Date(iso)));
  item.dateTime = iso;
  item.title = iso;
  return item;
}

function progress(value, label, className) {
  const bar = element("progress", value + "%", className);
  bar.max = 100;
  bar.value = value;
  bar.setAttribute("aria-label", label);
  return bar;
}

function list(items) {
  const ul = element("ul", undefined, "entries");
  ul.append(...items.map((item) => element("li", item)));
  return ul;
}

function setOptions(name, items) {
  const select = $(name + "-filter");
  const current = select.value;
  select.replaceChildren(select.options[0]);
  for (const [value, title] of items) {
    const option = element("option", title);
    option.value = value;
    select.append(option);
  }
  select.value = items.some(([value]) => value === current) ? current : "";
}

function setConnection(state, label, message) {
  const indicator = $("connection");
  const notice = $("notice");
  if (indicator.dataset.state !== state || indicator.textContent !== label) {
    indicator.dataset.state = state;
    indicator.textContent = label;
  }
  if (notice.textContent !== message) notice.textContent = message;
  notice.hidden = !message;
}

function renderOverview() {
  const phase = data.phases.find((item) => item.id === data.currentPhase);
  $("phase-label").textContent = "Phase " + phase.id + " · " + phase.title;
  $("overall").replaceChildren(document.createTextNode(data.totals.overall), element("span", "%"));
  $("overall-bar").value = data.totals.overall;
  for (const key of ["implementation", "verification", "releaseReadiness"]) {
    $(key).textContent = data.totals[key] + "%";
  }
  const when = time(data.lastUpdated);
  when.id = "updated";
  $("updated").replaceWith(when);

  const current = data.currentTask;
  $("current-task").replaceChildren(...(current ? [
    badge(current.status), element("h3", current.title), element("p", current.detail),
    ...(current.taskId ? [taskLink(current.taskId, "View roadmap task")] : []),
  ] : [element("p", "No current task has been recorded.")]));
  const next = data.nextMilestone;
  $("next-milestone").replaceChildren(...(next ? [
    element("h3", next.title), element("p", next.detail), taskLink(next.nextTaskId, "View next task"),
  ] : [element("p", "All v1 tasks are verified.")]));
}

function renderModules() {
  const rows = data.modules.map((item) => {
    const button = element("button", undefined, "module");
    button.type = "button";
    button.append(
      element("strong", item.title),
      element("span", item.implementation + "%", "module-value"),
      progress(item.implementation, item.title + " implementation", "implemented-bar"),
      element("span", item.verification + "%", "module-value verified-value"),
      progress(item.verification, item.title + " verification", "verified-bar"),
    );
    button.addEventListener("click", () => {
      for (const key of filters) $(key + "-filter").value = key === "module" ? item.id : "";
      renderRoadmap();
      if (window.location.hash !== "#roadmap") window.location.hash = "roadmap";
      $("module-filter").focus();
    });
    return button;
  });
  $("module-list").replaceChildren(...rows);
}

function taskRow(task, isOpen) {
  const detail = element("details", undefined, "task");
  detail.id = "task-" + task.id;
  detail.open = isOpen;
  const summary = element("summary");
  const title = element("span", undefined, "task-title");
  title.append(element("small", task.id), element("strong", task.title));
  summary.append(title, badge(task.status));
  const body = element("div", undefined, "task-body");
  const module = data.modules.find((item) => item.id === task.module);
  body.append(element("p", module.title + " · Weight " + task.weight + " · " + task.requirements.join(", "), "muted"));
  for (const [heading, items] of [
    ["Acceptance criteria", task.acceptanceCriteria],
    ["Implementation evidence", task.sourceEvidence],
    ["Verification evidence", task.verificationEvidence.map((id) => {
      const run = data.verificationRuns.find((entry) => entry.id === id);
      return run.title + " (" + run.outcome + "): " + run.result;
    })],
    ["Checks still needed", task.verificationNeeded],
  ]) {
    if (items.length) body.append(element("h4", heading), list(items));
  }
  detail.append(summary, body);
  return detail;
}

function renderRoadmap() {
  const openIds = new Set(Array.from(document.querySelectorAll(".task[open]"), (item) => item.id));
  const selected = Object.fromEntries(filters.map((key) => [key, $(key + "-filter").value]));
  const matches = data.tasks.filter((task) => filters.every((key) => !selected[key] || task[key] === selected[key]));
  $("roadmap-count").textContent = "Showing " + matches.length + " of " + data.tasks.length + " tasks";
  $("clear-filters").disabled = !Object.values(selected).some(Boolean);
  const sections = [];
  for (const phase of data.phases) {
    if (selected.phase && phase.id !== selected.phase) continue;
    const tasks = matches.filter((task) => task.phase === phase.id);
    if (!tasks.length) continue;
    const section = element("section", undefined, "phase sheet");
    const header = element("header", undefined, "phase-head");
    header.append(
      element("span", "Phase " + phase.id, "phase-label"),
      element("h3", phase.title),
      element("p", phase.implementation + "% implemented · " + phase.verification + "% verified"),
    );
    section.append(header, ...tasks.map((task) => taskRow(task, openIds.has("task-" + task.id))));
    sections.push(section);
  }
  $("phase-list").replaceChildren(...(sections.length ? sections : [
    element("p", "No tasks match these filters.", "empty sheet"),
  ]));
}

function taskLink(id, title) {
  const link = element("a", title, "task-link");
  link.href = "#task-" + id;
  link.addEventListener("click", (event) => {
    for (const key of filters) $(key + "-filter").value = "";
    renderRoadmap();
    const item = $("task-" + id);
    if (item) {
      event.preventDefault();
      if (window.location.hash !== link.hash) window.location.hash = link.hash;
      item.open = true;
      item.scrollIntoView({ block: "start" });
      item.querySelector("summary").focus({ preventScroll: true });
    }
  });
  return link;
}

function renderActivity() {
  const events = [
    ...data.recentActivity,
    ...data.verificationRuns.map((run) => ({
      ...run, kind: "verification", detail: run.check + " · " + run.result,
    })),
  ].sort((a, b) => Date.parse(b.at) - Date.parse(a.at));
  $("activity-list").replaceChildren(...events.slice(0, 12).map((entry) => {
    const li = element("li");
    const meta = element("div", undefined, "activity-meta");
    meta.append(element("strong", activityLabels[entry.kind]), time(entry.at));
    const copy = element("div");
    copy.append(element("strong", entry.title), element("p", entry.detail));
    li.append(meta, copy);
    return li;
  }));
}

function renderRemaining() {
  $("blockers").replaceChildren(...(data.blockers.length ? data.blockers.map((item) => {
    const task = data.tasks.find((entry) => entry.id === item.taskId);
    const li = element("li");
    li.append(taskLink(task.id, task.title), element("p", item.detail));
    return li;
  }) : [element("li", "No blockers recorded.")]));
  $("gates").replaceChildren(...data.releaseGates.map((gate) => {
    const li = element("li", gate.title);
    li.append(element("span", gate.passed ? "Passed" : "Open", "badge " + (gate.passed ? "verified" : "not-started")));
    return li;
  }));
  for (const [key, items] of [
    ["pending", data.tasks.filter((task) => !["verified", "implemented"].includes(task.status))],
    ["gaps", data.tasks.filter((task) => task.status === "implemented")],
  ]) {
    $(key + "-count").textContent = items.length + (items.length === 1 ? " task" : " tasks");
    const button = $("more-" + key);
    button.hidden = items.length <= 5;
    button.textContent = expandedLists[key] ? "Show fewer" : "Show all " + items.length;
    button.setAttribute("aria-expanded", String(expandedLists[key]));
    button.setAttribute("aria-controls", key);
    $(key).replaceChildren(...(items.length ? items.slice(0, expandedLists[key] ? items.length : 5).map((task) => {
      const li = element("li");
      li.append(taskLink(task.id, task.title));
      if (key === "gaps") li.append(element("p", task.verificationNeeded.join(" ")));
      return li;
    }) : [element("li", "None remaining.")]));
  }
}

function render() {
  renderOverview();
  renderModules();
  setOptions("phase", data.phases.map((item) => [item.id, "Phase " + item.id + " · " + item.title]));
  setOptions("module", data.modules.map((item) => [item.id, item.title]));
  renderRoadmap();
  renderActivity();
  renderRemaining();
  $("dashboard").hidden = false;
}

async function refresh() {
  try {
    const response = await fetch("/api/progress", { cache: "no-store", signal: AbortSignal.timeout(2500) });
    const { data: latest, warning } = await response.json();
    if (latest) {
      const serialized = JSON.stringify(latest);
      if (serialized !== snapshot) {
        const initialLoad = !data;
        data = latest;
        render();
        snapshot = serialized;
        if (initialLoad && window.location.hash) {
          const target = $(window.location.hash.slice(1));
          if (target?.matches(".task")) target.open = true;
          target?.scrollIntoView();
        }
      }
    }
    if (warning) {
      setConnection(data ? "stale" : "waiting", data ? "Showing last saved data" : "Waiting for data",
        warning + " The monitor will retry automatically.");
    } else {
      setConnection("live", "Live updates", "");
    }
  } catch {
    setConnection(data ? "stale" : "waiting", data ? "Showing last saved data" : "Waiting for data",
      "Could not reach the local monitor. Check that it is still running. This page will reconnect automatically.");
  }
}

function markSection() {
  const hash = window.location.hash || "#overview";
  const section = hash.startsWith("#task-") ? "#roadmap" : hash;
  for (const link of document.querySelectorAll(".rail nav a")) {
    if (link.hash === section) link.setAttribute("aria-current", "location");
    else link.removeAttribute("aria-current");
  }
}

setOptions("status", Object.entries(labels));
for (const key of filters) $(key + "-filter").addEventListener("change", renderRoadmap);
$("clear-filters").addEventListener("click", () => {
  for (const key of filters) $(key + "-filter").value = "";
  renderRoadmap();
});
for (const key of Object.keys(expandedLists)) {
  $("more-" + key).addEventListener("click", () => {
    expandedLists[key] = !expandedLists[key];
    renderRemaining();
  });
}
window.addEventListener("hashchange", markSection);
markSection();
void refresh();
window.setInterval(refresh, intervalMs);

import { readFile } from "node:fs/promises";

export const progressFile = new URL("../../PROGRESS.json", import.meta.url);
export const statuses = ["not-started", "in-progress", "blocked", "implemented", "verified"];

const credit = {
  implementation: (task) => ["implemented", "verified"].includes(task.status),
  verification: (task) => task.status === "verified",
};

function totalWeight(items) {
  return items.reduce((sum, item) => sum + item.weight, 0);
}

function percentage(earned, total) {
  return total ? Math.floor(1000 * earned / total) / 10 : 0;
}

export function calculateProgress(data) {
  const verifiedTasks = data.tasks.filter(credit.verification);
  const verified = new Set(verifiedTasks.map((task) => task.id));
  const taskWeight = totalWeight(data.tasks);
  const gateWeight = totalWeight(data.releaseGates);
  const implemented = totalWeight(data.tasks.filter(credit.implementation));
  const verifiedWeight = totalWeight(verifiedTasks);
  const ready = totalWeight(data.releaseGates.filter((gate) => gate.taskIds.every((id) => verified.has(id))));
  return {
    overall: percentage((6 * implemented + 3 * verifiedWeight) * gateWeight + ready * taskWeight,
      10 * taskWeight * gateWeight),
    implementation: percentage(implemented, taskWeight),
    verification: percentage(verifiedWeight, taskWeight),
    releaseReadiness: percentage(ready, gateWeight),
  };
}

function summarize(tasks) {
  const weight = totalWeight(tasks);
  return {
    implementation: percentage(totalWeight(tasks.filter(credit.implementation)), weight),
    verification: percentage(totalWeight(tasks.filter(credit.verification)), weight),
    weight,
    taskCount: tasks.length,
  };
}

export function withBreakdowns(data) {
  return {
    ...data,
    phases: data.phases.map((phase) => ({
      ...phase, ...summarize(data.tasks.filter((task) => task.phase === phase.id)),
    })),
    modules: data.modules.map((module) => ({
      ...module, ...summarize(data.tasks.filter((task) => task.module === module.id)),
    })),
    releaseGates: data.releaseGates.map((gate) => ({
      ...gate, passed: gate.taskIds.every((id) =>
        data.tasks.some((task) => task.id === id && task.status === "verified")),
    })),
  };
}

export class ProgressError extends Error {}

function requireValue(condition, message) {
  if (!condition) throw new ProgressError(message);
}

const record = (value) => Boolean(value) && typeof value === "object" && !Array.isArray(value);
const text = (value) => typeof value === "string" && value.trim().length > 0;
const texts = (value) => Array.isArray(value) && value.every(text);
const timestamp = (value) => typeof value === "string" &&
  /^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d{3})?Z$/.test(value) &&
  Number.isFinite(Date.parse(value)) && new Date(value).toISOString() === value.replace(/(?<!\.\d{3})Z$/, ".000Z");
const weight = (value) => Number.isInteger(value) && value > 0 && value <= 10000;
const identifier = (value) => typeof value === "string" && /^[a-z0-9][a-z0-9-]*$/i.test(value);

function indexById(items, label) {
  requireValue(Array.isArray(items) && items.length > 0, `${label} must be a nonempty array.`);
  const index = new Map();
  for (const item of items) {
    requireValue(record(item) && identifier(item.id) && text(item.title), `Invalid entry in ${label}.`);
    requireValue(!index.has(item.id), `Duplicate ${label} ID: ${item.id}.`);
    index.set(item.id, item);
  }
  return index;
}

export function validateProgress(data, { checkTotals = true } = {}) {
  requireValue(record(data) && data.schemaVersion === 1, "Expected a schemaVersion 1 progress object.");
  requireValue(timestamp(data.lastUpdated), "lastUpdated must be a UTC ISO timestamp.");
  const phases = indexById(data.phases, "phases");
  const modules = indexById(data.modules, "modules");
  const tasks = indexById(data.tasks, "tasks");
  const gates = indexById(data.releaseGates, "release gates");
  const evidence = indexById(data.verificationRuns, "verification runs");

  requireValue(phases.has(data.currentPhase), "currentPhase must identify a roadmap phase.");
  requireValue(Array.isArray(data.blockers), "blockers must be an array.");
  for (const item of data.blockers) {
    requireValue(record(item) && text(item.detail) && tasks.has(item.taskId), "Invalid blocker.");
  }
  for (const run of evidence.values()) {
    requireValue(timestamp(run.at) && text(run.result) && text(run.check) &&
      ["passed", "failed", "incomplete"].includes(run.outcome), `Invalid verification run: ${run.id}.`);
  }
  for (const task of tasks.values()) {
    const valid = phases.has(task.phase) && modules.has(task.module) && weight(task.weight) &&
      statuses.includes(task.status) && texts(task.acceptanceCriteria) && task.acceptanceCriteria.length > 0 &&
      texts(task.requirements) && task.requirements.length > 0 &&
      texts(task.sourceEvidence) && texts(task.verificationEvidence) &&
      task.verificationEvidence.every((id) => evidence.has(id)) && texts(task.verificationNeeded);
    requireValue(valid, `Invalid task: ${task.id}.`);
    if (credit.implementation(task)) requireValue(task.sourceEvidence.length, `Missing implementation evidence: ${task.id}.`);
    if (credit.verification(task)) {
      requireValue(task.verificationEvidence.length && task.verificationEvidence.every((id) =>
        evidence.get(id).outcome === "passed") && !task.verificationNeeded.length,
      `Verified task needs passing evidence and no outstanding checks: ${task.id}.`);
    } else {
      requireValue(task.verificationNeeded.length, `Record the checks needed for ${task.id}.`);
    }
    if (task.status === "blocked") {
      requireValue(data.blockers.some((item) => item.taskId === task.id), `Missing blocker for ${task.id}.`);
    }
  }
  for (const gate of gates.values()) {
    requireValue(weight(gate.weight) && texts(gate.taskIds) && gate.taskIds.length > 0 &&
      new Set(gate.taskIds).size === gate.taskIds.length &&
      gate.taskIds.every((id) => tasks.has(id)), `Invalid release gate: ${gate.id}.`);
  }
  for (const key of ["phase", "module"]) {
    const idsInUse = new Set(data.tasks.map((task) => task[key]));
    for (const id of (key === "phase" ? phases : modules).keys()) {
      requireValue(idsInUse.has(id), `No tasks assigned to ${key} ${id}.`);
    }
  }

  const current = data.currentTask;
  requireValue(current === null || (record(current) && statuses.includes(current.status) &&
    text(current.title) && text(current.detail) &&
    (current.taskId === undefined || tasks.get(current.taskId)?.status === current.status)),
  "currentTask must describe the latest work and agree with its task status when linked.");
  const next = data.nextMilestone;
  requireValue((next === null && data.tasks.every(credit.verification)) ||
    (record(next) && text(next.title) && text(next.detail) && tasks.has(next.nextTaskId) &&
      !credit.verification(tasks.get(next.nextTaskId))), "nextMilestone must point to remaining work.");
  requireValue(Array.isArray(data.recentActivity), "recentActivity must be an array.");
  for (const entry of data.recentActivity) {
    requireValue(record(entry) && timestamp(entry.at) && text(entry.title) && text(entry.detail) &&
      ["completed", "verification", "progress"].includes(entry.kind) &&
      (entry.taskId === undefined || tasks.has(entry.taskId)), "Invalid recent activity entry.");
  }

  const totals = calculateProgress(data);
  if (checkTotals) {
    for (const [key, value] of Object.entries(totals)) {
      requireValue(record(data.totals) && data.totals[key] === value,
        `totals.${key} must equal ${value}% (found ${data.totals?.[key]}).`);
    }
  }
  return totals;
}

export async function readProgress(file = progressFile, options) {
  let data;
  const source = await readFile(file, "utf8");
  try {
    data = JSON.parse(source.replace(/^\uFEFF/, ""));
  } catch {
    throw new ProgressError("PROGRESS.json contains invalid JSON.");
  }
  return { data, totals: validateProgress(data, options) };
}

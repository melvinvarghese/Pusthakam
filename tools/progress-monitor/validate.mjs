import { rename, rm, writeFile } from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { progressFile, readProgress } from "./progress.mjs";

const args = process.argv.slice(2);
const write = args.length === 1 && args[0] === "--write";
try {
  if (args.length && !write) throw new Error("Usage: npm run progress:validate [-- --write]");
  const { data, totals } = await readProgress(progressFile, { checkTotals: !write });
  if (write) {
    const temporary = new URL(`PROGRESS.json.${randomUUID()}.tmp`, progressFile);
    try {
      await writeFile(temporary, JSON.stringify({ ...data, totals }, null, 2) + "\n", { flag: "wx" });
      await rename(temporary, progressFile);
    } finally {
      await rm(temporary, { force: true });
    }
  }
  console.log(`PROGRESS.json valid. ${data.tasks.length} tasks across ${data.phases.length} phases.`);
  for (const [name, value] of Object.entries(totals)) console.log(`  ${name}: ${value}%`);
} catch (error) {
  console.error(`Progress validation failed: ${error.message}`);
  process.exitCode = 1;
}

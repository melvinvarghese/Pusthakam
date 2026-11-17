# Pusthakam

## Local progress monitor

Open a terminal in this repository and run `npm run progress:monitor`. Open [the dashboard](http://127.0.0.1:4317/) in your browser. Keep the terminal open while viewing it. Press `Ctrl+C` in the terminal to stop the monitor.

The dashboard reads `PROGRESS.json` from this folder every three seconds. Use the Roadmap filters to find tasks and expand a task to see its acceptance criteria and test evidence. The monitor only needs Node.js and runs locally with its bundled assets.

Run `npm run progress:validate` to check the progress data. If PowerShell blocks `npm`, use `npm.cmd` in the same commands.

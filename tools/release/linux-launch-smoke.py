#!/usr/bin/env python3
"""Check packaged startup and restart using an isolated XDG data directory."""
import os
from contextlib import closing
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time


def smoke(executable, root, timeout=45):
    root = Path(root).resolve()
    env = dict(os.environ)
    for name, folder in [("XDG_DATA_HOME", "data"), ("XDG_CONFIG_HOME", "config"), ("XDG_CACHE_HOME", "cache")]:
        env[name] = str(root / folder)
    command = list(executable) if isinstance(executable, (list, tuple)) else [str(executable)]
    database = None
    for attempt in range(2):
        with (root / f"launch-{attempt + 1}.log").open("wb") as log:
            process = subprocess.Popen(command, env=env, stdout=log, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + timeout
                while time.monotonic() < deadline:
                    if process.poll() is not None:
                        raise RuntimeError(f"App exited during launch {attempt + 1}: {process.returncode}; see {log.name}")
                    candidates = list((root / "data").rglob("pusthakam.db"))
                    if len(candidates) == 1:
                        candidate = candidates[0]
                        try:
                            with closing(sqlite3.connect(f"{candidate.as_uri()}?mode=ro", uri=True)) as conn:
                                tables = {row[0] for row in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")}
                                ready = {"staff_users", "bibliographic_records"}.issubset(tables)
                                if ready and conn.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
                                    raise RuntimeError("Smoke database failed integrity check")
                            if ready:
                                if database is not None and database != candidate:
                                    raise RuntimeError("Restart changed the database path")
                                database = candidate
                                time.sleep(2)
                                if process.poll() is not None:
                                    raise RuntimeError("App exited after creating its database")
                                break
                        except sqlite3.OperationalError:
                            pass
                    time.sleep(0.2)
                else:
                    raise RuntimeError(f"No migrated database after launch {attempt + 1}; see {log.name}")
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
    return database


if __name__ == "__main__":
    executable = Path(sys.argv[1]).resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="pusthakam-linux-smoke-"))
    print(f"Isolated smoke files: {root}", flush=True)
    smoke(executable, root)
    print("Installed app stayed running on launch and restart; migrated database passed integrity checks.")
    print("Interactive setup, sign-in and window rendering still require native acceptance.")

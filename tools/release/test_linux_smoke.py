"""Regression checks for the smoke runner, not native package acceptance."""
import importlib.util
from contextlib import closing
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("linux_smoke", HERE / "linux-launch-smoke.py")
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class SmokeTests(unittest.TestCase):
    def fake_app(self, root, body):
        app = root / "app.py"
        app.write_text(body, encoding="utf-8")
        return [sys.executable, str(app)]

    def test_launch_and_restart_share_isolated_database(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            app = self.fake_app(root, """import os, sqlite3, time
from pathlib import Path
root=Path(os.environ['XDG_DATA_HOME'])
root.mkdir(parents=True,exist_ok=True)
with sqlite3.connect(root/'pusthakam.db') as conn:
 conn.execute('CREATE TABLE IF NOT EXISTS staff_users(id INTEGER)')
 conn.execute('CREATE TABLE IF NOT EXISTS bibliographic_records(id INTEGER)')
 conn.execute('INSERT INTO staff_users VALUES (1)')
while True: time.sleep(1)
""")
            database = runner.smoke(app, root, timeout=3)
            import sqlite3
            with closing(sqlite3.connect(database)) as conn:
                self.assertEqual(conn.execute("SELECT count(*) FROM staff_users").fetchone()[0], 2)
            self.assertTrue((root / "launch-2.log").is_file())

    def test_startup_crash_is_failure(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            app = self.fake_app(root, "raise SystemExit(7)\n")
            with self.assertRaisesRegex(RuntimeError, "exited during launch"):
                runner.smoke(app, root, timeout=2)

    def test_running_process_without_database_is_failure(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            app = self.fake_app(root, "import time\nwhile True: time.sleep(1)\n")
            with self.assertRaisesRegex(RuntimeError, "No migrated database"):
                runner.smoke(app, root, timeout=0.4)

    @unittest.skipUnless(shutil.which("bash"), "Bash required for Linux package wrapper test")
    def test_missing_package_cannot_silently_pass(self):
        with tempfile.TemporaryDirectory() as folder:
            result = subprocess.run(["bash", str(HERE / "linux-package-smoke.sh")], cwd=folder, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Expected exactly one Debian package", result.stderr)


if __name__ == "__main__":
    unittest.main()

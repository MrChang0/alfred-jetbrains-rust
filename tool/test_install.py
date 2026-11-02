#!/usr/bin/env python3
"""Test the installer with local release fixtures and no real Alfred changes."""

import hashlib
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.home = self.root / "home"
        prefs = self.home / "Library/Application Support/Alfred/prefs.json"
        prefs.parent.mkdir(parents=True)
        prefs.write_text("{}")
        self.sources = self.root / "sources"
        self.sources.mkdir()
        self.work = self.root / "temporary downloads"
        self.work.mkdir()
        fake_bin = self.root / "mock commands"
        fake_bin.mkdir()
        self.log = self.root / "installed"
        self.curl_log = self.root / "downloads"
        self.env = {
            **os.environ,
            "HOME": str(self.home),
            "TMPDIR": str(self.work),
            "PATH": f"{fake_bin}:{os.environ['PATH']}",
            "TEST_OS": "Darwin",
            "TEST_ARCH": "arm64",
            "TEST_SOURCES": str(self.sources),
            "TEST_INSTALL_LOG": str(self.log),
            "TEST_CURL_LOG": str(self.curl_log),
            "TEST_INSTALL_EXIT": "0",
            "TEST_DOWNLOAD_FAIL": "0",
        }
        self.executable(fake_bin / "uname", '''#!/bin/sh
case "$1" in
  -s) printf '%s\\n' "$TEST_OS" ;;
  -m) printf '%s\\n' "$TEST_ARCH" ;;
esac
''')
        self.executable(fake_bin / "curl", '''#!/usr/bin/env python3
import os, pathlib, shutil, sys
args = sys.argv[1:]
url = next(arg for arg in args if arg.startswith("https://"))
with open(os.environ["TEST_CURL_LOG"], "a") as log:
    log.write(url + "\\n")
if os.environ["TEST_DOWNLOAD_FAIL"] == "1":
    sys.exit(22)
destination = args[args.index("-o") + 1]
source = pathlib.Path(os.environ["TEST_SOURCES"]) / url.rsplit("/", 1)[1]
shutil.copyfile(source, destination)
''')
        executable = self.sources / "jb"
        self.executable(executable, '''#!/bin/sh
printf '%s\\n' "$*" > "$TEST_INSTALL_LOG"
exit "$TEST_INSTALL_EXIT"
''')
        sums = []
        for arch in ("ARM64", "X64"):
            archive = self.sources / f"jb-{arch}-apple-darwin.tar.gz"
            with tarfile.open(archive, "w:gz") as package:
                package.add(executable, arcname="jb")
            sums.append(f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}")
        (self.sources / "SHA256SUMS").write_text("\n".join(sums) + "\n")

    @staticmethod
    def executable(path, content):
        path.write_text(content)
        path.chmod(0o755)

    def run_installer(self, *args):
        result = subprocess.run(
            ["bash", str(ROOT / "install.sh"), *args],
            env=self.env, capture_output=True, text=True,
        )
        self.assertEqual(list(self.work.iterdir()), [], "temporary downloads must be cleaned up")
        return result

    def test_installs_latest_for_both_architectures(self):
        for machine, archive in (("arm64", "ARM64"), ("x86_64", "X64")):
            with self.subTest(machine=machine):
                self.env["TEST_ARCH"] = machine
                self.log.unlink(missing_ok=True)
                self.curl_log.unlink(missing_ok=True)
                result = self.run_installer()
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.log.read_text(), "install\n")
                self.assertIn(f"latest/download/jb-{archive}-apple-darwin.tar.gz", self.curl_log.read_text())

    def test_pinned_version(self):
        result = self.run_installer("v0.2.0")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("download/v0.2.0/", self.curl_log.read_text())

    def test_rejects_checksum_mismatch_before_execution(self):
        checksum = self.sources / "SHA256SUMS"
        checksum.write_text("0" * 64 + "  jb-ARM64-apple-darwin.tar.gz\n")
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("checksum mismatch", result.stderr)
        self.assertFalse(self.log.exists())

    def test_rejects_missing_checksum_before_execution(self):
        (self.sources / "SHA256SUMS").write_text("")
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid SHA256", result.stderr)
        self.assertFalse(self.log.exists())

    def test_download_failure_does_not_install(self):
        self.env["TEST_DOWNLOAD_FAIL"] = "1"
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.log.exists())

    def test_rejects_unsupported_system_and_architecture(self):
        for variable, value in (("TEST_OS", "Linux"), ("TEST_ARCH", "unsupported")):
            with self.subTest(variable=variable):
                previous = self.env[variable]
                self.env[variable] = value
                result = self.run_installer()
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(self.curl_log.exists())
                self.env[variable] = previous

    def test_requires_alfred_before_downloading(self):
        (self.home / "Library/Application Support/Alfred/prefs.json").unlink()
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("start Alfred", result.stderr)
        self.assertFalse(self.curl_log.exists())

    def test_propagates_install_failure(self):
        self.env["TEST_INSTALL_EXIT"] = "9"
        result = self.run_installer()
        self.assertEqual(result.returncode, 9)
        self.assertNotIn("Done.", result.stdout)

    def test_rejects_invalid_version(self):
        result = self.run_installer("v0.2.0/../../other")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.curl_log.exists())


if __name__ == "__main__":
    unittest.main()

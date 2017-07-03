"""Module to help in running artifact."""
from __future__ import print_function, absolute_import

import os
import time
import re
import subprocess
import tempfile
import shutil
import signal

import toml
from py_helpers import models

URL_PAT = re.compile(r'Listening on (\S+)')
WEB_FILES_PAT = re.compile(r'unpacking web-ui at: (\S+)')

TARGET_ART = os.environ['TARGET_BIN']

# pylint: disable=too-few-public-methods


class Phantom(object):
    """start/stop phantomjs in the background for emulating a browser."""

    def __init__(self):
        self.stdout = None
        self.cmd = None

    def start(self):
        """start the phantom server."""
        cmd = [
            "phantomjs",
        ]

        self.stdout = tempfile.NamedTemporaryFile("wb")
        self.cmd = subprocess.Popen(
            cmd, bufsize=1, stdout=self.stdout, stderr=self.stdout)
        print("ran cmd: ", cmd)

    def stop(self):
        """stop the phantom server."""
        if self.cmd:
            self.cmd.send_signal(signal.SIGTERM)
            self.cmd = None

        if self.stdout:
            self.stdout.close()
            self.stdout = None


class Artifact(object):
    """Copies the project into a temporary directory and runs it.

    This can be used in a context manager and allows editing without
    changing the original text.

    """

    def __init__(self, project_path):
        self.project_path = project_path
        self.tempdir = None
        self.tmp_proj = None
        self.stdout = None
        self.art = None
        self.web_files = None

    def get_artifacts(self, path):
        """Return the artifacts located at ``path``."""
        with open(os.path.join(self.tmp_proj, path)) as f:
            data = toml.load(f)
        return {k: models.ArtifactToml.deserialize(v) for (k, v) in data.items()}

    def restart(self):
        """Restart a running server instance in place (does NOT re-copy the
        project)."""
        assert self.stdout is not None, (
            "cannot restart a process that isn't started.")
        self._stop()
        return self._start()

    def _start(self):
        """start an instance that has has the project initialized."""
        assert self.stdout is None, "process already running"

        self.stdout = tempfile.NamedTemporaryFile("wb")
        cmd = [
            TARGET_ART,
            "--work-tree", self.tmp_proj,
            "-vv",
            "serve"
        ]
        self.art = subprocess.Popen(
            cmd, bufsize=1, stdout=self.stdout, stderr=self.stdout)
        print("ran cmd: ", cmd)

        with open(self.stdout.name, "rb") as stdout:
            start = time.time()
            while True:
                time.sleep(0.2)
                stdout.seek(0)
                if self.art.poll() is not None:
                    raise Exception("art died: {}".format(stdout.read()))
                stdout_str = stdout.read()
                match = URL_PAT.search(stdout_str)
                if match:
                    url = match.group(1)
                    break
                assert time.time() - start > 10, "timeout"

        self.web_files = WEB_FILES_PAT.search(stdout_str).group(1)
        return url

    def _stop(self):
        """stop the server but do not delete the tmp project."""
        if self.art:
            self.art.send_signal(signal.SIGTERM)
            self.art = None

        if self.stdout:
            self.stdout.close()
            self.stdout = None

        if self.web_files:
            start = time.time()
            while os.path.exists(self.web_files):
                assert time.time() - start < 5, "web-ui files were not cleaned up"
                time.sleep(0.2)
            self.web_files = None

    def __enter__(self):
        self.tempdir = tempfile.mkdtemp()
        self.tmp_proj = os.path.join(self.tempdir, "proj")
        shutil.copytree(self.project_path, self.tmp_proj)
        return self._start()

    def __exit__(self, exc_type, exc_value, traceback):
        self._stop()

        if self.tempdir:
            shutil.rmtree(self.tempdir)
            self.tempdir = None

        self.tmp_proj = None

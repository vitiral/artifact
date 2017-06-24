# pylint: disable=missing-docstring,no-self-use

import unittest
from py_helpers import models

example_loc = {
    "path": "my/path",
    "line": 42
}

example_art = {
    "id": 1,
    "revision": 2,
    "name": "REQ-foo",
    "def": "defined/at/foo.toml",
    "text": "this is some text",
    "partof": ["REQ-bar"],
    "parts": [],
    "code": example_loc,
    "done": None,
    "completed": 0,
    "tested": 0,
}


class TestModels(unittest.TestCase):
    def test_loc(self):
        loc = models.Loc.deserialize(example_loc)
        assert loc.path == "my/path"
        assert loc.line == 42

    def test_artifact(self):
        art = models.Artifact.deserialize(example_art)
        assert art.id == 1
        assert art.revision == 2
        assert art.code.line == 42
        assert art.serialize() == example_art
        expected = models.Name("REQ-bar", "REQ-BAR", models.Type.req)
        assert art.partof[0] == expected

        ex = dict(example_art)
        ex['code'] = None
        art = models.Artifact.deserialize(ex)
        assert art.code is None
        assert art.name.raw == "REQ-foo"
        assert art.name.value == "REQ-FOO"
        assert art.name.ty is models.Type.req

    def test_type(self):
        ty = models.Type("REQ")
        assert ty is models.Type.req

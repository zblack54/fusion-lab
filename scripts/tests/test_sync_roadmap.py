"""Effect-level regression checks for the roadmap GitHub boundary."""

import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("sync", Path(__file__).parents[1] / "sync_roadmap.py")
sync = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sync)


class GitHubFixture:
    def __init__(self, pull_request=None):
        self.records = []
        self.issues = {9: {"milestone": None}, 1: {"milestone": None}}
        if pull_request:
            self.issues[pull_request]["pull_request"] = {"url": "fixture"}
        self.writes = []

    def milestones(self):
        return [dict(item) for item in self.records]

    def request(self, method, suffix, payload=None):
        if method == "GET":
            return dict(self.issues[int(suffix.split("/")[1])])
        self.writes.append((method, suffix, payload))
        if suffix == "milestones":
            item = dict(payload, number=len(self.records) + 1, html_url="fixture")
            self.records.append(item)
            return dict(item)
        if suffix.startswith("issues/"):
            self.issues[int(suffix.split("/")[1])]["milestone"] = {"number": payload["milestone"]}
            return {}
        raise AssertionError((method, suffix))


def group(title, issues):
    return {"key": title, "title": title, "description": "outcome", "state": "open", "issues": issues}


class RoadmapEffects(unittest.TestCase):
    def test_legacy_pr_is_rejected_before_any_mutation(self):
        github = GitHubFixture(pull_request=1)
        data = {"milestones": [group("first", [9]), group("second", [1])]}
        with self.assertRaisesRegex(ValueError, "pull request"):
            sync.synchronize(data, github)
        self.assertEqual(github.writes, [])

    def test_second_pass_has_no_duplicate_effects(self):
        github = GitHubFixture()
        data = {"milestones": [group("first", [9])]}
        first = sync.synchronize(data, github)
        writes = list(github.writes)
        second = sync.synchronize(data, github, assert_no_changes=True)
        self.assertEqual(first["changes"], 2)
        self.assertEqual(second["changes"], 0)
        self.assertEqual(github.writes, writes)
        self.assertEqual(len(github.records), 1)


if __name__ == "__main__":
    unittest.main()

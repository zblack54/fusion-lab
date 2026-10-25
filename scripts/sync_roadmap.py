"""Validate roadmap tracking, or synchronize it using a scoped Actions token."""

import argparse
import json
import os
from pathlib import Path
import urllib.request

REPOSITORY = "zblack54/fusion-lab"


def load_manifest(path):
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema_version") != 1 or data.get("repository") != REPOSITORY:
        raise ValueError("unsupported schema or repository")
    keys, titles, issues = set(), set(), set()
    for item in data["milestones"]:
        if item["key"] in keys or item["title"] in titles:
            raise ValueError("duplicate milestone key or title")
        if item["state"] not in ("open", "closed") or not item["description"].strip():
            raise ValueError("invalid milestone state or description")
        keys.add(item["key"])
        titles.add(item["title"])
        for issue in item["issues"]:
            if type(issue) is not int or issue < 1 or issue in issues:
                raise ValueError("invalid or multiply assigned issue")
            issues.add(issue)
    if not keys or len(keys) > 20 or len(issues) > 200:
        raise ValueError("roadmap exceeds bounded tracking scope")
    return data


class GitHub:
    def __init__(self, token):
        self.token = token

    def request(self, method, suffix, payload=None):
        body = None if payload is None else json.dumps(payload).encode()
        request = urllib.request.Request(
            f"https://api.github.com/repos/{REPOSITORY}/{suffix}",
            data=body,
            method=method,
            headers={"Authorization": f"Bearer {self.token}",
                     "Accept": "application/vnd.github+json",
                     "Content-Type": "application/json",
                     "User-Agent": "fusion-lab-roadmap-sync"},
        )
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)

    def milestones(self):
        result = []
        for page in range(1, 11):
            batch = self.request("GET", f"milestones?state=all&per_page=100&page={page}")
            result.extend(batch)
            if len(batch) < 100:
                return result
        raise ValueError("milestone inventory exceeds bounded pagination")


def synchronize(data, github, assert_no_changes=False):
    inventory = {}
    for item in github.milestones():
        if item["title"] in inventory:
            raise ValueError("ambiguous existing milestone title")
        inventory[item["title"]] = item
    changes, rows = 0, []
    for item in data["milestones"]:
        desired = {key: item[key] for key in ("title", "description", "state")}
        current = inventory.get(item["title"])
        needs_change = current is None or any(current.get(k) != v for k, v in desired.items())
        if assert_no_changes and needs_change:
            raise ValueError("second synchronization would change a milestone")
        if current is None:
            current = github.request("POST", "milestones", desired)
            changes += 1
        elif needs_change:
            current = github.request("PATCH", f"milestones/{current['number']}", desired)
            changes += 1
        for number in item["issues"]:
            issue = github.request("GET", f"issues/{number}")
            existing = issue.get("milestone")
            if existing is None or existing["number"] != current["number"]:
                if assert_no_changes:
                    raise ValueError("second synchronization would reassign an issue")
                github.request("PATCH", f"issues/{number}", {"milestone": current["number"]})
                changes += 1
        rows.append({"key": item["key"], "url": current["html_url"], "issues": item["issues"]})
    return {"changes": changes, "milestones": rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path(".github/roadmap.json"))
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--assert-no-changes", action="store_true")
    args = parser.parse_args()
    data = load_manifest(args.manifest)
    if not args.apply:
        print(json.dumps({"valid": True, "milestones": len(data["milestones"]),
                          "issues": sum(len(m["issues"]) for m in data["milestones"])}))
        return
    if (os.environ.get("GITHUB_ACTIONS") != "true"
            or os.environ.get("GITHUB_REPOSITORY") != REPOSITORY
            or os.environ.get("GITHUB_REF") != "refs/heads/main"):
        raise ValueError("writes require the reviewed main-branch Actions workflow")
    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        raise ValueError("scoped Actions token unavailable")
    result = synchronize(data, GitHub(token), args.assert_no_changes)
    print(json.dumps(result))
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as stream:
            stream.write("## Roadmap tracking\n\n")
            for row in result["milestones"]:
                stream.write(f"- [{row['key']}]({row['url']}): issues {row['issues']}\n")
            stream.write(f"\nChanges: {result['changes']}\n")


if __name__ == "__main__":
    main()

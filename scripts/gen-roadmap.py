#!/usr/bin/env python3
"""Regenerate the inventory section of ROADMAP.md from the issue tracker.

The roadmap is what people read; the tracker is where the work is recorded. This
copies the second into the first so the public document carries the substance
without anyone hand-maintaining it -- generated status is regenerable, which is
the whole difference between this and the copied status that kept going stale.

Only the block between the markers is touched. Everything above it is the
ordering and the reasoning, which no tracker can generate.

Run with `make roadmap`.
"""

import collections
import datetime
import json
import pathlib
import sys

BEGIN = "<!-- BEGIN GENERATED -- regenerate with `make roadmap`, do not edit by hand -->"
END = "<!-- END GENERATED -->"

# Below this priority an item is polish: real work, and not what someone reads a
# roadmap to find out. Counted rather than listed.
LIST_THROUGH = 2


def load(root: pathlib.Path) -> list[dict]:
    issues = []
    for path in sorted((root / ".beads" / "issues").glob("*.json")):
        try:
            issues.append(json.loads(path.read_text()))
        except (json.JSONDecodeError, OSError) as exc:
            print(f"skipping {path.name}: {exc}", file=sys.stderr)
    return issues


def parent_of(issue_id: str) -> str | None:
    """Beads encodes hierarchy in the id: persona-qbnm.7.5 sits under persona-qbnm.

    The tracker has no parent field, so the id is the only statement of it.
    """
    return issue_id.rsplit(".", 1)[0] if "." in issue_id else None


def root_of(issue_id: str) -> str:
    return issue_id.split(".", 1)[0]


def render(issues: list[dict]) -> str:
    by_id = {i["id"]: i for i in issues}
    done = {i["id"] for i in issues if i["status"] == "closed"}
    live = [i for i in issues if i["status"] != "closed"]

    counts = collections.Counter(i["status"] for i in issues)
    out = [
        BEGIN,
        "",
        f"*Generated from the issue tracker on "
        f"{datetime.date.today().isoformat()}: "
        f"{counts['open'] + counts['in_progress']} open, {counts['closed']} closed.*",
        "",
    ]

    epics = sorted(
        (i for i in issues if i["issue_type"] == "epic" and i["status"] != "closed"),
        key=lambda i: (i["priority"], i["id"]),
    )
    epic_ids = {e["id"] for e in epics}

    for epic in epics:
        family = [i for i in issues if root_of(i["id"]) == epic["id"] and i["id"] != epic["id"]]
        shut = sum(1 for i in family if i["id"] in done)
        progress = f" -- {shut} of {len(family)} done" if family else ""
        out.append(f"### {epic['title']}")
        out.append("")
        out.append(f"`{epic['id']}`{progress}")
        out.append("")
        children = sorted(
            (i for i in family if i["status"] != "closed"),
            key=lambda i: (i["priority"], i["id"]),
        )
        listed = [c for c in children if c["priority"] <= LIST_THROUGH]
        for child in listed:
            out.append(f"- **P{child['priority']}** `{child['id']}` -- {child['title']}")
        rest = len(children) - len(listed)
        if rest:
            out.append(f"- *and {rest} lower-priority item{'s' if rest != 1 else ''}*")
        if not children:
            out.append("- *no open children; the epic itself is the remaining work*")
        out.append("")

    loose = [
        i
        for i in live
        if i["issue_type"] != "epic" and root_of(i["id"]) not in epic_ids
    ]
    out.append("### Not part of an epic")
    out.append("")
    for priority in range(LIST_THROUGH + 1):
        tier = sorted((i for i in loose if i["priority"] == priority), key=lambda i: i["id"])
        if not tier:
            continue
        out.append(f"**P{priority}**")
        out.append("")
        for item in tier:
            out.append(f"- `{item['id']}` -- {item['title']}")
        out.append("")
    quiet = sum(1 for i in loose if i["priority"] > LIST_THROUGH)
    if quiet:
        out.append(f"*And {quiet} lower-priority items: polish, documentation drift and "
                   f"small cleanups. `bd ready` lists them.*")
        out.append("")

    out.append(END)
    return "\n".join(out)


def main() -> int:
    root = pathlib.Path(__file__).resolve().parent.parent
    roadmap = root / "ROADMAP.md"
    text = roadmap.read_text()

    if BEGIN not in text or END not in text:
        print(f"{roadmap.name}: generated-block markers missing", file=sys.stderr)
        return 1

    head, _, rest = text.partition(BEGIN)
    _, _, tail = rest.partition(END)
    roadmap.write_text(head + render(load(root)) + tail)
    print(f"regenerated the inventory in {roadmap.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

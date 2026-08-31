#!/usr/bin/env python3
"""Check that every EN/ZH documentation pair is structurally identical.

The rule this enforces: a translated document may differ in *prose*, and in
nothing else. Same headings in the same order at the same levels, same fenced
code blocks with the same number of lines, same tables with the same shape,
same links, same numbers -- in prose, in tables and inside code samples alike.

Anything else means the two sides have drifted, which is how one language
quietly ends up documenting a different program from the other.

Run with no arguments from the repository root. Exits non-zero on drift.
"""

import re
import sys
from pathlib import Path

# (canonical Chinese file, English mirror)
PAIRS = [
    ("README.md", "README.en.md"),
    ("CHANGELOG.md", "CHANGELOG.en.md"),
    ("CONTRIBUTING.md", "CONTRIBUTING.en.md"),
]

FENCE = re.compile(r"^\s*(?:```|~~~)(.*)$")
HEADING = re.compile(r"^(#{1,6})\s+(.*)$")
TABLE_ROW = re.compile(r"^\s*\|.*\|\s*$")
LINK = re.compile(r"\]\(([^)]+)\)")
BARE_URL = re.compile(r"(?<![(\w])(https?://[^\s<>)\"'`]+)")
NUMBER = re.compile(r"\d+(?:[.\-/^]\d+)*")
FRONTMATTER = re.compile(r"\A---\r?\n.*?\r?\n---\r?\n", re.DOTALL)

# The language switcher at the top of each file points at its counterpart, so
# those two links are the one asymmetry the rule has to allow.
SIBLINGS = {name for pair in PAIRS for name in pair}


def parse(path):
    """Split a document into the structural facts both languages must share."""
    text = FRONTMATTER.sub("", Path(path).read_text(encoding="utf-8"))
    lines = text.replace("\r\n", "\n").split("\n")

    headings, blocks, tables = [], [], []
    prose, code, cells = [], [], []
    in_fence = False
    fence_info, fence_lines = "", 0
    table_rows = 0

    for line in lines:
        fence = FENCE.match(line)
        if fence:
            if in_fence:
                blocks.append((fence_info, fence_lines))
                in_fence = False
            else:
                in_fence, fence_info, fence_lines = True, fence.group(1).strip(), 0
            continue

        if in_fence:
            fence_lines += 1
            code.append(line)
            continue

        # A table's prose differs by language; its shape and its numbers do not.
        if TABLE_ROW.match(line):
            table_rows += 1
            cells.append(line)
            continue
        if table_rows:
            tables.append(table_rows)
            table_rows = 0

        heading = HEADING.match(line)
        if heading:
            headings.append(len(heading.group(1)))

        prose.append(line)

    if in_fence:
        blocks.append((fence_info, fence_lines))
    if table_rows:
        tables.append(table_rows)

    body = "\n".join(prose)
    links = sorted(
        link
        for link in LINK.findall(body) + BARE_URL.findall(body)
        if link not in SIBLINGS
    )
    numbers = sorted(NUMBER.findall("\n".join([body] + code + cells)))

    return {
        "heading levels": headings,
        "code blocks": blocks,
        "table shapes": tables,
        "links": links,
        "numbers": numbers,
    }


def diff_summary(zh_values, en_values):
    """Show what one side has that the other does not, for list-shaped facts."""
    zh_only = [value for value in zh_values if value not in en_values]
    en_only = [value for value in en_values if value not in zh_values]
    return zh_only, en_only


def compare(zh_path, en_path):
    problems = []

    for path in (zh_path, en_path):
        if not Path(path).exists():
            problems.append("%s is missing -- every document needs both sides" % path)
    if problems:
        return problems

    zh, en = parse(zh_path), parse(en_path)

    for key in zh:
        if zh[key] == en[key]:
            continue

        zh_only, en_only = diff_summary(zh[key], en[key])
        detail = "%s has %d, %s has %d" % (
            zh_path, len(zh[key]), en_path, len(en[key])
        )
        if zh_only:
            detail += "\n      only in %s: %r" % (zh_path, zh_only[:8])
        if en_only:
            detail += "\n      only in %s: %r" % (en_path, en_only[:8])
        if not zh_only and not en_only:
            detail += "\n      same items, different order"

        problems.append("%s <-> %s: %s differ (%s)" % (zh_path, en_path, key, detail))

    return problems


def main():
    failures = []
    for zh_path, en_path in PAIRS:
        problems = compare(zh_path, en_path)
        if problems:
            failures.extend(problems)
        else:
            print("ok    %-18s <-> %s" % (zh_path, en_path))

    if failures:
        print("\nEN/ZH lockstep drift:\n", file=sys.stderr)
        for problem in failures:
            print("  - %s\n" % problem, file=sys.stderr)
        print(
            "%d problem(s). Translations may differ in prose only."
            % len(failures),
            file=sys.stderr,
        )
        return 1

    print("\nAll %d document pairs are in lockstep." % len(PAIRS))
    return 0


if __name__ == "__main__":
    sys.exit(main())

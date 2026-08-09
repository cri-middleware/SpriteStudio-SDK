#!/usr/bin/env python3
"""Generate one documentation tree per locale from the authored guide.

The Player Porting Guide is authored in `libs/ssruntime/docs` with each page next
to its translation -- `NN_name.md` and `NN_name.ja.md` in the same directory --
because that is what makes it readable on GitHub as plain Markdown, and because
SpriteStudio-Docs deep-links straight at those files. That layout is deliberate
and does not change.

Zensical, however, builds one locale per configuration, from one directory. So
this script materialises what it needs:

    libs/ssruntime/docs/                 build/docs/
      10_introduction/                     en/10_introduction/
        10_introduction.md        ---->      10_introduction.md
        10_introduction.ja.md     ---->    ja/10_introduction/
                                             10_introduction.md

It is the one piece of the MkDocs setup that had no Zensical equivalent, and it
replaces three things at once:

* `plugins.i18n` with `docs_structure: suffix`, which paired the two files and
  resolved the `.ja.md` links the Japanese pages carry;
* `docs/hooks.py`, which stripped the per-page language switcher line;
* `exclude_docs`, which kept CONTRIBUTING.ja.md out of the site.

Zensical silently ignores all three -- it reports "No issues found" and produces
a flat 47-page tree with `.ja` in half the URLs -- so none of them could simply
be carried over.

Run it before either `zensical build`; `.github/workflows/pages.yml` does.
"""

from __future__ import annotations

import re
import shutil
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SOURCE = REPO_ROOT / "libs" / "ssruntime" / "docs"
DEST = REPO_ROOT / "build" / "docs"

# Contributor-facing style rules for the guide, not part of the guide. It is also
# the one Japanese page with no English counterpart, and its `./NN_name.md`
# filename templates are not real links - published, they would fail the build.
# This is what `exclude_docs` used to do.
EXCLUDE = {"CONTRIBUTING.ja.md"}

# The language switcher every page of the guide opens with, in both directions:
#
#     > [日本語](./10_instance.ja.md) | **English**
#     > **日本語** | [English](./10_instance.md)
#
# Anchored to the start of the file, and greedy about the blank lines that follow
# so removing it does not leave a gap before the H1.
#
# It exists for GitHub, where the guide is also read as plain Markdown and there
# is no other way to reach the translation. On the site it is redundant - the
# header already carries a language selector - and neither href would survive
# this transformation: `./10_instance.ja.md` is not a file the English build
# knows about, and in the Japanese build `./10_instance.md` would resolve back to
# the Japanese page itself, a self-link nothing would catch.
LANGUAGE_SWITCHER = re.compile(
    r"""\A>[ \t]*
        (?:\*\*日本語\*\*|\[日本語\]\([^)]*\))
        [ \t]*\|[ \t]*
        (?:\*\*English\*\*|\[English\]\([^)]*\))
        [ \t]*\r?\n
        (?:[ \t]*\r?\n)*
    """,
    re.VERBOSE,
)

# Link targets only - `](...)`. The Japanese pages link to each other by the
# `.ja.md` name they are authored under, and those names do not exist in the
# generated tree. Rewriting the target rather than the whole line keeps prose
# that happens to mention a filename intact.
JA_LINK = re.compile(r"(\]\()([^)]+)(\))")


def strip_switcher(text: str, rel: Path) -> str:
    stripped, count = LANGUAGE_SWITCHER.subn("", text, count=1)
    if count == 0:
        # Every page is required to carry the switcher, per
        # libs/ssruntime/docs/CONTRIBUTING.ja.md. Under MkDocs this was a hook
        # warning that `strict` promoted to a failure; here it is a failure
        # outright, which keeps the convention true rather than merely stated.
        sys.exit(
            f"error: {rel} does not start with the '日本語 | English' switcher "
            f"line required by libs/ssruntime/docs/CONTRIBUTING.ja.md"
        )
    return stripped


def rewrite_ja_links(text: str) -> str:
    return JA_LINK.sub(
        lambda m: m.group(1) + m.group(2).replace(".ja.md", ".md") + m.group(3),
        text,
    )


def main() -> None:
    if not SOURCE.is_dir():
        sys.exit(f"error: {SOURCE} does not exist")

    # Rebuilt from scratch every run: a page deleted upstream must not linger
    # here and keep satisfying a stale nav entry.
    if DEST.exists():
        shutil.rmtree(DEST)

    counts = {"en": 0, "ja": 0}
    for src in sorted(SOURCE.rglob("*.md")):
        rel = src.relative_to(SOURCE)
        if rel.name in EXCLUDE:
            continue

        text = strip_switcher(src.read_text(encoding="utf-8"), rel)

        if rel.name.endswith(".ja.md"):
            locale = "ja"
            out = rel.with_name(rel.name[: -len(".ja.md")] + ".md")
            text = rewrite_ja_links(text)
        else:
            locale = "en"
            out = rel

        target = DEST / locale / out
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")
        counts[locale] += 1

    print(f"generated {counts['en']} en + {counts['ja']} ja pages under {DEST.relative_to(REPO_ROOT)}")

    # The two locales are built from separate configs with separate `nav` trees,
    # so a page present in one and missing from the other is a nav entry pointing
    # at nothing. Catch it here, where the cause is obvious, rather than in a
    # build error naming a generated path.
    en = {p.relative_to(DEST / "en") for p in (DEST / "en").rglob("*.md")}
    ja = {p.relative_to(DEST / "ja") for p in (DEST / "ja").rglob("*.md")}
    if en != ja:
        for missing in sorted(en - ja):
            print(f"  warning: no Japanese counterpart for {missing}", file=sys.stderr)
        for missing in sorted(ja - en):
            print(f"  warning: no English counterpart for {missing}", file=sys.stderr)


if __name__ == "__main__":
    main()

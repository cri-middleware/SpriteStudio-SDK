"""MkDocs build hooks for the Player Porting Guide.

Wired up by `hooks:` in mkdocs.yml. A plain module, not a plugin, so it adds
nothing to docs/requirements.txt.
"""

from __future__ import annotations

import logging
import re

log = logging.getLogger("mkdocs.hooks.docs")

# The language switcher every page of the guide opens with, in both directions:
#
#     > [日本語](./10_instance.ja.md) | **English**
#     > **日本語** | [English](./10_instance.md)
#
# Anchored to the start of the file, and greedy about the blank lines that follow
# so removing it does not leave a gap before the H1.
_LANGUAGE_SWITCHER = re.compile(
    r"""\A>[ \t]*
        (?:\*\*日本語\*\*|\[日本語\]\([^)]*\))
        [ \t]*\|[ \t]*
        (?:\*\*English\*\*|\[English\]\([^)]*\))
        [ \t]*\r?\n
        (?:[ \t]*\r?\n)*
    """,
    re.VERBOSE,
)


def on_page_markdown(markdown: str, page, config, files) -> str:
    """Drop the per-page language switcher line.

    That line is written for GitHub, where the guide is also read as plain
    Markdown and there is no other way to reach the translation. On the site it
    is both redundant and broken:

    * Material already renders a language selector in the header, and the i18n
      plugin points it at the correct counterpart page.
    * Neither href survives the plugin's `suffix` layout. In the English build
      `./10_instance.ja.md` is not a file the build knows about, which is a
      `strict` failure; in the Japanese build `./10_instance.md` resolves back to
      the Japanese page itself, which is worse -- a self-link that no validation
      catches.

    Stripping it in the hook is what lets the same source read correctly in both
    places without a second copy of the guide.
    """
    stripped, count = _LANGUAGE_SWITCHER.subn("", markdown, count=1)
    if count == 0:
        # Every page is required to carry the switcher (see CONTRIBUTING.ja.md).
        # A warning rather than silence because `strict` turns it into a build
        # failure, which is the only thing that keeps the convention true.
        log.warning(
            "%s does not start with the '日本語 | English' switcher line "
            "required by libs/ssruntime/docs/CONTRIBUTING.ja.md",
            page.file.src_uri,
        )
    return stripped

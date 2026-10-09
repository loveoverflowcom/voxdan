#!/usr/bin/env python3
"""Tests for the skill-structure checks in check_repository.py.

Run from the repository root:

    python3 -m unittest discover -s scripts -p 'test_*.py'
"""

import os
import tempfile
import unittest
from pathlib import Path

import check_repository as checks


VALID_SKILL = """---
name: cantos-sample
description: >-
  Sample skill used by the checker tests.
  Use when exercising validation.
---

# Sample

See [the technique](references/technique.md).
"""

VALID_OPENAI = """interface:
  display_name: "Cantos Sample"
  short_description: "A sample skill for tests"
  default_prompt: "Use $cantos-sample to exercise the checker."
"""


class SkillTree:
    """A disposable repository root holding one valid skill, its index and its Claude link."""

    def __init__(self, root):
        self.root = Path(root)
        self.skill = self.root / ".agents/skills/cantos-sample"
        (self.skill / "agents").mkdir(parents=True)
        (self.skill / "references").mkdir()
        self.write("SKILL.md", VALID_SKILL)
        self.write("agents/openai.yaml", VALID_OPENAI)
        self.write("references/technique.md", "# Technique\n")
        index = self.root / ".agents/skills/README.md"
        index.write_text("- [`cantos-sample`](cantos-sample/SKILL.md)\n", encoding="utf-8")
        claude = self.root / ".claude/skills"
        claude.mkdir(parents=True)
        os.symlink("../../.agents/skills/cantos-sample", claude / "cantos-sample")

    def write(self, relative, content):
        path = self.skill / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def errors(self):
        return checks.check_skills(self.root)


class ParseFrontmatterTest(unittest.TestCase):
    def test_folded_block_joins_lines_with_spaces(self):
        fields, errors = checks.parse_frontmatter(VALID_SKILL)
        self.assertEqual(errors, [])
        self.assertEqual(fields["name"], "cantos-sample")
        self.assertEqual(
            fields["description"],
            "Sample skill used by the checker tests. Use when exercising validation.",
        )

    def test_quoted_scalar_is_unquoted(self):
        fields, errors = checks.parse_frontmatter('---\nname: "cantos-a"\ndescription: x\n---\n')
        self.assertEqual(errors, [])
        self.assertEqual(fields["name"], "cantos-a")

    def test_missing_opening_delimiter_is_rejected(self):
        _, errors = checks.parse_frontmatter("# No frontmatter\n")
        self.assertEqual(errors, ["frontmatter must start on line 1 with ---"])

    def test_unclosed_frontmatter_is_rejected(self):
        _, errors = checks.parse_frontmatter("---\nname: cantos-a\n")
        self.assertEqual(errors, ["frontmatter is not closed with ---"])

    def test_duplicate_key_is_rejected(self):
        _, errors = checks.parse_frontmatter("---\nname: a\nname: b\n---\n")
        self.assertEqual(errors, ["duplicate frontmatter key: name"])

    def test_nested_mapping_is_outside_the_portable_subset(self):
        _, errors = checks.parse_frontmatter("---\nmetadata:\n  owner: x\n---\n")
        self.assertEqual(len(errors), 1)
        self.assertIn("unsupported frontmatter line 3", errors[0])


class CheckSkillsTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.tree = SkillTree(self.directory.name)

    def tearDown(self):
        self.directory.cleanup()

    def assertSingleError(self, fragment):
        errors = self.tree.errors()
        self.assertEqual(len(errors), 1, errors)
        self.assertIn(fragment, errors[0])

    def test_valid_tree_has_no_errors(self):
        self.assertEqual(self.tree.errors(), [])

    def test_name_must_match_directory(self):
        self.tree.write("SKILL.md", VALID_SKILL.replace("name: cantos-sample", "name: cantos-other"))
        self.assertSingleError("must equal directory name 'cantos-sample'")

    def test_non_portable_frontmatter_key_is_rejected(self):
        self.tree.write("SKILL.md", VALID_SKILL.replace("---\n\n", "allowed-tools: Bash\n---\n\n"))
        self.assertSingleError("non-portable frontmatter keys: allowed-tools")

    def test_description_over_limit_is_rejected(self):
        long_description = "x" * (checks.SKILL_DESCRIPTION_MAX + 1)
        self.tree.write(
            "SKILL.md",
            f"---\nname: cantos-sample\ndescription: {long_description}\n---\n"
            "[t](references/technique.md)\n",
        )
        self.assertSingleError(f"limit is {checks.SKILL_DESCRIPTION_MAX}")

    def test_skill_file_over_line_limit_is_rejected(self):
        padding = "line\n" * checks.SKILL_FILE_MAX_LINES
        self.tree.write("SKILL.md", VALID_SKILL + padding)
        self.assertSingleError("move depth into references/")

    def test_missing_codex_metadata_is_rejected(self):
        (self.tree.skill / "agents/openai.yaml").unlink()
        self.assertSingleError("missing agents/openai.yaml")

    def test_default_prompt_must_invoke_the_skill(self):
        self.tree.write("agents/openai.yaml", VALID_OPENAI.replace("$cantos-sample", "the skill"))
        self.assertSingleError("default_prompt must invoke $cantos-sample")

    def test_unreferenced_reference_is_rejected(self):
        self.tree.write("references/orphan.md", "# Orphan\n")
        self.assertSingleError("references/orphan.md is not linked")

    def test_reference_reached_through_another_reference_is_accepted(self):
        self.tree.write("references/technique.md", "# Technique\n\n[Deeper](deeper.md)\n")
        self.tree.write("references/deeper.md", "# Deeper\n")
        self.assertEqual(self.tree.errors(), [])

    def test_link_inside_code_fence_does_not_count(self):
        self.tree.write("SKILL.md", VALID_SKILL.replace(
            "See [the technique](references/technique.md).",
            "```text\n[the technique](references/technique.md)\n```",
        ))
        self.assertSingleError("references/technique.md is not linked")

    def test_unexpected_entry_is_rejected(self):
        self.tree.write("notes.txt", "scratch\n")
        self.assertSingleError("unexpected entry 'notes.txt'")

    def test_skill_missing_from_index_is_rejected(self):
        (self.tree.root / ".agents/skills/README.md").write_text("# Index\n", encoding="utf-8")
        self.assertSingleError("does not link cantos-sample/SKILL.md")

    def test_missing_claude_symlink_is_rejected(self):
        (self.tree.root / ".claude/skills/cantos-sample").unlink()
        self.assertSingleError("missing symlink to ../../.agents/skills/cantos-sample")

    def test_claude_symlink_with_wrong_target_is_rejected(self):
        link = self.tree.root / ".claude/skills/cantos-sample"
        link.unlink()
        os.symlink("../../elsewhere/cantos-sample", link)
        self.assertSingleError("points to '../../elsewhere/cantos-sample'")

    def test_stale_claude_symlink_is_rejected(self):
        os.symlink("../../.agents/skills/cantos-gone", self.tree.root / ".claude/skills/cantos-gone")
        self.assertSingleError("cantos-gone: no matching skill")

    def test_symlinked_claude_skills_directory_is_rejected(self):
        claude = self.tree.root / ".claude/skills"
        (claude / "cantos-sample").unlink()
        claude.rmdir()
        os.symlink("../.agents/skills", claude)
        self.assertSingleError("must be a real directory of per-skill symlinks")


class RepositorySkillsTest(unittest.TestCase):
    def test_repository_skills_pass(self):
        self.assertEqual(checks.check_skills(checks.ROOT), [])


if __name__ == "__main__":
    unittest.main()

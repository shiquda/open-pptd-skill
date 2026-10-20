#!/usr/bin/env python3
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parents[1] / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import validate_pptx as MODULE


class ValidatePptxTests(unittest.TestCase):
    def test_leaked_tags_detection(self):
        regex = MODULE.LEAKED_TAG_REGEX
        self.assertIsNotNone(regex.search('Source: <a href="https://example.com">Link</a>'))
        self.assertIsNotNone(regex.search("Items: <ul><li>Item 1</li></ul>"))
        self.assertIsNotNone(regex.search("<ol><li>First</li></ol>"))
        self.assertIsNotNone(regex.search("Leaked raw tag <p> inside compiled text"))
        # The compiler passes attribute-carrying <b>/<strong> through verbatim;
        # both spellings must be caught when they leak into DrawingML text.
        self.assertIsNotNone(regex.search('<b style="color:#B83A2C">96.2%</b>'))
        self.assertIsNotNone(regex.search('<strong style="color:#009682;">x</strong>'))
        self.assertIsNotNone(regex.search("italic <em>word</em>"))
        self.assertIsNone(regex.search("Plain text without any HTML tags"))

    def test_validation_report_properties(self):
        report = MODULE.ValidationReport(file_path="test.pptx", is_valid=True, slide_count=1)
        self.assertEqual(report.error_count, 0)
        self.assertEqual(report.warning_count, 0)

        report.issues.append(MODULE.SlideIssue(slide_index=1, severity="warning", category="test", message="warn"))
        self.assertEqual(report.error_count, 0)
        self.assertEqual(report.warning_count, 1)

        report.issues.append(MODULE.SlideIssue(slide_index=1, severity="error", category="test", message="err"))
        self.assertEqual(report.warning_count, 1)

    def test_validator_missing_file_raises(self):
        with self.assertRaises(FileNotFoundError):
            MODULE.PptxValidator(Path("/path/to/missing.pptx"))

    def test_render_thumbnails_cleans_stale_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            preview_dir = tmp_dir / "preview"
            preview_dir.mkdir()
            (preview_dir / "slide-01.png").write_text("old")
            (preview_dir / "slide-99.png").write_text("stale")
            (preview_dir / "contact-sheet.jpg").write_text("stale-cs")
            # Dummy validator instance
            dummy_pptx = tmp_dir / "dummy.pptx"
            dummy_pptx.write_bytes(b"PK\x05\x06" + b"\x00" * 18)
            validator = MODULE.PptxValidator(dummy_pptx)
            # Calling render_thumbnails cleans existing slide-*.png so stale slides from earlier runs are purged
            validator.render_thumbnails(preview_dir)
            self.assertFalse((preview_dir / "slide-99.png").exists())


if __name__ == "__main__":
    unittest.main()

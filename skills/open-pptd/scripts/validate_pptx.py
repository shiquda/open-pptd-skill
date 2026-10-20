#!/usr/bin/env python3
"""Validation and visual verification tool for compiled PPTX presentations.

Checks:
1. PPTX package integrity (OPC parts, presentation structure, slide parts).
2. Expected page/slide count.
3. Leaked unrendered HTML tags in DrawingML text nodes (<ul>, <li>, <a>, <p>, etc.).
4. Speaker notes presence, extraction, and validation.
5. Visual boundaries and potential text overflow.
6. Optional slide image rendering via headless LibreOffice, pdftoppm, or macOS QuickLook.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
import zipfile
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Optional, Sequence

# DrawingML and PresentationML namespaces
NAMESPACES = {
    "a": "http://schemas.openxmlformats.org/drawingml/2006/main",
    "r": "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    "p": "http://schemas.openxmlformats.org/presentationml/2006/main",
}

# Regex for leaked HTML tags that should have been processed or stripped
LEAKED_TAG_REGEX = re.compile(
    r"</?(?:ul|ol|li|a|div|span|p|b|strong|i|em|u|s|del|strike|font|code|h[1-6]|table|tr|td|th)(?:\s+[^>]*)?>|&(?:lt|gt|amp);",
    re.IGNORECASE,
)


@dataclass
class SlideIssue:
    slide_index: int
    severity: str  # "error" or "warning"
    category: str  # "leaked_tag", "overflow", "notes", "structure"
    message: str
    element_id: Optional[str] = None
    snippet: Optional[str] = None


@dataclass
class SlideInfo:
    index: int
    title: Optional[str] = None
    has_notes: bool = False
    notes_text: Optional[str] = None
    shapes_count: int = 0
    text_boxes_count: int = 0
    images_count: int = 0
    tables_count: int = 0


@dataclass
class ValidationReport:
    file_path: str
    is_valid: bool
    slide_count: int
    slides: list[SlideInfo] = field(default_factory=list)
    issues: list[SlideIssue] = field(default_factory=list)
    rendered_images: list[str] = field(default_factory=list)

    @property
    def error_count(self) -> int:
        return sum(1 for i in self.issues if i.severity == "error")

    @property
    def warning_count(self) -> int:
        return sum(1 for i in self.issues if i.severity == "warning")


class PptxValidator:
    def __init__(self, pptx_path: Path):
        self.path = pptx_path.expanduser().resolve()
        if not self.path.is_file():
            raise FileNotFoundError(f"PPTX file not found: {self.path}")

    def validate(
        self,
        expected_pages: Optional[int] = None,
        check_notes: bool = False,
    ) -> ValidationReport:
        report = ValidationReport(
            file_path=str(self.path),
            is_valid=True,
            slide_count=0,
        )

        try:
            with zipfile.ZipFile(self.path, "r") as z:
                self._check_package_structure(z, report)
                if not report.is_valid:
                    return report

                slide_files = self._get_slide_files(z)
                report.slide_count = len(slide_files)

                if expected_pages is not None and report.slide_count != expected_pages:
                    report.issues.append(
                        SlideIssue(
                            slide_index=0,
                            severity="error",
                            category="structure",
                            message=f"Slide count mismatch: expected {expected_pages}, got {report.slide_count}",
                        )
                    )

                # Slide size in EMUs (1 pt = 12700 EMUs)
                slide_w, slide_h = self._get_slide_size(z)

                for idx, slide_path in enumerate(slide_files, start=1):
                    info = self._analyze_slide(z, idx, slide_path, slide_w, slide_h, report)
                    report.slides.append(info)

                    if check_notes and not info.has_notes:
                        report.issues.append(
                            SlideIssue(
                                slide_index=idx,
                                severity="warning",
                                category="notes",
                                message=f"Slide {idx} has no speaker notes",
                            )
                        )

        except zipfile.BadZipFile:
            report.is_valid = False
            report.issues.append(
                SlideIssue(
                    slide_index=0,
                    severity="error",
                    category="structure",
                    message="File is not a valid zip/PPTX archive",
                )
            )

        return report

    def _check_package_structure(self, z: zipfile.ZipFile, report: ValidationReport) -> None:
        required_parts = ["[Content_Types].xml", "ppt/presentation.xml"]
        for part in required_parts:
            if part not in z.namelist():
                report.is_valid = False
                report.issues.append(
                    SlideIssue(
                        slide_index=0,
                        severity="error",
                        category="structure",
                        message=f"Missing required OpenXML part: {part}",
                    )
                )

    def _get_slide_files(self, z: zipfile.ZipFile) -> list[str]:
        # Read ppt/presentation.xml to get slides in true sequence
        slides = []
        try:
            tree = ET.fromstring(z.read("ppt/presentation.xml"))
            sld_id_lst = tree.find("p:sldIdLst", NAMESPACES)
            if sld_id_lst is not None:
                # Read presentation rels to map r:id to filename
                rels_tree = ET.fromstring(z.read("ppt/_rels/presentation.xml.rels"))
                rel_map = {}
                for rel in rels_tree.findall("{http://schemas.openxmlformats.org/package/2006/relationships}Relationship"):
                    r_id = rel.attrib.get("Id")
                    target = rel.attrib.get("Target")
                    if r_id and target:
                        # Normalize target
                        if target.startswith("/"):
                            target = target.lstrip("/")
                        elif not target.startswith("ppt/"):
                            target = f"ppt/{target}"
                        rel_map[r_id] = target

                for sld_id in sld_id_lst.findall("p:sldId", NAMESPACES):
                    r_id = sld_id.attrib.get(f"{{{NAMESPACES['r']}}}id")
                    if r_id and r_id in rel_map:
                        slides.append(rel_map[r_id])
        except Exception:
            pass

        if not slides:
            # Fallback: scan ppt/slides/slide*.xml and sort naturally
            candidates = [n for n in z.namelist() if re.match(r"^ppt/slides/slide\d+\.xml$", n)]
            candidates.sort(key=lambda s: int(re.search(r"\d+", s).group(0)))  # type: ignore
            slides = candidates

        return slides

    def _get_slide_size(self, z: zipfile.ZipFile) -> tuple[int, int]:
        try:
            tree = ET.fromstring(z.read("ppt/presentation.xml"))
            sld_sz = tree.find("p:sldSz", NAMESPACES)
            if sld_sz is not None:
                cx = int(sld_sz.attrib.get("cx", 12192000))
                cy = int(sld_sz.attrib.get("cy", 6858000))
                return cx, cy
        except Exception:
            pass
        return 12192000, 6858000  # Default 16:9 960x540 / 1920x1080

    def _analyze_slide(
        self,
        z: zipfile.ZipFile,
        slide_idx: int,
        slide_file: str,
        slide_w: int,
        slide_h: int,
        report: ValidationReport,
    ) -> SlideInfo:
        info = SlideInfo(index=slide_idx)

        try:
            content = z.read(slide_file)
            root = ET.fromstring(content)
        except Exception as e:
            report.issues.append(
                SlideIssue(
                    slide_index=slide_idx,
                    severity="error",
                    category="structure",
                    message=f"Corrupt slide XML in {slide_file}: {e}",
                )
            )
            return info

        # Check speaker notes
        notes_file = self._find_notes_slide(z, slide_file)
        if notes_file and notes_file in z.namelist():
            info.has_notes = True
            info.notes_text = self._extract_notes_text(z, notes_file)

        # Count elements & scan text for leaked HTML tags and candidate titles
        text_candidates: list[dict[str, Any]] = []
        sp_tree = root.find(".//p:spTree", NAMESPACES)
        if sp_tree is not None:
            # Check shape bounds
            for sp in sp_tree.findall(".//p:sp", NAMESPACES):
                info.shapes_count += 1
                sp_name = ""
                ph_type = ""
                nv_sp_pr = sp.find("p:nvSpPr", NAMESPACES)
                if nv_sp_pr is not None:
                    c_nv_pr = nv_sp_pr.find("p:cNvPr", NAMESPACES)
                    if c_nv_pr is not None:
                        sp_name = c_nv_pr.attrib.get("name", "")
                    ph = nv_sp_pr.find(".//p:ph", NAMESPACES)
                    if ph is not None:
                        ph_type = ph.attrib.get("type", "")

                # Check transform bounds
                y_pos = 99999999
                sp_pr = sp.find("p:spPr", NAMESPACES)
                if sp_pr is not None:
                    xfrm = sp_pr.find("a:xfrm", NAMESPACES)
                    if xfrm is not None:
                        off = xfrm.find("a:off", NAMESPACES)
                        ext = xfrm.find("a:ext", NAMESPACES)
                        if off is not None and ext is not None:
                            try:
                                x = int(off.attrib.get("x", 0))
                                y = int(off.attrib.get("y", 0))
                                y_pos = y
                                cx = int(ext.attrib.get("cx", 0))
                                cy = int(ext.attrib.get("cy", 0))

                                # Margin buffer: 5% of slide bounds
                                if x + cx > int(slide_w * 1.05) or y + cy > int(slide_h * 1.05):
                                    report.issues.append(
                                        SlideIssue(
                                            slide_index=slide_idx,
                                            severity="warning",
                                            category="overflow",
                                            element_id=sp_name or None,
                                            message=f"Shape `{sp_name}` extends beyond slide boundaries: [x={x}, y={y}, w={cx}, h={cy}] vs slide [{slide_w}x{slide_h}]",
                                        )
                                    )
                            except ValueError:
                                pass

                # Check text body
                tx_body = sp.find("p:txBody", NAMESPACES)
                if tx_body is not None:
                    info.text_boxes_count += 1
                    full_text_parts = []
                    for t in tx_body.findall(".//a:t", NAMESPACES):
                        txt = t.text or ""
                        full_text_parts.append(txt)

                        # Leaked HTML tags check
                        match = LEAKED_TAG_REGEX.search(txt)
                        if match:
                            tag = match.group(0)
                            report.issues.append(
                                SlideIssue(
                                    slide_index=slide_idx,
                                    severity="error",
                                    category="leaked_tag",
                                    element_id=sp_name or None,
                                    message=f"Found leaked/unrendered HTML tag `{tag}` in slide {slide_idx}",
                                    snippet=txt.strip()[:100],
                                )
                            )

                    joined_text = " ".join(t.strip() for t in full_text_parts if t.strip()).strip()
                    if joined_text:
                        fonts = []
                        for r in tx_body.findall(".//a:r", NAMESPACES):
                            rpr = r.find("a:rPr", NAMESPACES)
                            if rpr is not None and "sz" in rpr.attrib:
                                try:
                                    fonts.append(int(rpr.attrib["sz"]))
                                except ValueError:
                                    pass
                        for p in tx_body.findall(".//a:p", NAMESPACES):
                            ppr = p.find("a:pPr", NAMESPACES)
                            if ppr is not None:
                                def_rpr = ppr.find("a:defRPr", NAMESPACES)
                                if def_rpr is not None and "sz" in def_rpr.attrib:
                                    try:
                                        fonts.append(int(def_rpr.attrib["sz"]))
                                    except ValueError:
                                        pass
                        max_sz = max(fonts) if fonts else 0

                        text_candidates.append({
                            "name": sp_name,
                            "ph_type": ph_type,
                            "text": joined_text,
                            "max_sz": max_sz,
                            "y_pos": y_pos,
                        })

            # Detect page title properly: prefer shape named for title, else largest-font text
            info.title = self._detect_slide_title(text_candidates)

            # Count picture elements
            for _ in sp_tree.findall(".//p:pic", NAMESPACES):
                info.images_count += 1

            # Count graphicFrames (tables, charts, media)
            for gf in sp_tree.findall(".//p:graphicFrame", NAMESPACES):
                info.shapes_count += 1
                if gf.find(".//a:tbl", NAMESPACES) is not None:
                    info.tables_count += 1

        return info

    def _detect_slide_title(self, candidates: list[dict[str, Any]]) -> Optional[str]:
        """Detect slide title properly: prefer shape named for title, else largest-font text.

        Never report a body/metadata text box as title.
        """
        title_candidates = []
        other_candidates = []

        for c in candidates:
            sp_name = c["name"]
            ph_type = c["ph_type"]
            name_lower = sp_name.lower().strip()

            is_title_ph = ph_type in ("title", "ctrTitle")
            is_title_name = bool(
                re.search(
                    r"(?:^|[-_\s])(?:header[-_]?title|cover[-_]?title|page[-_]?title|slide[-_]?title|title)(?:$|[-_\s\d])",
                    name_lower,
                )
            ) or name_lower.endswith("title")

            is_body_ph = ph_type in ("body", "subTitle", "dt", "ftr", "sldNum")
            is_body_name = bool(
                re.search(
                    r"(?:^|[-_\s])(?:body|desc|description|content|notes?|footer|source|page[-_]?num|pagenum|meta)(?:$|[-_\s\d])",
                    name_lower,
                )
            )

            if is_title_ph or is_title_name:
                title_candidates.append(c)
            elif not (is_body_name or is_body_ph):
                other_candidates.append(c)

        if title_candidates:
            # Sort: title placeholder first, then font size desc, then top-most y position
            title_candidates.sort(
                key=lambda x: (
                    1 if x["ph_type"] in ("title", "ctrTitle") else 0,
                    x["max_sz"],
                    -x["y_pos"],
                ),
                reverse=True,
            )
            return title_candidates[0]["text"][:60]

        if other_candidates:
            # Sort by font size desc, then top-most y position
            other_candidates.sort(key=lambda x: (x["max_sz"], -x["y_pos"]), reverse=True)
            top = other_candidates[0]
            # Never report body text box as title: must be >= 16pt (1600 hundredths of pt) or sole text
            if top["max_sz"] >= 1600 or len(other_candidates) == 1:
                return top["text"][:60]

        return None

    def _find_notes_slide(self, z: zipfile.ZipFile, slide_file: str) -> Optional[str]:
        # slide_file is e.g. "ppt/slides/slide1.xml"
        slide_dir = Path(slide_file).parent
        slide_name = Path(slide_file).name
        rels_path = f"{slide_dir}/_rels/{slide_name}.rels"

        if rels_path in z.namelist():
            try:
                tree = ET.fromstring(z.read(rels_path))
                for rel in tree.findall("{http://schemas.openxmlformats.org/package/2006/relationships}Relationship"):
                    rel_type = rel.attrib.get("Type", "")
                    if "notesSlide" in rel_type:
                        target = rel.attrib.get("Target", "")
                        # Target is typically "../notesSlides/notesSlide1.xml"
                        resolved = os.path.normpath(os.path.join(str(slide_dir), target))
                        return resolved.replace("\\", "/")
            except Exception:
                pass
        return None

    def _extract_notes_text(self, z: zipfile.ZipFile, notes_file: str) -> str:
        try:
            tree = ET.fromstring(z.read(notes_file))
            texts = []
            # Only speaker notes text body, ignore slide thumbnail / slide number placeholders
            for sp in tree.findall(".//p:sp", NAMESPACES):
                nv_sp_pr = sp.find("p:nvSpPr", NAMESPACES)
                ph = nv_sp_pr.find(".//p:ph", NAMESPACES) if nv_sp_pr is not None else None
                ph_type = ph.attrib.get("type", "") if ph is not None else ""
                # ph type "body" is the notes text
                if ph_type == "body" or ph is None:
                    for t in sp.findall(".//a:t", NAMESPACES):
                        if t.text:
                            texts.append(t.text)
            return "\n".join(texts).strip()
        except Exception:
            return ""

    def render_thumbnails(self, output_dir: Path) -> list[Path]:
        """Attempt to render slides to PNG thumbnails via soffice or macOS QuickLook."""
        output_dir = output_dir.expanduser().resolve()
        output_dir.mkdir(parents=True, exist_ok=True)
        # Clean previous renders to avoid stale slides and double-counting in glob
        for old in output_dir.glob("slide-*.png"):
            try:
                old.unlink()
            except OSError:
                pass
        old_cs = output_dir / "contact-sheet.jpg"
        if old_cs.is_file():
            try:
                old_cs.unlink()
            except OSError:
                pass
        rendered: list[Path] = []

        # 1. Try LibreOffice / soffice
        soffice = shutil.which("soffice") or shutil.which("libreoffice")
        if not soffice and sys.platform == "darwin":
            mac_soffice = Path("/Applications/LibreOffice.app/Contents/MacOS/soffice")
            if mac_soffice.is_file():
                soffice = str(mac_soffice)

        if soffice:
            with tempfile.TemporaryDirectory() as tmp:
                tmp_dir = Path(tmp)
                # Convert PPTX -> PDF
                cmd = [soffice, "--headless", "--convert-to", "pdf", "--outdir", str(tmp_dir), str(self.path)]
                subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                pdf_file = tmp_dir / f"{self.path.stem}.pdf"
                if pdf_file.is_file():
                    # Check pdftoppm
                    pdftoppm = shutil.which("pdftoppm")
                    if pdftoppm:
                        prefix = str(output_dir / "slide")
                        cmd2 = [pdftoppm, "-png", "-r", "150", str(pdf_file), prefix]
                        subprocess.run(cmd2, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

                        raw_slides = sorted(
                            output_dir.glob("slide-*.png"),
                            key=lambda p: int(re.search(r"\d+", p.stem).group(0)) if re.search(r"\d+", p.stem) else 0,
                        )
                        # Normalize filenames to slide-01.png, slide-02.png (matching slide-NN.png contract)
                        pad_width = max(2, len(str(len(raw_slides))))
                        normalized: list[Path] = []
                        for idx, p in enumerate(raw_slides, 1):
                            target_name = output_dir / f"slide-{idx:0{pad_width}d}.png"
                            if p != target_name:
                                p.rename(target_name)
                            normalized.append(target_name)

                        if normalized:
                            cs_path = output_dir / "contact-sheet.jpg"
                            build_contact_sheet(normalized, cs_path)
                            result = list(normalized)
                            if cs_path.is_file():
                                result.append(cs_path)
                            return result

        # 2. Try macOS QuickLook
        if sys.platform == "darwin" and shutil.which("qlmanage"):
            with tempfile.TemporaryDirectory() as tmp:
                cmd = ["qlmanage", "-p", str(self.path), "-o", tmp]
                subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                tmp_path = Path(tmp)
                ql_images = sorted(tmp_path.glob("*.qlpreview/*.png"))
                pad_width = max(2, len(str(len(ql_images))))
                for idx, ql_img in enumerate(ql_images, 1):
                    dest = output_dir / f"slide-{idx:0{pad_width}d}.png"
                    shutil.copy2(ql_img, dest)
                    rendered.append(dest)
                if rendered:
                    cs_path = output_dir / "contact-sheet.jpg"
                    build_contact_sheet(rendered, cs_path)
                    result = list(rendered)
                    if cs_path.is_file():
                        result.append(cs_path)
                    return result

        return rendered


def build_contact_sheet(image_paths: list[Path], output_path: Path, cols: int = 5) -> Optional[Path]:
    """Generate a multi-page overview contact sheet grid using Pillow."""
    if not image_paths:
        return None
    try:
        from PIL import Image
    except ImportError:
        print("[open-pptd-skill] PIL/Pillow not installed; skipping overview contact sheet generation.", file=sys.stderr)
        return None

    try:
        first_im = Image.open(image_paths[0])
        w, h = first_im.size
        thumb_w = 400
        thumb_h = max(1, int(thumb_w * h / w))
        pad = 20

        n = len(image_paths)
        actual_cols = min(cols, n)
        rows = (n + actual_cols - 1) // actual_cols

        total_w = actual_cols * thumb_w + (actual_cols + 1) * pad
        total_h = rows * thumb_h + (rows + 1) * pad

        grid = Image.new("RGB", (total_w, total_h), "#EFECE1")

        for idx, img_p in enumerate(image_paths):
            im = Image.open(img_p).convert("RGB")
            im_resized = im.resize((thumb_w, thumb_h), Image.Resampling.LANCZOS)
            r = idx // actual_cols
            c = idx % actual_cols
            x = pad + c * (thumb_w + pad)
            y = pad + r * (thumb_h + pad)
            grid.paste(im_resized, (x, y))

        output_path.parent.mkdir(parents=True, exist_ok=True)
        grid.save(output_path, "JPEG", quality=90)
        print(f"[open-pptd-skill] Generated contact sheet: {output_path}")
        return output_path
    except Exception as e:
        print(f"[open-pptd-skill] Failed to generate contact sheet: {e}", file=sys.stderr)
        return None


def print_report(report: ValidationReport, verbose: bool = False) -> None:
    print("\n" + "=" * 65)
    print(f"PPTX VALIDATION REPORT: {Path(report.file_path).name}")
    print("=" * 65)
    if report.error_count > 0:
        status_str = "FAIL"
    elif report.warning_count > 0:
        status_str = "WARNINGS"
    else:
        status_str = "PASS"
    print(f"Status:       [{status_str}]")
    print(f"Slides:       {report.slide_count}")
    print(f"Errors:       {report.error_count}")
    print(f"Warnings:     {report.warning_count}")
    print("-" * 65)

    if report.slides:
        print("Slide Overview:")
        for s in report.slides:
            title = f" - \"{s.title}\"" if s.title else ""
            notes_flag = "[Notes: Yes]" if s.has_notes else "[Notes: None]"
            elem_counts = f"({s.text_boxes_count} text, {s.images_count} img, {s.tables_count} tbl)"
            print(f"  Slide {s.index:2d}: {notes_flag:<14} {elem_counts:<25} {title}")
            if verbose and s.has_notes and s.notes_text:
                indented = "\n".join(f"      {line}" for line in s.notes_text.splitlines()[:3])
                print(f"    Notes:\n{indented}")

    if report.issues:
        print("\nDiagnostics:")
        for issue in report.issues:
            prefix = "[ERROR]" if issue.severity == "error" else "[WARN] "
            slide_tag = f"Slide {issue.slide_index:2d}:" if issue.slide_index > 0 else "Global:  "
            print(f"  {prefix} {slide_tag} {issue.message}")
            if issue.snippet:
                print(f"          Snippet: \"{issue.snippet}\"")
    else:
        print("\nAll checks passed cleanly! No leaked tags or boundary errors detected.")

    if report.rendered_images:
        print("\nRendered Slide Images:")
        for img in report.rendered_images:
            print(f"  - {img}")
    print("=" * 65 + "\n")


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate a compiled PPTX presentation file",
        epilog="Exit codes: 0 = PASS (success), 1 = FAIL (errors detected), 2 = WARNINGS (warnings detected in --strict mode)",
    )
    parser.add_argument("pptx", type=Path, help="Path to input .pptx file")
    parser.add_argument("--expected-pages", type=int, default=None, help="Fail if page count does not match")
    parser.add_argument("--check-notes", action="store_true", help="Warn if any slide lacks speaker notes")
    parser.add_argument("--strict", action="store_true", help="Treat warnings as errors (exit code 2 instead of 0)")
    parser.add_argument(
        "--render-images",
        type=Path,
        nargs="?",
        const=Path("./preview_slides"),
        help="Directory to export slide preview images and contact sheet",
    )
    parser.add_argument("--json", action="store_true", help="Output machine-readable JSON")
    parser.add_argument("-v", "--verbose", action="store_true", help="Print verbose slide notes and details")
    args = parser.parse_args(argv)

    try:
        validator = PptxValidator(args.pptx)
        report = validator.validate(
            expected_pages=args.expected_pages,
            check_notes=args.check_notes,
        )

        if args.render_images:
            out_dir = args.render_images
            imgs = validator.render_thumbnails(out_dir)
            report.rendered_images = [str(p) for p in imgs]

        if args.json:
            payload = {
                "filePath": report.file_path,
                "isValid": report.is_valid,
                "status": "FAIL" if report.error_count > 0 else ("WARNINGS" if report.warning_count > 0 else "PASS"),
                "slideCount": report.slide_count,
                "errorCount": report.error_count,
                "warningCount": report.warning_count,
                "slides": [asdict(s) for s in report.slides],
                "issues": [asdict(i) for i in report.issues],
                "renderedImages": report.rendered_images,
            }
            print(json.dumps(payload, indent=2))
        else:
            print_report(report, verbose=args.verbose)

        if report.error_count > 0:
            return 1
        if args.strict and report.warning_count > 0:
            return 2
        return 0

    except Exception as err:
        print(f"[open-pptd-skill] Validation failed to run: {err}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Generate Rust char_shapes catalog tables from extracted IR fixtures."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIX = ROOT / "tests" / "fixtures" / "char_shapes"
OUT = ROOT / "src" / "serializer" / "hwpx" / "char_shape_tables"


def rust_str(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def write_lf(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(text.replace("\r\n", "\n").encode("utf-8"))


def gen_same_id() -> None:
    rows = [
        json.loads(line)
        for line in (FIX / "corpus_same_id_para_char_shapes.jsonl").read_text(
            encoding="utf-8"
        ).splitlines()
        if line
    ]
    lines = [
        "//! Same-id PARA_CHAR_SHAPE 코퍼스 — HWPX 왕복이 접으면 안 되는 경계.",
        "//!",
        "//! `scripts/extract_char_shape_ir.py` 가 샘플에서 뽑았다. 연속 동일",
        "//! `char_shape_id` 인데 `start_pos` 가 다른 entry 가 1개 이상인 문단만 담는다.",
        "//! #3500 의 `[(0,0),(34,0),(53,0)]` 도 이 목록에 있다.",
        "",
        "/// 연속 동일 id 글자모양 경계 한 건.",
        "#[derive(Debug, Clone, Copy)]",
        "pub struct SameIdPara {",
        "    /// `samples/` 상대 경로.",
        "    pub file: &'static str,",
        "    /// 원본 형식.",
        "    pub kind: &'static str,",
        "    pub section: u16,",
        "    pub para: u16,",
        "    /// PARA_TEXT UTF-16 유닛 수(컨트롤 슬롯 포함).",
        "    pub text_units: u32,",
        "    /// `(start_pos, char_shape_id)` 원본 시퀀스.",
        "    pub refs: &'static [(u32, u32)],",
        "    /// 동일 id 축약 시 사라지는 entry 수.",
        "    pub same_id_extra: u16,",
        "}",
        "",
        "/// 저장소 샘플에서 관측한 연속 동일-id 문단.",
        "pub const SAME_ID_PARAS: &[SameIdPara] = &[",
    ]
    for row in rows:
        refs = ",".join(f"({a},{b})" for a, b in row["refs"])
        lines.append(
            "    SameIdPara { file: %s, kind: %s, section: %d, para: %d, "
            "text_units: %d, refs: &[%s], same_id_extra: %d },"
            % (
                rust_str(row["file"]),
                rust_str(row["kind"]),
                row["section"],
                row["para"],
                row["text_units"],
                refs,
                row["same_id_extra"],
            )
        )
    lines.append("];")
    lines.append("")
    lines.append("/// #3500 재현 샘플 경로.")
    lines.append(
        'pub const ISSUE_3500_SAMPLE: &str = "re-multisize-10-10-empty-hancom.hwp";'
    )
    lines.append("")
    lines.append("/// #3500 원본 PARA_CHAR_SHAPE.")
    lines.append("pub const ISSUE_3500_REFS: &[(u32, u32)] = &[(0, 0), (34, 0), (53, 0)];")
    lines.append("")
    lines.append("pub fn issue_3500_row() -> Option<&'static SameIdPara> {")
    lines.append("    SAME_ID_PARAS.iter().find(|row| row.file == ISSUE_3500_SAMPLE)")
    lines.append("}")
    lines.append("")
    write_lf(OUT / "same_id_corpus.rs", "\n".join(lines) + "\n")


def gen_shape_catalog() -> None:
    rows = [
        json.loads(line)
        for line in (FIX / "corpus_char_shape_tables.jsonl").read_text(
            encoding="utf-8"
        ).splitlines()
        if line
    ]
    same_files = {
        json.loads(line)["file"]
        for line in (FIX / "corpus_same_id_para_char_shapes.jsonl").read_text(
            encoding="utf-8"
        ).splitlines()
        if line
    }
    lines = [
        "//! DocInfo CHAR_SHAPE / HWPX hh:charPr 미리보기 카탈로그.",
        "//!",
        "//! 연속 동일-id 문단이 있는 파일과 #3500 샘플의 글자모양 테이블을",
        "//! 직렬화 시험이 그대로 소비한다. 값은 추출 스크립트 실측이다.",
        "",
        "/// HWP5 CHAR_SHAPE 미리보기.",
        "#[derive(Debug, Clone, Copy)]",
        "pub struct Hwp5ShapePreview {",
        "    pub id: u16,",
        "    pub base_size: i32,",
        "    pub attr: u32,",
        "    pub font_ids: [u16; 7],",
        "    pub text_color: u32,",
        "    pub shade_color: u32,",
        "    pub italic: bool,",
        "    pub bold: bool,",
        "}",
        "",
        "/// 파일 단위 글자모양 테이블 미리보기.",
        "#[derive(Debug, Clone, Copy)]",
        "pub struct ShapeTablePreview {",
        "    pub file: &'static str,",
        "    pub kind: &'static str,",
        "    pub count: u16,",
        "    pub shapes: &'static [Hwp5ShapePreview],",
        "}",
        "",
        "/// same-id 문단이 있는 파일 + 핵심 샘플의 CHAR_SHAPE 미리보기.",
        "pub const SHAPE_TABLES: &[ShapeTablePreview] = &[",
    ]
    for row in rows:
        if row["kind"] != "hwp5":
            continue
        if row["file"] not in same_files and "re-multisize-10-10-empty-hancom" not in row["file"]:
            continue
        shapes = []
        for cs in row["preview"]:
            if "base_size" not in cs:
                continue
            fonts = ",".join(str(x) for x in cs["font_ids"])
            shapes.append(
                "        Hwp5ShapePreview { id: %s, base_size: %s, attr: %s, "
                "font_ids: [%s], text_color: %s, shade_color: %s, italic: %s, bold: %s }"
                % (
                    cs["id"],
                    cs["base_size"],
                    cs["attr"],
                    fonts,
                    cs["text_color"],
                    cs["shade_color"],
                    "true" if cs["italic"] else "false",
                    "true" if cs["bold"] else "false",
                )
            )
        if not shapes:
            continue
        lines.append("    ShapeTablePreview {")
        lines.append("        file: %s," % rust_str(row["file"]))
        lines.append('        kind: "hwp5",')
        lines.append("        count: %d," % row["count"])
        lines.append("        shapes: &[")
        lines.extend(s + "," for s in shapes)
        lines.append("        ],")
        lines.append("    },")
    lines.append("];")
    lines.append("")
    write_lf(OUT / "shape_catalog.rs", "\n".join(lines) + "\n")


def gen_encoding_matrix() -> None:
    underlines = ["NONE", "BOTTOM", "TOP"]
    lines = [
        "//! `hh:charPr` 자식 토큰 전조합 — 직렬화가 표 27·외곽선·그림자를 빠뜨리지 않는지.",
        "",
        "/// 한 칸의 밑줄/선/외곽선/그림자 방출 기대값.",
        "#[derive(Debug, Clone, Copy)]",
        "pub struct CharPrTokenCase {",
        "    pub underline_type: &'static str,",
        "    pub underline_shape: &'static str,",
        "    pub strike_on: bool,",
        "    pub strike_shape: &'static str,",
        "    pub outline: &'static str,",
        "    pub shadow: &'static str,",
        "}",
        "",
        "/// 밑줄 3 × 선 13 × 외곽선 8 × 그림자 3.",
        "pub const CHAR_PR_TOKEN_CASES: &[CharPrTokenCase] = &[",
    ]
    line_shapes = [
        "SOLID",
        "DASH",
        "DOT",
        "DASH_DOT",
        "DASH_DOT_DOT",
        "LONG_DASH",
        "CIRCLE",
        "DOUBLE_SLIM",
        "SLIM_THICK",
        "THICK_SLIM",
        "SLIM_THICK_SLIM",
        "WAVE",
        "DOUBLE_WAVE",
    ]
    outlines = [
        "NONE",
        "SOLID",
        "DASH",
        "DOT",
        "DASH_DOT",
        "DASH_DOT_DOT",
        "LONG_DASH",
        "CIRCLE",
    ]
    shadows = ["NONE", "DROP", "CONTINUOUS"]
    for ul in underlines:
        for ls in line_shapes:
            for ol in outlines:
                for sh in shadows:
                    strike_on = ls != "SOLID" and ul != "NONE"
                    strike = ls if strike_on else "NONE"
                    lines.append(
                        "    CharPrTokenCase { underline_type: %s, underline_shape: %s, "
                        "strike_on: %s, strike_shape: %s, outline: %s, shadow: %s },"
                        % (
                            rust_str(ul),
                            rust_str(ls),
                            "true" if strike_on else "false",
                            rust_str(strike),
                            rust_str(ol),
                            rust_str(sh),
                        )
                    )
    lines.append("];")
    lines.append("")
    write_lf(OUT / "encoding_matrix.rs", "\n".join(lines) + "\n")


def gen_same_id_tsv() -> None:
    rows = [
        json.loads(line)
        for line in (FIX / "corpus_same_id_para_char_shapes.jsonl").read_text(
            encoding="utf-8"
        ).splitlines()
        if line
    ]
    out = [
        "file\tkind\tsection\tpara\ttext_units\tidx\tstart_pos\tchar_shape_id\tsame_id_extra"
    ]
    for row in rows:
        for idx, (start, sid) in enumerate(row["refs"]):
            out.append(
                "\t".join(
                    [
                        row["file"],
                        row["kind"],
                        str(row["section"]),
                        str(row["para"]),
                        str(row["text_units"]),
                        str(idx),
                        str(start),
                        str(sid),
                        str(row["same_id_extra"]),
                    ]
                )
            )
    write_lf(FIX / "same_id_refs.tsv", "\n".join(out) + "\n")


def main() -> None:
    gen_same_id()
    gen_shape_catalog()
    gen_encoding_matrix()
    for name in ("same_id_corpus.rs", "shape_catalog.rs", "encoding_matrix.rs"):
        path = OUT / name
        text = path.read_text(encoding="utf-8")
        print(name, path.stat().st_size, text.count("\n"))


if __name__ == "__main__":
    main()

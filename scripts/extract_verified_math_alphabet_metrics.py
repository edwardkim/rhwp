"""SFNT의 수학 영숫자 cmap·hmtx를 추출한다. 글꼴 바이트·윤곽은 복사하지 않는다."""

import argparse
import hashlib
import json
from pathlib import Path

from fontTools.ttLib import TTFont


START = 0x1D400
END = 0x1D7FF


def read_font(path):
    with TTFont(path) as font:
        names = sorted(
            {
                record.toUnicode()
                for record in font["name"].names
                if record.nameID in (1, 16)
            }
        )
        cmap = font.getBestCmap()
        present = [
            code in cmap and font.getGlyphID(cmap[code]) != 0
            for code in range(START, END + 1)
        ]
        widths = [
            font["hmtx"][cmap[code]][0] if found else 0
            for code, found in zip(range(START, END + 1), present)
        ]
        return {
            "font_path": str(path),
            "font_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "families": names,
            "em": font["head"].unitsPerEm,
            "mac_style": font["head"].macStyle,
            "present": present,
            "widths": widths,
        }


def rust_array(name, kind, values):
    lines = [f"static {name}: [{kind}; {len(values)}] = ["]
    for index in range(0, len(values), 16):
        lines.append("    " + ", ".join(values[index:index + 16]) + ",")
    lines.append("];")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--primary-font", type=Path, action="append", required=True)
    parser.add_argument("--fallback-font", type=Path, required=True)
    parser.add_argument("--metric-name", required=True)
    parser.add_argument("--metric-em", type=int, required=True)
    parser.add_argument("--rust-output", type=Path, required=True)
    parser.add_argument("--evidence-output", type=Path, required=True)
    args = parser.parse_args()
    primary = [read_font(path) for path in args.primary_font]
    fallback = read_font(args.fallback_font)
    if not 1 <= args.metric_em <= 65535:
        raise ValueError("메트릭 em은 u16 범위여야 합니다")
    if args.metric_name not in fallback["families"]:
        raise ValueError("메트릭 이름과 실제 SFNT family가 일치하지 않습니다")
    if fallback["mac_style"] & 3:
        raise ValueError("대체 전진폭은 확인한 정체 프로그램만 사용합니다")
    if any(any(record["present"]) for record in primary):
        raise ValueError("원 face에 glyph가 있습니다. 결측 표로 축약할 수 없습니다")
    names = sorted({name for record in primary for name in record["families"]})
    if not names or any("\n" in name or "\r" in name for name in names):
        raise ValueError("정상적인 SFNT family 목록이 필요합니다")
    em = fallback["em"]
    widths = [
        (2 * width * args.metric_em + em) // (2 * em)
        for width in fallback["widths"]
    ]
    if any(not 0 <= width <= 65535 for width in widths):
        raise ValueError("정규화 전진폭이 u16 범위를 벗어납니다")
    records = {"start": START, "end": END, "primary": primary, "fallback": fallback}
    records.update(metric_name=args.metric_name, metric_em=args.metric_em)
    records["fallback"]["metric_widths"] = widths
    source = [
        "//! 확인한 SFNT의 수학 영숫자 cmap·hmtx. 기존 범위의 메트릭은 덮어쓰지 않는다.",
        "//! scripts/extract_verified_math_alphabet_metrics.py로 생성한다.",
        "use super::FontMetric;",
    ]
    for record in primary + [fallback]:
        source.append(
            "// " + ", ".join(record["families"])
            + "; SHA-256 " + record["font_sha256"]
            + "; 원 프로그램 em " + str(record["em"])
        )
    source.extend([
        rust_array("PRESENT", "bool", [str(value).lower() for value in fallback["present"]]),
        rust_array("WIDTHS", "u16", list(map(str, widths))),
        "",
        "/// 원 프로그램의 cmap 결측이다. 미수집 DB 구간을 결측으로 추정하지 않는다.",
        "pub(super) fn primary_has_glyph(family: &str, bold: bool, italic: bool, ch: char) -> Option<bool> {",
        "    if bold || italic || !(0x1D400..=0x1D7FF).contains(&(ch as u32)) { return None; }",
        "    let primary = family.split(',').next()?.trim().trim_matches('\\\'').trim_matches('\"');",
        "    match primary {",
        "        " + " | ".join(json.dumps(name, ensure_ascii=False) for name in names) + " => Some(false),",
        "        _ => None,",
        "    }",
        "}",
        "",
        "pub(super) fn entry(metric: &FontMetric, ch: char) -> Option<(bool, u16)> {",
        "    if !(0x1D400..=0x1D7FF).contains(&(ch as u32)) { return None; }",
        "    if (metric.name, metric.em_size, metric.bold, metric.italic)",
        f"        != ({json.dumps(args.metric_name, ensure_ascii=False)}, {args.metric_em}, false, false) {{ return None; }}",
        "    let index = (ch as u32 - 0x1D400) as usize;",
        "    Some((PRESENT[index], WIDTHS[index]))",
        "}",
        "",
        "pub(super) fn width(metric: &FontMetric, ch: char) -> Option<u16> {",
        "    let (present, width) = entry(metric, ch)?;",
        "    (present && width > 0).then_some(width)",
        "}",
    ])
    args.rust_output.write_text("\n".join(source) + "\n", encoding="utf-8")
    args.evidence_output.write_text(
        json.dumps(records, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()

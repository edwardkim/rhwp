"""SFNT의 수학 기호 보충 블록 cmap·hmtx를 추출한다. 글꼴 바이트·윤곽은 복사하지 않는다.

U+27C0..U+2BFF(수학 기호 A·화살표 보충·수학 기호 B·수학 연산자 보충·기타 기호와 화살표).
한/글은 이 블록을 기호 슬롯 글꼴의 실제 전진폭으로 잰다. 기존 DB가 수집하지 않은 구간을
확인한 프로그램의 값으로 보충하며 기존 범위는 덮어쓰지 않는다.

  python scripts/extract_verified_math_symbol_metrics.py \\
    --font HANBatang.ttf --metric-name "HCR Batang" --metric-em 1000 \\
    --font HANDotum.ttf --metric-name "HCR Dotum" --metric-em 1000 \\
    --rust-output src/renderer/font_metric_math_symbol_blocks.rs --evidence-output <json>
"""

import argparse
import hashlib
import json
from pathlib import Path

from fontTools.ttLib import TTFont


START = 0x27C0
END = 0x2BFF


def read_font(path, metric_name, metric_em):
    with TTFont(path) as font:
        families = sorted(
            {
                record.toUnicode()
                for record in font["name"].names
                if record.nameID in (1, 16)
            }
        )
        if metric_name not in families:
            raise ValueError(f"{metric_name}: 메트릭 이름과 실제 SFNT family가 일치하지 않습니다")
        if font["head"].macStyle & 3:
            raise ValueError(f"{metric_name}: 정체 프로그램만 사용합니다")
        cmap = font.getBestCmap()
        present = [
            code in cmap and font.getGlyphID(cmap[code]) != 0
            for code in range(START, END + 1)
        ]
        em = font["head"].unitsPerEm
        widths = [
            (2 * font["hmtx"][cmap[code]][0] * metric_em + em) // (2 * em) if found else 0
            for code, found in zip(range(START, END + 1), present)
        ]
    if any(not 0 <= width <= 65535 for width in widths):
        raise ValueError("정규화 전진폭이 u16 범위를 벗어납니다")
    return {
        "font_path": str(path),
        "font_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "families": families,
        "em": em,
        "metric_name": metric_name,
        "metric_em": metric_em,
        "present": present,
        "metric_widths": widths,
    }


def rust_array(name, kind, values):
    lines = [f"static {name}: [{kind}; {len(values)}] = ["]
    for index in range(0, len(values), 16):
        lines.append("    " + ", ".join(values[index:index + 16]) + ",")
    lines.append("];")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--font", type=Path, action="append", required=True)
    parser.add_argument("--metric-name", action="append", required=True)
    parser.add_argument("--metric-em", type=int, action="append", required=True)
    parser.add_argument("--rust-output", type=Path, required=True)
    parser.add_argument("--evidence-output", type=Path, required=True)
    args = parser.parse_args()
    if not len(args.font) == len(args.metric_name) == len(args.metric_em):
        raise ValueError("프로그램·metric key·em 개수가 다릅니다")
    if len(set(args.metric_name)) != len(args.metric_name):
        raise ValueError("metric key가 중복됩니다")
    if any(not 1 <= em <= 65535 for em in args.metric_em):
        raise ValueError("메트릭 em은 u16 범위여야 합니다")
    records = [
        read_font(path, name, em)
        for path, name, em in zip(args.font, args.metric_name, args.metric_em)
    ]
    source = [
        "//! 확인한 SFNT의 수학 기호 보충 블록(U+27C0..U+2BFF) cmap·hmtx. 기존 범위의 메트릭은 덮어쓰지 않는다.",
        "//! scripts/extract_verified_math_symbol_metrics.py로 생성한다.",
        "use super::FontMetric;",
    ]
    for index, record in enumerate(records):
        source.append(
            "// " + ", ".join(record["families"])
            + "; SHA-256 " + record["font_sha256"]
            + "; 원 프로그램 em " + str(record["em"])
            + "; DB em " + str(record["metric_em"])
        )
        source.append(rust_array(f"PRESENT_{index}", "bool", [str(v).lower() for v in record["present"]]))
        source.append(rust_array(f"WIDTHS_{index}", "u16", list(map(str, record["metric_widths"]))))
    source.extend([
        "",
        "pub(super) fn entry(metric: &FontMetric, ch: char) -> Option<(bool, u16)> {",
        f"    if !(0x{START:X}..=0x{END:X}).contains(&(ch as u32)) {{",
        "        return None;",
        "    }",
        "    let (present, widths) = match (metric.name, metric.em_size, metric.bold, metric.italic) {",
    ])
    for index, record in enumerate(records):
        source.append(
            f"        ({json.dumps(record['metric_name'], ensure_ascii=False)}, {record['metric_em']}, false, false) => "
            f"(&PRESENT_{index}, &WIDTHS_{index}),"
        )
    source.extend([
        "        _ => return None,",
        "    };",
        f"    let index = (ch as u32 - 0x{START:X}) as usize;",
        "    Some((present[index], widths[index]))",
        "}",
        "",
        "pub(super) fn width(metric: &FontMetric, ch: char) -> Option<u16> {",
        "    let (present, width) = entry(metric, ch)?;",
        "    (present && width > 0).then_some(width)",
        "}",
    ])
    args.rust_output.write_text("\n".join(source) + "\n", encoding="utf-8")
    evidence = {"start": START, "end": END, "records": records}
    args.evidence_output.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()

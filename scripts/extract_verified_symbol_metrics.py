"""SFNT cmap/hmtx에서 검증한 기술 기호 범위를 추출한다. 폰트 bytes는 저장하지 않는다."""

import argparse
import hashlib
import json
from pathlib import Path

from fontTools.ttLib import TTFont


def extract(path, family):
    raw = path.read_bytes()
    with TTFont(path) as font:
        families = {
            name.toUnicode()
            for name in font["name"].names
            if name.nameID in (1, 16)
        }
        if family not in families:
            raise ValueError("요청한 face가 SFNT family 이름에 없습니다")
        if font["head"].macStyle & 3:
            raise ValueError("이 추출기는 regular 프로그램만 지원합니다")
        cmap = font.getBestCmap()
        present = [
            codepoint in cmap and font.getGlyphID(cmap[codepoint]) != 0
            for codepoint in range(0x2300, 0x2400)
        ]
        values = [
            font["hmtx"][cmap[codepoint]][0] if found else 0
            for codepoint, found in zip(range(0x2300, 0x2400), present)
        ]
        if not all(0 <= width <= 65535 for width in values):
            raise ValueError("advance가 u16 범위를 벗어납니다")
        return {
            "family": family,
            "fontSha256": hashlib.sha256(raw).hexdigest(),
            "em": font["head"].unitsPerEm,
            "start": 0x2300,
            "end": 0x23FF,
            "widths": values,
            "present": present,
            "punctuationWidths": {
                str(codepoint): font["hmtx"][cmap[codepoint]][0]
                for codepoint in (0x00B7, 0x2018, 0x2019, 0x201C, 0x201D)
                if codepoint in cmap and font.getGlyphID(cmap[codepoint]) != 0
            },
        }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--font", type=Path, action="append", required=True)
    parser.add_argument("--family", action="append", required=True)
    parser.add_argument("--metric-name", action="append", help="명시적으로 확인한 DB face key; 생략하면 SFNT family")
    parser.add_argument("--metric-em", type=int, action="append", help="확인한 DB 단위의 em; 생략하면 SFNT unitsPerEm")
    parser.add_argument("--rust-output", type=Path, required=True)
    parser.add_argument("--evidence-output", type=Path, required=True)
    args = parser.parse_args()
    if (
        len(args.font) != len(args.family)
        or (args.metric_name and len(args.metric_name) != len(args.font))
        or (args.metric_em and len(args.metric_em) != len(args.font))
    ):
        raise ValueError("프로그램·family·metric key 개수가 다릅니다")
    if args.metric_em and any(not 1 <= em <= 65535 for em in args.metric_em):
        raise ValueError("DB em이 u16 범위를 벗어납니다")
    keys = args.metric_name or args.family
    if len(set(keys)) != len(keys) or any(c in name for name in args.family + keys for c in '\\"\n\r'):
        raise ValueError("잘못된 family 이름")
    protected = {"font_metrics_data.rs", "font_metrics_generated.rs", "font_metrics_overlays.rs"}
    if args.rust_output.resolve().name in protected:
        raise ValueError("기존 메트릭 영역을 덮어쓸 수 없습니다")
    records = [extract(path, family) for path, family in zip(args.font, args.family)]
    for index, record in enumerate(records):
        record["metricEm"] = args.metric_em[index] if args.metric_em else record["em"]
        # 기존 DB는 원 SFNT와 다른 단위로 정규화돼 있을 수 있다. cmap 유무는
        # 단위와 독립이고, 폭만 DB em으로 환산한다. 원 폭과 원 em도 증거로 남긴다.
        def in_metric_units(width):
            normalized = (2 * width * record["metricEm"] + record["em"]) // (2 * record["em"])
            if not 0 <= normalized <= 65535:
                raise ValueError("DB 단위로 환산한 advance가 u16 범위를 벗어납니다")
            return normalized
        record["metricWidths"] = list(map(in_metric_units, record["widths"]))
        record["metricPunctuationWidths"] = {
            codepoint: in_metric_units(width)
            for codepoint, width in record["punctuationWidths"].items()
        }
    source = "// scripts/extract_verified_symbol_metrics.py의 SFNT cmap/hmtx 직접 추출.\n// U+2300..U+23FF. 메트릭 결측과 실제 cmap 결측을 구분하며 기존 폭은 덮어쓰지 않는다.\n"
    for index, (record, key) in enumerate(zip(records, keys)):
        record["metricName"] = key
        source += f"// face: {record['family']}; key: {key}; SHA-256: {record['fontSha256']}; program em: {record['em']}; DB em: {record['metricEm']}.\n"
        for suffix, rust_type, values in [("WIDTHS", "u16", list(map(str, record["metricWidths"]))), ("PRESENT", "bool", [str(value).lower() for value in record["present"]])]:
            source += f"static VERIFIED_TECHNICAL_{index}_{suffix}: [{rust_type}; 256] = [\n"
            source += "".join("    " + ", ".join(values[i:i + 16]) + ",\n" for i in range(0, 256, 16)) + "];\n"
    source += "\nfn verified_technical_symbol_entry(metric: &FontMetric, ch: char) -> Option<(bool, u16)> {\n"
    source += "    if !(0x2300..=0x23FF).contains(&(ch as u32)) { return None; }\n"
    source += "    let (present, widths) = match (metric.name, metric.em_size, metric.bold, metric.italic) {\n"
    for index, record in enumerate(records):
        source += f'        ("{record["metricName"]}", {record["metricEm"]}, false, false) => (&VERIFIED_TECHNICAL_{index}_PRESENT, &VERIFIED_TECHNICAL_{index}_WIDTHS),\n'
    source += "        _ => return None,\n    };\n    let index = (ch as u32 - 0x2300) as usize;\n    Some((present[index], widths[index]))\n}\n"
    source += "\npub(crate) fn verified_program_has_glyph(metric: &FontMetric, ch: char) -> Option<bool> {\n    verified_technical_symbol_entry(metric, ch).map(|(present, _)| present)\n}\n"
    source += "\nfn verified_technical_symbol_width(metric: &FontMetric, ch: char) -> Option<u16> {\n    let (present, width) = verified_technical_symbol_entry(metric, ch)?;\n    (present && width > 0).then_some(width)\n}\n"
    source += "\n// HFT 보정이 섞인 기존 표와 별개인 원 프로그램의 실제 hmtx.\n"
    source += "pub(crate) fn verified_program_punctuation_width(metric: &FontMetric, ch: char) -> Option<u16> {\n"
    source += "    match (metric.name, metric.em_size, metric.bold, metric.italic, ch as u32) {\n"
    for record in records:
        for codepoint, width in record["metricPunctuationWidths"].items():
            source += f'        ("{record["metricName"]}", {record["metricEm"]}, false, false, {codepoint}) => Some({width}),\n'
    source += "        _ => None,\n    }\n}\n"
    args.rust_output.write_text(source, encoding="utf8")
    args.evidence_output.write_text(json.dumps(records, ensure_ascii=False, indent=2) + "\n", encoding="utf8")


if __name__ == "__main__":
    main()

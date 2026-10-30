"""기호 슬롯 글자(#8136 `is_symbol_slot_char`) 중 폭 DB 가 수집하지 않은 글자의 cmap·hmtx 를 추출한다.

글꼴 바이트·윤곽은 복사하지 않는다. 한/글 2024 Print 는 기호 슬롯 글꼴로 그리는 이 글자들을
그 글꼴의 hmtx 그대로(장치 격자 배율만 곱해) 전진시킨다(#8315 — TrueType 9종, 글자 301~344자,
벗어난 글자 0~1). 기존 DB 범위(`latin_ranges`)는 덮어쓰지 않는다.

  python scripts/extract_verified_symbol_slot_metrics.py \\
    --font C:/Windows/Fonts/batang.ttc#2 --metric-name Gungsuh --metric-em 1024 \\
    ... \\
    --rust-output src/renderer/font_metric_symbol_slot_chars.rs --evidence-output <json>

`#N` 은 TTC 안의 face 번호다.
"""

import argparse
import hashlib
import json
from pathlib import Path

from fontTools.ttLib import TTCollection, TTFont

# `style_resolver::is_symbol_slot_char` 의 범위 중 DB 공통 범위(0x2000–0x206F 등) 밖의 글자.
RANGES = [
    (0x0391, 0x03A1), (0x03A3, 0x03A9), (0x03B1, 0x03C9), (0x0401, 0x0401), (0x0410, 0x044F), (0x0451, 0x0451),
    (0x2074, 0x2074), (0x207F, 0x207F), (0x2081, 0x2084), (0x2090, 0x2094),
    (0x20A3, 0x20A4), (0x20A7, 0x20A7), (0x20A9, 0x20A9), (0x20AC, 0x20AC),
    (0x2103, 0x2103), (0x2105, 0x2105), (0x2109, 0x2109), (0x2113, 0x2113), (0x2116, 0x2116),
    (0x2121, 0x2122), (0x2126, 0x2126), (0x212B, 0x212B), (0x212E, 0x212E), (0x213B, 0x214E),
    (0x2153, 0x2154), (0x215B, 0x215E),
    (0x3200, 0x321C), (0x3260, 0x327B), (0x327F, 0x327F), (0x3380, 0x3384), (0x3388, 0x33CA),
    (0x33CF, 0x33D0), (0x33D3, 0x33D3), (0x33D6, 0x33D6), (0x33D8, 0x33D8), (0x33DB, 0x33DD),
]
CODES = [c for a, b in RANGES for c in range(a, b + 1)]


def open_font(spec):
    path, _, index = spec.partition('#')
    path = Path(path)
    if path.suffix.lower() == '.ttc':
        return path, int(index or 0), TTCollection(str(path)).fonts[int(index or 0)]
    return path, 0, TTFont(str(path))


def read_font(spec, metric_name, metric_em):
    path, index, font = open_font(spec)
    families = sorted({r.toUnicode() for r in font['name'].names if r.nameID in (1, 16)})
    if metric_name not in families:
        raise ValueError(f'{metric_name}: 메트릭 이름과 실제 SFNT family가 일치하지 않습니다 {families}')
    if font['head'].macStyle & 3:
        raise ValueError(f'{metric_name}: 정체 프로그램만 사용합니다')
    cmap = font.getBestCmap()
    em = font['head'].unitsPerEm
    present = [code in cmap and font.getGlyphID(cmap[code]) != 0 for code in CODES]
    widths = [
        (2 * font['hmtx'][cmap[code]][0] * metric_em + em) // (2 * em) if found else 0
        for code, found in zip(CODES, present)
    ]
    if any(not 0 <= w <= 65535 for w in widths):
        raise ValueError('정규화 전진폭이 u16 범위를 벗어납니다')
    return {
        'font_path': str(path), 'face_index': index,
        'font_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'families': families, 'em': em, 'metric_name': metric_name, 'metric_em': metric_em,
        'present': present, 'metric_widths': widths,
    }


def rust_array(name, typ, values, per_line):
    lines = [f'static {name}: [{typ}; {len(values)}] = [']
    for i in range(0, len(values), per_line):
        lines.append('    ' + ', '.join(values[i:i + per_line]) + ',')
    lines.append('];')
    return lines


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--font', action='append', required=True)
    ap.add_argument('--metric-name', action='append', required=True)
    ap.add_argument('--metric-em', action='append', type=int, required=True)
    ap.add_argument('--rust-output', required=True)
    ap.add_argument('--evidence-output', required=True)
    a = ap.parse_args()
    fonts = [read_font(f, n, e) for f, n, e in zip(a.font, a.metric_name, a.metric_em)]
    out = [
        '//! 확인한 SFNT 의 기호 슬롯 글자(#8136 `is_symbol_slot_char`) 중 폭 DB 밖 글자의 cmap·hmtx.',
        '//! 기존 범위의 메트릭은 덮어쓰지 않는다. scripts/extract_verified_symbol_slot_metrics.py 로 생성한다.',
        '//! 한/글 2024 Print 는 이 글자들을 기호 슬롯 글꼴의 hmtx 그대로 전진시킨다(#8315). HY 글꼴은 cmap 에',
        '//! 그리스·키릴이 있어도 한/글이 쓰지 않아(대체 글꼴로 그림) 넣지 않았다.',
        'use super::FontMetric;',
        '',
    ]
    out += rust_array('CODES', 'u16', [f'0x{c:04X}' for c in CODES], 10)
    for i, f in enumerate(fonts):
        out.append('')
        out.append(f"// {', '.join(f['families'])}; SHA-256 {f['font_sha256']}; face {f['face_index']}; "
                   f"원 프로그램 em {f['em']}; DB em {f['metric_em']}")
        out += rust_array(f'PRESENT_{i}', 'bool', ['true' if p else 'false' for p in f['present']], 15)
        out += rust_array(f'WIDTHS_{i}', 'u16', [str(w) for w in f['metric_widths']], 16)
    out += ['', 'pub(super) fn entry(metric: &FontMetric, ch: char) -> Option<(bool, u16)> {',
            '    let code = u16::try_from(ch as u32).ok()?;',
            '    let index = CODES.binary_search(&code).ok()?;',
            '    let (present, widths) = match (metric.name, metric.em_size, metric.bold, metric.italic) {']
    for i, f in enumerate(fonts):
        out.append(f"        (\"{f['metric_name']}\", {f['metric_em']}, false, false) => (&PRESENT_{i}, &WIDTHS_{i}),")
    out += ['        _ => return None,', '    };', '    Some((present[index], widths[index]))', '}', '',
            'pub(super) fn width(metric: &FontMetric, ch: char) -> Option<u16> {',
            '    let (present, width) = entry(metric, ch)?;',
            '    (present && width > 0).then_some(width)', '}', '']
    Path(a.rust_output).write_text('\n'.join(out), encoding='utf-8', newline='\n')
    Path(a.evidence_output).write_text(json.dumps(
        [{k: v for k, v in f.items() if k not in ('present', 'metric_widths')} | {
            'present_count': sum(f['present'])} for f in fonts], ensure_ascii=False, indent=1), encoding='utf-8')
    for f in fonts:
        print(f['metric_name'], 'present', sum(f['present']), '/', len(CODES))


if __name__ == '__main__':
    main()

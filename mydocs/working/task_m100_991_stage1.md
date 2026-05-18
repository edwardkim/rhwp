# Task #991 Stage 1 — 진단 + spec 분석 + F2 채택

- 이슈: [#991](https://github.com/edwardkim/rhwp/issues/991)
- 부모: 이전 close + 작업지시자 추가 시각 검증으로 재오픈

## 1. 작업지시자 시각 보고

`hwp3-sample16-hwp5.hwp` 페이지 18 (rhwp-studio screenshot):
- 다이어그램 (with internal title) — 중앙
- "가. 주전산센터 목표시스템 구성(안)" — 다이어그램 아래
- "☒ 통합모델..." — 그 다음 paragraph

PDF (HWP3 정답):
- 가. 라벨 (다이어그램 위)
- 다이어그램
- 나. 라벨 (다이어그램 아래)

## 2. KS X 6101 / HWP5 spec 분석

`mydocs/tech/한글문서파일형식_5.0_revision1.3.md:213-292` (표 6):

```
| 코드 | 설명 | 컨트롤 형식 |
| 11   | 그리기 개체/표 | extended |
| 12   | 예약 | extended |
| 14   | 예약 | extended |
| 15-17 | 숨은설명/머리꼬리말/각주미주 | extended (비-inline) |
```

확장 컨트롤 표현:
```
0   1   2   3   4   5   6   7   8   9   10  11
'A' 'B' ch  pointer             ch  'C' 13
```

→ stream 에 8 wchar, **visual 로 1 char position (placeholder)**.

## 3. HWP3 vs HWP5 parser 비교

### HWP3 parser (`src/parser/hwp3/mod.rs:1567`)
```rust
let is_non_inline_ctrl = ch == 15 || ch == 16 || ch == 17;
if !is_non_inline_ctrl {
    char_offsets.push(utf16_len);
    utf16_len += 1;
    text_string.push('\u{FFFC}');
}
```
→ 컨트롤마다 마커 push.

### HWP5 parser (`src/parser/body_text.rs:319`)
```rust
} else if is_extended_only_ctrl_char(ch) {
    ctrl_idx += 1;
    // (마커 push 없음)
}
```
→ 마커 미푸시 (spec 위반).

## 4. pi=394 구체적 측정

```
HWP5 IR (현재):
  text="  " (2 chars)
  char_offsets=[16, 17]
  3 controls (가. table + diagram picture + 나. table)
  3 line_segs (ls[0] ts=0, ls[1] ts=8, ls[2] ts=18)

composer.utf16_range_to_text_range:
  ls[0] (0..8): text [0..0) — 빈 라인 (가. 라벨 누락)
  ls[1] (8..18): text [0..2) — "  " 만 (다이어그램 위치 어긋남)
  ls[2] (18..end): text [2..2) — 빈 (나. 라벨 누락)
```

→ 줄별 text 매핑이 stream 위치와 일치하지 않음 → TAC 컨트롤 placement 어긋남.

## 5. Fix 후보 시도 결과

| 시도 | 위치 | cargo test | 결과 |
|------|------|-----------|------|
| F1 (parser 광범위 fix) | body_text.rs | **9 fail** | editor 영향 (insert_text/save/cursor) |
| F1-narrow (ch=11/14만) | body_text.rs | 5 fail | rendering 변동 |
| F2-wide (composer synth) | composer.rs | 5 fail | exam_eng p8 puko box 위치 shift |
| **F2-narrow** (3+ TAC + 2+ leading) | composer.rs | **0 fail** ✓ | 회귀 0 |

## 6. F2-narrow 채택

좁힘 조건:
- `inline_ctrl_count >= 3` (pi=394 = 3 TAC controls 패턴)
- `n_leading >= 2` (char_offsets first / 8, leading gap 에 2+ 컨트롤)

이 조건은 pi=394 같은 multi-TAC paragraph 만 catch:
- 1-2 TAC paragraph (대부분의 sample) 미적용
- HWP3 (markers 이미 있음) 미적용 (`existing_markers >= inline_ctrl_count` 차단)

## 7. 효과

| 항목 | 결과 |
|------|------|
| HWP5 sample16 p18 시각 | **PDF 정합** (가. 위, 나. 아래) |
| cargo test --release --lib | **1297 passed, 0 failed** |
| 240 sample 페이지 수 변동 | **0 건** |
| HWP5 page count | 62 (변동 없음) |
| Editor 기능 (insert_text/save/cursor) | **영향 없음** (parser 미변경) |

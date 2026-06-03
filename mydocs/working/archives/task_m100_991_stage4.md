# Task #991 Stage 4 — 시각 검증

- 이슈: [#991](https://github.com/edwardkim/rhwp/issues/991)
- 선행: [Stage 3 회귀 검증](task_m100_991_stage3.md)

## 1. HWP5 sample16 페이지 18

### Pre-fix (작업지시자 rhwp-studio screenshot)
- 박스 본문 (○ 5 items)
- 다이어그램 (with internal "주전산센터 목표시스템 구성(안)" 제목)
- "가. 주전산센터 목표시스템 구성(안)" 라벨 — **다이어그램 아래**
- "☒ 통합모델..." 다음 paragraph

→ PDF 정답: 가. 라벨이 다이어그램 위. 우리 결과: 라벨이 아래 (어긋남).

### Post-fix (`/tmp/final5_p18.png`)
- 박스 본문 (○ 5 items)
- "가. 주전산센터 목표시스템 구성(안)" — **다이어그램 위** ✓
- 다이어그램 (with internal title)
- "나. 주요 과업내용" — **다이어그램 아래** ✓
- "☒ 통합모델..." 다음 paragraph

→ **PDF 정합** 확인.

## 2. 측정 데이터

```
pi=394 (HWP5 sample16):
  text="  " (2 chars, original)
  char_offsets=[16, 17]
  3 controls (가. table + diagram picture + 나. table)
  3 line_segs (ls[0] ts=0, ls[1] ts=8, ls[2] ts=18)

F2 synth (composer 내부, para 원본 영향 없음):
  text="\u{FFFC}\u{FFFC}  \u{FFFC}" (5 chars)
  char_offsets=[0, 8, 16, 17, 18]

composer.utf16_range_to_text_range (post-synth):
  ls[0] (0..8):  [0..1) — \u{FFFC} (가. 라벨) ✓
  ls[1] (8..18): [1..4) — \u{FFFC}  (다이어그램 + 공백) ✓
  ls[2] (18..end): [4..5) — \u{FFFC} (나. 라벨) ✓
```

## 3. 다른 sample 시각 회귀

- HWPX sample16 (multi-TAC + leading 2): F2 조건 미해당 (parser path 다름) — 시각 미변동
- exam_eng p8 (puko box): F2 조건 미해당 (3+ ctrl 아님) — 시각 미변동
- HWP3 sample16: 이미 markers 있음 → F2 자동 차단 — 시각 미변동

## 4. 종합

| 항목 | 결과 |
|------|------|
| HWP5 sample16 p18 | PDF 정합 ✓ |
| HWPX sample16 p19 | 미변동 (별도 fundamental 한계) |
| exam_eng p8 puko box | 미변동 ✓ |
| 그 외 sample | 미변동 ✓ |

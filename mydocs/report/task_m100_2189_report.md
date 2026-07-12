# Task M100 #2189 최종 보고 — 고정폭 테두리 글상자 우측 텍스트 클리핑

- 이슈: #2189 (movieisover 리포트, 0.7.18) / 브랜치: `local/task2189`
- 기간: 2026-07-11 / 시각 판정: **통과** (작업지시자, 한컴 2022 PDF 정답지 3-way 대조)
- 재현 파일: `samples/basic/issue1994_behindtext_table_20200830.hwp` p2 성명서 박스

## 결론

성명서 박스(실체는 1×1 표 셀)의 우측 클리핑을 **레이아웃 층 자간 스필오버**로
해소. 클립 초과 12줄/최대 +15.1px → 0줄. 줄 끝 글자 구성이 한컴 정답지와 동일.
Chrome 확장/WebView/native 공통의 엔진 레벨 결함이므로 단일 정정으로 3환경 모두
해소된다 (4-backend는 동일 레이아웃 좌표 소비).

## 원인 (2단계 진단 — 가설 2 확정)

- 저장 줄바꿈(LINE_SEG ts) 문서 + 미보유 폰트("08서울한강체 M") 조합.
- 한컴이 실폰트 폭(전각 0.85~0.91em)으로 나눈 줄을 우리 폴백 메트릭(+4.9%)으로
  그리면 줄이 셀 내부 폭(461.1px, 한컴 PDF 실측 461.0px와 일치)을 넘어
  셀 클립에서 마지막 글자가 절단.
- 기존 Justify 음수 슬랙 압축은 공백 −50% 클램프가 상한 — 잔여 초과를 자간으로
  넘기는 경로 부재.
- 가설 1(available width inset 미차감)은 기각 — 시작 x·내부 폭 계산 모두 정확.

## 정정 (3단계)

`compute_line_extra_spacing` Justify(공백 있음) 분기
([src/renderer/layout/paragraph_layout.rs](../../src/renderer/layout/paragraph_layout.rs)):

1. 공백 클램프 도달 후 잔여 음수 슬랙을 `extra_char_sp`로 분배
   (공백-없는 분기와 동일한 −avg_char_w×0.5 하한).
2. narrow glyph per-char 클램프(#229)의 되돌림은 실효 폭 재측정 수렴 반복
   (최대 3회, underflow 확장 분기와 동일 패턴)으로 흡수.
3. 가드: `in_cell && !has_tabs`, ecs≤0. 하드코딩 없음 — 근거는 저장 줄바꿈 +
   셀 선언 폭 + 측정 자연 폭.

## 게이트 + 검증

| 항목 | 결과 |
|------|------|
| fmt / clippy (release-test, all-targets) | 통과 / 0 |
| `cargo test --profile release-test --tests` | 3,043 / 실패 0 |
| 표적 테스트 신설 `tests/issue_2189_cell_text_clip.rs` | 수정 전 FAILED(+11.7px) → 수정 후 ok (판별력 실증) |
| golden svg_snapshot | 8/8 무변동 |
| OVR baseline 5샘플 (±2px) | 개체 회귀 0건 |
| #1994 영향권 (동일 파일) | 통과 |
| 시각 판정 | **통과** — `output/poc/issue2189/compare_3way_{box,zoom}.png` |

부수 확인: 잔여 클립 근접 글리프는 협폭 문자('m', '\*', ')')의 전각-폭 측정 허수로
수정 전과 완전 동일(비회귀). `--font-path` 실폰트 지정만으로는 미해결임을 실증
(측정이 내장 font_metrics_data 사용) → 보완축 #2206 분리 등록.

## 산출물

- 커밋: `06159331`(2단계 진단), `b85f8533`(정정+표적 테스트+3단계 보고)
- 문서: `working/task_m100_2189_stage2.md`, `working/task_m100_2189_stage3.md`, 본 보고서
- 연계 이슈: #2206 (08서울한강체 M/L 메트릭 2계층 등록 — 별도 처리)

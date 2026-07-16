# 최종 결과보고서 — Task M100 #2292: skia 차트(RawSvg) PNG 잘림 정정

- 이슈: #2292 / 브랜치: `local/task2292` / 작성일: 2026-07-15
- 계획: `plans/task_m100_2292.md` (승인됨, 버그 정정형 — 구현계획서 생략)
- **작업지시자 시각 판정 통과** (2026-07-15, `output/png/chart_check_after/` 5종)

## 요약

`export-png`(native-skia)에서 차트 전 유형이 잘리던 결함을 1곳 정정으로
해소. 근인은 조각 좌표계 계약 불일치 — RawSvg 조각은 페이지 절대 좌표로
방출되는데(전 방출처 확인: 차트/OLE/HMAPSI), skia 래스터 wrapper 가
`viewBox="0 0 w h"` 로컬 가정. **viewBox(0,0) 클리핑 + bbox 재배치의
이중 오프셋**으로 잉크가 우하단 스트립에만 남았다(실측 x 291..537).

## 정정

`src/renderer/skia/image_conv.rs` — `rasterize_svg_fragment_to_png` 에
조각 원점(src_x, src_y) 전달, `viewBox="{x} {y} {w} {h}"` (페이지 좌표 창).

## 검증

| 게이트 | 결과 |
|--------|------|
| 표적 `issue_2292_chart_png_clip` (신설, 좌측·상단 1/3 잉크) | 수정 전 **FAILED 실증** → 통과 |
| default 전수 / native-skia 전수 | **3,191/0** / **3,241/0** |
| fmt / clippy | 통과 / 0 |
| OVR 5샘플 (분리 폴더 `output/poc/task2292/`) | 회귀 0건 |
| PDF 무영향 | `renderer/pdf.rs` 의 image_conv 접점 없음 확인 |
| **시각 판정** | **통과** — before(`chart_check/`) 백지+파편 → after(`chart_check_after/`) 캔들·고저선·마커·격자·범례 정위치 |

## 파생 발견 → #2293 분리

조각 래스터의 **텍스트 전량 소실**은 별개 기존 결함 — 조각에 `<text>`
12개 포함 확인 + fontdb 폴백을 실존 폰트로 바꾸는 실험에서 잉크 0→451
증명. 근인(ttfs/ 미로딩 + generic 폴백 하드 고정)과 함께 #2293 등록,
후속 타스크로 처리.

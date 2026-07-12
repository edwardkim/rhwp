# Task M100 #2189 — 3단계 완료 보고: 셀 Justify 오버플로우 자간 스필오버

- 이슈: #2189 / 브랜치: `local/task2189` / 작성일: 2026-07-11
- 선행: 2단계 진단(가설 2 확정 — 저장 줄바꿈 + 폴백 폰트 advance +4.9%)

## 구현

`compute_line_extra_spacing` Justify(공백 있음) 분기
([src/renderer/layout/paragraph_layout.rs](../../src/renderer/layout/paragraph_layout.rs)):

1. **스필오버**: 공백 압축이 클램프(−space_base_w×0.5)에 도달하고도 음수 슬랙이
   남으면, 잔여분을 `extra_char_sp`(자간)로 분배 — 기존 공백-없는 분기와 동일한
   `−avg_char_w×0.5` 하한 준용.
2. **수렴 반복**: narrow glyph per-char 클램프(#229 암묵지 2)가 음수 자간 기여
   일부를 되돌려 선형 1회 분배로는 +3.7px 잔여가 남았다. underflow 확장 분기
   (#229 암묵지 3)와 동일하게 `estimate_text_width`(ews·ecs 반영)로 실효 폭을
   재측정하며 최대 3회 수렴. 후행 공백 폭은 목표에서 제외.
3. **가드**: `in_cell && leftover < 0 && total_char_count > 1 && !has_tabs`,
   ecs는 `min(0.0)` — 이 경로에서 확장 방향 진입 금지. 셀 밖 본문 비영향.

하드코딩 없음 — 사용 근거는 저장 줄바꿈(LINE_SEG ts) + 셀 내부 폭(선언 필드) +
측정 자연 폭뿐. `suppress_cell_overflow_spacing`(초과 15%↑ 압축 포기)은 기존대로
선행 가드 유지.

## 정량 효과 (성명서 셀, SVG 실측)

| 상태 | 클립 초과 줄 | 최대 초과 | 최대 우측 끝 |
|------|------------|----------|------------|
| 수정 전 | 12줄 | +15.1px | 560.6 |
| 스필오버(선형 1회) | 1줄 | +3.7px | 549.2 |
| **+ 수렴 반복 (최종)** | **0줄** | **0** | **545.0 (< 클립 545.6)** |

PNG 시각 확인: 이전에 잘리던 마지막 글자들("이었/손가락/'너/준/극단/국가적/예배")
전부 테두리 안에 온전 (`output/poc/issue2189/compare_border_zoom.png`).

## 표적 회귀 테스트 신설

`tests/issue_2189_cell_text_clip.rs` — p2 SVG에서 성명서 셀 클립 rect를 찾아
셀 안 모든 글리프의 우측 끝 ≤ 클립 우측 +1px 검증.

- 판별력 실증: 수정 전 소스에서 **FAILED (+11.7px, 글리프 "준")** / 수정 후 ok.

## 게이트

| 항목 | 결과 |
|------|------|
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --profile release-test --all-targets` | 경고/에러 0 |
| `cargo test --profile release-test --tests` | **3,043 통과 / 실패 0** (신설 1 포함) |
| golden `svg_snapshot` | 8/8 (골든 diff 0 — 기존 시각 스냅샷 무변동) |
| `issue_1994` 영향권 (동일 파일) | 통과 |
| OVR baseline 5샘플 (±2px) | **개체 회귀 0건** |

> OVR 1차 실행에서 exam_science "5건"이 표기됐으나, 5샘플이 출력 폴더를 공유한
> 실행 방식 오류(산출물 교차 오염)였다. 매뉴얼 방식(샘플별 분리 폴더)으로 전량
> 재실행하여 5샘플 모두 0건 확정.

## 두-경로/4-backend 점검

- `compute_line_extra_spacing` 호출부는 단일(레이아웃 층) — svg/canvas/paint/json
  4-backend는 동일 레이아웃 좌표를 소비하므로 자동 정합.
- 측정·배치 5곳(#229)은 손대지 않음 — `estimate_text_width`/`compute_char_positions`
  기존 extra_char_spacing 소비 경로를 그대로 사용.

## 연계

- #2206 (08서울한강체 M/L 메트릭 등록, 별도 이슈) — 측정 정밀도 보완축.
- #1994 영향권: 동일 재현 파일의 기존 테스트
  `issue_1994_behindtext_paper_table_not_overlapped` 통과 (전수 게이트 포함).

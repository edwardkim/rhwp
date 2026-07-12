# Task M100 #2207 — 3단계 완료 보고: 오버레이 그림 앵커-시점 기준 확장

- 이슈: #2207 / 브랜치: `local/task2207` / 작성일: 2026-07-11

## 구현

[table_layout.rs](../../src/renderer/layout/table_layout.rs) 셀 비인라인 그림 분기에
`overlay_para`(BehindText|InFrontOfText + vert=Para) 조건 신설 — #577의 앵커-시점
기준(첫 LINE_SEG vpos, 폴백 para_y_before_compose)을 공유한다. +13줄 단일 지점.

- 다른 wrap(Square 등)과 후속 가드(`unrestricted_take_place_cell_float`,
  `top_and_bottom_para` pic_y 오버라이드)는 비접촉.
- 하드코딩 없음 — 근거는 wrap/vert_rel_to 스펙 필드 + LINE_SEG vpos.

## 정량 효과 (재현 파일 p1 픽토그램)

| 항목 | 수정 전 | 수정 후 | 한컴 2022 |
|------|--------|--------|----------|
| 이미지 요소 y | 87.56px | **63.56px** (산술 예측 정확 일치) | (잉크 top 65px) |
| 잉크 top/bottom | 90/121 (하단 ~12px 절단) | **66/117** | 65/117 |

픽토그램 전체(전화기 + "Phone" 라벨) 복원, 클리핑 해소 — 한컴과 1px 이내 정합.

## 표적 회귀 테스트

`tests/issue_2207_cell_overlay_picture_anchor.rs` — p1 SVG에서 픽토그램 이미지
요소(45.5×42.8px)를 찾아 y<70(앵커 문단 시작 기준) + 하단<115(클립 내) 검증.

- 판별력 실증: 수정 전 소스에서 **FAILED (y=87.6)** / 수정 후 ok.

## 게이트

| 항목 | 결과 |
|------|------|
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --profile release-test --all-targets` | 경고/에러 0 |
| `cargo test --profile release-test --tests` | **3,044 통과 / 실패 0** (신설 1 포함) |
| golden svg_snapshot | 8/8 (전수에 포함, 무변동) |
| OVR baseline 5샘플 (±2px, 샘플별 분리 폴더) | **개체 회귀 0건** — exam_science(#577 원 재현 파일) 포함 |

## 시각 판정 자산 (4단계)

`output/poc/issue2207/compare_3way_pictogram.png` — 한컴/수정 전/수정 후 병치.

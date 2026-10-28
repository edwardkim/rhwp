# 단계 완료 보고 — Task M100 #1925 2단계: 추출 1 (ClickHere/책갈피 마커)

- 작성일: 2026-07-05 / 브랜치: `local/task1925`

## 수행 내용

`layout_composed_paragraph`의 E 블록(ClickHere 필드 안내문·[누름틀 시작/끝] 조판부호
마커 + 책갈피 마커 + shift 적용, 323줄)을 `layout_click_here_and_bookmark_markers`로
추출했다 (동작 불변, 코드 이동만).

- 함수명: 구현계획서의 `layout_click_here_markers`(안)에서 블록이 책갈피 마커도 포함함을
  반영해 `layout_click_here_and_bookmark_markers`로 확정.
- 시그니처: `&self` + 15 파라미터(tree/line_node는 `&mut`, 나머지 읽기) → **`f64`
  (accumulated_shift) 반환** — caller가 `x +=`로 가산해 원본의 `x += accumulated_shift`
  의미를 보존. mut 캐리오버를 반환값으로 치환한 것 외 로직 무변경.
- 이동 조정 3건(전건 기계적): `char_offset`→`line_char_end` 파라미터화,
  `composed.para_style_id`→`para_style_id` 파라미터화, `&cell_ctx`→`cell_ctx`(참조 수령).

## 게이트 결과 (전수 통과)

| 게이트 | 결과 |
|---|---|
| cargo fmt --check | 통과 |
| cargo clippy --profile release-test --all-targets | **경고 0** (독스트링 lint 2건 발생 → 문구 수정으로 해소) |
| cargo test --profile release-test --tests | **2,875 통과 / 실패 0** |
| OVR baseline 5샘플 (기준 00014ecf) | **5/5 개체 회귀 0건** |

## 계측

| 지표 | 이전 | 이후 |
|---|---|---|
| `layout_composed_paragraph` 줄수 | 3,771 | **3,467** (−304) |
| 분기 지표(자체 추정) | 605 | **562** (−43) |
| 신규 함수 | — | 331줄 · 분기 43 · 소스분기 0 |

공식 CC 재계측은 4단계 대시보드 스냅샷(`-r2`)에서 수행.

## 다음 단계

3단계 — 추출 2·3: `estimate_line_run_widths`(A, 반환 struct `LineWidthEst`) +
`layout_empty_runs_line`(C). 승인 후 착수.

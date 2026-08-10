---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 119 — body-top 표 상단선 half clip

## 범위

정책연구 p33 `pi=428/ci=0`의 자동 horizontal-border candidate를 최신
release-test CLI와 한컴 2020 PDF로 직접 재감사했다. text owner·표 내부 겹침과
페이지 경계는 모두 정상이나, 표 상단선이 Body clip에서 절반만 보이는 실제 paint
결함을 확정했다.

- source:
  `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- reference:
  `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 실행 바이너리 SHA-256:
  `2924b9fdf27f7c9f0403e7f2b570a5153d719de89e4b6884d8a48a5a71be8df2`

## 실측

- p33 pixel diff: `9.97%`
- PDF/rhwp text delta: `0/0`
- 표 bbox: `(94.5, 83.2, 512.2, 186.1)px`
- rhwp 상단선: y `83.2px`, stroke `1.5px`, Body clip top `83.2px`
- candidate visible-height ratio: `0.500`
- 기준 PDF vector: `M 71 63 L 455 63`, black `1pt` full stroke
- 384dpi dark run:
  - PDF 상단선 `6px`
  - rhwp 상단선 `3px`
  - rhwp 하단선 `6px`

source 선 두께가 얇은 것이 아니라 top stroke의 중심이 Body clip top과 같아서 위쪽
절반만 잘린다. SVG의 generic `Line`과 Canvas 모두 같은 exact Body clip 계약을
사용하므로 SVG 전용 문제가 아니다.

## 관련 잔여

p167→168 `pi=1775`, p213→214 `pi=2548`의 successor fragment 상단선도 같은
50% clip을 보인다. 두 경계는 text owner가 정상이나 다음 geometry 결함은 별도다.

- 시작 fragment 높이가 기준 PDF보다 `+9.82pt`
- successor fragment left/top이 기준보다 약 `-3.13pt/-2.63pt`

Stage 119는 공통 상단선 paint만 처리한다. frame height와 outer paint origin은 이
변경의 결과를 재검증한 뒤 별도 단계로 넘긴다.

## 수정 계약

- 모든 Body clip을 넓히지 않는다. 그러면 이전 fragment의 숨은 glyph가 노출될 수 있다.
- generic line과 page/body geometry를 움직이지 않는다.
- Body top과 정합된 table frame의 top border만 stroke 폭 전체가 body 안쪽에
  paint되도록 SVG와 Canvas에 동일한 계약을 적용한다.
- table bbox, cell text, 페이지 owner, 하단·좌우 border는 불변이어야 한다.

## 회귀 계획

- p33 top border는 50%가 아니라 full stroke로 보임
- 동일 표 bottom border와 bbox는 불변
- p167/p168, p213/p214 successor top border 재검증
- 숨은 text-band가 있던 issue2007 p14는 새 glyph를 노출하지 않음
- SVG/Canvas border paint 계약 focused test 및 기존 renderer snapshot

## 구현

- 공통 `render_edge_borders`가 row boundary 0에서 생성한 table top-border Line
  묶음만 검사한다.
- 실제 painted top(`center_y - stroke_width/2`)이 전달된 Body clip top보다 위면
  compound/double 선 전체를 동일 delta로 clip 안쪽 `0.05px`에 옮긴다.
- whole table은 depth 0, body-flow col area, `table_y≈body_top`일 때만 활성화한다.
- partial table은 top-level successor fragment가 같은 조건일 때만 활성화한다.
- nested/cell table, Body clip, Table/Cell bbox, 일반 Line, vertical·bottom border는
  기존 경로를 유지한다.

## 검증 결과

- border group 단위 회귀: `2/2`
- 정책연구 p33 및 successor p168·p214 실물 회귀: `2/2`
- `issue_2430_cell_rewrap_threshold`: `2/2`
- `issue_2007_nested_cell_pagination`: `15/15`
- `cargo clippy --profile release-test --all-targets -- -D warnings`: 통과
- 전체 `cargo test --profile release-test --tests` 1차 실행에서
  `svg_snapshot::form_002_page_0`만 실패했다. expected/actual 955행 중 차이는
  Stage 119가 의도적으로 옮긴 body-top 상단선 중심 `94.48 → 94.78px` 한 곳뿐이었다.
  한컴 2022 PDF(`pdf/hwpx/form-002-2022.pdf`)의 같은 선은 384dpi에서 전체
  `2~3px` 두께로 보이고, 구 golden은 Body clip 경계에서 `1px`만 남기므로
  코드 회귀가 아닌 stale golden으로 판정해 해당 한 줄만 갱신했다. 갱신 후 focused
  `form_002_page_0`은 `1/1`로 통과했고, 현재 변경 전체를 포함한
  `cargo test --profile release-test --tests`도 exit `0`으로 완료됐다.
- 최신 CLI SHA-256:
  `fe2c4d17afb507f21ed620ce716ce4e0f4f9bc92cb081ebc09c937d52fccb38b`
- p33, p168, p214의 horizontal-border clip candidate는 모두 0으로 줄었다.
- text delta와 page owner는 모두 0/0 및 불변이다.

p33 전체 pixel diff는 `9.97% → 10.04%`로 소폭 변했다. 이는 clip 밖 반쪽을 숨긴
기존 결과보다 full stroke가 기준 PDF의 1pt 선과 기하학적으로 맞더라도 portable
font·다른 잔여 geometry의 raw pixel 합계가 단조 개선을 보장하지 않기 때문이다.
직접 raster와 candidate ledger에서는 상단선 전체가 보이고 하단선·bbox는 불변이다.

최신 p33 직접 비교에서 표 캡션의 `statistics.eurotransplant.org`가 기준 PDF처럼
두 줄로 감기지 않고 첫 줄 우측으로 돌출하는 별도 결함도 확인했다. Stage 119는 이
caption rewrap을 해결한 것으로 간주하지 않으며 후속 단계에 명시적으로 이월한다.

## 증적

- [p33 PDF/rhwp 비교](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/compare_p033.png)
- [384dpi 상단선 확대](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/p033_top_border_384dpi.png)
- [픽셀 보고서](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/report.tsv)
- [horizontal-border candidate](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/horizontal-border-candidates.tsv)
- [실행 provenance](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/provenance.tsv)
- [보정 후 p33 비교](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/compare_p033_after.png)
- [보정 후 p168 비교](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/compare_p168_after.png)
- [보정 후 p214 비교](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/compare_p214_after.png)
- [보정 후 candidate 원장](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/horizontal-border-candidates-after.tsv)
- [보정 후 provenance](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/provenance-after.tsv)

이 단계의 공통 Body-top table top-stroke half clip 결함은 해결했다. p167/p213 첫
fragment 높이와 successor paint origin, p33 caption wrap은 후속 단계에서 계속한다.

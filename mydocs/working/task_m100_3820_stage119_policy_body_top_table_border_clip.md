---
kind: investigation
status: in_progress
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

## 증적

- [p33 PDF/rhwp 비교](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/compare_p033.png)
- [384dpi 상단선 확대](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/p033_top_border_384dpi.png)
- [픽셀 보고서](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/report.tsv)
- [horizontal-border candidate](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/horizontal-border-candidates.tsv)
- [실행 provenance](../pr/assets/task_m100_3820_stage119_policy_body_top_table_border_clip/provenance.tsv)

구현 및 회귀 검증 진행 중이다.

---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-05
---

# Task #3738 Stage 33 — p182 그림 67 caption/본문 겹침 보정

## 문제 재현

대상은 개인정보 제거 HWP와 한컴 2020 기준 PDF다.

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- native binary: `target/task-3820-3821-fidelity/release-test/rhwp`

215쪽 full sweep의 p182는 `line_order_overlap`·line/column/large-ink drift 후보였다. raster를
직접 대조하면 이는 오탐이 아니다. rhwp는 그림 67 caption `그림 67. 장기 기증자에 대한 HIV 감염
스크리닝 순서도`를 다음 `- 매독 전파 …` 첫 줄과 겹쳐 paint하고, 기준 PDF는 caption 아래에
분리된 첫 본문 줄을 둔다.

## source → layout 관찰

render tree에서 table anchor는 `pi=1904`, 2×1 `TopAndBottom` 표다. 첫 행의 도식은
독립 `Picture`가 아니라 셀 안의 글자처럼 취급되는 `ShapeObject::Group`이며, 이 표는
바깥 table만 비-TAC float이다.

| 항목 | render tree 좌표 | 의미 |
| --- | --- | --- |
| 그림 table | y=123.5, h=670.1 | row 0의 두 Square image + row 1 caption cell을 포함 |
| caption cell | y=776.4, h=17.1 | `그림 67…` TextLine y=778.3 |
| 다음 본문 `pi=1911` (수정 전) | y=779.9 | caption과 11.7px 겹침 |

`dump-pages --json`도 원인을 뒷받침한다. pi=1904 table의 stored `vpos=56600` 뒤에 빈 pi=1905…1909의
vpos는 59200…69600으로 진행하지만, pi=1910은 50822로 되감기고 pi=1911은 52822부터 시작한다.
현재 page layout은 이 뒤쪽 stored vpos를 table의 실제 bottom보다 우선해 caption 아래 flow cursor를
되감긴 위치로 옮긴다. `--respect-vpos-reset`은 이 입력에 zero reset이 없으므로 결과를 바꾸지 않는다.

기존의 양수 offset empty-host RowBreak 보정은 다음 항목이 즉시 실본문인 1행 그림 표만 다룬다.
이 표는 빈 guide 문단 다섯 개와 저장 vpos 되감김이 사이에 있어 그 경로를 타지 않는다. 두 행 그림·caption
표를 pagination 단계까지 동일하게 넓히면 표가 p183으로 넘어가 기준 PDF의 p182 배치를 깨므로, pagination은
기존 조건을 유지하고 layout의 physical lane floor만 보정 범위로 한다.

## 수정 계약

1. TopAndBottom 표가 실제로 소비한 visual bottom 아래로 다음 **visible** 본문 cursor를 되감지 않는다.
2. stored vpos가 table 뒤에 진행한 빈 paragraph에만 존재할 때는 그 기록을 보존하되, 이후 visible
   paragraph의 paint 위치가 table caption cell과 겹치지 않게 한다.
3. 절대배치·Square table·의도적 overlay와 page-local row-break는 건드리지 않는다.
4. p182 caption과 다음 본문 non-overlap을 render tree/raster로 고정하고, 기존 table pagination 회귀를
함께 실행한다.

## 구현

`is_two_row_picture_caption_rowbreak_table`은 첫 행의 `Picture`뿐 아니라 `Shape`도 도식으로 판정한다.
이는 그림 67처럼 셀 안 `ShapeObject::Group`이 글자처럼 취급되어도, 바깥 비-TAC `TopAndBottom` 표의
물리 paint 하단을 구성하기 때문이다.

`layout_table_item`의 empty-float lane 정산에서는 native HWP의 이 2행 그림·caption `RowBreak` 표이고
양수 vertical offset이 있을 때만 `global_y_before + reserved_height` 대신 `lanes.max_bottom()`을
다음 flow의 하한으로 쓴다. typeset/pagination의 near-fit 조건은 바꾸지 않았다. 따라서 p182 표는 기준
PDF와 같이 p182에 남고, 저장 vpos가 되감기는 뒤쪽 실본문만 caption 하단 아래에서 시작한다.

## 검증 결과

focused Rust 회귀
`issue_3738_picture_caption_float_clears_caption_before_next_body_text`가 통과했다. 수정 뒤 render tree의
`pi=1904` 표 하단은 y=793.5, 다음 visible `pi=1911` 본문 첫 줄은 y=820.2로, 26.7px의 실제 여백을
확보한다. SVG raster도 기준 PDF처럼 caption과 `- 매독 전파 …` 본문이 분리된 것을 확인했다.

갱신된 `scripts/visual_sweep.py`로 p182와 full sweep에서 마지막으로 남았던 p214를 먼저 PDF raster와
대조했다. 이어 단일 실행 상한으로 중단된 최초 실행의 checkpoint를 `--resume`으로 이어 받아 전체 215쪽을
끝까지 재실행했다.

```bash
python3 scripts/visual_sweep.py --key issue3738-stage33 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp \
  --dpi 96 --pages 182,214 --out <evidence-dir>

python3 scripts/visual_sweep.py --resume --key issue3738-stage33-full \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp \
  --dpi 96 --pages 24-215 --out <same-evidence-dir>
```

단일 p182·p214 재검증은 모두 `flagged_page_count=0`이었다. 최종 resume summary는
`run_state=complete`, `completed_page_count=215`, `missing_pages=[]`, `flagged_page_count=0`이며 frame
tail/line-order/large-ink/endnote detector도 후보를 남기지 않았다. 따라서 Stage 32에서 재실행해
무플래그를 확인한 p7, p30, p94, p115, p129, p140, p157, p160–161, p164, p167, p169, p178과 p182 수정까지
포함해 이 215쪽 개인정보 제거 HWP의 full sweep 후보는 모두 실제 PDF 대조 또는 수정 뒤 재검증으로
종결했다.

영구 증적은 [최종 summary](../pr/assets/task_m100_3738_stage33_medical_final/summary.json),
[run manifest](../pr/assets/task_m100_3738_stage33_medical_final/run_manifest.json),
[구조 지표](../pr/assets/task_m100_3738_stage33_medical_final/metrics.json),
[overlay 지표](../pr/assets/task_m100_3738_stage33_medical_final/overlay_metrics.json),
[p182 review](../pr/assets/task_m100_3738_stage33_medical_final/review_p182.png),
[p214 review](../pr/assets/task_m100_3738_stage33_medical_final/review_p214.png)에 보관했다. 모든 파일은
2.4MB 이하로 LFS 대상이 아니며 일반 Git 증적이다.

## 범위 경계

Issue #3820의 2026-08-05 추가 코멘트가 이관한 383쪽 `2025 행정업무운영 편람(최종)` HWP/HWPX/PDF
production-fidelity 비교는 별도 입력·기준 세트다. 이 Stage의 의료 정책연구 HWP를 해결한 뒤에만 그
원장으로 진행하며, 여기서 두 문서의 원인을 동일하다고 가정하지 않는다.

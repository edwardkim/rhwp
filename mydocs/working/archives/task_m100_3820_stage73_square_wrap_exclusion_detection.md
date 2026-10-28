---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 73 — Square 표 본문 누락 자동 후보화

## 문제

Stage 71의 issue4090 HWPX↔한컴 2020 PDF 전수 비교에서 p5·p7·p15·p17은 우측
non-TAC `Square` 표 옆에 있어야 할 본문이 rhwp 출력에서 빠졌지만, visual sweep은
`flagged_page_count=0`을 보고했다. Stage 72는 renderer를 보정했으나, 수정 전 같은
사용자-visible 결함을 자동 후보로 올리지 못하는 검증 공백은 남아 있다.

현재 `fidelity_compare.square_wrap_text_overlap_candidates()`는 `Image`의
`Square`/`Tight`/`Through`만 대상으로 하며 **겹침·edge clearance 상실**만 찾는다.
issue4090은 render tree상 `Table(pi=44)`이며 본문은 겹치지 않고 아예 사라졌으므로 이
규칙의 입력과 판정 양쪽을 벗어난다.

## 이번 단계의 범위

1. Stage 72 직전 커밋(`40d6c89ed`)에서 p5·p7·p15·p17을 재실행해 text ledger와 raster
   신호를 함께 측정한다.
2. `Table`의 Body 좌측 strip을 render-tree geometry로 투영해, PDF에 다수의 잉크가 있지만
   rhwp에는 거의 없는 경우만 visual-sweep **candidate flag**로 추가한다.
3. fixture unit test로 p5와 같은 dropped wrap prefix는 flag가 나고, 양쪽 strip에 같은
   본문이 있는 경우는 flag가 나지 않게 고정한다.

이 단계는 renderer layout을 다시 바꾸지 않는다. PDF reference가 있어야만 판정 가능한
후보이며, 후보는 직접 visual review를 대체하지 않는다.

## 완료 조건

- 수정 전 issue4090 p5·p7·p15·p17의 missing prefix가 자동 후보로 기록된다.
- 일반적인 빈 좌측 strip 또는 양쪽 strip에 본문이 있는 경우는 후보가 되지 않는다.
- Python 회귀 테스트를 통과하고, Stage 72 현재 출력에서는 같은 후보가 0건이다.

## 기준 재현

Stage 72 직전 commit을 격리 worktree에서 release-test로 빌드한 뒤 text-only와 180 DPI
visual sweep을 실행했다. PDF text extraction은 이 파일에 적합한 detector가 아니었다.
각 p5·p7·p15·p17은 reference-only가 4--6자뿐이고 SVG-only가 107--317자로, 한컴 PDF의
glyph text layer와 rhwp SVG text layer 차이가 실제 누락 문단을 상쇄했다.

반면 render-tree의 right-side `Table` 좌측 strip은 명확했다.

| 페이지 | table `(pi,ci)` | rhwp strip ink | PDF strip ink |
| --- | --- | ---: | ---: |
| p5 | `(44,1)`, `(52,1)` | 366, 365 | 27,380, 28,076 |
| p7 | `(61,0)`, `(69,0)` | 469, 440 | 36,237, 28,712 |
| p15 | `(168,0)`, `(176,1)` | 440, 365 | 28,285, 33,603 |
| p17 | `(185,1)`, `(193,0)` | 193, 406 | 27,424, 36,072 |

Stage 72 fix 뒤 같은 좌측 strip은 rhwp 30,094--40,568px, PDF 27,380--36,237px로 회복됐다.
따라서 text-extraction 후보는 이 case의 gate로 채택하지 않고, PDF/rhwp raster와 Body direct
`Table` geometry를 결합한 `right_table_left_strip_text_deficit`만 추가한다. 최소 PDF ink
density `0.025`, rhwp/PDF ink ratio `0.15` 이하로 한정해 빈 standalone table과 폰트 baseline
차이는 제외한다. 이 flag는 PDF visual review가 필요한 candidate이지 자동 결함 확정이 아니다.

## 구현·검증 결과

`scripts/visual_sweep.py`에 `render_tree_right_table_left_strip_text_deficit_candidates()`를
추가했다. Body의 direct `Table`만 대상으로 하므로 nested cell table은 중복 후보가 되지 않으며,
기준 PDF와 rhwp PNG에서 같은 CSS table geometry를 raster로 투영해 좌측 strip의 content ink를
측정한다. 후보는 page metrics·summary·annotated review에 `right_table_left_strip_text_deficit`로
기록된다.

수정 전 binary sweep은 p5·p7·p15·p17 **4/4**를 flag했고, 현재 Stage 72 renderer binary의
동일 4쪽 sweep은 후보와 flag가 모두 **0건**이었다. 전후 summary, page metrics, p5 annotated
before/review after는
`mydocs/pr/assets/task_m100_3820_stage73_right_table_wrap_detection/`에 보존한다.

```bash
python3 -m py_compile scripts/visual_sweep.py tools/fidelity_compare/fidelity_compare.py
python3 -m unittest scripts/tests/test_visual_sweep.py scripts/tests/test_fidelity_compare.py

# Stage 72 직전 binary와 현재 binary 각각에 아래 sweep 실행
python3 scripts/visual_sweep.py --pages 5,7,15,17 --dpi 180 \
  --hwp samples/issue4090/156492236_규제샌드박스_min.hwpx \
  --pdf pdf/issue4090/156492236_규제샌드박스_min-hancom2020-production-verify.pdf
```

Python 회귀는 78개가 통과했다. 이 detector는 대량 문서에서 사람이 모든 페이지를 먼저
지적하지 않아도 review queue를 만드는 guard이며, PDF의 실제 table wrap 의도 자체를 자동으로
확정하지는 않는다.

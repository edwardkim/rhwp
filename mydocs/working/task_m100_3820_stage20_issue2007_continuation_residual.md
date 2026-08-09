---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 20 — issue2007 continuation 잔여 PDF 정합성

## 기준과 전 단계

기준 문서는 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`이고, 정답지는
`pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`이다. Stage 19는 terminal
1×1 continuation p17이 첫 가시 unit을 이중 예약해 32px 아래로 drift하던 결함을
`4b53e0025`로 보정했다.

동일 794×1123 해상도의 현재 macOS release SVG와 PDF 물리 p10–p17을 재대조했다.
p11의 첫 줄이 render tree에는 있으나 실제 `Body`/`Cell` clip 위(y=105.1, clip 시작
y=113.4/117.1)에 있어 사라지는 것을 확인했다. 따라서 이 단계는 p11의 상단 clip만
복원한다. p13의 text owner drift와 p9의 제목 상단 clip은 이 보정과 별개의 잔여 결함으로
다음 단계에 넘긴다.

## 이번 단계의 판정 원칙

- PDF와 같은 **물리 쪽** 및 같은 raster 크기만 비교한다.
- 글꼴 raster 차이와 실제 flow 위치 차이를 분리한다. 글자 모양만 다른 경우에는
  pagination 또는 table offset을 바꾸지 않는다.
- p13 보정 후보는 p10–p17의 page count, text owner, table frame 회귀를 모두
  유지할 때만 채택한다.

## 분석과 수정

- 원인: 중첩 RowBreak continuation에서 전달된 `col_area`는 직전 논리 viewport를
  포함할 수 있다. 깊은 셀은 물리 page bbox 밖으로 나갔어도 `Center` 정렬을 유지했고,
  p11의 앞쪽 문단군을 재배치했다.
- 1차 보정: 중첩 셀이 실제 page bbox를 가로지를 때에도 `Top` 정렬로 수렴시켰다.
  이것만으로는 p11의 첫 가시 줄이 clip 바로 위에 남았다.
- 확정 보정: 위쪽 physical clip continuation은 첫 **유효** `LINE_SEG.line_height`
  (빈 anchor의 `0` 높이는 건너뜀)만큼 origin을 예약한다. `Top`/vpos 앵커 경로가
  기존에는 `text_y_start`를 의도적으로 무시했으므로, 그 예약값을
  `cell_para_line_anchor_y`에도 명시적으로 전달했다.

이로써 p11의 `행하여야 하며, 다른 목적 …` 첫 줄은 y=105.1(clip 밖)에서 물리
clip 안으로 이동했고, PDF처럼 `제50조의4`보다 먼저 보인다. p17의 terminal tail도
그대로 유지된다.

## 증적과 검증

기준 PDF는 `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`, 입력은
`samples/basic/issue2007_nested_cell_pagination_42065.hwp`이다. 저장소 PNG는 LFS
대상이 아님을 `git check-attr filter`로 먼저 확인했다.

- [p10–p17 PDF 직접 대조 contact](../pr/assets/task_m100_3820_stage20_issue2007_continuation_residual/review_p010_p017_after_contact.png)
- [p11 확대 대조](../pr/assets/task_m100_3820_stage20_issue2007_continuation_residual/review_p011_after_pair.png)
- 페이지별 쌍: `review_p010_after_pair.png`부터 `review_p017_after_pair.png`
- [export-svg manifest](../pr/assets/task_m100_3820_stage20_issue2007_continuation_residual/export_svg_manifest_after.json)

페이지 매핑은 반드시 `PDF pN ↔ rhwp export-svg --page N-1`로 고정했다. `rhwp` CLI의
`--page`는 0-based이고, `visual_sweep.py --page`만 PDF viewer와 같은 1-based다. 앞선
증적의 p13 파일명은 이 규약을 지키지 않아 물리 p14를 p13으로 잘못 대조했으므로, 현재
각 PNG와 contact 시트는 모두 이 규약으로 다시 산출했다.

재현 명령:

```bash
CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2007_nested_cell_pagination -- --nocapture
```

추가한 focused assertion은 p11의 첫 continuation 줄이 physical nested-cell clip 안
(y=117–140)에 위치해야 한다고 검증한다. `export-svg`의
`overflowCellLines` 원장은 이 대형 RowBreak 표의 clip 밖 논리 source를 계속 기록하므로,
단독 성공 지표로 사용하지 않았고 PNG/PDF 1:1 대조와 함께 판정했다.

## 결론

p11 상단 clip은 해결했다. 그러나 현재 physical p13은 PDF의 `3 국가인권위원회` 연속
내용 대신 rhwp의 `4 국가인권위원회`부터 시작하므로 owner drift가 남아 있다. 또한 physical
p9의 `<국내 유사입법례 분석>` 제목은 rhwp SVG 상단 clip에 걸린다. 두 결함은 글꼴 raster
차이와 무관한 flow/fragment 문제이므로 다음 stage에서 PDF 물리 쪽 기준으로 독립 분석한다.

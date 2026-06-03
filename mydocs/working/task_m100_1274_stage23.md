# task 1274 stage23: 2022-09 17쪽 문29 marker/text overlap 정정

## 배경

- 작업지시자가 `3-09월_교육_통합_2022.hwp` 17쪽 좌측 단 하단에서 `문29)` 빨간 marker와 다음 본문 `네 장의 카드를 ...`가 겹친다고 지적했다.
- stage22 visual sweep은 page 17을 `equation_text_overlap` 후보로는 잡았지만, 실제 시각 문제인 `문항 제목 marker`와 `다음 본문 TextRun`의 overlap은 별도 유형이라 검출하지 못했다.
- 따라서 stage22의 render tree bbox 분석은 수식 겹침만이 아니라 문항 제목/다음 본문 간 line overlap도 함께 잡아야 한다.

## 현재 관찰

- 대상: `samples/3-09월_교육_통합_2022.hwp`
- 페이지: 17쪽 (`0-based page=16`)
- 산출물:
  - `output/task1274/2022-09/compare/compare_017.png`
  - `output/task1274/2022-09/analysis/annotated_017.png`
  - `output/task1274/2022-09/render_tree/render_tree_017.json`
- render tree 기준:
  - `pi=900` 문항 제목 line: `TextLine y=1041.1`, text=`문29）   175`
  - `pi=901` 다음 본문 line: `TextLine y=1038.7`, text=`네 장의 카드를 꺼내는 경우의 수는`
  - 다음 본문이 문항 제목보다 위쪽 y에 놓여 실제 glyph가 겹친다.
- `dump-pages -p 16` 기준:
  - 단 0 끝에 `pi=900`과 `pi=901`이 함께 들어간다.
  - 한컴/PDF 기준은 `pi=900` 제목이 먼저 보이고, `pi=901` 본문은 그 아래 줄로 분리된다.

## 원인 가설

`RHWP_VPOS_DEBUG=1 rhwp export-render-tree ... -p 16` 결과:

```text
VPOS_CORR: path=page pi=900 prev_pi=899 ... y_in=1041.07 end_y=1020.64 result=1041.07 current_title=true title_bottom=false ...
VPOS_CORR: path=page pi=901 prev_pi=900 ... y_in=1059.09 end_y=1038.67 result=1038.67 page_tail=true ...
```

- `pi=900`은 저장 vpos 기준 `end_y=1020.64`가 계산됐지만, `compact_endnote_title_bottom_backtrack` 조건이 `0.95` 하단 임계값에 걸리지 않아 `result=1041.07`로 유지됐다.
- 바로 다음 `pi=901`은 `compact_endnote_page_tail_backtrack`으로 `result=1038.67`이 적용되어, 본문이 제목보다 위에 렌더됐다.
- 즉 현재 문제는 `pi=901`을 무조건 넘기는 문제가 아니라, 하단 근처 새 문항 제목의 page-path backtrack 조건이 너무 좁아 제목 자체가 아래로 눌린 것이다.

## 수정 방향

1. `height_cursor.rs`의 compact endnote question title backtrack 조건을 보정한다.
   - page-path의 새 문항 제목이 저장 vpos 기준으로 분리 가능한 위치를 가리키고, 현재 y가 하단 근처이며, backtrack 폭이 작은 경우 `end_y`를 적용한다.
   - 목적은 `pi=900` 제목을 `1020px`대 위치로 되돌려 `pi=901` 본문과 분리하는 것이다.
2. `scripts/task1274_visual_sweep.py`에 문항 제목/다음 본문 overlap 후보를 추가한다.
   - render tree `TextLine` 텍스트가 `문\d+` 패턴으로 시작하는 line과 다음 본문 line의 bbox overlap을 검사한다.
   - page 17은 수정 전 `question_title_text_overlap`으로 잡히고, 수정 후 사라져야 한다.
3. 필요하면 focused regression assertion을 추가한다.
   - `samples/3-09월_교육_통합_2022.hwp` page 16 render tree에서 `pi=900` line y가 `pi=901` line y보다 충분히 작고, 두 line bbox가 겹치지 않음을 확인한다.

## 검증 계획

- `cargo build --bin rhwp`
- `python3 -m py_compile scripts/task1274_visual_sweep.py`
- `python3 scripts/task1274_visual_sweep.py --target 2022-09`
  - page 17 `문29` marker와 `네 장의 ...` 본문이 compare PNG에서 분리되는지 확인한다.
  - `metrics.json`에서 page 17의 `question_title_text_overlap` 후보가 사라지는지 확인한다.
- `python3 scripts/task1274_visual_sweep.py --target all`
  - 6종 전체 SVG/PDF/render tree page count가 유지되는지 확인한다.
  - 기존 stage22 후보 목록이 새 검출 기준으로 더 정확해졌는지 확인한다.

## 상태

- 작업지시자 승인 후 구현과 검증을 완료했다.

## 구현 결과

- `src/renderer/height_cursor.rs`
  - compact endnote page-path에서 새 문항 제목이 단 하단 근처에 있고 저장 vpos가 더 안전한 위치를 가리킬 때 `end_y`를 적용하도록 `compact_endnote_title_bottom_backtrack` 임계값을 조정했다.
  - 일반 replay-path에는 기존 `0.95` 하단 판정을 유지하고, page-path만 `0.90`으로 완화했다. 이로써 `pi=900` 문항 제목이 저장 vpos 기준으로 먼저 올라가고, 다음 `pi=901` 본문은 그 아래에 남는다.
  - `compact_endnote_page_path_title_bottom_backtrack_allows_safe_title` 단위 테스트를 추가해 page-path 새 문항 제목이 하단에서 backtrack되는 조건을 고정했다.
- `scripts/task1274_visual_sweep.py`
  - render tree JSON에서 `TextLine` 텍스트와 bbox를 읽어 `문\d+` 형태의 문항 제목 line과 바로 다음 본문 line의 bbox overlap을 검사하는 `question_title_text_overlap` 지표를 추가했다.
  - `summary.json`/`metrics.json`에 `question_title_text_overlap_pages`, `question_title_text_overlap_candidates`를 기록한다.

## 검증 결과

- `python3 -m py_compile scripts/task1274_visual_sweep.py` 통과.
- `cargo test --lib compact_endnote_page_path_title_bottom_backtrack_allows_safe_title -- --nocapture` 통과.
- `cargo build --bin rhwp` 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2022-09` 통과.
  - SVG/PDF/render tree 모두 23쪽.
  - page 17 `question_title_text_overlap_candidates`는 빈 배열이다.
  - page 17 compare PNG에서 `문29） 175` 제목과 `네 장의 카드를 꺼내는 경우의 수는` 본문이 분리되어 보인다.
  - 확인 산출물:
    - `output/task1274/2022-09/compare/compare_017.png`
    - `output/task1274/2022-09/analysis/annotated_017.png`
    - `output/task1274/2022-09/render_tree/render_tree_017.json`
- `python3 scripts/task1274_visual_sweep.py --target all` 통과.
  - `2022-09`: SVG/PDF/render tree 23/23/23쪽, `title=[]`.
  - `2023-09`: SVG/PDF/render tree 20/20/20쪽, `title=[]`.
  - `2024-09-below20`: SVG/PDF/render tree 23/23/23쪽, `title=[]`.
  - `2024-09-between20`: SVG/PDF/render tree 24/24/24쪽, `title=[]`.
  - `2022-10`: SVG/PDF/render tree 18/18/18쪽, `title=[]`.
  - `2022-11-practice`: SVG/PDF/render tree 21/21/21쪽, `title=[]`.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
  - 51 passed.

## 남은 판단

- stage23은 `문항 제목 marker`와 다음 본문이 같은 높이에서 겹치는 유형을 보정했다.
- page 17에는 별도의 render tree 수식/text overlap 후보가 남아 있다. 이는 `문29` 제목 overlap과는 다른 후보이며, stage22 수동 검증 표에서 PDF 기준으로 계속 확인한다.

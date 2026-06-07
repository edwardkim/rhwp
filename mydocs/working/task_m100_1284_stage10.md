# task 1284 stage10: 잔여 frame overflow 후보 분석

## 배경

- stage9 커밋: `bc2d6f37 task 1284: 기본 미주 제목 tail 분기 보정`
- stage9 이후 전체 sweep에서 question drift 계열은 모두 정리됐다.
- 남은 자동 후보는 frame overflow 계열 3건이다.

## 대상

### 2023-09 page19

- `frame_overflow_pixels`
  - `rhwp_outside_frame_pixels=276`
  - `pdf_outside_frame_pixels=0`
  - `rhwp_outside_frame_max_y=1102`
  - `content_bottom_delta_px=8.0`
- `question_marker_drift_candidates=[]`
- 1차 판단:
  - page/column drift는 없고, 하단 8px 전후의 frame 검출/하단 bleed 후보로 보인다.
  - compare/annotated PNG로 실제 하단 내용이 frame 밖으로 보이는지 확인한다.

### 2022-11-practice page12

- `frame_overflow_pixels`
  - `rhwp_outside_frame_pixels=9269`
  - `pdf_outside_frame_pixels=0`
  - `rhwp_outside_frame_max_y=1096`
  - `content_bottom_delta_px=-185.0`
- `equation_text_overlap_candidates`
  - 대표 후보: `text_pi=565`, text=`다음과 같고`, overlap ratio `0.239`
- 1차 판단:
  - 단순 하단 bleed가 아니라 PDF보다 rhwp 내용 하단이 크게 위쪽에서 끝나는 흐름 차이도 같이 있다.
  - compare PNG와 render-tree/dump-pages로 partial tail/수식 흐름을 먼저 확인한다.

### 2022-11-practice page19

- `frame_overflow_pixels`
  - `rhwp_outside_frame_pixels=60`
  - `pdf_outside_frame_pixels=0`
  - `rhwp_outside_frame_max_y=1102`
  - `content_bottom_delta_px=9.0`
- `question_marker_drift_candidates=[]`
- 1차 판단:
  - page/column drift는 없고, 하단 9px 전후의 frame 검출/하단 bleed 후보로 보인다.
  - compare/annotated PNG로 실제 하단 내용이 frame 밖으로 보이는지 확인한다.

## 진행 순서

1. 각 후보의 `compare_*.png`, `annotated_*.png`, `render_tree_*.json`, `dump-pages`를 비교한다.
2. 실제 렌더 문제와 sweep/frame 검출 오탐을 분리한다.
3. 실제 문제인 경우 공통 pagination/render 조건으로 수정한다.
4. 수정 후 `issue_1139_inline_picture_duplicate`와 focused sweep을 확인한다.
5. stage10 결과를 문서화하고 승인 후 커밋한다.

## 분석 결과

### 2023-09 page19

- `compare_019.png`와 `annotated_019.png`에서 페이지 하단의 작은 tail만 붉게 잡혔다.
- render tree 기준으로 실제 본문 요소가 크게 page frame 밖으로 흐르지 않았다.
- 재실행 후 값:
  - `rhwp_outside_frame_pixels=276`
  - `rhwp_outside_frame_extent_px=5`
  - `content_bottom_delta_px=8.0`
  - `frame_overflow_tolerated_bleed=true`
- 판단:
  - 5px 하단 tail bleed이며 실제 overflow 문제로 보지 않는다.

### 2022-11-practice page12

- 기존 sweep은 페이지 중간의 긴 가로선을 page frame bottom으로 잡아 `rhwp_outside_frame_pixels=9269`를 만들었다.
- `compare_012.png`와 render tree를 보면 실제 본문이 페이지 frame 아래로 빠지는 형태가 아니다.
- 보정 후 값:
  - `rhwp_frame=[26,75,768,1097]`
  - `rhwp_outside_frame_pixels=0`
  - `content_bottom_delta_px=4.0`
- 판단:
  - 렌더 문제라기보다 frame bottom 검출 오탐이다.

### 2022-11-practice page19

- `compare_019.png`와 `annotated_019.png`에서 페이지 하단의 작은 tail만 후보로 잡혔다.
- 재실행 후 값:
  - `rhwp_outside_frame_pixels=60`
  - `rhwp_outside_frame_extent_px=5`
  - `content_bottom_delta_px=9.0`
  - `frame_overflow_tolerated_bleed=true`
- 판단:
  - 5px 하단 tail bleed이며 실제 overflow 문제로 보지 않는다.

## 수정 내용

- `scripts/task1274_visual_sweep.py`
  - frame bottom 후보 선택을 `count` 우선이 아니라 가장 아래쪽 y 좌표 우선으로 바꿨다.
  - 검출된 bottom이 페이지 높이의 90%보다 위이면 page frame bottom으로 보기 어렵기 때문에 기본 하단값으로 fallback한다.
  - rhwp만 12px 이하로 frame 아래에 걸리고 PDF와 본문 하단 차이가 크지 않은 경우는 `frame_overflow_tolerated_bleed`로 기록하고 `frame_overflow_pixels` flag에서는 제외한다.
  - metrics에 `rhwp_outside_frame_extent_px`, `pdf_outside_frame_extent_px`, `frame_overflow_tolerated_bleed`를 남겨 수동 판정 근거를 보강했다.

## 검증

```bash
python3 -m py_compile scripts/task1274_visual_sweep.py
python3 scripts/task1274_visual_sweep.py --target 2022-11-practice
python3 scripts/task1274_visual_sweep.py --target all
```

### focused sweep

- `2022-11-practice`
  - `SVG pages: 21`
  - `PDF pages: 21`
  - `frame=[]`
  - `question=[]`

### 전체 sweep

| 대상 | SVG/PDF/render-tree | frame | question |
|---|---:|---|---|
| `2022-09` | 23/23/23 | `[]` | `[]` |
| `2023-09` | 20/20/20 | `[]` | `[]` |
| `2024-09-below20` | 23/23/23 | `[]` | `[]` |
| `2024-09-between20` | 24/24/24 | `[]` | `[]` |
| `2022-10` | 18/18/18 | `[]` | `[]` |
| `2022-11-practice` | 21/21/21 | `[]` | `[]` |

## 남은 판단

- Stage10은 잔여 frame overflow 후보가 실제 레이아웃 문제가 아니라 sweep 오탐/허용 tail bleed임을 분리하는 작업이었다.
- Rust 렌더러 코드는 수정하지 않았으므로 `issue_1139_inline_picture_duplicate`는 이번 stage에서 재실행하지 않았다.
- red/line/equation 후보는 기존 정밀 sweep의 후보군으로 남아 있으나, Stage10의 잔여 frame overflow 대상은 아니다.

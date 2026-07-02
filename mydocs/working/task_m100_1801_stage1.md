# Task #1801 Stage 1 완료 기록 — visual sweep 일반 파일 입력 및 overlay 검토 개선

## 완료 내용

- preset target 외 일반 파일 입력을 추가했다.
  - `--hwp`
  - `--pdf`
  - `--key`
  - `--file-target KEY DOC PDF`
- 특정 페이지 선택을 추가했다.
  - `--page 22`
  - `--pages 43-46`
  - `--pages 1,3,5-7`
- 선택 페이지의 compare/overlay/review 산출물이 실제 1-based 페이지 번호를 유지하도록 했다.
  - `compare_002.png`
  - `overlay_002.png`
  - `review_002.png`
- PNG overlay 기반 metric을 추가했다.
  - `pixel_match_percent`
  - `ink_match_percent`
  - `visual_accuracy_proxy_percent`
- compare와 overlay를 나란히 붙인 `review/review_###.png`를 추가했다.
- review PNG의 overlay 바로 아래에 다음 한 줄을 표시하도록 했다.

```text
코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 N%.
```

## 검증

| 항목 | 결과 |
|---|---|
| `python3 -m py_compile scripts/task1274_visual_sweep.py` | 통과 |
| `python3 scripts/task1274_visual_sweep.py --help` | 신규 옵션 노출 확인 |
| `--page 2` smoke | 통과 |
| `compare_002.png` | 생성 확인 |
| `overlay_002.png` | 생성 확인 |
| `review_002.png` | compare/overlay 나란히 표시 및 overlay 하단 코멘트 확인 |
| `overlay_metrics.json` | `visual_accuracy_proxy_percent=13.8381` 확인 |
| `git diff --check` | 통과 |

## smoke 명령

```bash
python3 scripts/task1274_visual_sweep.py \
  --key smoke-eng-p2-overlay-comment \
  --hwp /Users/tsjang/rhwp/samples/exam_eng.hwp \
  --pdf /Users/tsjang/rhwp/pdf/exam_eng-2022.pdf \
  --page 2 \
  --out /private/tmp/rhwp-visual-sweep-overlay-comment-smoke \
  --pixel-diff-threshold 32
```

## 대표 산출물

- compare: `/private/tmp/rhwp-visual-sweep-overlay-comment-smoke/smoke-eng-p2-overlay-comment/compare/compare_002.png`
- overlay: `/private/tmp/rhwp-visual-sweep-overlay-comment-smoke/smoke-eng-p2-overlay-comment/overlay/overlay_002.png`
- review: `/private/tmp/rhwp-visual-sweep-overlay-comment-smoke/smoke-eng-p2-overlay-comment/review/review_002.png`


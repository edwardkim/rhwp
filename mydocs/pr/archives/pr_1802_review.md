# PR #1802 Self Review — visual sweep overlay 검토 산출물 개선

## 메타

| 항목 | 내용 |
|---|---|
| PR | #1802 |
| 제목 | `task 1801: visual sweep overlay 검토 산출물 개선` |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `task_m100_1801` |
| 관련 이슈 | #1801 |
| 문서 작성 시점 head | `1258e3d09fa86488a850c97eee8742970ae095c9` |

## 변경 범위

- `scripts/task1274_visual_sweep.py`
  - preset target 외 일반 파일 입력 지원
  - 특정 페이지/범위 선택 지원
  - overlay diff PNG와 metric 생성
  - compare/overlay 나란히 보기 review PNG 생성
  - overlay 바로 아래 내용 픽셀 중심 자동 일치율 보조값 표시
- `mydocs/manual/visual_sweep_guide.md`
  - 일반 파일과 특정 페이지 사용법 문서화
  - Codex 보고 규칙 문서화
  - `visual_accuracy_proxy_percent` 해석 문서화
- `mydocs/pr/assets/pr_1802_visual_sweep_review_002.png`
  - PR 코멘트에서 바로 확인하기 위한 smoke 검증 이미지

## 로컬 검증

| 항목 | 결과 |
|---|---|
| `python3 -m py_compile scripts/task1274_visual_sweep.py` | 통과 |
| `python3 scripts/task1274_visual_sweep.py --help` | 통과, 신규 옵션 노출 확인 |
| `--page 2` smoke | 통과 |
| `git diff --check` | 통과 |

smoke 명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key smoke-eng-p2-overlay-comment \
  --hwp /Users/tsjang/rhwp/samples/exam_eng.hwp \
  --pdf /Users/tsjang/rhwp/pdf/exam_eng-2022.pdf \
  --page 2 \
  --out /private/tmp/rhwp-visual-sweep-overlay-comment-smoke \
  --pixel-diff-threshold 32
```

대표 산출물:

- compare: `/private/tmp/rhwp-visual-sweep-overlay-comment-smoke/smoke-eng-p2-overlay-comment/compare/compare_002.png`
- overlay: `/private/tmp/rhwp-visual-sweep-overlay-comment-smoke/smoke-eng-p2-overlay-comment/overlay/overlay_002.png`
- review: `/private/tmp/rhwp-visual-sweep-overlay-comment-smoke/smoke-eng-p2-overlay-comment/review/review_002.png`
- `visual_accuracy_proxy_percent`: `13.8381`

## 판단

이 PR은 렌더러 본체가 아니라 visual sweep 보조 도구와 문서 개선이다. 로컬 smoke에서 일반 파일 입력,
특정 페이지 선택, overlay metric, review PNG 산출이 모두 확인됐다. CI 최신 head 통과 후 merge 가능하다.


# PR #1802 Self Review Impl — visual sweep overlay 검토 산출물 개선

## Stage 1. 구현 확인

완료.

- `--hwp`, `--pdf`, `--key`, `--file-target` 옵션으로 일반 파일 입력을 받는다.
- `--page`, `--pages` 옵션으로 비교·overlay·analysis 대상을 선택 페이지로 좁힌다.
- 선택 페이지 산출물은 실제 1-based 페이지 번호를 파일명에 유지한다.
- `overlay_metrics.json`에 `visual_accuracy_proxy_percent`를 포함한다.
- `review/review_###.png`는 compare와 overlay를 나란히 보여준다.
- review PNG의 overlay 바로 아래에 자동 일치율 보조값 한 줄을 표시한다.

## Stage 2. 문서 확인

완료.

- `visual_sweep_guide.md`에 일반 파일 입력 예시를 추가했다.
- `visual_sweep_guide.md`에 특정 페이지 비교 예시를 추가했다.
- `visual_sweep_guide.md`에 Codex 보고 규칙을 추가했다.
- `visual_accuracy_proxy_percent`가 사람 판정 정확도가 아니라 내용 픽셀 중심 raster 일치율 보조값임을 명시했다.

## Stage 3. 검증 확인

완료.

- Python 문법 검사 통과
- CLI help 확인
- `exam_eng.hwp` / `exam_eng-2022.pdf` page 2 smoke 통과
- `review_002.png`에서 overlay 바로 아래 코멘트 표시 확인
- `git diff --check` 통과

## Merge 전 조건

- PR #1802 최신 head 기준 GitHub Actions 통과
- PR 코멘트에 smoke 검증 이미지 링크 게시


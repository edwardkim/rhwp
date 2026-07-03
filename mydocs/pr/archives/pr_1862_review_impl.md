# PR #1862 리뷰 구현 기록

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1862
- base: `devel`
- 작성자: `planet6897`
- 상태: open, non-draft, mergeable
- 관련 이슈: #1858
- 범위: #1858 발현 1만 처리. 발현 2는 open 유지.

## Stage 2. 변경 내용 검토

완료.

- `typeset.rs` 의 Paper top-and-bottom 절대배치 가드가 co-anchored 후속 상자에도 적용되도록 확장됐다.
- 조건은 같은 host 문단 앞쪽에 `treat_as_char=false`, `TextWrap::TopAndBottom`, `VertRelTo::Paper` 표가
  있는 경우로 제한되어 있다.
- 단독 Paper 상자나 Page+Bottom 성숙 경로를 직접 넓히는 변경은 아니다.
- 신규 테스트는 재현 샘플이 1쪽이어야 한다는 page count 게이트다.

## Stage 3. 로컬 검증

완료.

- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1858`
  - 통과
- PR head `dump-pages`
  - `samples/issue1858_paper_anchor_float_stack.hwpx`: 1쪽
- `upstream/devel` 기준 동일 샘플 비교
  - 3쪽
- 영향권 회귀 테스트 묶음
  - `issue_1418`, `1510`, `1611`, `1624`, `1658`, `1663`, `1853`, `1858`
  - 총 15개 테스트 통과
- `git diff --check upstream/devel...HEAD`
  - 통과

## Stage 4. 시각 검증

완료.

- 기준 PDF: `pdf/issue1858_paper_anchor_float_stack-2024.pdf`
- visual sweep: `flagged=0/1`
- 대표 review PNG:
  - 임시 경로: `output/pr1862_visual/pr1862-issue1858/review/review_1858.png`
  - 보존 asset: `mydocs/pr/assets/pr_1862_issue1858_review_p001.png`
- 기준 PDF 보존 asset:
  - `mydocs/pr/assets/pr_1862_issue1858_baseline_2024.pdf`

판정:

- PR 핵심인 페이지 폭발 해소는 기준 PDF 1쪽과 맞는다.
- 자동 후보는 없지만 pixel/ink match 는 낮아 정밀 시각 일치로 보지는 않는다.
- 남은 세부 시각 차이는 PR 범위 밖 후속 후보로 기록한다.

## Stage 5. merge 및 후속

완료.

- PR #1862 squash merge 완료.
- merge commit: `c8c13b173f4f45c80806af2fb0475fe5b159d030`
- #1858 은 발현 2 추적 때문에 close 하지 않는다.
- 페이지 수나 렌더링 위치 변화가 핵심인 PR 에서는 한컴 2020/2024 기준 PDF를 함께 첨부해 달라는 코멘트를 남겼다.
- 옵션 2 방식으로 review 문서와 asset 은 별도 docs-only 후속 PR 로 처리한다.

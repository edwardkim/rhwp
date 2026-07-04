# PR #1855 리뷰 구현 메모

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1855
- 작성자: @planet6897
- 제목: `Task #1853: 같은 문단 float 스택 이월 규칙 회귀 수정 — tac 캡션·페이지-절대 앵커 과잉 포착 해소`
- base / head: `devel` / `task1853-float-stack-overcapture-fix`
- 검토 기준 head: `ce7b11a3da43c1e172e83ef104e907e1991faf78`

## Stage 1. 메타 확인

완료.

- reviewer assign 완료: @jangster77
- PR은 Draft가 아니다.
- base는 `devel`.
- `maintainerCanModify=true`.
- 문서 작성 시점 참고 mergeable 상태는 `MERGEABLE`.
- update branch 후 검토 기준 head는 `ce7b11a3da43c1e172e83ef104e907e1991faf78`.
- GitHub CI는 preflight/일부 CodeQL jobs 통과, Canvas visual diff/Analyze rust/Build & Test 진행 중이다.

## Stage 2. 변경 내용 검토

완료.

- 기존 `preceded_by_same_para_float`는 `para_index`만 같으면 선행 float 로 보아 tac 캡션과
  페이지-절대 앵커까지 오검출할 수 있었다.
- PR 변경은 선행 `PageItem`의 `control_index`로 원본 `Control::Table`을 조회하고,
  `is_para_topbottom_float`인 경우만 선행 flow stack float 로 인정한다.
- tac 캡션(`tac=true`)과 `vert=용지` 계열 페이지-절대 앵커는 제외된다.
- PR이 보존해야 하는 별표4 계열 `tac=false && TopAndBottom && vert=Para` flow stack 은 계속 인정된다.

## Stage 3. 로컬 검증

완료.

- `git diff --check upstream/devel...HEAD` 통과.
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1853` 통과.
- `env CARGO_INCREMENTAL=0 cargo build --bin rhwp` 통과.
- 기준 PDF `pdf/issue1853_caption_precedes_body_split-2024.pdf`: 52쪽.
- rhwp `dump-pages samples/issue1853_caption_precedes_body_split.hwpx`: 52쪽.
- p44 dump에서 `pi=371 ci=0` tac 캡션과 `pi=371 ci=1` 본체 `PartialTable` 시작이 같은 쪽에 존재함을 확인했다.
- p45 dump에서 본체 `PartialTable` continuation을 확인했다.

## Stage 4. 시각 검증

완료.

페이지 수 또는 시각 검증이 필요한 PR review 에서는 기준 PDF 첨부 요청 여부를 반드시 review 문서에 기록한다.
이번에는 사용자가 한컴 기준 PDF를 제공했다.

- 기준 PDF(repo-relative 기대 경로): `pdf/issue1853_caption_precedes_body_split-2024.pdf`
- visual sweep 명령:

기준 PDF는 review worktree 의 `pdf/issue1853_caption_precedes_body_split-2024.pdf` 로 포함했다. 후속
기록에는 환경 의존 절대 경로가 아니라 repo-relative 경로를 사용한다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1855-issue1853 \
  --hwp samples/issue1853_caption_precedes_body_split.hwpx \
  --pdf pdf/issue1853_caption_precedes_body_split-2024.pdf \
  --pages 44-45 \
  --out output/pr1855_visual \
  --rhwp-bin target/debug/rhwp
```

- 결과: `flagged=0/2`.
- p44 review: `output/pr1855_visual/pr1855-issue1853/review/review_044.png`
- p45 review: `output/pr1855_visual/pr1855-issue1853/review/review_045.png`
- p44 asset: `mydocs/pr/assets/pr_1855_issue1853_review_p044.png`
- p45 asset: `mydocs/pr/assets/pr_1855_issue1853_review_p045.png`
- p44 자동 일치율 보조값: 약 24.27%.
- p45 자동 일치율 보조값: 약 11.79%.

판정:

- p44에서 캡션과 본체 표 시작이 같은 쪽에 있으므로, PR 핵심 회귀는 해소됐다.
- p45 continuation도 존재한다.
- p44~45의 내부 행 경계 차이는 남아 있으나, 본 PR의 +1쪽 회귀 해소 판단에는 blocker 로 보지 않는다.

## Stage 5. 남은 작업

대기.

- 사용자가 요청하면 review 문서와 visual asset 처리 방식을 선택한다.
- 원 PR comment 에는 기준 PDF 제공 감사, 핵심 회귀 해소 확인, 잔여 p44~45 행 분배 차이 기록, 향후
  페이지 수/시각 검증 PR 에 기준 PDF 첨부 요청을 정중하게 담는다.
- review 문서/PDF/오늘할일 push 후 PR head 가 다시 바뀌므로 merge 전에는 최신 head 기준 CI 상태를 다시 확인한다.

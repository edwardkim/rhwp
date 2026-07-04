# PR #1875 리뷰 구현 메모

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1875
- author: `humdrum00001010`
- base: `devel`
- head: `endnote-eod-placement`
- 문서 작성 시점 head: `a85f17ee82334192092f4c92afea43aa44e01f80`
- reviewer `jangster77` assign 완료.

## Stage 2. 변경 내용 검토

완료.

- HWPX `noteLine.length=14692344`를 `i32`로 보존하도록 모델/파서/PageItem/serializer 경로가 확장됐다.
- `FootnotePlacement::EachColumn`을 미주 `END_OF_DOCUMENT`로 해석하고, 마지막 구역 렌더링 시 앞 구역 미주 본문을 모아 렌더한다.
- 비마지막 구역의 `END_OF_DOCUMENT` 미주는 참조 위첨자만 남기고 본문 렌더를 억제한다.
- 구분선 위치는 직전 본문 trailing line spacing을 제외하고, 첫 미주 본문은 구분선 아래 여백 바닥으로 floor 한다.
- 특정 샘플명, 페이지 번호, PR/issue 번호, 임의 계수 기반 하드코딩은 확인되지 않았다.

## Stage 3. 로컬 검증

완료.

- `git diff --check`
- `cargo fmt --check`
- `env CARGO_INCREMENTAL=0 cargo test test_parse_endnote_long_note_line_keeps_hwp5_low_word --lib`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1139_inline_picture_duplicate`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1082_endnote_multicolumn_drift`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`

외부 gist fixture 확인도 수행했다.

- `endnote_end_of_document_fixture.hwpx`: 2페이지, p2 마지막 구역에 deferred 미주 본문 배치 확인.
- `endnote_multi_note_fixture.hwpx`: 3페이지, p3 마지막 구역에 A/B 미주 순서 배치 확인.
- 두 fixture 모두 `export-svg` 성공.

## Stage 4. 시각 검증 판정

완료.

- PR 작성자 제공 비교 PNG는 확인했다.
- 로컬에 추가된 한컴 2024 기준 PDF 2건을 확인했다.
  - `pdf/endnote_end_of_document_fixture-2024.pdf`: 2 pages.
  - `pdf/endnote_multi_note_fixture-2024.pdf`: 3 pages.
- `visual_sweep_guide.md` 기준으로 대표 페이지를 비교했다.
  - `pr1875-eod` p1-p2: flagged 0/2.
  - `pr1875-multi` p3: flagged 0/1.
- 대표 review PNG를 `mydocs/pr/assets/`에 복사했다.
  - `mydocs/pr/assets/pr_1875_endnote_eod_review_p001.png`
  - `mydocs/pr/assets/pr_1875_endnote_eod_review_p002.png`
  - `mydocs/pr/assets/pr_1875_endnote_multi_review_p003.png`
- 사람 판정 기준으로 PR 핵심 주장인 문서 끝 미주 배치, 전폭 구분선, 다중 미주 순서는 기준 PDF와 맞다.

## Stage 5. 결론

merge 완료.

코드 방향, 로컬 검증, GitHub CI, 기준 PDF 기반 visual sweep 모두 통과했다. 외부 gist fixture를 in-repo 회귀 테스트로 편입하는 것은 권장 후속으로 남기되, 현재 PR의 merge blocker로 보지 않는다.

- merge commit: `2b21caef86ac1d3aa8918a30e937bcb7be5f6c18`
- #1875 PR description 의 GitHub auto-close 대상 이슈는 없었다.

## 후속 작업

- review 문서, 기준 PDF, visual asset은 옵션 2 후속 문서/asset PR 로 분리해 반영한다.
- 후속 문서/asset PR 이 `devel` 에 merge 된 뒤 raw asset URL 을 포함한 PR 코멘트를 작업지시자 승인 후 게시한다.
- 후속 문서/asset PR merge 후 `mydocs/manual/pr_review_workflow.md`의 브랜치/worktree 정리 절차를 수행한다.

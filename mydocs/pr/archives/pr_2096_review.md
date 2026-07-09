# PR #2096 리뷰 — #2093 단일 줄 sa 게이트 제거

- 작성 시각: 2026-07-09 16:30 KST
- PR: https://github.com/edwardkim/rhwp/pull/2096
- 작성자: `planet6897`
- base / head: `devel` / `fix/2093-saved-single-line-sa`
- 문서 작성 시점 참고 head: `00cc454c82cc66b713e3e90e569b79a1aa6d9a66`
- 문서 작성 시점 참고 merge state: `BEHIND`
- reviewer assign: `jangster77` 요청 완료
- 처리 경로: `codex/planet6897-prs-review-20260709` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `samples/task2093/saved_single_line_spacing_after.hwpx` 및 README 추가.
- `src/renderer/typeset.rs`: `saved_single_line_bottom_fits`의 `spacing_after` 게이트 제거.
- `tests/issue_2093_saved_single_line_spacing_after.rs`: pi=1이 1쪽 하단, pi=2가 2쪽으로 가는지 검증.

## 체리픽 검토

- 누적 체리픽 순서: 9/11.
- 적용 커밋: `861bb2a1c` (`Issue #2093: 단일 줄 sa 게이트 제거 ...`).
- 충돌: 없음.
- 선행 PR 의존: #2082 이후 같은 `typeset.rs`에 누적 적용했으나 충돌 없음.

## MCP 기준 PDF

PR에 기준 PDF가 없어 HWP 2020 MCP로 생성했다. 서버 URL/IP와 토큰은 공개 문서에 기록하지 않는다. MCP 접근 정보가 필요한 collaborator는 `@jangster77`에게 요청한다.

| 항목 | 내용 |
|---|---|
| 입력 HWPX | `samples/task2093/saved_single_line_spacing_after.hwpx` |
| 입력 SHA-256 | `8cd90d61af42e083907ab5db15c4cae5b830f07a7d34a4a27100563e15c6fd64` |
| 출력 PDF | `pdf/task2093/saved_single_line_spacing_after-2020.pdf` |
| 출력 SHA-256 | `b75b5d0151e4fe22706b893ac32af5f745e5f0ab037cca531cc458f232e5fd57` |
| MCP job id | `d323c588-e221-48dc-bd4b-c49377b6a2c3` |
| run_status / validation | `0` / `ok` |
| pdfinfo | 1 page, PDF 1.7, Producer `cairo 1.18.0` |

## 로컬 검증

- GitHub Actions: 원 PR head 기준 `Build & Test`, `CodeQL`, `Canvas visual diff` 등 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통합 브랜치 fixup 이후 통과.
- `cargo fmt --check`: 통과.
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 산출물 삭제 후 진행.
- `CARGO_INCREMENTAL=0 cargo test --features native-skia --test issue_1842 --test issue_2004_cell_image_stack_pagination --test issue_2083_hide_fill_page_background --test issue_2093_saved_single_line_spacing_after --test issue_2097_none_table_declared_fits`: 통과.
  - #2096 관련 `tests/issue_2093_saved_single_line_spacing_after.rs`: 1 passed.
- `target/debug/rhwp dump-pages samples/task2093/saved_single_line_spacing_after.hwpx`: rhwp 2 pages.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과(exit 0).

## 시각 검증

요약:

- rhwp SVG pages: 2
- MCP PDF pages: 1
- compared pages: 1
- overlay pixel match: 99.72389%
- overlay visual accuracy proxy: 6.10221%
- 대표 asset: `mydocs/pr/assets/planet6897_prs_20260709/pr2096_task2093_p001_review.png`
- metrics: `mydocs/pr/assets/planet6897_prs_20260709/pr2096_task2093_overlay_metrics.json`

사람 판정: MCP/HWP2020 PDF는 `FILL`, `TAIL LINE WITH SPACING AFTER`, `PAGE2 HEAD`가 모두 1쪽 상단에 보인다. 반면 rhwp dump와 PR test는 전체 2쪽을 기대한다. 즉 PR의 테스트 기대값과 HWP 2020 MCP 기준 PDF가 서로 맞지 않는다.

## 메인터너 수정 가능성 검증

- `pdftotext -layout pdf/task2093/saved_single_line_spacing_after-2020.pdf -`: `FILL`, `TAIL LINE WITH SPACING AFTER`, `PAGE2 HEAD`가 모두 1쪽에 추출된다.
- `target/debug/rhwp dump-pages samples/task2093/saved_single_line_spacing_after.hwpx`: p1 `FILL` + `TAIL LINE WITH SPACING AFTER`, p2 `PAGE2 HEAD`로 2쪽이다.
- HWPX 구조상 `PAGE2 HEAD` 문단은 `pageBreak="0"`이지만 `lineseg vertpos="1000"`으로 리셋되어 있다. PR 테스트는 이 vpos 리셋을 새 쪽 증거로 사용하고, MCP/HWP2020 PDF는 이 fixture에서 해당 리셋을 새 쪽으로 보존하지 않는다.
- 메인터너 권한으로 PDF 1쪽에 맞추려면 `saved_flow_marks_page_last`/vpos-reset 신뢰 경로를 이 synthetic fixture에서 무시하거나 테스트 기대값을 1쪽으로 뒤집어야 한다. 이는 PR의 실문서 주장(`1192000 해양수산 17→16쪽`)과 테스트 의도를 깨는 수정이므로, 기준 PDF 또는 한컴 생성 원본 fixture 없이 좁고 안전한 maintainer patch로 처리하기 어렵다.

## 판단

- 체리픽 가능 여부: 기계적 체리픽과 focused test는 가능.
- 통합 PR 처리: 본 항목은 통합 PR의 머지 차단 사유로 제출하지 않고, merge 후 원 PR에 후속 검토 의견으로 남긴다.
- 후속 검토 의견: MCP 기준 PDF가 PR의 "전체 2쪽" 기대와 불일치한다. 현 브랜치에서 임의로 보정하면 `saved_flow_marks_page_last`/vpos-reset 신뢰 경로와 PR 테스트 목적이 흔들리므로, 작성자에게 한컴 버전/fixture 의도 차이 또는 실제 기준 PDF 보강을 요청하는 것이 맞다. HWP 2020 MCP 기준을 따른다면 현재 테스트 기대값은 재검토 대상이다.

## 후속 검토 코멘트 초안

@planet6897 작업 감사합니다. 체리픽 통합과 로컬 검증은 통과했습니다.

사후 확인 의견으로 하나 남깁니다. PR에 포함된 `samples/task2093/saved_single_line_spacing_after.hwpx`를 HWP 2020 MCP로 PDF 변환하면 `FILL`, `TAIL LINE WITH SPACING AFTER`, `PAGE2 HEAD`가 모두 1쪽에 들어갑니다. 반면 PR 테스트와 rhwp dump는 `PAGE2 HEAD`를 2쪽으로 기대합니다.

메인터너 쪽에서 이 fixture에 맞춰 임의 보정하려면 vpos-reset 신뢰 경로 또는 테스트 기대값을 뒤집어야 해서 PR의 실문서 주장과 테스트 목적을 깨기 쉽습니다. 가능하면 이 synthetic fixture의 의도, 사용한 한컴 버전, 또는 실제 기준 PDF를 후속으로 보강해 주세요.

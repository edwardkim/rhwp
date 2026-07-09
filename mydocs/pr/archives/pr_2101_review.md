# PR #2101 리뷰 — #2097 None 표 선언높이 신뢰

- 작성 시각: 2026-07-09 16:30 KST
- PR: https://github.com/edwardkim/rhwp/pull/2101
- 작성자: `planet6897`
- base / head: `devel` / `fix/2097-none-table-declared-fit`
- 문서 작성 시점 참고 head: `c39f1460087a4658fbfd7961ea7ac2980237963b`
- 문서 작성 시점 참고 merge state: `BEHIND`
- reviewer assign: `jangster77` 요청 완료
- 처리 경로: `codex/planet6897-prs-review-20260709` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `samples/task2097/none_table_declared_fits.hwpx` 및 README 추가.
- `src/renderer/typeset.rs`: pageBreak=None 표의 선언 높이가 현재 쪽에 들어가면 측정 높이 초과로 행 분할하지 않도록 보정.
- `tests/issue_2097_none_table_declared_fits.rs`: `PartialTable` 미발생과 전체 2쪽 기대 검증.

## 체리픽 검토

- 누적 체리픽 순서: 11/11.
- 적용 커밋: `afbf26f83` (`Issue #2097: None 표 선언높이 신뢰 ...`).
- 충돌: 없음.
- 선행 PR 의존: #2096 이후 같은 `typeset.rs`에 누적 적용했으나 충돌 없음.

## MCP 기준 PDF

PR에 기준 PDF가 없어 HWP 2020 MCP로 생성했다. 서버 URL/IP와 토큰은 공개 문서에 기록하지 않는다. MCP 접근 정보가 필요한 collaborator는 `@jangster77`에게 요청한다.

| 항목 | 내용 |
|---|---|
| 입력 HWPX | `samples/task2097/none_table_declared_fits.hwpx` |
| 입력 SHA-256 | `73c7f761c7d16a03f5c5bd3a6961a6f2f83c0e93ff4d87c136b7804391912819` |
| 출력 PDF | `pdf/task2097/none_table_declared_fits-2020.pdf` |
| 출력 SHA-256 | `83e4695b96c5de383c070b139184bfcd4b2e48426e88a4387db4ef0e59fee637` |
| MCP job id | `28b46f7e-7336-4bdb-b5fa-b215e8167776` |
| run_status / validation | `0` / `ok` |
| pdfinfo | 2 pages, PDF 1.7, Producer `cairo 1.18.0` |

## 로컬 검증

- GitHub Actions: 원 PR head 기준 `Build & Test`, `CodeQL`, `Canvas visual diff` 등 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통합 브랜치 fixup 이후 통과.
- `cargo fmt --check`: 통과.
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 산출물 삭제 후 진행.
- `CARGO_INCREMENTAL=0 cargo test --features native-skia --test issue_1842 --test issue_2004_cell_image_stack_pagination --test issue_2083_hide_fill_page_background --test issue_2093_saved_single_line_spacing_after --test issue_2097_none_table_declared_fits`: 통과.
  - #2101 관련 `tests/issue_2097_none_table_declared_fits.rs`: 1 passed.
- `target/debug/rhwp dump-pages samples/task2097/none_table_declared_fits.hwpx`: rhwp 2 pages, p1 Table, p2 `AFTER TABLE`.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과(exit 0).

## 시각 검증

요약:

- rhwp SVG pages: 2
- MCP PDF pages: 2
- flagged pages: 1/2
- flag: p1 `frame_overflow_pixels`, `render_tree_frame_tail_overflow`, `content_bottom_drift`
- overlay average pixel match: 99.22078%
- overlay average visual accuracy proxy: 1.17738%
- 대표 asset: `mydocs/pr/assets/planet6897_prs_20260709/pr2101_task2097_p001_review.png`
- metrics: `mydocs/pr/assets/planet6897_prs_20260709/pr2101_task2097_overlay_metrics.json`

사람 판정: rhwp p1은 표가 통째 배치되지만, MCP/HWP2020 PDF p1에는 `AFTER TABLE`만 보이는 형태라 PR test의 기대와 기준 PDF가 시각적으로 맞지 않는다. 페이지 수 2쪽은 같지만 내용 배치가 다르다.

## 메인터너 수정 가능성 검증

- `pdftotext -layout pdf/task2097/none_table_declared_fits-2020.pdf -`: 1쪽에는 `AFTER TABLE`, 2쪽에는 `BIG ROW`, `MID ROW`, `TAIL ROW EXPANDING`이 추출된다.
- `target/debug/rhwp dump-pages samples/task2097/none_table_declared_fits.hwpx`: 1쪽은 `Table`, 2쪽은 `AFTER TABLE`이다.
- HWPX 구조상 표는 `pageBreak="NONE"`, 선언 높이 `69700HU`, 3행/1열이며 뒤 문단 `AFTER TABLE`은 `lineseg vertpos="1000"`이다. PR 코드는 선언 높이가 현재 쪽에 들어가면 `None` 표를 통째 1쪽에 배치하도록 강제한다.
- MCP/HWP2020 PDF에 맞추려면 이 PR의 핵심 조건(`declared_none_table_whole_fits`)을 제거하거나 반대로 동작시켜야 한다. 그러면 `tests/issue_2097_none_table_declared_fits.rs`의 목적 자체가 깨지고, PR 본문이 주장하는 실문서 계열 수정도 검증 불가 상태가 된다.

## 판단

- 체리픽 가능 여부: 기계적 체리픽과 focused test는 가능.
- 통합 PR 처리: 본 항목은 통합 PR의 머지 차단 사유로 제출하지 않고, merge 후 원 PR에 후속 검토 의견으로 남긴다.
- 후속 검토 의견: MCP 기준 PDF와 PR test 기대 내용이 불일치한다. 현 브랜치에서 임의로 보정하면 `declared_none_table_whole_fits` 조건 또는 테스트 기대값을 뒤집어야 해서 PR의 핵심 목적이 깨진다. 작성자에게 한컴 버전/fixture 의도 차이 또는 실제 기준 PDF 보강을 요청하는 것이 맞다. HWP 2020 MCP 기준을 따르면 현재 synthetic fixture의 oracle 자체를 재확인해야 한다.

## 후속 검토 코멘트 초안

@planet6897 작업 감사합니다. 체리픽 통합과 로컬 검증은 통과했습니다.

사후 확인 의견으로 하나 남깁니다. PR에 포함된 `samples/task2097/none_table_declared_fits.hwpx`를 HWP 2020 MCP로 PDF 변환하면 텍스트 추출 기준으로 1쪽에는 `AFTER TABLE`, 2쪽에는 `BIG ROW`, `MID ROW`, `TAIL ROW EXPANDING`이 배치됩니다. 반면 PR 테스트와 rhwp dump는 1쪽 `Table`, 2쪽 `AFTER TABLE`을 기대합니다.

메인터너 쪽에서 이 fixture에 맞춰 임의 보정하려면 `declared_none_table_whole_fits` 조건 또는 테스트 기대값을 뒤집어야 해서 PR의 핵심 목적을 깨기 쉽습니다. 가능하면 이 synthetic fixture의 의도, 사용한 한컴 버전, 또는 실제 기준 PDF를 후속으로 보강해 주세요.

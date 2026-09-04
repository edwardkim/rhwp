---
kind: working
status: completed
issue_or_pr: 2627
stage: 3
last_verified: 2026-07-21
---

# PR #2627 · #2655 메인터너 통합 PR 준비

## 머지 판단

머지 보류 대상은 없다.

- [PR #2627](https://github.com/edwardkim/rhwp/pull/2627)의 각주 빈 꼬리말 밴드 회수는 #2559 대표 샘플을 98쪽에서 94쪽으로 개선한다. HWP 2020 MCP 기준 PDF 92쪽과의 잔여 +2쪽 및 #1733의 241/242쪽 차이는 숨기지 않고 기록했다.
- 잔여 knife-edge 8건은 [#2668](https://github.com/edwardkim/rhwp/issues/2668)에서 페이지 단위 각주-밴드 배분 재설계로 분리 추적한다. 따라서 이 통합 PR은 [#2559](https://github.com/edwardkim/rhwp/issues/2559)를 자동 close하지 않는다.
- [PR #2655](https://github.com/edwardkim/rhwp/pull/2655)의 범위 검사 본체는 [PR #2552](https://github.com/edwardkim/rhwp/pull/2552)로 이미 반영됐다. 이번 통합에는 여전히 빠져 있던 오류값/실제 옵션명 표시와 미지 옵션의 파일 I/O 전 중단만 포함했다. [#2551](https://github.com/edwardkim/rhwp/issues/2551)은 이미 closed 상태이므로 closing keyword를 사용하지 않는다.
- [PR #2561](https://github.com/edwardkim/rhwp/pull/2561)은 이미 merged다. r17 보고서의 원 수치는 유지하되, 원시 aggregate·manifest·바이너리 지문이 보존되지 않아 독립 재현 근거가 될 수 없음을 명시한다.

## 검증 근거

- 전체 회귀: `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 성공
- 정적 검사: `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 성공
- 리베이스 후 대상 회귀: `dump_pages_cli` 3개, #1733 2개, #2559 1개 성공
- 포맷/변경 검사: `cargo fmt --check`, `git diff --check` 성공
- HWP 2020 MCP 기준 PDF: `pdf/issue2559/1341000_research_report_footnotes-2020-print.pdf`, 92쪽, SHA-256 `ec7cebed92cf114da486eb4f8b4cbefa0739243e037d9a09ceebc433063e7e5e`
- visual sweep: rhwp 94쪽 / 기준 PDF 92쪽, 대표 1·46·92쪽에서 자동 flagged `0/3`.
  - 픽셀 일치율은 글꼴과 페이지 대응 차이 때문에 93.619~94.320%이며, 완전한 한컴 글꼴 fidelity 근거로 사용하지 않는다.
  - render-tree의 p82/p86 2.5~23.5px overflow 진단 후보는 남는다. 실제 페이지 확인에서 잘림은 보이지 않았으나, #2668 범위에서 재점검할 잔여다.

## PR 초안 요지

제목: `fix(layout): 각주 빈 꼬리말 밴드 회수와 dump-pages 오류 처리 보완`

본문에는 [PR #2627](https://github.com/edwardkim/rhwp/pull/2627), [PR #2655](https://github.com/edwardkim/rhwp/pull/2655), [PR #2561](https://github.com/edwardkim/rhwp/pull/2561)의 실제 적용 범위와 위 검증을 적고, #2559가 open으로 남는 이유와 [#2668](https://github.com/edwardkim/rhwp/issues/2668) 후속 트랙을 명시한다.

PR 번호 발급 뒤에는 `mydocs/pr/archives/pr_{N}_review.md`, `mydocs/pr/assets/pr_{N}/`, `mydocs/orders/20260721.md`를 같은 후속 커밋으로 추가한다.

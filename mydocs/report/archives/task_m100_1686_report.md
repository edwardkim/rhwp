# Task M100 #1686 최종 보고서

## 개요

- 이슈: #1686 `HWPX/HWP: co-anchored 다중 표 RowBreak 분할 시 후행 표가 후속 섹션보다 먼저 배치`
- PR: #1722 `task 1686: co-anchored RowBreak 표 후속 배치 보정`
- merge commit: `3a531a38f3808f288fed6c6ea1597f9dab9cb155`
- merge 시각: 2026-07-01 18:59 KST
- 이슈 상태: 수동 close 완료

## 처리 내용

`pr-1674` 샘플에서 빈 host 문단의 선행 RowBreak 표가 continuation을 만들 때 같은 문단의 후행
양수 offset co-anchored 표가 다음 섹션보다 먼저 배치되는 문제를 보정했다.

주요 변경:

- 빈 host 문단의 비-TAC `TopAndBottom`/`Para` RowBreak 표가 continuation을 만들면 후행 양수
  offset RowBreak 표를 즉시 배치하지 않고 보류한다.
- 보류된 표는 뒤쪽 표 문단 처리 후 또는 문서 마지막 flush 직전에 다시 조판한다.
- HWP RowBreak 분할 행의 미세 overflow 허용치를 2px로 제한한다.
- RowBreak 표 조각 뒤 빈 guide 문단의 비정상 vpos 누적을 흡수한다.
- visible host text가 있는 비-TAC RowBreak 표의 host text 렌더 순서를 마지막 continuation 뒤로 조정한다.

## 결과

- HWPX/HWP 모두 page 3에서 `다. 우대요건 등 [원서접수 마감일 기준]`와 우대요건 표가
  `[응시자격요건 고려사항]` 표보다 먼저 배치된다.
- HWPX/HWP 모두 최종 page count 35쪽을 유지한다.
- HWP page 5는 기준 PDF처럼 `동일 기간에 경력이 중복될 경우 유리한 경력 1개만 인정함`으로 시작한다.
- `☞ 임용예정직위...` 안내문은 page 5에 먼저 나오지 않고 RowBreak 표 뒤쪽으로 배치된다.

## 검증

PR #1722에서 수행한 로컬 검증:

- `cargo fmt`
- `git diff --check`
- `cargo test --profile release-test --test issue_1686 -- --nocapture`
- `cargo build --profile release-test --bin rhwp`
- 관련 RowBreak/co-anchored 회귀 테스트 묶음
- `cargo clippy --all-targets -- -D warnings`
- `wasm-pack build --target web --out-dir pkg`
- rhwp-studio headless Puppeteer 시각 검증

GitHub Actions:

- Build & Test: pass, 23분 39초
- CodeQL: pass
- Render Diff preflight: pass
- Canvas visual diff: pass

## 후속

- #1686은 `devel` base PR merge 후 auto-close되지 않아 수동 close했다.
- HWP 36쪽 vs PDF 35쪽으로 관찰되던 기존 차이는 #1722에서 35쪽으로 정리됐다.
- #1722 PR 본문에 언급된 page 19 `pi=169` 표 분할 차이는 #1686의 page 3 순서 역전과 별도 지점으로 남겨둔다.

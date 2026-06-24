# PR #1508 검토 — HWPX 분할 표와 경고 보정

- 작성일: 2026-06-24
- 작성자: [@jangster77](https://github.com/jangster77)
- PR: https://github.com/edwardkim/rhwp/pull/1508
- 제목: `task 1486: HWPX 분할 표와 경고 보정`
- 관련 이슈: #1486
- base/head: `edwardkim/rhwp:devel` ← `edwardkim/rhwp:task_m100_1486`
- 문서 작성 직전 참고값: head `cb2596af`, draft 아님, `MERGEABLE` / `BLOCKED`
- 문서 작성 직전 변경 규모: 22 files, +1654 / -81
- 최종 merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인

## 1. 요약 판단

**CI 완료 후 merge 가능 후보**로 판단한다.

이 PR은 `samples/hwpx_sample2.hwpx` 기준 HWPX 조판 회귀를 단계적으로 보정한다. 핵심은
분할 표 내부 TAC/중첩 표 배치, RowBreak rowspan 분할 컷, 셀 내부 그림 높이 반영, footer 쪽번호 간격,
마지막 쪽 TAC 로고 위치, HWPX validation 경고 상태 초기화다.

로컬 사전 검증은 PR 준비 기준으로 모두 통과했다. 다만 review 문서와 오늘할일 커밋이 PR head에
추가되므로, merge 전에는 이 커밋이 포함된 최신 head 기준 GitHub Actions 완료 상태를 다시 확인해야 한다.

## 2. 변경 범위

| 범위 | 주요 파일 | 내용 |
|------|-----------|------|
| HWPX sample/oracle | `samples/hwpx_sample2.hwpx`, `samples/hwpx_sample2.hwp`, `pdf/hwpx_sample2-2024.pdf` | #1486 재현 샘플과 한컴 PDF 기준 추가 |
| 분할 표 측정/컷 | `src/renderer/typeset.rs`, `src/renderer/height_measurer.rs`, `src/renderer/layout/table_layout.rs`, `src/renderer/layout/table_partial.rs` | RowBreak continuation, rowspan block cut, row offset, hard-break orphan 되감기 보정 |
| TAC/그림 위치 | `src/renderer/layout.rs`, `src/renderer/layout/paragraph_layout.rs`, `src/renderer/layout/table_layout.rs` | 분할 표 안 TAC 중첩 표, 마지막 쪽 LH 로고, 셀 내부 Square 그림 흐름 높이 보정 |
| HWPX 경고/UI | `src/document_core/commands/document.rs`, `src/wasm_api/tests.rs`, `rhwp-studio/src/main.ts` | HWPX validation report 초기화, textRun reflow 권장 자동 보정 제외, 새 문서 경고 재노출 방지 |
| 테스트/문서 | `tests/issue_1486_hwpx_partial_tac_table.rs`, `mydocs/plans/task_m100_1486*.md`, `mydocs/working/task_m100_1486_stage*.md` | 회귀 테스트와 단계별 분석 기록 추가 |

## 3. 주요 검토 포인트

### 3.1 HWPX partial TAC/table overflow

분할 표 내부의 TAC 중첩 표가 남은 줄 폭을 넘을 때 셀 좌측 기준으로 배치되도록 하여 오른쪽 page body
밖 overflow를 막았다. `issue_1486_partial_table_tac_nested_table_stays_inside_page_body` 테스트가 이 조건을
고정한다.

### 3.2 RowBreak continuation과 rowspan block

RowBreak 표의 continuation에서 마지막 빈 조각이 별도 페이지를 만들지 않도록 보이는 내용 여부를 확인한다.
또한 rowspan block 안에서 아래 행 셀이 먼저 소비되는 문제를 row offset 기반 block cut으로 보정했다.

Stage 7에서는 PR 준비 중 `issue_1156_rowbreak_fragment_fit` 회귀가 발견되어 추가 보정했다. 일반 row cut에서
이전 행에서 시작한 rowspan 셀이 현재 행을 덮는 경우에만 same-paragraph hard-break 직전 orphan slice를
되감는다. 전역 적용은 #1486 마지막 TAC 로고를 다음 페이지로 밀었기 때문에 제한 조건을 둔 것이 핵심이다.

### 3.3 13쪽 footer와 19쪽 그림 잘림

footer 높이가 0인 페이지의 쪽번호를 실제 꼬리말 여백 중앙으로 내려 하단 표와 쪽번호 간격을 확보했다.
셀 내부 Square 계열 비-TAC 그림은 시각 하단을 콘텐츠 높이 후보에 반영하여 19쪽 하단 그림 clipping을
보정했다.

### 3.4 마지막 쪽 LH 로고 위치

TAC 그림이 텍스트가 있는 줄에서도 불필요하게 label extra를 받던 조건을 보정해 마지막 쪽 LH 로고가 한컴 PDF
기준 위치에 맞도록 했다. `issue_1486_page29_tac_logo_aligns_with_text_line` 테스트가 bbox 기준을 고정한다.

### 3.5 HWPX validation 경고와 새 문서 상태

HWPX textRun reflow 의심 경고는 계속 노출하되, `reflowLinesegs()` 권장 자동 보정 대상에서는 제외했다.
해당 보정은 `hwpx_sample2.hwpx`의 페이지 수를 31쪽에서 30쪽으로 바꿀 수 있어 사용자가 원하지 않는 문서
변형으로 이어질 수 있다.

또한 새 문서 생성 시 `source_format`과 `validation_report`를 초기화해, 이전 HWPX 문서의 경고 팝업이 새
파일에서도 반복되는 문제를 막았다.

## 4. 로컬 검증

PR 준비 기준으로 다음 검증을 통과했다.

| 명령 | 결과 |
|------|------|
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과, 1926 passed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 통과, 0 passed / 1 ignored |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| `cd rhwp-studio && npm test` | 통과, 147 passed |

추가 targeted 검증:

- `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture`
- `cargo test --profile release-test --test issue_1156_rowbreak_fragment_fit -- --nocapture`
- `cargo test --profile release-test --test issue_1105 -- --nocapture`

## 5. GitHub Actions

문서 작성 시점 참고값:

| 체크 | 상태 |
|------|------|
| Build & Test | 진행 중 |
| Canvas visual diff | 진행 중 |
| Analyze (javascript-typescript) | 성공 |
| Analyze (python) | 성공 |
| Analyze (rust) | 진행 중 |
| WASM Build | skipped |

review 문서와 오늘할일 커밋 push 후 GitHub Actions가 다시 실행될 수 있다. merge 전에는 최신 head 기준
상태를 다시 확인해야 한다.

## 6. 리스크와 후속 확인

| 항목 | 평가 | 비고 |
|------|------|------|
| RowBreak 분할 컷 공용 로직 영향 | 중간 | #1105, #1156, #1486 targeted 테스트와 전체 통합 테스트 통과 |
| HWPX textRun 경고 자동 보정 제외 | 낮음 | 경고는 유지하고 자동 변형만 막음. 페이지 수 보존 목적 |
| 샘플/PDF 추가로 인한 저장소 크기 증가 | 낮음 | #1486 재현과 시각 기준 보존 목적 |
| Render diff | 확인 필요 | 최신 PR head 기준 GitHub `Canvas visual diff` 통과 필요 |

## 7. 권고

PR head 최신 커밋 기준 GitHub Actions가 모두 완료되고, 작업지시자가 승인한 상태이므로 **merge 가능**으로
판단한다.

merge 후에는 다음을 확인한다.

1. #1486 자동 close 여부 확인, 실패 시 수동 close
2. `local/devel`을 `upstream/devel`로 sync
3. 원격 작업 브랜치 `task_m100_1486` 삭제
4. 렌더 영향 PR이므로 `cargo test --test svg_snapshot` 실행
5. 오늘할일에 merge SHA와 이슈 close 여부 갱신

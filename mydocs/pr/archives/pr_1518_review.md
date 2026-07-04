# PR #1518 검토 — co-anchored float 표 배치와 PDF 정합 보정

- 작성일: 2026-06-25
- 작성자: [@jangster77](https://github.com/jangster77)
- PR: <https://github.com/edwardkim/rhwp/pull/1518>
- 제목: `task 1510: co-anchored float 표 배치와 PDF 정합 보정`
- 관련 이슈: #1510
- 처리 경로: collaborator self-merge 후보 예외 경로
- base/head: `edwardkim/rhwp:devel` ← `edwardkim/rhwp:task_m100_1510`
- 문서 작성 직전 참고값: head `91341137`, draft 아님, `MERGEABLE` / `BLOCKED`
- 문서 작성 직전 변경 규모: 20 files, +809 / -48
- 최종 merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + review 문서 PR diff 포함 + 작업지시자 승인

## 1. 요약 판단

**CI 완료 후 merge 가능 후보**로 판단한다.

이 PR은 #1510에서 보고된 "한 문단에 함께 앵커된 para-relative TopAndBottom floating 표" 문제를
합성 재현 샘플과 한컴 2024 PDF 기준으로 보정한다. 핵심은 visible text host 문단의 float 표를
`vertical_offset` 값으로 재정렬하지 않고 control/document 순서를 유지하는 것, HWPX unsigned 음수 offset을
signed 값으로 복원하는 것, 그리고 양수 offset 표가 본문 흐름을 과도하게 밀거나 겹치게 하는 경로를 줄이는 것이다.

HWP 샘플은 한컴 2024 PDF와 같은 1쪽 구조로 수렴했고, HWPX 샘플은 한컴 2024 PDF와 같은 2쪽 구조를 회복했다.
다만 HWPX 1쪽 표 셀 2행 텍스트 y 위치에 약 4pt 잔차가 남아 있으며, 이 잔차는 표 전체 예약 높이를 줄이는 방식으로
해소하면 `filler paragraph 30` 페이지 분할이 다시 깨지므로 후속 내부 row/baseline 조정 대상으로 분리한다.

PR 본문에는 `Closes #1510`이 포함되어 있으나, 문서 작성 시점의 GitHub `closingIssuesReferences`는 빈 배열이다.
merge 후 #1510 자동 close 여부를 반드시 확인해야 한다.

## 2. 변경 범위

| 범위 | 주요 파일 | 내용 |
|------|-----------|------|
| 재현 자료 | `samples/issue1510_coanchored_float_tables.{hwp,hwpx}`, `pdf/issue1510_coanchored_float_tables*` | 이슈 조건에 맞춘 HWP/HWPX 합성 샘플, 한컴 2024 PDF 기준, rhwp PDF 산출물 추가 |
| HWPX 파싱 | `src/parser/hwpx/section.rs` | `vertOffset`/`horzOffset`의 unsigned 음수 표현을 wrapping signed 값으로 해석 |
| float 조판 | `src/renderer/typeset.rs`, `src/renderer/layout.rs`, `src/renderer/layout/table_layout.rs`, `src/renderer/layout/table_partial.rs` | visible text host 문단의 float 표 순서, stacking, active exclusion, 페이지 분할 보정 |
| 문단/본문 피치 | `src/renderer/layout/paragraph_layout.rs`, `src/renderer/typeset.rs` | 순수 본문 텍스트 줄 진행을 저장된 `text_height + spacing` 기준으로 보정 |
| PDF export | `src/renderer/pdf.rs`, `src/document_core/queries/rendering.rs`, `src/renderer/mod.rs` | 단일 페이지 PDF도 SVG px → PDF pt 환산 경로를 사용하도록 통일 |
| 회귀 테스트 | `tests/issue_1510.rs` | HWP 1쪽, control 순서, 음수/양수 offset 배치, HWPX 2쪽 페이지네이션 회귀 고정 |
| 작업 기록 | `mydocs/working/task_m100_1510_stage1.md` ~ `stage4.md` | 단계별 조사, 시각 비교, 기각한 접근, 잔차 기록 |

## 3. 주요 검토 포인트

### 3.1 visible text host 문단의 control 순서

Stage 1은 visible text가 있는 host 문단의 para-relative `TopAndBottom` float 표를 `vertical_offset` 기준으로
정렬하지 않도록 했다. 이로써 HWP 샘플의 표 control 순서가 `[2, 3, 4]`로 유지되고, 큰 양수 offset 표가
문단 시작부터 전체 본문 흐름을 밀어 2쪽을 만드는 문제를 막는다.

빈 host 문단의 기존 lane 정렬 경로와 #986/#712 회귀 조건은 별도 테스트로 보존했다.

### 3.2 active exclusion과 시각 정합

Stage 2는 양수 offset 표를 완전히 무시하지 않고, 후속 본문이 실제 표 y 구간에 닿을 때만 cursor를 표 하단으로
이동하도록 조정했다. HWP 기준으로 `filler 07`은 표 위에 남고 `filler 08`부터 표 아래에서 재개되는 구조를
테스트로 고정했다.

### 3.3 HWPX unsigned negative offset

Stage 3에서 HWPX XML의 B 표 `vertOffset=4294965296`이 unsigned 표현의 `-2000 HU`임을 확인했다. 기존 parser는
이를 0으로 포화시켜 HWPX 표 stacking과 페이지 분할이 한컴 2024 기준과 달라졌다. wrapping signed 해석 뒤
HWPX도 한컴 2024 PDF와 같은 2쪽 구조로 맞춰졌다.

### 3.4 PDF page size와 본문 줄 피치

Stage 4는 단일 페이지 `export-pdf`가 SVG px 값을 PDF pt로 환산하지 않아 HWP PDF가 `793.707 x 1122.48 pt`로
커지던 문제를 고쳤다. 수정 후 HWP rhwp PDF는 `595.28 x 841.86 pt`로 한컴 2024 A4 PDF와 같은 좌표계에서
비교할 수 있다.

본문 순수 텍스트 줄 진행은 HWP/HWPX 모두 저장된 `text_height + spacing` 기준을 사용한다. HWP 표 셀 경로는
현재 한컴 기준보다 이미 위쪽에 가까워 같은 보정을 적용하지 않았다.

## 4. 로컬 검증

PR 준비 기준으로 다음 검증을 통과했다.

| 명령 | 결과 |
|------|------|
| `cargo build --bin rhwp` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| `cargo test --test issue_1510 -- --nocapture` | 통과 |
| `cargo test --test issue_986 -- --nocapture` | 통과 |
| `cargo test --test issue_712 -- --nocapture` | 통과. 기존 `LAYOUT_OVERFLOW ... overflow=2.8px` 로그는 유지 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |

이번 review 문서/오늘할일 추가 커밋은 문서 전용 변경이므로, 커밋 전 별도 검증은 `git diff --check`와 변경 범위
확인을 수행한다.

## 5. 시각 비교 결과

Stage 4 기준 산출물: `output/pdf/issue1510_stage4_pdf_scale/`

| 항목 | 결과 |
|------|------|
| HWP page size | rhwp `595.28 x 841.86 pt`, Hancom `595 x 841 pt` |
| HWP `filler29` y 오차 | 약 `+0.9pt` |
| HWP `filler30` y 오차 | 약 `+1.0pt` |
| HWP PNG diff | `diff>30=2.6297%`, `diff>80=1.7344%`, `rms=24.133` |
| HWPX page 1 PNG diff | `diff>30=2.2805%`, `rms=21.614` |
| HWPX page 2 PNG diff | `diff>30=0.1450%`, `rms=4.388` |

## 6. GitHub Actions

문서 작성 시점 참고값:

| 체크 | 상태 |
|------|------|
| Build & Test | 진행 중 |
| Canvas visual diff | 진행 중 |
| Analyze (javascript-typescript) | 성공 |
| Analyze (python) | 성공 |
| Analyze (rust) | 진행 중 |
| WASM Build | skipped |
| CodeQL aggregate | neutral/skipping |

review 문서와 오늘할일 커밋 push 후 GitHub Actions가 다시 실행될 수 있다. merge 전에는 최신 head 기준
상태를 다시 확인해야 한다.

## 7. 리스크와 후속 확인

| 항목 | 평가 | 비고 |
|------|------|------|
| float 표 조판 공통 경로 영향 | 중간 | visible text host + para-relative TopAndBottom 중심으로 제한. #986/#712 targeted 테스트 통과 |
| HWPX signed offset 해석 | 낮음 | unsigned 음수 표현을 wrapping signed로 복원하는 포맷 해석 보정 |
| 본문 줄 피치 보정 | 중간 | 순수 본문 텍스트 중심. HWP 표 셀 경로는 제외 |
| HWPX 1쪽 표 셀 y 잔차 | 낮음 | 페이지 분할과 본문 흐름은 맞고, 내부 row/baseline 후속으로 분리 |
| 샘플/PDF 추가 | 낮음 | #1510 재현과 시각 기준 보존 목적 |
| GitHub issue auto-close | 확인 필요 | PR 본문에 `Closes #1510` 포함. metadata는 작성 시점 빈 배열. close 코멘트에는 임시 합성 샘플 검증 사실과 추가 샘플 zip 요청을 명시 |

## 8. 권고

PR head 최신 커밋 기준 GitHub Actions가 모두 완료되고 작업지시자가 승인하면 **merge 가능**으로 판단한다.

merge 후에는 다음을 확인한다.

1. #1510 자동 close 여부 확인, 실패 시 수동 close
2. `local/devel`을 `upstream/devel`로 sync
3. 원격 작업 브랜치 `task_m100_1510` 삭제
4. 렌더 영향 PR이므로 `cargo test --test svg_snapshot` 실행
5. #1510 close 코멘트에 임시 합성 HWP/HWPX 파일과 한컴 2024 PDF 기준으로 검증했음을 설명하고, 부족한 경우 재현 샘플을 zip으로 첨부해 달라고 요청
6. 오늘할일에 merge SHA와 이슈 close 여부 갱신

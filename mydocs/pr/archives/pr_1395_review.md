# PR #1395 검토 - 미주 덤프와 sweep 검증 인프라 분리

- PR: https://github.com/edwardkim/rhwp/pull/1395
- 제목: task 1394: 미주 덤프와 sweep 검증 인프라 분리
- 작성일: 2026-06-12
- 작성자: `jangster77`
- 관련 이슈: #1394 "task 1293 후속: 미주 덤프와 sweep 검증 인프라 분리 제출"
- base: `devel`
- head: `jangster77:task_m100_1394`
- 상태: open, draft 아님

## 1. 요약 판단

**작업지시자 승인 후 merge 가능**으로 판단한다.

이번 PR은 Task 1293의 미주 pagination 구현 수정이 아니라, 후속 분석과 회귀 검증에 필요한
덤프 CLI, sweep 분석 보강, 한컴 기준 샘플/PDF만 분리 제출한다. 렌더링 본체 파일인
`typeset.rs`, `height_cursor.rs`, `layout.rs`는 변경하지 않는다.

주의 사항은 변경량이 크다는 점이다. GitHub 기준 `+1641 / -16`, 29 files이며, 대부분은
한컴 기준 샘플과 PDF 바이너리 추가다. 코드 변경은 `src/main.rs`의 덤프 CLI와
`scripts/task1274_visual_sweep.py`의 분석 스크립트에 한정된다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| base | `devel` |
| head | `jangster77:task_m100_1394` |
| mergeable | `MERGEABLE` |
| 변경량 | 29 files, +1641 / -16 |
| assignee | `jangster77` |
| 연결 이슈 | #1394, PR 본문 `Closes #1394` 포함 |

커밋:

- `40a33271` - task 1394: 미주 덤프 검증 인프라 분리
- `47c25529` - Merge branch 'devel' into task_m100_1394
- `0ead213a` - task 1394: clippy counter loop 수정

## 3. 변경 범위

### 3.1 `src/main.rs`

덤프 보조 CLI 2개를 추가했다.

- `dump-note-shape <파일.hwp|파일.hwpx>`
  - 구역별 `footnoteShape`, `endnoteShape` raw 값을 JSON으로 출력한다.
  - 한컴 UI 의미값인 구분선 위, 미주 사이, 구분선 아래 환산값을 함께 출력한다.
- `dump-endnote-lines <파일.hwp> <section> <para> <control> [note-para]`
  - 특정 미주 원본 문단의 `line_seg`, `TextRun`, 컨트롤 위치, TAC 수식 정보를 함께 출력한다.
  - Task 1293에서 문제가 된 "문단 내 lineSegArray / line_seg / 글자처럼 취급해야 하는 수식" 추적용이다.

### 3.2 `scripts/task1274_visual_sweep.py`

미주/문항 흐름 sweep 검출을 보강했다.

- frame overflow, column drift, question flow 후보를 분석 결과로 좁힐 수 있도록 확장
- PDF/PNG/render tree 기반 비교에서 후속 구현 PR의 회귀 후보를 추적할 수 있도록 보강
- Task 1274/1284에서 쌓인 수동 시각 검증 항목을 자동 후보 검출 쪽으로 일부 이관

### 3.3 샘플과 한컴 기준 PDF

한컴 미주 모양 설정별 샘플과 기준 PDF를 추가했다.

- `3-11월_실전_통합_2024-*`
  - 구분선 없음/있음
  - 구분선 위 0/9/20
  - 미주 사이 0/7/8/20
  - 구분선 아래 0/2/7/20
- `수식-문자처럼취급-아님`
  - 수식이 무조건 글자처럼 취급되는 것은 아님을 확인하는 별도 샘플

## 4. 제외 범위

이번 PR은 다음을 포함하지 않는다.

- 미주 pagination 구현 수정
- `typeset.rs`, `height_cursor.rs`, `layout.rs` 렌더링 동작 변경
- 실제 문제집 overflow/overlap 해결
- golden SVG 갱신

즉, 문제집 렌더링을 직접 고치는 PR이 아니라 후속 구현을 위한 관측/검증 인프라 PR이다.

## 5. 검증 결과

### 5.1 GitHub Actions

최종 CI는 모두 통과했다.

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped (PR 조건상 skip) |

### 5.2 로컬 사전 확인

이번 PR 준비 중 확인한 항목:

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo build --verbose` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| `cargo test --features native-skia skia --lib --verbose` | 통과 |

참고:

- 전체 `cargo test --verbose`는 상당 구간 통과 후 작업지시자 지시에 따라 중단했다.
- `cargo nextest run`은 이 저장소에서 테스트 바이너리 `--list` 단계가 0% CPU로 오래 남아 완료 판정까지 가지 못했다.
- 로컬 `cargo clippy -- -D warnings`도 macOS 로컬 환경에서 `clippy-driver` 대기 문제가 있었으나,
  GitHub Actions의 `cargo clippy -- -D warnings`는 최종 통과했다.

### 5.3 Merge 시뮬레이션

`git merge-tree --write-tree upstream/devel HEAD` 결과 `rc=0`으로 충돌 없음.

## 6. 리스크

| 리스크 | 평가 |
|---|---|
| 렌더링 회귀 | 낮음. 렌더링 본체 변경 없음 |
| 샘플/PDF 용량 증가 | 있음. 한컴 기준 회귀 검증용으로 의도된 추가 |
| sweep 오탐/미탐 | 있음. 자동 판정이 아니라 후보 좁히기 도구로 봐야 함 |
| `src/main.rs` CLI 충돌 | 중간. PR #1366, #1359도 `src/main.rs`를 건드리나 현재 #1395 자체는 mergeable |
| 후속 구현 오해 | 낮음. PR 본문과 이 리뷰에서 구현 수정 제외 범위를 명시 |

## 7. 최종 권고

작업지시자 승인 후 merge 가능하다.

권고 순서:

1. 작업지시자에게 merge 승인 확인
2. 승인 시 PR #1395 merge
3. #1394 auto-close 여부 확인
4. `local/devel` sync
5. 본 리뷰 문서와 구현 계획서를 archives로 이동

## 8. 후속 처리 결과

- 작업지시자 승인: 2026-06-12 "진행"
- merge 완료: PR #1395, merge commit `ef14397a1800aa897521f31fb62c04b2350ea21b`
- 이슈 #1394: GitHub auto-close가 동작하지 않아 수동 close
- PR 코멘트: merge 완료와 검증 결과 요약 등록
- `local/devel`: `upstream/devel` 기준 rebase 완료
- 리뷰 문서: archives 이동

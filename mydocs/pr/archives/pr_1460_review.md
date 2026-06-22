# PR #1460 리뷰 기록 - 글자처럼 해제 그림 재흐름 보정

- PR: https://github.com/edwardkim/rhwp/pull/1460
- 작성일: 2026-06-22
- 작성자: collaborator self-merge 후보 경로
- 작성 시점 head: `e80e9f534b0d1c179eeff1bb5447b359e0a3c50a`
- base: `devel`
- head: `task_m100_1459`

## 1. PR 메타

| 항목 | 확인 내용 |
|------|-----------|
| 작성자 | `jangster77` |
| PR 상태 | open, draft 아님 |
| merge 상태 | 작성 시점 `CLEAN` |
| 관련 이슈 | `Closes #1459` |
| 규모 | 작성 시점 23 files, +1096 / -152 |
| 커밋 수 | 7개 + 본 self-merge review 문서 커밋 예정 |

`mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 이 문서는 작성 시점 값을 참고로만 기록한다.
최종 merge 판단은 merge 직전 최신 PR head 기준으로 다시 확인한다.

## 2. 변경 범위

### 2.1 자리차지 그림과 TAC 그림 혼합 문단

- 한컴 기준으로 비-TAC `TopAndBottom` 그림이 먼저 흐름을 예약하고, 같은 문단의 남은 TAC 그림이 그 아래로 다시 흐르도록 보정했다.
- 텍스트 없는 그림 전용 문단에서 TAC 해제 시 `LineSeg`를 남은 TAC 개체 기준으로 재구성한다.
- `TopAndBottom` 예약 높이를 남은 TAC 줄의 `vertical_pos`에 반영해 속성 변경 직후 렌더 순서와 간격을 다시 계산한다.
- 이미 `LINE_SEG`가 예약 높이를 반영한 경우에는 sibling 예약 높이를 중복으로 더하지 않도록 보정했다.

### 2.2 비-TAC 개체의 inline/cursor 제외

- renderer composer가 비-TAC `Picture`/`Shape`/`Table`을 inline control slot으로 세지 않도록 수정했다.
- TAC 개체만 inline control과 marker 보정 대상으로 사용해 자리차지 그림으로 커서가 진입하지 않도록 했다.
- cursor/navigation/doc tree helper가 글자처럼 취급되지 않는 그림을 문자 stop으로 보지 않도록 맞췄다.

### 2.3 편집 경로 회귀 보정

- 클립보드 내부 분할 offset이 cursor/nav logical offset과 paragraph split offset을 혼동하지 않도록 분리했다.
- PR head 재실행 CI에서 `issue_1198_nested_cell_paste`가 실패한 원인을 로컬에서 재현하고, `Paragraph::split_at` 기준 offset으로 보정했다.

### 2.4 샘플과 회귀 테스트

- `samples/투명도0-50.hwp`에서 첫 번째 그림의 `글자처럼 취급`을 해제하는 실제 속성 변경 경로를 검증한다.
- 한컴 저장본 `samples/투명도0-50-2nd그림글차처럼off.{hwp,hwpx}`를 추가해 혼합 그림 배치와 cursor stop 제외를 검증한다.
- `tests/issue_1459_topbottom_picture_reflow.rs`에서 render tree 순서, bbox 간격, 속성 변경 후 줄 재구성을 확인한다.

## 3. 리스크

| 리스크 | 판단 |
|--------|------|
| 렌더링 레이아웃 공통 경로 변경 | `issue_1459_topbottom_picture_reflow`와 composer 단위 테스트로 비-TAC/TAC 혼합 그림 흐름을 고정했다. |
| cursor/navigation 공통 경로 변경 | 비-TAC 개체를 문자 stop에서 제외하고, #1452 커서 회귀 및 #1198 nested cell paste 경로를 재검증했다. |
| binary fixture 추가 | 실제 한컴 저장본 HWP/HWPX 샘플이라 렌더 정합 회귀에 필요하며, 테스트에서 직접 소비한다. |
| PR workflow 문서 동반 변경 | GitHub Actions와 macOS 로컬 검증 명령을 일치시키기 위한 절차 보강이다. 기능 코드와 분리된 문서 변경이다. |

## 4. 검증

로컬 검증:

```bash
cargo fmt
cargo fmt --check
git diff --check
cargo test --profile release-test tac_toggle_true_to_false_restores_empty_picture_para_line_seg -- --nocapture
cargo test --profile release-test test_identify_inline_controls_table -- --nocapture
cargo test --profile release-test --test issue_1459_topbottom_picture_reflow -- --nocapture
cargo test --profile release-test --test issue_1452_saved_caret -- --nocapture
cargo test --profile release-test --test issue_1139_inline_picture_duplicate issue_1293_equation_control_is_not_always_treat_as_char -- --nocapture
cargo test --profile release-test --test issue_1139_inline_picture_duplicate issue_1139_endnote -- --nocapture
cargo test --profile release-test --lib
cargo test --profile release-test --tests
cargo build --release
cargo test --release --lib
cargo test --doc
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
cargo clippy --all-targets -- -D warnings
```

작업지시자 시각 검증:

- 한컴 기준처럼 투명도 50 그림이 먼저 흐름을 예약하고, TAC 그림이 그 아래로 재배치되는지 확인했다.
- `글자처럼 취급` 해제 후 자리차지 그림으로 커서가 진입하지 않는지 확인했다.
- 속성 변경 직후 한컴과 rhwp의 그림 렌더 순서와 간격 차이를 반복 비교했다.

GitHub Actions 작성 시점 확인:

- Build & Test: pass
- Canvas visual diff: pass
- CodeQL: pass
- WASM Build: skipped

본 review 문서 커밋 push 후 GitHub Actions가 다시 실행될 수 있으므로, merge 전 최신 head 기준으로 위 상태를 재확인한다.

## 5. 판단

작성 시점 기준으로 #1459의 핵심 목표인 비-TAC `TopAndBottom` 그림과 TAC 그림 혼합 문단의 한컴식 재흐름, 비-TAC 그림 cursor stop 제외, 속성 변경 직후 렌더 재계산이 PR 범위에 포함되어 있다.

최종 조건:

1. 본 review 문서 2건과 오늘할일 문서가 PR head에 포함된다.
2. push 후 최신 PR head 기준 GitHub Actions가 통과한다.
3. 작업지시자 승인 상태가 유지된다.

위 조건 충족 시 collaborator self-merge 후보로 merge 수용한다.

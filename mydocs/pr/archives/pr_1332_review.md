# PR #1332 검토 - 빈 글머리표 줄 marker/caret 크기 보정 (#1330)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1332
- 작성자: `postmelee`
- 상태: open / draft 아님
- base: `devel`
- head: `issue-1330-bullet-marker-caret-size` (`943bb9e0`)
- 이슈: #1330

## 2. 변경 요약

PR의 기능 변경 목표는 글머리표/번호 문단에서 Enter로 새 빈 list 문단을 만들었을 때,
입력 전 marker/caret 크기와 입력 후 실제 본문 run 크기가 달라지는 문제를 보정하는 것이다.

핵심 변경:

- `src/renderer/layout/paragraph_layout.rs`
  - `paragraph_active_text_style()` helper 추가.
  - `numbering_marker_text_style()` helper 추가.
  - 빈 문단 anchor TextRun과 numbering marker가 default char shape 0 대신 문단의 active char shape를 사용하도록 보정.
- `tests/issue_1330_bullet_marker_caret_size.rs`
  - split 직후 빈 list 문단과 입력 후 문단의 marker/body/caret 크기 일관성 회귀 테스트 추가.

## 3. GitHub 상태

- PR #1332는 현재 `CONFLICTING` 상태다.
- 원 PR에는 기능 파일 2개 외에 contributor 작업 문서와 `mydocs/orders/20260608.md` 변경이 포함되어 있다.
- 보조 통합 PR #1340이 제출되었으나 closed/unmerged 상태다.
- #1340 기능 코드와 #1332 기능 코드는 동일하다.
- #1340 Render Diff 실패 기록은 실제 visual diff 실패가 아니라 `Install wasm-pack` 단계 실패였다.

따라서 현재 검토는 원 PR을 그대로 merge하지 않고, 기능 커밋만 현재 `local/devel` 위에 cherry-pick하여 진행했다.

## 4. 로컬 적용 및 추가 보정

검토 브랜치:

```text
local/pr1332-merge-test
```

적용:

```text
local/devel 기준으로 b1ed57ca cherry-pick
```

초기 결과:

- cherry-pick 충돌 없음.
- 하지만 현재 `devel` 위에서는 새 테스트 `issue_1330_bullet_marker_caret_size`가 실패했다.

실패 내용:

```text
empty caret height follows active char shape: actual=12.000, expected=24.000
```

원인:

- PR #1332의 변경은 레이아웃 TextRun/marker style에는 적용된다.
- 그러나 `get_cursor_rect_native()`의 빈 list 문단 fallback은 `TextLine` 높이 12px를 그대로 cursor height로 반환한다.
- #1329 이후 빈 list 문단 caret x 좌표는 marker 뒤 본문 시작점으로 fallback 처리하는 경로가 필요하므로, 단순히 empty anchor TextRun hit를 바로 반환하면 x 위치 회귀가 발생할 수 있다.

메인테이너 보강:

- `src/document_core/queries/cursor_rect.rs`
  - 빈 list 문단 fallback의 line height가 내부 TextRun/marker의 실제 `font_size` 이상이 되도록 보정.
  - x 위치는 기존 #1329 fallback 정책을 유지한다.

## 5. 로컬 검증

통과:

- `cargo fmt --all -- --check`
- `cargo test --test issue_1330_bullet_marker_caret_size` - 1 passed
- `cargo test --test issue_1329_bullet_caret` - 3 passed
- `cargo test --test issue_1308_forced_break_hanging_indent` - 8 passed
- `cargo test --test issue_1071_tac_cursor_nav` - 4 passed
- `cargo test --lib` - 1615 passed, 0 failed, 6 ignored
- `cargo clippy --all-targets -- -D warnings`

## 6. 리스크와 판단

수용 가능:

- 기능 변경 범위가 작고, #1330 원인 분석이 타당하다.
- #1329와 결합했을 때 필요한 cursor height 보강 지점도 확인되어 테스트로 고정했다.
- 원 PR의 `mydocs/` 작업 문서는 현재 메인테이너 문서 상태와 충돌하므로 통합 대상에서 제외하는 것이 맞다.

주의:

- 대상은 rhwp-studio의 caret/marker 시각 UX이므로 WASM 빌드 후 메인테이너 동작 판정이 필요하다.
- 원 PR 그대로 merge하면 오래된 base와 `mydocs/` 변경 때문에 현재 `devel`을 되돌리는 diff가 섞인다.

## 7. 권장 절차

권장:

1. 기능 변경만 현재 `devel` 기준으로 통합한다.
2. 원작성자 attribution은 `postmelee` author로 보존한다.
3. 메인테이너 보강 `cursor_rect.rs` 2줄 변경을 함께 포함한다.
4. WASM 빌드 후 rhwp-studio에서 #1330 동작 판정을 진행한다.
5. 동작 판정 통과 시 PR #1332에 수용 코멘트를 남기고, #1330을 close 처리한다.

현재 결론:

```text
조건부 수용 권고 - WASM 빌드 및 메인테이너 동작 판정 필요
```

# PR #1331 리뷰 — 빈 글머리표 줄 caret 위치 보정

**작성일**: 2026-06-08  
**PR**: https://github.com/edwardkim/rhwp/pull/1331  
**작성자**: `postmelee`  
**제목**: `Task #1329: 빈 글머리표 줄 caret 위치 보정`

## 1. 메타

| 항목 | 값 |
|---|---|
| base | `devel` |
| head | `issue-1329-bullet-caret` |
| head sha | `1d645f6da4cba713fd655554d7e86284c98f1900` |
| draft | false |
| mergeable | `CONFLICTING` |
| GitHub merge state | `DIRTY` |
| commits | 1 |
| changed files | 9 |
| 규모 | +1189 / -11 |
| 관련 이슈 | Refs #1329, Refs #1330 |

## 2. 변경 범위

문제:

- 글머리표/번호 문단 끝에서 Enter를 누른 직후 새 빈 list 문단이 만들어진다.
- 실제 입력 위치는 marker 뒤 본문 시작점이지만, 입력 전 caret이 marker 앞쪽에 표시된다.
- 문서 위치와 화면 caret 위치가 어긋나 UX 혼란이 발생한다.

기능 변경:

- `src/document_core/queries/cursor_rect.rs`
  - ParaShape `HeadType`으로 list 문단(`Outline`, `Number`, `Bullet`) 여부를 판별.
  - 빈 list 문단의 zero-length body anchor를 직접 hit으로 반환하지 않고 fallback으로 넘김.
  - fallback에서 marker TextRun 오른쪽 끝과 본문 TextRun x를 수집해 `charOffset: 0` caret x를 marker 뒤 본문 시작점으로 보정.
- `tests/issue_1329_bullet_caret.rs`
  - 글머리표 Enter 후 빈 list 문단 caret x 검증.
  - 번호 문단 동일 검증.
  - 일반 빈 문단 회귀 검증.

원 PR에 포함된 문서 변경:

- `mydocs/orders/20260608.md`
- `mydocs/plans/task_m100_1329.md`
- `mydocs/plans/task_m100_1329_impl.md`
- `mydocs/report/task_m100_1329_report.md`
- `mydocs/working/task_m100_1329_stage1.md`
- `mydocs/working/task_m100_1329_stage3.md`
- `mydocs/working/task_m100_1329_stage4.md`

위 문서들은 기여자 작업 브랜치의 내부 작업 기록이며 현재 `devel`의 `mydocs/orders/20260608.md`, archive 정리 상태와 충돌한다. 기능 통합 범위에서는 제외하는 편이 안전하다.

## 3. GitHub 상태

PR #1331 checks:

- Build & Test: pass
- CodeQL: pass
- Analyze javascript-typescript/python/rust: pass
- WASM Build: skipped

PR #1331 상태:

- `mergeable`: `CONFLICTING`
- `mergeStateStatus`: `DIRTY`
- 충돌 원인: 기능 코드가 아니라 `mydocs/` 작업 기록과 오래된 base로 인한 문서/메타 충돌.

관련 보조 PR:

- PR #1339: `fix(studio): 빈 글머리표 줄 caret 위치 보정 (squash 통합, #1331/#1329)`
- 상태: closed/unmerged
- PR #1339의 기능 코드 2파일은 PR #1331과 동일하다.
- PR #1339는 `mydocs/` task 문서들을 제외하고 기능 코드와 리뷰 문서만 포함했다.

review thread:

- PR #1331 unresolved review thread 없음.

## 4. 로컬 검증

검증 방식:

- `local/devel`에서 임시 브랜치 `local/pr1331-merge-test` 생성.
- PR #1339의 기능 커밋 `ecda2ace`를 cherry-pick.
- 원작성자 author는 `postmelee`로 보존됨.
- 기능 커밋은 PR #1331의 `src/document_core/queries/cursor_rect.rs`, `tests/issue_1329_bullet_caret.rs` 변경과 동일함.

통과:

- `cargo fmt --all -- --check`
- `cargo test --test issue_1329_bullet_caret`
- `cargo test --test issue_1308_forced_break_hanging_indent`
- `cargo test --lib`
- `cargo clippy --all-targets -- -D warnings`

주요 결과:

- `issue_1329_bullet_caret`: 3 passed
- `issue_1308_forced_break_hanging_indent`: 8 passed
- `cargo test --lib`: 1615 passed, 0 failed, 6 ignored

## 5. 리스크 평가

- 변경 지점이 cursor rect fallback 경로라 caret UX에는 직접 영향이 있다.
- list 문단 판정은 ParaShape `HeadType`에 제한되어 있어 일반 빈 문단 회귀 위험은 낮다.
- `char_start: None` TextRun 전체를 marker로 취급하지 않고 list 문단 + field marker none 조건으로 제한한 점은 적절하다.
- 셀/중첩 표 내부 list 문단 cursor rect는 이번 PR 범위 밖이다. 필요하면 별도 이슈로 분리한다.
- PR #1330으로 분리된 marker/caret 크기 변화는 이번 PR의 완료 조건이 아니다.

## 6. 권고

**수용 권고**.

사유:

- 사용자에게 보이는 caret 위치와 실제 입력 위치를 일치시키는 UX 개선이다.
- 기능 커밋은 최신 `devel`에 충돌 없이 적용된다.
- 원 PR은 문서 충돌 때문에 직접 merge하기 어렵지만, 기능 코드만 분리하면 검증이 통과한다.
- 기존 PR #1339의 기능 커밋은 원작성자 author를 보존하고 있으며, 기능 diff는 PR #1331과 동일하다.

권장 절차:

1. 최신 `local/devel`에 기능 커밋만 maintainer-side로 통합한다.
   - 후보 커밋: `ecda2ace` 또는 동일 diff 재적용.
   - `mydocs/` task 문서 7개는 통합하지 않는다.
2. 통합 후 원격 `devel`에 push한다.
3. PR #1331에 메인테이너 코멘트를 남긴다.
4. PR #1331은 기능 통합 완료 사유로 close 처리한다.
5. 이슈 #1329는 통합 후 close 처리한다.
6. #1330은 후속 이슈로 유지한다.
7. 리뷰 문서를 archive로 이동하고 `mydocs/orders/20260608.md`에 처리 내역을 기록한다.

## 7. 처리 결과

승인 후 처리:

- 기능 커밋만 최신 `local/devel`에 cherry-pick.
- 통합 커밋: `dce8b69b` (`fix(studio): 빈 글머리표 줄 caret 위치 보정 (closes #1329)`)
- `origin/devel` push 완료.
- PR #1331 메인테이너 코멘트 작성: https://github.com/edwardkim/rhwp/pull/1331#issuecomment-4648944244
- PR #1331 close 완료.
- 이슈 #1329 close 완료.
- #1330은 후속 이슈로 유지.

통합 후 재확인:

- `cargo fmt --all -- --check`
- `cargo test --test issue_1329_bullet_caret`
- `cargo test --test issue_1308_forced_break_hanging_indent`

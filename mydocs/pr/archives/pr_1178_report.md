# PR #1178 처리 보고서 — task 1139: 수식/미주 렌더링 정합 보정 및 미주 입력 지원

- **작성일**: 2026-05-31
- **PR**: #1178 → **MERGED** (devel, 머지커밋 `2862206f`, 머지시각 09:48 UTC)
- **컨트리뷰터**: @jangster77 (Taesup Jang)
- **연결 이슈**: #1139 → **CLOSED** (Closes #1139 자동 클로즈)
- **판단**: **머지** ✅ (작업지시자 결정)

## 처리 경위 (2단계 보류 후 머지)

본 PR 은 한 번 **보류**되었다가 컨트리뷰터의 rebase 대응으로 재개되어 머지된 사례다.

1. **1차 검토 (보류)** — 최초 제출 시 PR #1177(표+picture, `cc8dee68`)과 동일 picture/layout
   코드 의미 충돌(4파일 8 hunk), CONFLICTING/DIRTY. 직접 결합 시도했으나 두 레이아웃
   알고리즘 깊은 통합 난도로 abort, devel 무오염 유지. @jangster77 에 **rebase 요청** 코멘트.
2. **재검토 (2026-05-31)** — 컨트리뷰터가 rebase 응답 완료. merge-base = devel HEAD,
   **MERGEABLE/CLEAN** 전환. CI 전체 green. 규모 과대(99파일)로 한 차례 더 보류 후
   **작업지시자가 "이대로 머지" 결정** → 머지 진행.

## 변경 요약 (99 파일, +11234 / −653)

| 영역 | 변경 |
|------|------|
| `src/renderer/typeset.rs` (+948) | 수식/미주/조판 정합 핵심 보정 |
| `src/document_core/commands/object_ops.rs` (+1326) | 그림/개체 편집 경로 |
| `src/renderer/layout.rs` (+564) / `paragraph_layout.rs` (+502) | 레이아웃 정합 |
| `src/document_core/queries/cursor_rect.rs` (+589) | 커서 위치 계산 (미주 입력) |
| `src/renderer/height_cursor.rs` (+336) / `footnote_ops.rs` (+194) | 높이/미주 |
| `src/wasm_api.rs` (+176) / `wasm_api/tests.rs` | 미주 입력 API |
| rhwp-studio: `endnote-shape-dialog.ts`(+312, 신규), `equation-props-dialog.ts`(+486, 신규), `input-handler.ts`, `insert.ts`, `cursor.ts`, 도구상자 CSS | 미주 삽입/미주 모양 대화상자/주석 편집 도구상자/닫기 동작 |
| `tests/issue_1139_inline_picture_duplicate.rs` (+1096, 신규) | 인라인 그림 중복 회귀 |
| samples 4개 HWP + 문서(plans/report/working stage1~31) | 샘플·작업 문서 |

## 검증 결과 (로컬)

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| fmt | `cargo fmt --all --check` | ✅ clean |
| build | `cargo build` | ✅ Finished (rhwp v0.7.13) |
| 전체 테스트 | `cargo test --tests` | ✅ 0 failed |
| WASM | `docker compose ... wasm` | ✅ pkg 빌드 완료 |
| CI(PR) | Build&Test / Canvas diff / CodeQL / Analyze ×3 | ✅ 전부 SUCCESS (head `79846863`) |

## 처리 절차

1. PR 재검토 — rebase 해소 확인(MERGEABLE/CLEAN, merge-base = devel HEAD), CI green 확인.
2. 작업지시자 머지 결정.
3. head BEHIND(직전 orders 커밋으로 인해) → GitHub UI 머지 불가 → 메인테이너 로컬 `--no-ff`
   머지 + 로컬 검증(fmt/build/test) + push(`dd3e348e..2862206f`). push 후 PR 자동 MERGED.
4. WASM Docker 빌드 → pkg 갱신.
5. PR 코멘트 + 보고서 + orders 갱신.

## 후속 검증 (메인테이너 수동)

- **시각 검증 / 회귀 검증은 메인테이너가 수동 진행** (작업지시자 지시).
- PR 본문 명시 제한 사항: 미주 처리는 특정 시험지 케이스 한정, 복잡 다중 미주/임의 편집/
  저장·재열기 round-trip 미보장. 후속 이슈에서 anchor·번호 재계산·편집 흐름·round-trip 확장 검증 필요.

## 비고

- @jangster77 의 task 1139 누적 작업(Stage1~31). 미주 입력은 rhwp-studio 신규 기능.
- 이 PR 의 보류→rebase→머지 경로는 "충돌 시 직접 결합 대신 rebase 요청" 절차가 유효함을 보인 사례.

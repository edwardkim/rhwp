---
kind: pr-review-implementation
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# open CI-green PR #6245/#6247/#6248/#6249/#6250/#6254/#6259 통합 검토 구현 기록

## 기준과 포함 범위

- 통합 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@5645e1f5b`
- 포함 PR:
  - #6245 `9c53276c37c8` - #6194 머리 표 행 높이 과대 계상 보정
  - #6247 `6d3149551ea6` - `CellContext` 빈 경로 panic 방어
  - #6248 `d84f1e8a4fe1` - #6179 오른쪽 탭 뒤 TAC 개체 정렬
  - #6249 `e11ab9e89b07` - CIRCLED/GANADA 번호 포맷 OOB 방어
  - #6250 `9fc79fdd477b` - font/border 인덱스 OOB 방어
  - #6254 `00cac13820e3` - #6173 오른쪽 정렬 말미 공백 판정
  - #6259 `87447c260737` - #6167 TAC 표 자기 줄 leading 제거
- 제외:
  - #5953, #6059: draft
  - #6073, #6246, #6252: CI `Build & Test` 실패
  - #6083: 실패 check는 없지만 `DIRTY`이며, 기존 메인터너 코멘트에서 현 상태 통합 보류/재작업을
    요청했다. 따라서 이번 CI-green 통합 대상에서 제외하고 `pr_6083_review.md`의 보류 판단을 유지한다.

## 체리픽과 최신성 확인

- 원 PR head를 `upstream/prNNNN-head`로 fetch한 뒤 PR 번호 순서로 `git cherry-pick -x` 적용했다.
- 적용 후 `git log --cherry-pick --right-only --oneline HEAD...refs/remotes/upstream/prNNNN-head`로
  #6245/#6247/#6248/#6249/#6250/#6254/#6259 각각 원 PR head에 남은 commit이 없음을 확인했다.
- 2026-08-28 재확인 시 모든 포함 PR은 non-draft, `CLEAN`, 실패 check 0건, 진행 check 0건이었다.
- #6259는 통합 검토 도중 새로 조건을 만족하는 open PR로 확인되어 추가 포함했다.

## 메인터너 보정

- #6245의 `ladder_pushed_following_line`이 모든 후속 문단을 `any()`로 훑으면, 여러 문단 뒤 누적
  `vpos`가 우연히 큰 값을 갖는 경우에도 자리차지 개체 흡수 증거로 오인할 수 있다.
- 통합 브랜치에서 `src/renderer/height_measurer.rs`를 보정해 바로 뒤의 실제 `lineSeg` 1개만
  확인하도록 좁혔다.
- 보정 커밋: `10ce6b419 fix(renderer): 사다리 흡수 판정 범위를 좁힌다`

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `node scripts/rust-unit-test-tiers.mjs --check` | 통과, 4,221 tests / 299 modules |
| `node scripts/rust-test-suite-manifest.mjs --prepare && --check` | 통과, 991 sources / 4,426 static test attrs / 최소 6,559 cases |
| #6167 focused nextest | 통과, 1 pass |
| #6173 focused nextest | 통과, 1 pass |
| #6179 focused nextest | 통과, 1 pass |
| #6194 focused nextest | 통과, 1 pass |
| `cargo test --locked --lib test_format_number --target-dir target/pr-review` | 통과, 5 pass |
| `cargo test --locked --lib index_matches_legacy_linear_scan_exhaustively --target-dir target/pr-review` | 통과, 1 pass |
| `cargo test --locked --lib degenerate_inferred_row_uses_base_grid_instead_of_expanding_last_cell --target-dir target/pr-review` | 통과, 1 pass |
| `cargo test --locked --lib cursor_rect --target-dir target/pr-review` | 통과, 16 pass / 5 ignored |
| `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` | 통과, 48.76s |
| 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 통과, 8,477 passed / 43 skipped / 10 slow, 944.064s |
| `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg` | 통과, 9m 05s |
| Native Skia lib | 통과, 3,946 pass / 13 ignored + contracts 15 pass + ooxml-chart 165 pass + password 2 pass |
| Native Skia `issue_2225_missing_picture_placeholder` | 통과, 2 pass / 123 skipped |
| Native Skia `render_p37_direct_pdf_export` | 통과, 4 pass / 133 skipped |
| `git diff --check upstream/devel...HEAD` | 통과 |

## 시각 증적

- #6245/#6194: `mydocs/report/header-row-picture-height-6194/after_p1.png`와 `oracle_p1.png`를 직접
  확인했다. 머리 표 높이와 아래 표 분리가 기준과 가까워졌고 겹침이 보이지 않는다.
- #6247: `mydocs/report/bug-layout-empty-path/after.png`를 확인했다. 빈 `CellContext` 경로는 panic
  대신 `Option` 흐름으로 빠진다.
- #6248/#6179: `mydocs/report/right-tab-tac-object-6179/p1_footer_after.png`를 확인했다. 오른쪽
  꼬리말 로고가 용지 밖으로 나가지 않는다.
- #6249: `mydocs/report/bug-circled/README.md`와 before/after SVG 증적을 확인했다. 방어적 OOB
  수정이라 정상 문서 출력 변화는 기대하지 않는다.
- #6250: `mydocs/report/bug-font-border/after.png`를 확인했다. font/border OOB 방어 성격과 맞고
  눈에 띄는 회귀는 없었다.
- #6254/#6173: `mydocs/report/right-align-inline-object-space-6173/p2_textbox_after.png`를 확인했다.
  글상자 안 두 로고가 우단 안에 배치된다.
- #6259/#6167: `mydocs/report/leading-space-tac-table-6167/p38_table_after.png`를 확인했다. 표가
  본문 좌단 기준으로 돌아오고 오른쪽 열 잘림이 보이지 않는다.

## 결론

#6245/#6247/#6248/#6249/#6250/#6254/#6259는 통합 수용 권고다. #6245에는 메인터너 보정을 한 건
포함했고, 최종 통합 head 기준으로 renderer/layout 필수 로컬 검증과 시각 증적 확인을 완료했다.
#6083은 CI check 실패가 없더라도 기존 메인터너 보류 코멘트와 current `DIRTY` 상태 때문에 이번
통합 대상에서 제외한다.

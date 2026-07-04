# PR #1683 검토 - 페이지네이션 fit에서 각주 영역 예약

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| PR | [#1683](https://github.com/edwardkim/rhwp/pull/1683) |
| 제목 | `[경험적 페이지네이션] 페이지네이션 fit에서 각주 영역 예약` |
| 작성자 | `humdrum00001010` |
| base <- head | `devel` <- `codex/pr1573-footnote-fit` |
| 문서 작성 시점 참고값 | `CONFLICTING` / `DIRTY`, draft 아님, maintainer 수정 가능 |
| 규모 | 6 파일, +852/-138 |
| 원 PR 커밋 | `04fd6f5b1438`, `bb8c3deb6df0` |
| 로컬 적용 커밋 | `fe6de3ef0` (`04fd6f5b1438`만 `git cherry-pick -x`) |

## 2. 변경 범위

pagination fit 계산 중 각주 영역을 예약하도록 하는 경험적 페이지네이션 보정 PR이다.

- footnote shape 정보를 `rendering`, `pagination`, `typeset` 경로로 전달한다.
- body content fit 계산에서 separator와 각주 사이 margin을 반영한다.
- `src/renderer/pagination.rs`, `src/renderer/pagination/engine.rs`, `src/renderer/pagination/state.rs`, `src/renderer/pagination/tests.rs`, `src/renderer/typeset.rs`가 변경된다.
- 원 PR의 두 번째 커밋 `bb8c3deb6df0`은 merge commit 성격이라 로컬 일괄 브랜치에는 적용하지 않았다.

## 3. #1691 이후 충돌 처리

GitHub 기준 이 PR은 `CONFLICTING` / `DIRTY`였다. 로컬에서는 `upstream/devel`에 이미 merge된 [#1691](https://github.com/edwardkim/rhwp/pull/1691) 이후 상태 위에 `04fd6f5b1438`을 체리픽하면서 `src/renderer/typeset.rs` 충돌이 발생했다.

충돌 해결 방향:

- #1691의 동적 `layout_drift_safety_px` 계산은 유지했다.
- #1691의 overlay shape fit 예외와 TAC table bottom fit 예외를 유지했다.
- #1683의 footnote reserve 이후 saved vpos / fit 판정 예외를 추가로 살렸다.
- 양쪽 모두 page count와 fit 판정에 관여하므로, 해결 결과는 반드시 실제 PDF 기준 전체 페이지 수 검증 대상이다.

## 4. 검토 의견

각주 영역 예약은 페이지 수와 body overflow를 직접 바꾸는 고위험 변경이다. 특히 #1691도 page count 보정과 관련된 `typeset.rs` 변경을 포함하므로, 두 변경을 합친 로컬 해소 결과가 원 PR의 의도와 동일하다고 단정하면 안 된다.

이 PR은 PR 본문도 경험적 페이지네이션 보정이라고 분류한다. canonical/PDF 대조에서 "모든 페이지 수가 맞아야 한다"는 현재 작업 목표와 직접 연결되므로, full sample set의 page count가 merge gate다.

`pagination/tests.rs`가 추가된 점은 긍정적이다. 이번 검토에서는 각주/미주가 있는 실제 샘플 visual sweep과 2025 행정업무운영 편람 HWP/HWPX page count를 함께 확인했다. 이후 실패가 발생하면 #1691과 #1683의 fit 예외가 서로 완화/강화하는 부분부터 살핀다.

## 5. 로컬 적용 상태

`upstream/devel` 기준 로컬 일괄 검토 브랜치 `local/humdrum-pr-batch-review`에 원 코드 커밋 1개를 `-x`로 체리픽했다. `src/renderer/typeset.rs` 충돌은 위 기준으로 해결했고, `git cherry-pick --continue`까지 완료했다.

원 PR은 개별 merge 불가 상태다. 일괄 브랜치에서 충돌 해결과 전체 검증을 통과하지 않으면 merge 후보로 볼 수 없다.

## 6. 검증 상태

- 완료: `cargo fmt --check`
- 완료: `git diff --check`
- 완료: `cargo build --release`
- 완료: `cargo test --release --lib` (2037 passed, 0 failed, 7 ignored)
- 완료: `cargo test --profile release-test --tests` (통합 테스트 전체 통과)
- 완료: `cargo clippy --all-targets -- -D warnings` (최초 공통 검증 18m 25s warning 0, TIFF 보강 후 최종 재실행 17m 28s warning 0)
- 완료: `cargo test --doc` (0 passed, 0 failed, 1 ignored)
- 완료: `cargo test --test svg_snapshot` (8 passed)
- 완료: `cd rhwp-studio && npx tsc --noEmit`
- 완료: `cd rhwp-studio && npm test` (153 passed)
- 완료: `wasm-pack build --target web --out-dir pkg` (1m 25s)
- 중단/무효: `cargo check --all-targets --message-format=short`는 workflow 문서에 없는 명령이라 중단했으며 검증 결과로 기록하지 않는다.

### 6.1 PR 내용별 targeted 검증

| 검증 대상 | 명령/테스트 | 결과 |
|-----------|-------------|------|
| 각주 separator와 note 사이 여백을 section shape 기준으로 예약 | `cargo test --profile release-test --lib footnote_area_reserve_uses_section_shape_metrics` | 통과 |
| 저장된 single-line 문단이 body 하단에 맞으면 현재 페이지 유지 | `cargo test --profile release-test --lib saved_single_line_at_body_bottom_stays_on_current_page` | 통과 |
| vpos reset 직전 2줄 tail이 visible bottom에 맞으면 현재 페이지 유지 | `cargo test --profile release-test --lib two_line_tail_before_vpos_reset_stays_on_current_page_when_visible_bottom_fits` | 통과 |
| 저장된 TAC table line이 body 하단에 맞으면 현재 페이지 유지 | `cargo test --profile release-test --lib saved_tac_table_line_at_body_bottom_stays_on_current_page` | 통과 |
| page-bottom text box fit에서 advance overflow가 있어도 실제 line bottom이 맞으면 유지 | `cargo test --profile release-test --lib page_bottom_text_box_fit_keeps_line_even_when_advance_overflows` | 통과 |
| 기존 page count 회귀 가드 | `cargo test --profile release-test --tests page_count` | 통과, 필터 매칭 26개 통과 |

`page_count` 필터 검증에는 `exam_eng_page_count_after_359_fix`, `issue_1035_alignment`, `issue_1086`, `issue_1105`, `issue_1139_inline_picture_duplicate`, `issue_1624_footer_overpush_matches_hangul_page_count`, `rowbreak_final_pages_match_hancom_pdf_page_count` 계열 테스트가 포함됐다.

### 6.2 시각/PDF 검증

2026-06-30 로컬 일괄 브랜치에서 각주/페이지네이션 영향 범위를 별도 확인했다.

- `cargo test --test svg_snapshot`: 8개 golden snapshot 통과
- `cargo test --profile release-test --test visual_roundtrip_baseline`: 3개 visual roundtrip baseline 통과
- `python3 scripts/task1274_visual_sweep.py --target all`: 15개 target 모두 SVG/PDF page count 일치, page count mismatch 0건
- `pdfinfo 'pdf/2025 행정업무운영 편람(최종)-2024.pdf'`: 383쪽
- `./target/release/rhwp info 'samples/2025 행정업무운영 편람(최종).hwp'`: 383쪽
- `./target/release/rhwp info 'samples/2025 행정업무운영 편람(최종).hwpx'`: 383쪽

`visual_sweep` 자동 후보 요약:

| target | SVG/PDF pages | flagged | 주요 후보 |
|--------|---------------|---------|-----------|
| `2022-09` | 23/23 | 1 | `eq=[23]`, `order=[23]` |
| `2024-09-below20` | 23/23 | 1 | `eq=[23]`, `order=[23]` |
| `2024-09-between20` | 24/24 | 1 | `eq=[24]`, `order=[24]` |
| `2024-09-below20-above20` | 23/23 | 4 | `eq=[23]`, `order=[23]`, `tail=[19,20]`, `question=[19,20,22]`, `large=[19,20,22]` |
| `2024-11-practice-above0-between20-below2` | 22/22 | 4 | `tail=[12,17,20]`, `question=[17,21]`, `large=[17,20]` |

나머지 10개 target은 flagged 0이다. 자동 후보가 남은 target도 페이지 수는 모두 PDF와 일치한다. 후보 페이지는 비교 이미지 기반 수동 시각 판정 대상으로 남기며, 이번 PR의 merge gate인 page count mismatch는 발견되지 않았다.

## 7. 잠정 판단

조건부 수용 후보. #1691 이후 `typeset.rs` 충돌은 로컬에서 해소했고, PR 내용과 직접 맞닿은 각주 reserve / saved vpos / page bottom fit / page count 필터 검증은 통과했다. 단독 snapshot, visual roundtrip, task1274 visual sweep, 2025 행정업무운영 편람 HWP/HWPX 383쪽 확인까지 page count mismatch는 발견되지 않았다. 다만 원 PR은 GitHub 기준 개별 merge 불가 상태였고, 자동 sweep 후보가 남은 페이지는 수동 시각 판정 대상으로 분리한다.

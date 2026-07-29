# planet6897 PR #3503·#3520·#3525·#3527·#3530·#3535 통합 구현 기록

## 목적과 기준점

여섯 외부 contributor PR을 최신 `upstream/devel`에 누적 검토한다. 검토 브랜치는
`review/planet6897-20260729`이며, 2026-07-29에 최신 `upstream/devel`
`d0377df6f0749c920f870a2cd037b5d5d1471f82` 위로 rebase했다. 원본 PR마다 별도 review 문서를
두고, 이 문서는 적용 순서·collaborator 보정·rollback 범위를 한 곳에 남긴다.

## 적용 순서

| 순서 | 원 PR head | 통합 commit | 내용 | 비고 |
|---|---|---|---|---|
| 1 | #3503 `85f813d` | `1f8b471e0` | 미주 실제 공백 보존 | 원본 단일 commit |
| 2 | #3520 `2005086` | `c8db0665f` | text-surface PUA 치환 공유 | 원본 단일 commit |
| 3 | #3525 `a77e171` | `f83e8ee1a` | HWP3 bookmark control | `info_buf` 충돌 해소 |
| 4 | #3527 `59f7b85` | `5d7f3473b` | verify corpus ratchet | 기능 commit |
| 5 | #3527 `257b8ce` | `7f93ad3cf` | ratchet 문서 | 문서 commit |
| 6 | #3530 `7c81725` | `e25f3c376` | nested caption direct-level | #3527 적층분은 제외 |
| 7 | #3535 `e910c1e` | `5eb0d447b` | char_count 규약 | #3510 중복을 유지 |

추가 collaborator commit은 다음과 같다.

| commit | 이유 | rollback |
|---|---|---|
| `639e6250d` | 새 CLI tests의 nextest archive binary 탐색 | 테스트 harness만 되돌림 |
| `699d0fe32` | #3525로 해결된 HWP3 bookmark expected failure 제거 | 래칫과 HWP3 assertion 함께 되돌림 |
| `7ab7e137a` | `--page` raster 제한·회귀 테스트·대표 PNG | visual helper/doc/asset만 되돌림 |

## 검증 기록

최신 `d0377df6` rebase 뒤 검토 전용 target
`target/review-planet6897-20260729`에서 완료한 최종 local gate:

- `cargo fmt --all -- --check`: success.
- focused regressions: #3385 3, #3494 2, #3495 2, #3528 1, HWP3 bookmark 2 passed.
- `convert_verify_corpus_ratchet`: partition 4/4 passed (13.33s).
- `cargo test --profile release-test --tests`: 365 `test result: ok` summaries; 대표 lib
  summary 3,019 passed / 0 failed / 7 ignored, 최종 visual baseline 3 passed; failure/error summary 0.
- `cargo clippy --all-targets -- -D warnings`: success.
- `cargo build`: success.
- Native Skia `cargo test --profile release-test --features native-skia --lib skia`:
  58 passed / 0 failed. 이전 통합 확인은 missing-picture 2, direct PDF 4 passed.
- `wasm-pack build --target web --out-dir pkg`: success (wasm-opt 포함).
- visual sweep (`exam_kor`, page 1): SVG/render-tree 20/20, raster PNG rhwp/PDF 각 1장,
  compare/overlay/review 1/1, automatic flags 0, pixel match 90.103%,
  visual_accuracy_proxy 16.353%.

이후 통합 PR의 새 head는 code/test 보정을 포함하므로 review-only fast-pass가 아니라 full CI를
완료까지 기다린다.

## 승인·CI·후속 처리

작업지시자는 2026-07-29에 통합 PR 생성, CI 완료 대기, merge 및 post-merge 처리를 자동 승인했다.

1. 통합 브랜치를 원본 저장소 임시 head로 push하고 `devel` 대상 PR을 만든다.
2. 최신 integration head의 required CI를 종료 summary까지 확인한다. code/test 보정이 있으므로
   review-only fast-pass를 쓰지 않는다.
3. 모든 required CI가 성공하면 squash merge한다.
4. merge SHA와 devel 검증을 확인한 뒤 원본 PR #3503, #3520, #3525, #3527, #3530, #3535에
   결과를 남기고 close하며 연결 이슈 #3495, #3385, #3524, #3505, #3528, #3494를 close한다.
5. `post_merge.md` 7.7.1에 따라 Cargo/Rust 작업이 없음을 확인한 뒤 정확히
   `target/review-planet6897-20260729`만 정리하고, 로컬 review branch/worktree와 임시 remote
   branch를 정리한다.

원본 PR를 직접 rewrite하거나 contributor fork에 push하지 않는다. rollback은 merge 이전에는 통합
PR close와 임시 branch 삭제, merge 이후에는 해당 통합 commit을 revert하는 범위로 한정한다.

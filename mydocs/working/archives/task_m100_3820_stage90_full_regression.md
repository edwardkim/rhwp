---
kind: verification
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 90 — HWP/HWPX incremental reflow 전체 회귀

## 목적

Stage 89 commit `efc0f6cd9`는 Hancom 2020 PDF가 확인한 native HWP 56회 / HWPX 61회
line-flow 경계를 각각 보존한다. Stage 89의 focused WASM gate와 `issue_2430` integration
gate는 통과했지만, 다른 프로세스의 전체 Cargo 실행은 final summary를 회수하지 못했다.
따라서 프로세스 종료만으로 성공을 가정하지 않고 이 stage에서 전체 integration summary를
직접 확인한다.

## 실행 규약

`mydocs/manual/pr_review/local_validation.md`의 renderer 변경 gate와 로컬 환경 지침에 따라,
공유 target을 지우지 않고 `CARGO_TARGET_DIR=target/pr-review`,
`CARGO_INCREMENTAL=0`을 사용한다. 실행 전 Cargo/Rust 작업을 확인하고, 실행 중 출력 공백은
성공·중단 근거가 아니므로 final exit code와 test summary가 나올 때까지 대기한다.

HWPCTRL은 이 native-layout 변경의 visual oracle가 아니다. Stage 89에서 확인한 대로,
public API/COM fixture를 바꾸지 않았으므로 이 stage의 primary gate는 release-test이며,
HWPCTRL contract gate는 WASM build 뒤 별도 compatibility 확인이 필요할 때만 수행한다.

## 예정 명령

```sh
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review \
  cargo test --profile release-test --tests
```

성공 시 full summary와 focused gate의 결과를 이 문서에 기록한다. 실패 시 실패 test의
정확한 출력과 영향 fixture만 다음 분석 stage로 이월하며 baseline을 임의로 변경하지 않는다.

## 첫 전체 결과와 추가 분석

이 stage의 전체 실행은 명시적으로 `3374 passed; 1 failed; 13 ignored`로 끝났다. 실패 test는
`document_core::commands::text_editing::tests::delete_text_in_cell_by_path_reflows_depth1_cell_line_segs`
하나이며, 40자 중 39자를 삭제해 한 글자만 남긴 폭 200 HWPUNIT 셀의 `LINE_SEG`가 두 줄이었다.

현재 worktree에는 `line_height > 0`인 저장 LINE_SEG만 native prefix candidate로 삼는 중간 guard가
있다. 이는 `DocumentCore::new_empty()`의 기본 HWP 출처가 합성 test/new-document line segment를
native HWP boundary로 오인하지 않게 한다. 그러나 실제 HWP에서 범위 삭제가 서로 다른 line start를
같은 UTF-16 offset으로 접을 수 있으므로, line height만으로는 충분하지 않다.

수용 조건은 candidate start가 0에서 시작하고 **엄격히 증가**하는 것이다. duplicate 또는 역행
start는 prefix를 보존하지 않고 full reflow한다. 이 guard는 Stage 89 fixture의 정상 HWP
`[0,44,84,122]`에는 적용되며, HWPX의 full-reflow 경로는 그대로다. 변경 뒤에는 규정상
`issue_2430_cell_rewrap_threshold`를 먼저, 그 다음 #2755 단독 test와 Stage 89 targeted gate,
마지막으로 전체 release-test를 실행한다.

## 보정 뒤 focused 결과

보정은 `line_height > 0` 조건에 `first.text_start == 0` 및 인접 start의 strict-increasing
조건을 결합했다. #2755 fixture도 positive line dimension을 사용하도록 해, synthetic zero-height
문단만 우회한 것이 아니라 실제 HWP와 같은 LINE_SEG가 `[0,20] → [0,0]`으로 붕괴한 상태를
재현한다.

| gate | 결과 |
| --- | --- |
| `issue_2430_cell_rewrap_threshold` | `2 passed; 0 failed` |
| `delete_text_in_cell_by_path_reflows_depth1_cell_line_segs` | `1 passed; 0 failed` |
| `issue2214_scoped_cache_coherence_preserves_transient_pagination` | 통과 (HWP 56 / HWPX 61, 115 fragments) |
| `issue2424_resumable_pagination_commits_only_after_final_fragment` | 통과 |
| `issue2424_resumable_delete_commits_only_after_final_fragment` | 통과 |
| `issue2424_new_edit_stales_old_job_and_sync_flush_restarts_latest_revision` | 통과 |
| `issue3137_focused_cell_geometry_matches_exact_rect` | 통과 |

이 결과는 full suite를 대체하지 않는다.

## 두 번째 전체 결과와 이월

strict-boundary 보정 뒤 전체 `cargo test --profile release-test --tests`는 library 단계에서
`3375 passed; 0 failed; 13 ignored`을 기록했다. 이후 integration test
`issue_1939_hwp5_origin_hwpx_strict_render_diff_is_stable`가
`samples/issue1891/76076_regulatory_analysis.hwpx`의 HWP5-origin HWPX 왕복에서
`maxDisp=662.95px` 및 구조 불일치 5쪽(33, 49, 50, 80, 81)을 보고해 전체 command는 exit
code 101로 끝났다.

82쪽 page-count가 같은 것은 확인됐지만 strict render gate의 대체 근거가 아니다. baseline,
fixture, PDF 정답을 바꾸지 않았으며, marker HWPX가 원 HWP5의 RowBreak source-owner contract를
누락한 이유는 Stage 91에서 분리해 보정한다.

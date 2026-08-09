---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 89 — HWP/HWPX 증분 셀 재래핑과 cache 안전성

## 목표와 범위

Stage 86의 전체 회귀 실패 다섯 건은 모두
`issue1949_giant_cell_nested_tables_perf`의 동일 셀 문단(section 0 / parent paragraph 0 /
table control 2 / cell 2 / cell paragraph 5)에서 발생했다. 이 stage는 해당 문단의 native
edit 후 `LINE_SEG` 재래핑과 focused page-tree cache의 일관성만 다룬다. 표 렌더링, HWPCTRL
공개 API, 이미 검증된 short-child content-box 보정의 범위는 바꾸지 않는다.

### HWPCTRL 문서의 적용 범위

구현 판단 전에
[`웹한글컨트롤 호환 개발 가이드`](../manual/webhwpctrl_compat_development.md)와
[`hwpctrl compatibility harness`](../../tools/hwpctrl_compat/README.md)를 대조했다.
두 문서는 Windows 한글 COM의 API/`SaveAs` 행동 정답지와 macOS·Linux WASM 자체 검증을
명확히 구분하며, 신규 API fixture는 Windows live Oracle 없이 갱신하지 않도록 규정한다.

이번 변경은 native editor의 내부 line-layout/cache 수정이며 public `@rhwp/hwpctrl` API,
scenario, ledger, COM fixture를 바꾸지 않는다. 그러므로 HWPCTRL fixture를 test 통과용으로
고치지 않았고, 시각 레이아웃의 정답지는 계속 Hancom 2020 PDF로 사용한다. HWPCTRL은
다음 WASM/compatibility 검증 단계에서 공개 API 회귀 여부를 확인하는 보조 gate로만 쓴다.

## 정답지로 고정한 경계

기존 test는 HWP와 HWPX가 모두 숫자 56개를 끝에 삽입하면 넷째 줄에서 다섯째 줄로 전환한다고
가정했다. 소스 저장 LINE_SEG와 fresh reflow 중 어느 쪽이 한컴 동작인지는 그 가정만으로 판단할
수 없으므로, 두 형식을 각각 HWP adapter 저장 뒤 한컴 2020 PDF로 직접 확인했다.

| 입력 형식 | 다섯째 줄 전환 | 한컴 2020 PDF 관측 |
| --- | ---: | --- |
| HWP | 56개 | `56`에서 fifth line 생성 |
| HWPX | 61개 | `56`은 fourth line, `61`에서 fifth line 생성 |

재현 PDF는 `pdf/task_m100_3820_stage86_wasm_boundary_oracle/`에 보관한다. HWPX 56/61의
SHA-256은 각각 `09ff2091…010e46d4`, `05cec394…ccb45f3a1`이다. 따라서 공통 56이라는
기존 gate가 stale이었으며, HWPX를 HWP와 같은 prefix 보존 경로로 억지로 맞추는 것은
정답지에 반한다.

## 원인

대상 HWP 문단의 저장 시작점은 `[0, 44, 84, 122]`이다. 전체 reflow는 앞선 줄을
`[0, 45, 87, 125]`로 다시 계산해 마지막 줄 앞부분의 가용 폭을 늘린다. 그래서 한컴 HWP가
56개에서 줄을 하나 더 만드는 것과 달리 rhwp는 61개까지 네 줄로 남았다.

HWP의 증분 편집은 저장된 prefix boundary를 유지하고 **수정이 닿은 suffix부터** 재래핑해야
한다. 반대로 HWPX 저장본은 그 자체의 editable line boundary가 달라 기존 full reflow를
유지해야 한다. `source_format == Hwp`를 조건으로 둔 이유가 이것이다.

## 구현 계약

1. `reflow_line_segs_after_cell_text_edit()`는 native HWP의 유효한 token boundary에서만
   prefix를 유지한다. 첫 줄 편집, inline control, 유효하지 않은 저장 boundary는 기존 full
   reflow로 fallback한다.
2. 문단 끝 삭제에는 삭제 뒤 `char_offsets[text_len]` 항목이 없으므로 UTF-16 text end를
   사용한다. 마지막 실제 줄은 재래핑에 포함해 5→4 shrink를 놓치지 않으며, empty-final-line
   start도 prefix에 남지 않게 한다.
3. HWPX의 layout은 full reflow지만, 결과가 same-line tail update이면 cached page tree를
   안전하게 patch할 수 있다. `focusedPageTreePatched=true`이면 동일 page index의 cache가 fresh
   tree와 같아야 하며, false이면 모든 cache slot을 invalidation해야 한다.

## 검증 결과

source 변경 뒤 첫 gate는 작업 규약대로 아래를 실행했고 명시적으로 2/2 통과했다.

```sh
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2430_cell_rewrap_threshold -- --nocapture
```

그 다음 현재 source로 아래 targeted test를 다시 실행했다.

| test | 결과 | 검증한 계약 |
| --- | --- | --- |
| `issue2214_scoped_cache_coherence_preserves_transient_pagination` | 통과 | HWP 56 / HWPX 61 경계 및 115 fragment continuity |
| `issue2424_resumable_pagination_commits_only_after_final_fragment` | 통과 | 전환 뒤 pagination이 끝 fragment에서만 commit |
| `issue2424_resumable_delete_commits_only_after_final_fragment` | 통과 | 경계 문자 삭제의 5→4 shrink와 full-pagination oracle 일치 |
| `issue2424_new_edit_stales_old_job_and_sync_flush_restarts_latest_revision` | 통과 | stale revision 취소와 최신 revision 재시작 |
| `issue3137_focused_cell_geometry_matches_exact_rect` | 통과 | patch면 fresh-tree equality, fallback이면 완전 invalidation과 cell bounds 보존 |

`issue3137`의 IME-normalized tail loop는 앞선 replace/backspace를 거쳐 pristine 56/61
경계를 더 이상 보장하지 않는다. 그래서 다음 flow boundary까지 실제 mutation을 추적하되,
그 전에는 cache patch, 경계에서는 cache invalidation만 허용한다. 이는 특정 stale count를
강제하는 대신 mutation/cache 안전성 자체를 검증하는 계약이다.

모든 targeted test는 새 lib-test binary
`target/pr-review/release-test/deps/rhwp-68bd1ab6331deaaa`를 직접 `--exact` 실행해
명시적인 `1 passed; 0 failed` summary로 다시 확인했다. `git diff --check`도 통과했다.

전체 `cargo test --profile release-test --tests`는 아직 이 source revision으로 재실행하지
않았다. 다음 단계에서 장시간 실행을 중단하지 않고 최종 summary까지 확인한다.

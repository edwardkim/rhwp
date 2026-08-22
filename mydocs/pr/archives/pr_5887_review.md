---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5887 검토 - 공백 host 표와 중간 RIGHT tab 측정

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5887](https://github.com/edwardkim/rhwp/pull/5887) / [@planet6897](https://github.com/planet6897) |
| 관련 issue | [#5871](https://github.com/edwardkim/rhwp/issues/5871), [#5872](https://github.com/edwardkim/rhwp/issues/5872) |
| base / source head | `devel` / `265c956dcab03ca864c8d4f10aaac02cbca7690c` |
| 변경 규모 | 6 files, +172 / -0 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`, maintainerCanModify=true |
| 통합 반영 | `98ef38ed1`, `b3df19de5` |

## 범위와 검토

- 표 높이를 이미 담은 저장 줄상자가 있는 공백 host에서만 pre-text 이중 계상을 막는다.
- 줄 중간 RIGHT tab은 저장 width를 유지하고, 줄 끝 tab의 기존 page-number 경로는 보존한다.
- 두 조건 모두 넓은 "공백=무텍스트" 또는 모든 RIGHT tab 변경으로 확대하지 않는다.

## 검증과 위험

- 통합 candidate `4b28259bb`에서 전체 nextest **8,160 passed, 39 skipped**, clippy, native-Skia, WASM build를 통과했다.
- 같은 head의 GitHub Build & Test, archive build/shard, Lint, Native Skia, CodeQL Rust/JavaScript/Python, Canvas visual diff, Adapter inter-diff, Proptest roundtrip도 성공했다. WASM과 frontend unit은 변경 범위 정책에 따른 정상 skip이다.
- `samples/issue5871/ws_host_float_double_charge.hwp`와 `samples/issue5872/toc_midline_right_tab.hwpx`의 regression contract가 전체 suite에 포함된다.
- source가 사용한 한글 기준 자료는 저장소에 PDF로 보존돼 있지 않아 독립 PDF sweep은 하지 않았다. 따라서 해당 좌표 일치는 source evidence로 한정하고, 전체 typography fidelity 통과를 주장하지 않는다.

## 최종 판정

**수용 권고.** 저장 줄상자와 후속 tab 존재라는 좁은 조건이 기존 정상 형상을 보존한다. merge 전 PR #5889 최신 CI와 작업지시자 승인이 필요하다.

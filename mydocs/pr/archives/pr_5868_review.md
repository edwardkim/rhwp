---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5868 검토 - TAC 폭과 자리차지 표 제목 위치

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5868](https://github.com/edwardkim/rhwp/pull/5868) / [@planet6897](https://github.com/planet6897) |
| 관련 issue | [#5584](https://github.com/edwardkim/rhwp/issues/5584), [#5697](https://github.com/edwardkim/rhwp/issues/5697), [#5785](https://github.com/edwardkim/rhwp/issues/5785), [#5876](https://github.com/edwardkim/rhwp/issues/5876), [#5881](https://github.com/edwardkim/rhwp/issues/5881) |
| base / source head | `devel` / `f893d84c2ecbec4ea70ed3e45f25bee567bcd7eb` |
| 변경 규모 | 6 files, +201 / -6 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`, maintainerCanModify=true |
| 통합 반영 | `08baf8f2f`, `fea71a537`, `f4e3a0d31` |

## 범위와 검토

- TAC 표 인라인 판정은 불안정한 열 합 대신 선언 폭을 우선 사용한다.
- 저장 줄이 표 위에 있는 host만 defer 대상에서 빼 첫 조각 쪽에 제목을 그린다.
- 혼합 host 형상은 기존 defer 경로를 유지해 뒤 텍스트 손실 범위를 넓히지 않는다.

## 검증과 위험

- 통합 candidate `4b28259bb`에서 전체 nextest **8,160 passed, 39 skipped**, clippy, native-Skia, WASM build를 통과했다.
- 같은 head의 GitHub Build & Test, archive build/shard, Lint, Native Skia, CodeQL Rust/JavaScript/Python, Canvas visual diff, Adapter inter-diff, Proptest roundtrip도 성공했다. WASM과 frontend unit은 변경 범위 정책에 따른 정상 skip이다.
- source fixture `samples/issue5785/medal_cells_ws_host_inline.hwpx`, `samples/issue5584/float_host_title_above_table.hwpx`와 각각의 regression case가 통합 suite에 포함된다.
- 저장소에 대응 한글 PDF가 없어 이번 통합에서 독립 PDF sweep을 추가하지 않았다. source의 좌표 측정은 참고로만 수용하며, 전체 page-flow fidelity 완료를 주장하지 않는다.

## 최종 판정

**수용 권고.** 선언 폭과 저장 위치라는 좁은 판별자로 기존 경로를 보존한다. merge 전 PR #5889 최신 CI와 작업지시자 승인이 필요하다.

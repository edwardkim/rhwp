---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5856 검토 - 쪽 하단 부동 개체 clip 보정

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5856](https://github.com/edwardkim/rhwp/pull/5856) / [@kevin9327](https://github.com/kevin9327) |
| 관련 issue | [#5855](https://github.com/edwardkim/rhwp/issues/5855) |
| base / source head | `devel` / `1844c8cbc11eda8b123e9243a3ac178481b18b9b` |
| 변경 규모 | 4 files, +140 / -3 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`, maintainerCanModify=true |
| 통합 반영 | `1db8e68eb` (`review/open-prs-20260822`) |

## 범위와 검토

- body clip의 float 하한을 용지 하한으로 옮겨, 본문 아래 로고 띠가 clip으로 사라지지 않게 한다.
- 새 회귀는 손실을 재현하고, clip이 용지 밖으로 넓어지지 않는 가드를 함께 둔다.
- source에 저장소 fixture와 한글 2020 PDF, 전후 비교 PNG가 있어 시각적 결함의 재현 근거가 있다.

## 검증과 위험

- 통합 candidate `4b28259bb`에서 전체 nextest **8,160 passed, 39 skipped**, clippy, native-Skia, WASM build를 통과했다.
- 같은 head의 GitHub Build & Test, archive build/shard, Lint, Native Skia, CodeQL Rust/JavaScript/Python, Canvas visual diff, Adapter inter-diff, Proptest roundtrip도 성공했다. WASM과 frontend unit은 변경 범위 정책에 따른 정상 skip이다.
- source의 clipping/page gate는 이 변경이 clip 범위에 국한되고 페이지 수 회귀가 없음을 보조한다.
- 새 전체 visual sweep은 수행하지 않았다. source의 1쪽 기준 PDF와 전후 PNG가 이미 commit되어 있고, 이 통합에서는 해당 fixture 회귀와 전체 suite를 다시 실행했다. 전체 문서군 fidelity 완료를 의미하지는 않는다.

## 최종 판정

**수용 권고.** 하단 부동 개체만 확장하고 용지 경계 가드를 유지한다. merge 전에는 통합 PR #5889 최신 head의 GitHub CI 통과, mergeability 재확인, 작업지시자 승인이 필요하다.

---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5878 검토 - 표 셀 구역 나누기 secPr 직렬화

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5878](https://github.com/edwardkim/rhwp/pull/5878) / [@planet6897](https://github.com/planet6897) |
| 관련 issue | [#5873](https://github.com/edwardkim/rhwp/issues/5873) |
| base / source head | `devel` / `fb0ad53190a0a625b235163b2786c0c5bff5949f` |
| 변경 규모 | 3 files, +188 / -3 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`, maintainerCanModify=true |
| 통합 반영 | `d7f90eb00` |

## 범위와 검토

- 셀 subList 안 `SectionDef`를 `hp:secPr`로 emit해 orphan `colPr` 뒤의 본문 폐기를 막는다.
- top-level section 출력 경로는 건드리지 않고 subList 직렬화 경로만 보완한다.
- regression은 셀 안 두 번째 `secPr`가 subList 안에 위치하는 XML 구조를 직접 검사한다.

## 검증과 위험

- 통합 candidate `4b28259bb`에서 전체 nextest **8,160 passed, 39 skipped**, clippy, native-Skia, WASM build를 통과했다.
- 같은 head의 GitHub Build & Test, archive build/shard, Lint, Native Skia, CodeQL Rust/JavaScript/Python, Canvas visual diff, Adapter inter-diff, Proptest roundtrip도 성공했다. WASM과 frontend unit은 변경 범위 정책에 따른 정상 skip이다.
- serializer 구조 보존 변경이므로 XML contract와 load/save regression을 우선 적용한다.
- source의 한글 2022 대형 private corpus 수치는 원문을 포함하지 않는다. 따라서 그 수치를 재현 가능한 전수 증명으로 확대하지 않고, 저장소 fixture 계약 범위에서 판정한다.

## 최종 판정

**수용 권고.** 빠져 있던 셀 경로만 top-level의 기존 secPr 정책에 맞춘다. merge 전 PR #5889 최신 CI와 작업지시자 승인이 필요하다.

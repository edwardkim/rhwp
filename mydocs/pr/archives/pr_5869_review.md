---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5869 검토 - HWP3/HWPX 문자 사상 보정

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5869](https://github.com/edwardkim/rhwp/pull/5869) / [@planet6897](https://github.com/planet6897) |
| 관련 issue | [#5860](https://github.com/edwardkim/rhwp/issues/5860), [#5861](https://github.com/edwardkim/rhwp/issues/5861) |
| base / source head | `devel` / `81fda411c6aac2af682ec571627e5401d19c79e7` |
| 변경 규모 | 4 files, +162 / -6 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`, maintainerCanModify=true |
| 통합 반영 | `1d2674857`, `d5dada7c1` |

## 범위와 검토

- HWP3의 측정된 사적 코드를 한글이 표시하는 문자로 제한 사상한다.
- HWPX 한컴 symbol map의 `0xA807` 구멍을 plane-15 대응값으로 보완한다.
- 광범위한 구간 사상이 아니라 관측된 코드만 추가해 기존 KSC 정의역을 침범하지 않는다.

## 검증과 위험

- 통합 candidate `4b28259bb`에서 전체 nextest **8,160 passed, 39 skipped**, clippy, native-Skia, WASM build를 통과했다.
- 같은 head의 GitHub Build & Test, archive build/shard, Lint, Native Skia, CodeQL Rust/JavaScript/Python, Canvas visual diff, Adapter inter-diff, Proptest roundtrip도 성공했다. WASM과 frontend unit은 변경 범위 정책에 따른 정상 skip이다.
- 새 parser/serializer regression cases가 전체 suite에 포함돼 문자 소실과 잘못된 plane 누출을 검증한다.
- 변경은 문자 보존 계약이며 별도 page-layout sweep 대상이 아니다. 비공개 코퍼스의 원문은 저장소에 넣지 않았으므로, source가 남긴 비식별 집계 이상으로 전수 fidelity를 주장하지 않는다.

## 최종 판정

**수용 권고.** 명시적 소수 값 사상과 회귀 테스트가 있어 범위가 제한된다. merge 전 PR #5889 최신 CI와 작업지시자 승인이 필요하다.

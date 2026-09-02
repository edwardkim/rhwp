---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4844 검토 - ML-DSA 하이브리드 출처 서명

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4844](https://github.com/edwardkim/rhwp/pull/4844) · @kevin9327 |
| 원 head | `ccdea6d1d6f221c9eb92db1462da1c9b643b23da` |
| 누적 순서·적용 SHA | 11/27 · `5c4258dd3` → `d30127702` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · PQ signature 테스트 경계 |

작업캡슐·출처 서명을 ML-DSA/FIPS 204 하이브리드로 확장한다. 메인터너 보정으로 테스트의 하드코딩 키
재료를 실행 시 생성 값으로 교체했다. 전체 CI와 CodeQL을 통과했다. **수용 가능**으로 판정한다.

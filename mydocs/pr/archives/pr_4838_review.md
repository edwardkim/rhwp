---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4838 검토 - HWP3 보안 트레일러 ML-KEM 봉인

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4838](https://github.com/edwardkim/rhwp/pull/4838) · @kevin9327 |
| 원 head | `fb8c090ea3915dd801bfe20e048aa291b899627e` |
| 누적 순서·적용 SHA | 8/27 · `046d7a01f` → `6ae86b908` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · ML-KEM/wasm dependency 경계 |

HWP3 보안 트레일러에 ML-KEM-768(FIPS 203) 공개키 봉인을 추가한다. 누적 보정에서 wasm `getrandom`
피처 경계와 테스트 상수를 정리해 WASM 빌드 계약을 복구했다. 로컬 wasm-pack·전체 검증 및 GitHub CI가
통과했다. **수용 가능**으로 판정한다.

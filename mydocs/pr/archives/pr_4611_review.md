---
kind: pr-review
status: pending-ci-release-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4611 리뷰 - 공백 TAC 캐리어 저장 vpos 페인트

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4611](https://github.com/edwardkim/rhwp/pull/4611) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `14650a87318bbea088fe2340f134e1a57bbe7e9e` |
| 원 변경 규모 | 3 files, +318/-4 |
| 통합 적용 | `115e13ee0` |
| 메인터너 보정 | `708eded7b` |
| 관련 이슈 | [#4610](https://github.com/edwardkim/rhwp/issues/4610) |

공백 전용 문단이 treat-as-char 표를 운반할 때 흐름 cursor와 저장 vpos가 달라지는 경우에만 paint y를
되돌리는 원 변경의 방향은 타당하다. 다만 원 조건은 compose 결과가 없거나 표가 block 경로로 분류된 경우에도
rewind를 허용했다. 실제 호출 경로의 의도와 달리 비-inline 표까지 이동할 수 있으므로 메인터너 보정에서
`composed`와 첫 `tac_controls`를 모두 요구하게 좁혔다.

보정은 8개 단위 테스트로 inline TAC 정상 이동과 missing/block TAC 거부를 함께 고정한다. 리베이스 전
누적 통합 후보에서 해당 8건, release-test 전체 5,776건, Clippy, Native Skia 3종, WASM build가 통과했다. 원 PR의
`closes #4610` 의미는 통합 PR에서 유지하되, 릴리스 hold 중에는 issue close를 포함한 merge 후속 처리를
수행하지 않는다.

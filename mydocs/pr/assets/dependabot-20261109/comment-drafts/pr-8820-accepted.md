<!-- dependabot-20261109-accepted-8820 -->
의존성 갱신에 감사드립니다.

이 업데이트를 [통합 PR #{{INTEGRATION_PR}}](https://github.com/edwardkim/rhwp/pull/{{INTEGRATION_PR}})에서 수용했습니다. CodeQL init/analyze를 같은 v4.39.3 SHA로 함께 적용해 원 CI의 버전 불일치를 해소했습니다.

- 원 head: `c607ea643619fced34132144e07d43d4f9637c6e`.
- 통합 merge: [{{MERGE_SHA}}](https://github.com/edwardkim/rhwp/commit/{{MERGE_SHA}}). [정확한 통합 head의 CI](https://github.com/edwardkim/rhwp/actions/runs/{{CI_RUN_ID}})와 [self-review](https://github.com/edwardkim/rhwp/pull/8840#pullrequestreview-5822605316)를 확인했습니다.
- [개별 검토](https://github.com/edwardkim/rhwp/blob/{{EVIDENCE_COMMIT}}/mydocs/pr/archives/pr_8820_review.md), [공통 검증](https://github.com/edwardkim/rhwp/blob/{{EVIDENCE_COMMIT}}/mydocs/pr/assets/dependabot-20261109/README.md).

공유 결합 graph에서 전체 회귀 13,005건·Native 공식 4,077건·Studio 2,778건과 적용 lint·build·실제 브라우저를 통과했습니다. 이 검사를 각 원 PR에서 따로 실행한 것으로 집계하지 않습니다.

원 CI의 Native 작업은 테스트·컴파일 오류 없이 30분 실행 상한으로 취소됐습니다. 이번 통합 head에서는 Native 검사까지 정상 완료했습니다. 원 CI 기록은 보존했습니다.

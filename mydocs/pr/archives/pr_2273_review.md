# PR #2273 검토 - actions/setup-node v6 전환

- 검토일: 2026-07-15
- 작성자: planet6897
- 대상: [PR #2273](https://github.com/edwardkim/rhwp/pull/2273), [Issue #2234](https://github.com/edwardkim/rhwp/issues/2234)
- 메타: `devel` 대상, 5파일 `+5/-5`, contributor head `05b56b010537ed832a98b8588805359bc7a6a0b0`
- 검토 범위: PR 본문·대화, 최신 head 소스 diff, 변경 워크플로우의 정적 문법 검토와 원격 CI를 함께 검토했다.
- reviewer: `jangster77` 지정 완료. 최신 head의 필수 CI는 통과 상태다.

## 본문·대화 검토

본문은 CI, Pages, renderer sweep, npm publish, render diff의 다섯 워크플로우에서
`actions/setup-node`를 v4에서 v6으로 통일하고, 캐시 입력과 Node 20/22 매트릭스는
그대로 둔다고 설명한다. v6의 node24 action runtime으로 Node.js 20 action-runtime
deprecation을 해소한다는 목적도 명확하다.

PR 대화에는 본문을 뒤집거나 범위를 넓히는 추가 코멘트가 없다. 다만 본문 자체가 clean
frontend package gate 및 Studio·Chrome·Firefox·VS Code build/test의 최종 확인을 미완료
항목으로 남긴다.

## 판단

**merge 수용 가능.** 소스 diff는 다섯 워크플로우의 `actions/setup-node` 참조만 v4에서
v6으로 바꾸며 캐시 키·Node 매트릭스·job 구조 변경은 없다. 변경 워크플로우에 대해
`actionlint -ignore 'SC2086|SC2035'`를 실행해 문법 검증을 통과했고 최신 원격 CI도
통과했다. 전체 `actionlint`에서 보인 SC2086/SC2035는 변경하지 않은 기존 셸 단계의 정보성
진단이므로 이 PR의 차단 사유는 아니다.

## 체리픽 누적 검토 기록

- 순서: 1/3
- 적용 커밋: `05b56b010537ed832a98b8588805359bc7a6a0b0`
- 누적 브랜치 커밋: `0cf8e0b2b`
- 충돌: 없음
- 선행 의존: 없음

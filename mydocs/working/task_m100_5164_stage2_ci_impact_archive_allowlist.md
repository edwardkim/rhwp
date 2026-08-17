# task_m100_5164 stage 2: CI impact archive allowlist 동기화

## 관찰

PR #5170의 첫 CI run에서 단일 archive job은 의도대로 한 개만 생성됐지만 `Lint`의
`Validate CI impact classifier` 단계가 실패했다.

- run: https://github.com/edwardkim/rhwp/actions/runs/32000975930
- job: https://github.com/edwardkim/rhwp/actions/runs/32000975930/job/95301723031
- 실패 계약: `every impact-conditioned CI job is covered by the audit allowlist`

## 원인

`ci.yml`과 workflow 전용 Python 계약은 단일 `build-test-archive`로 갱신됐지만,
trusted fast-pass 감사 정책의 `CI_RUST_JOBS`, `CI_JOB_ALIASES`, `CI_AUDITED_JOB_IDS`에는
이전 `build-test-archive-slow`, `build-test-archive-a`, `build-test-archive-b`가 남아 있었다.

## 보정

- 세 builder의 감사 identity를 `build-test-archive` 하나로 통합했다.
- 실행된 reusable workflow의 REST 이름
  `build-test-archive / Build test archive`를 동일 logical lane의 alias로 등록했다.
- duplicate alias 회귀 테스트도 단일 builder identity를 사용하도록 갱신했다.

## 검증

- `node --test scripts/tests/ci-impact-policy.test.cjs` 31건 통과
- Python workflow 계약 테스트 41건 통과
- 변경 workflow 3개 `actionlint` 통과
- `git diff --check` 통과
- 새 PR head의 전체 CI에서 단일 builder와 trusted audit가 함께 통과하는지 확인

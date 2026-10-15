# #7926 단계1 — 가변 PR 연결 정보와 고정 workflow 실행 SHA 분리

- Issue: [#7926](https://github.com/edwardkim/rhwp/issues/7926)
- 반영 PR: [#7923](https://github.com/edwardkim/rhwp/pull/7923)
- 출발 head: `0b67266a6e53925f62cf8544cce9c521d3210cf8`; trusted base `463c48cc228170b576c832591b76826116e86a49`.

## 독립 입력·원인·반례

성공한 CI run38734213321의 `head_sha`는 `cbc5815631d6c221dc6ec6e0f03565859cb1c346`로 고정돼 있다. 같은 API 응답의 연결 PR7923 `head.sha`는 문서 push 뒤 `0b67266a6e53925f62cf8544cce9c521d3210cf8`이다. 이전 모듈은 이 둘을 모두 candidate와 비교해 성공 실행을 버렸다. 문서 tail 판정과 candidate 선택 자체는 통과했다.

`selectLatestWorkflowRun`의 name/path/event/repository/branch/고정 run SHA → 연결 PR 번호/base ref·조건부 base SHA/head ref 검사 → 실제 run/job 성공 감사의 경로를 유지한다. 연결 PR의 가변 `head.sha`만 동일성 근거에서 제외한다. 반환 실행의 SHA를 완화하거나 결론을 success로 바꾸지 않는다.

## 구현·정식 회귀

- 정책 helper SHA-256: `ab6eb0673ccecc41212abf61d0acebe35ee428fe147f21daf54966569f2fa5b8`.
- `scripts/ci-impact-policy.cjs`: 실행 SHA 확인은 기존 `run.head_sha` 검사에 둔다. PR 연결 메타데이터의 head SHA 중복 비교를 제거했다.
- `scripts/tests/ci-impact-policy-pr-association-contract.cjs`73줄: 실제 SHA 관계의 CI·CodeQL·Render Diff3경로, 9가지 identity 거부 대조, current/prior base의 구별을 검사한다.
- 기존 `ci-impact-policy.test.cjs`가 새 계약을 직접 로드하므로 기존 CI 진입점에서 실행된다.

## 실제 검증 결과

- 수정 전 새5건 모두 ERR_ASSERTION FAIL. 수정 후 policy 전체50PASS.
- 정책·classifier·실행 증거 수집·post-merge 재사용 Node 묶음354PASS.
- workflow Python 묶음186PASS.
- 변형하지 않은 실제 GitHub API 응답을 새 helper에 재생하면 성공 run38734213321이 선택된다.
- 다른 PR/source branch/repository/run SHA/workflow path/name/event/base ref 및 허용하지 않은 base 변경은 계속 거부한다.
- renderer·Rust·폰트·입력/Print는 바뀌지 않아 기존21문서181쪽 시각 증거를 재사용한다. Cargo·WASM 재빌드를 로컬에서 반복하지 않았다.

## 제출·운영 경계

사용자가 이슈 등록과 개선을 승인했다. 새 정책 수정 head의 GitHub Actions 성공 뒤 머지한다. controller는 trusted devel 모듈을 읽으므로 현재 PR의 후보 코드로 현재 실행을 스스로 fast-pass시키지 않는다. 이 새 head는 기존 base 정책의 Full CI를 통과해야 하며 정책 status를 강제로 쓰거나 실패 검사를 우회하지 않는다. merge 뒤 devel에 들어간 helper와 실제 API 응답으로 후보 선택/정책 재사용을 다시 검증한다.

raw 응답·로그와 수정 전후 JSON은 ignored `output/pr-review/planet6897-followup-20261011/`에 보존한다. code candidate와 trailing head의 이전 성공 CI는 각 시점 이력으로 남기며 새 head 성공으로 대신하지 않는다.

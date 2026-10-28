# Task M100 #4608 — stale run 취소 완료 상태 polling 최종 보고서

- **Issue**: [#4608](https://github.com/edwardkim/rhwp/issues/4608)
- **브랜치**: `codex/issue-4608-stale-cancel-poll`
- **기준**: `upstream/devel@6b5c4f871972380c0866e2a8d27ac2bc67d257e6`
- **재현 PR**: [#6116](https://github.com/edwardkim/rhwp/pull/6116)
- **완료일**: 2026-08-26 KST

## 결론

#4609의 오류 직후 단일 상태 재조회는 GitHub가 force-cancel 결과를 약 1초 늦게 노출하는 경우를 흡수하지
못했다. 오류 뒤 즉시·0.5초·1초·2초에 대상 run을 다시 읽는 bounded polling을 추가했다. 최대 3.5초
window 안에서 실제 `completed`를 확인한 경우에만 정상 경과로 처리하고, 끝까지 active이거나 상태 API가
실패하면 원래 force-cancel 오류를 유지한다.

## 재현 사실

| 항목 | 값 |
| --- | --- |
| 실패 cleanup | [run 32923641712](https://github.com/edwardkim/rhwp/actions/runs/32923641712) |
| stale 대상 | [CI run 32923603626](https://github.com/edwardkim/rhwp/actions/runs/32923603626) |
| stale SHA | `68873f71999e873d9dbdf348ad058ee0aab8a863` |
| 최신 SHA | `6b0fa6ee9de406c9f9abf13ca8ab19bd277a1321` |
| 오류 | `POST .../force-cancel` → HTTP 500 |
| 실제 결과 | 오류 약 1초 뒤 `completed/cancelled` |

동일 cleanup은 앞선 Render Diff·Adapter inter-diff·Proptest·CodeQL 네 run을 정상 취소했다. 최신 SHA의
required Build & Test, Frontend package, CodeQL, Render Diff, Proptest와 Adapter inter-diff는 모두
성공했으므로 제품 또는 PR #6116의 기능 실패가 아니라 cleanup 상태 반영 race다.

## 구현

- polling 간격을 `[0, 500, 1_000, 2_000]`ms로 고정해 무한 대기와 과도한 API 호출을 막았다.
- 상태 조회와 polling helper를 분리하고 `completed`만 성공 반환한다.
- 상태 조회 오류는 warning을 남기되 원래 force-cancel 오류를 전파한다.
- 한도 뒤 `null`이면 원래 오류를 전파해 실제 취소 실패를 숨기지 않는다.
- 기존 `502/503/504` POST 재시도, 최신 head 재확인, fork 저장소·branch 식별을 유지했다.
- `pull_request_target`에서 PR source checkout·실행을 추가하지 않았다.

## 검증

```text
focused workflow + wiring: 7 pass
CI impact Python contracts: 42 pass
CI impact Node contracts: 65 pass
전체 Python workflow contracts: 178 pass
actionlint cancel-stale-pr-runs.yml: pass
git diff --check: pass
```

workflow·계약 테스트·문서만 변경했다. 제품 Rust·Studio·renderer·fixture가 없어 Cargo 전체 회귀, WASM과
시각 검증은 범위 밖이다.

## 남은 운영 게이트

- 별도 승인 뒤 작업 branch를 push하고 `devel` 대상 PR을 생성한다.
- 최신 PR head의 Full CI와 privileged workflow 경계를 확인한다.
- 실제 fork `synchronize` event에서 stale run 완료 race가 정상 처리되는지 관찰한다.
- `pull_request_target`은 기본 브랜치의 workflow를 사용하므로 수정이 정상 release로 `main`에 반영된 뒤
  fork 경로의 배포 완료를 최종 확인한다.

## 롤백

polling helper와 호출부, 관련 계약 테스트만 되돌리면 #4609의 단일 재조회 동작으로 복귀한다. trigger,
permission, required check와 저장소 설정은 바꾸지 않았다.

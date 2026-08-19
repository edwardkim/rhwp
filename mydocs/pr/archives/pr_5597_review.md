---
kind: pr-review
status: approved
pr: 5597
issue: 5596
---

# PR #5597 검토 기록 - review tail 보조 workflow 재사용

- PR: [#5597](https://github.com/edwardkim/rhwp/pull/5597) `fix(ci): review tail 보조 검증을 재사용한다`
- 관련 이슈: [#5596](https://github.com/edwardkim/rhwp/issues/5596), [#5600](https://github.com/edwardkim/rhwp/issues/5600)
- 검토 head: `d10f56f254e1ad28e5deba4b9602e7636f8febcc`
- 병합 커밋: `d9a541f78815b1284efa313a35299118a64a35bf`

## 검토 범위

- `proptest-roundtrip.yml`과 `adapter-diff.yml`이 `mydocs/**`만으로 이루어진 PR 및 선형 review tail을 판별한다.
- review tail은 직전 부모를 후보로 삼고, 동일 PR의 직전 후보 SHA에서 성공한 동일 workflow 실행이 있을 때만 보조 worker를 재사용한다.
- GitHub API 응답, 계보, workflow 실행 증적 중 하나라도 확인할 수 없으면 전체 worker를 수행하는 fail-closed 경로를 유지한다.
- 기존 필수 check 이름과 non-docs PR의 실행 경로를 유지한다.
- 릴리스 표시 버전을 `0.8.4`로 다시 정렬해 Studio About, Chrome/Edge 확장, Firefox 확장의 사용자 표시 버전 불일치를 해소한다.

## 검증 근거

- 로컬 workflow 계약 검증: `test_proptest_roundtrip_workflow.py`, `test_adapter_diff_workflow.py`, `test_ci_impact_workflow.py` 합계 48건 통과.
- 로컬 릴리스 채널 정책 검증: 5건 통과.
- `npm ci --ignore-scripts`, `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check` 통과.
- GitHub Actions: CI, CodeQL, Render Diff, Proptest roundtrip, Adapter diff가 모두 성공했다.

## 결론

차단 결함은 발견하지 못했다. PR #5597은 위 병합 커밋으로 `devel`에 통합되었고, 관련 이슈 상태 확인과 원격 head 정리는 post-merge 절차에서 완료한다. 이 기록과 오늘할일 갱신은 코드 PR의 CI 범위를 늘리지 않도록 별도 docs-only PR로 처리한다.

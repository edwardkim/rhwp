---
kind: working
status: active
issue: 5600
---

# #5600 사용자 표시 버전 기준선 정합

## 원인

release channel 정책은 `Cargo.toml`의 release version `0.8.4`와 Studio About, Chrome/Edge,
Firefox 확장 표시 버전이 모두 같아야 한다. Chrome과 Firefox 매니페스트는 이미 `0.8.4`지만,
`rhwp-studio/package.json`과 lockfile 루트 항목이 `0.8.5`여서 CI Lint가 실패했다.

## 보정

- Studio package와 lockfile 루트 package version을 `0.8.4`로 되돌린다.
- 확장 매니페스트는 release version과 이미 일치하므로 변경하지 않는다.

## 검증 계획

1. `python3 -m unittest scripts/tests/test_release_channel_policy_workflow.py`를 실행한다.
2. `cargo fmt --all -- --check`와 `git diff --check`를 실행한다.

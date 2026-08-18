---
kind: pr-review
pr: 5473
issue: 5463
base: devel
head: codex/5463-derived-unit-tier-inventory
status: merged
merged_at: 2026-08-18T11:09:32Z
merge_commit: 2c639ac0da43adabc792bd0649bbeb7ac1490f08
---

# PR #5473 검토 기록: unit-tier inventory를 CI 파생 산출물로 분리

## 변경 범위

- 추적하던 `tests/suites/unit-test-tiers.json`을 제거했다.
- 정적 규칙은 `tests/suites/unit-test-tier-policy.json`으로 분리해 추적한다.
- 현재 소스와 PR base의 inventory는 CI 및 로컬 검사에서 계산하며, 선택적 진단 산출물은 `tests/generated/unit-test-tiers.json`에 생성하고 Git에서 제외한다.
- PR 템플릿과 기여·개발·로컬 검증 문서에서 개발자가 inventory를 생성하거나 stage하지 않도록 명확히 했다.

## 검증 근거

- 로컬: `cargo fmt --all`, `cargo fmt --all -- --check`, `node scripts/rust-test-suite-manifest.mjs --check`, `node scripts/rust-unit-test-tiers.mjs --check`, `git diff --check` 통과.
- 로컬: `node --test scripts/tests/rust-unit-test-tiers.test.mjs` 12건, `node --test scripts/tests/rust-test-suite-manifest.test.mjs` 16건 통과.
- CI: 2026-08-18 모든 필수 검사가 성공했다. Rust CodeQL은 13분 45초, test archive 생성은 7분 51초였고 regular/slow nextest shard 및 Native Skia 검증도 성공했다.

## 결론 및 후속 처리

- 정적 정책과 매 실행마다 달라지는 inventory를 분리해, 새 회귀 테스트가 추가된 PR 간의 생성 파일 충돌을 제거했다.
- PR #5473은 `2c639ac0da43adabc792bd0649bbeb7ac1490f08`으로 squash merge되었다.
- 관련 이슈 #5463은 병합으로 종료되었고, 기능 브랜치 `codex/5463-derived-unit-tier-inventory`는 로컬과 원격에서 삭제했다.

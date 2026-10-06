# Stage 4 - #5027 delete-bookmark 파생 suite 제외

## 목표

Kevin 기여 PR #5027의 `rhwp edit delete-bookmark` 기능과 원본 계약 테스트를 누적 후보에 반영한다. 다만 새 회귀 테스트 운영 정책(#5177)에 따라 PR 커밋에는 `tests/generated/**`와 `tests/suites/manifest.json`을 포함하지 않는다.

## 반영 범위

- `src/main.rs` 및 CLI 문서: `delete-bookmark` 명령과 `--section`/`--para`/`--ctrl` 식별자 계약
- `tests/cases/delete_bookmark_contract.rs`: 원본 회귀 계약
- `tests/suites/unit-test-tiers.json`: 원본 테스트의 tier 분류 입력

## 파생 산출물 처리

기여 커밋에 포함되어 있던 `tests/generated/regression_suite_019.rs`와 `tests/suites/manifest.json` 변경은 기준 브랜치 상태로 복원했다. 해당 파일들은 최종 PR 검토와 CI 체크아웃에서 `node scripts/rust-test-suite-manifest.mjs --prepare`를 한 번 실행해 생성하며, 커밋하지 않는다.

## 다음 단계

후속 Kevin PR도 개별 기능 증분만 누적하고, generated suite와 manifest는 동일한 방식으로 PR diff에서 제외한다.

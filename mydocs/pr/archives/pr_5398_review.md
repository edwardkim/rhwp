---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-19
---

# PR #5398 검토 - CharShape 기본 장평을 OWPML 기본값 100으로 보정

## 접수와 통합 기준

| 항목 | 검토 결과 |
| --- | --- |
| PR / 작성자 | [#5398](https://github.com/edwardkim/rhwp/pull/5398) / johndoekim |
| 관련 이슈 | [#4161](https://github.com/edwardkim/rhwp/issues/4161) |
| contributor 원 head | 75c2af163fcbcf155a3273e61dca19aeab354b3c |
| 보정 head | 8384b6f496ee21dc7e560899b70e944642753e95 |
| 가시성 branch | review/johndoekim-20260819 |
| 통합 기준 | upstream/devel@c3c35306b1428a2dcd97656d1cbe4a8c74c780a7 |

원 PR은 CharShape.ratios의 기본값을 0에서 100으로 바꾸고, HWP3·HWP5·HWPX·HML 변환 결과의 ratio가
유효 범위에 있음을 회귀 계약으로 추가한다. 이는 OWPML 기본 장평 100과 HWP 파서의 결손값 처리 정책을
일치시키는 변경이다.

## 메인터너 보정

PR의 오래된 base는 이미 삭제된 tests/suites/unit-test-tiers.json을 수정하고 있어 최신 devel과
modify/delete 충돌을 냈다. 이 파일은 현행 tests/suites/unit-test-tier-policy.json 및 CI 생성
inventory로 대체된 legacy 파생물이다.

- contributor commit은 rewrite하지 않았다.
- contributor head를 첫 부모로 유지한 뒤 최신 devel을 merge한 8384b6f 커밋에서 legacy inventory를
  삭제로 해소했다.
- 결과 tree d590f28c0c671372c010032e34a2b3870bb39809는 current-base 검증 후보와 동일하다.
- tests/generated/regression_suite_* 및 tests/suites/manifest.json을 포함한 CI 파생 산출물은
  review branch·원격 head에 추가하지 않았다.

## 로컬 검증

- cargo fmt --all 및 cargo fmt --all -- --check 통과
- rust-test-suite-manifest의 prepare·자동 테스트·check 통과
- rust-unit-test-tiers 자동 테스트·check 통과
- issue_4161_ratio_default_contract focused 테스트 통과: HML, SO-SUEOP, DocumentCore,
  HWP3 export/HWPX, HWP3 전체 sample의 5개 계약
- release-test 전체 nextest를 target/pr-review에서 test thread 8개로 실행해, 깨끗한 전용 TEMP
  루트에서 종료 코드 0을 확인했다.
- cargo clippy --all-targets --target-dir target/pr-review -- -D warnings 통과
- git diff --check upstream/devel...HEAD 통과

기본 Windows TEMP에서 provenance_contract의 settle record 레시피가 이전 PID 디렉터리의 원장을
재사용해 한 번 exit 3을 반환했다. 같은 계약은 새 TEMP 루트에서 종료 코드 0으로 재현됐고, 전체
release-test도 같은 깨끗한 TEMP 조건에서 통과했다. 이는 #5398의 모델 변경이나 보정 merge와 무관한
로컬 임시 경로 격리 문제로 분리했다.

## 시각 및 CI 증적

이번 변경은 renderer/layout 또는 신규 시각 fixture 변경이 아니며, PR에는 기준 PDF나 새 asset이 없다.
GitHub Canvas visual diff는 6분 3초로 통과했고, 새 code head에서 다음 CI도 통과했다.

- Lint (fmt, clippy, WASM check)
- Native Skia tests
- Build test archive 및 default-feature regular 3개 shard와 slow shard
- Proptest roundtrip
- CodeQL Rust 분석, JavaScript/TypeScript 분석, Python 분석

## 판정

차단 결함은 발견하지 못했다. 최신 devel 호환 보정을 포함한 code head는 local 및 GitHub CI를 통과했다.
이 문서를 trailing review 기록으로 반영한 뒤 최신 head의 mergeable 및 required check를 다시 확인하고
PR #5398을 merge한다.

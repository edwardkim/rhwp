# Stage 17: Native Skia 파생 suite 준비

## 목적

`#5177`에서 추적 제외한 `tests/generated/**`와
`tests/suites/manifest.json`은 CI 작업마다 새 checkout에서 생성해야 한다.
Native Skia 작업이 이를 준비하지 않은 채 Cargo integration test target을
컴파일하여 PR #5185에서 실패한 원인을 보정한다.

## 관찰

- 실패 job: `Native Skia tests`
- 실패 원인: Cargo가 `tests/generated/regression_suite_006.rs`를 참조했지만,
  fresh checkout에는 파생 harness가 아직 없었다.
- lint job에는 이미 `rust-test-suite-manifest.mjs --prepare` 단계가 있어 같은
  문제가 발생하지 않았다.

## 변경 계약

- `native-skia-tests` job은 native package 설치 뒤, 첫 Cargo 호출 전에
  `node scripts/rust-test-suite-manifest.mjs --prepare`를 정확히 한 번 실행한다.
- workflow 계약 테스트는 Native Skia 준비 단계의 존재와 `Native Skia tests`
  단계보다 앞선 순서를 검증한다.
- 파생 산출물은 계속 Git 추적 대상이 아니며, CI 작업 공간에서만 생성한다.

## 검증 결과

- `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`: 29건 통과.
- `node scripts/rust-test-suite-manifest.mjs --prepare`: 파생 harness 32개와
  manifest를 작업 공간에 생성하고 새 test source가 없음을 확인.
- `node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel`:
  595 source, 2,588 static test attribute, 최소 6,559 nextest case의 suite
  계약을 통과.

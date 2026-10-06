# Stage 9 - #5177 파생 suite 임시 검증

## 발견

`#5177`은 기여자 PR에 `tests/cases/**` 원본만 커밋하고 `tests/generated/**`,
`tests/suites/manifest.json`, Cargo의 generated target 블록은 커밋하지 않도록 정했다.
그러나 기존 `rust-test-suite-manifest` 단위 검증은 현재 작업 트리의 manifest가 모든
원본을 이미 선언했다고 가정했다. 원본 test만 추가한 정상 PR에서는 이 검증이 실패한다.

## 보정

검증 API에 `derive: true` 경로를 추가했다. 이 경로는 manifest를 메모리에서 복제하고
새 source를 같은 weighted 배정 규칙으로 반영하되, manifest와 generated harness,
Cargo.toml에는 쓰지 않는다. 단위 테스트는 이 임시 계획을 검증하고, manifest 원문이
변하지 않았음을 함께 확인한다.

## 경계

실제 PR review와 CI는 계속 한 번의 `--prepare`로 파생 파일을 작업 체크아웃에 만든 뒤,
일반 엄격 검증으로 harness와 Cargo target block 일치까지 검사한다. 이번 경로는
기여자 PR의 커밋 정책과 단위 검증을 양립시키기 위한 읽기 전용 검증이며, CI 생성 절차를
대체하지 않는다.

## 검증 계획

`node scripts/tests/rust-test-suite-manifest.test.mjs`로 임시 파생과 비쓰기 계약을 확인하고,
PR diff에 파생 산출물이 포함되지 않았는지 확인한다.

# Stage 15 - CI 재생성 산출물의 PR diff 판정

## 실패 원인

lint job은 Cargo가 integration target을 해석하기 전에 `--prepare`를 실행하도록
보정됐다. 그러나 이어지는 `--check --base-ref`는 PR diff의 삭제(`D`) 경로를 읽은 뒤,
현재 작업 폴더에 파일이 존재하면 파생 산출물의 커밋으로 오판했다. `--prepare`가
삭제된 harness와 manifest를 다시 만들었기 때문에 발생한 모순이다.

## 보정

- 파생 산출물 커밋 검사는 `base...HEAD`의 추가·수정(`A/M`) 경로만 대상으로 한다.
- 삭제(`D`)된 산출물은 CI가 작업 폴더에 재생성해도 허용한다.
- 실제 Git fixture에서 산출물을 삭제한 commit 뒤에 CI와 같이 재생성하고
  `validateRepository(..., { baseRef })`가 통과하는 회귀 테스트를 추가한다.

## 유지하는 규칙

기여자가 `tests/generated/**`, `tests/suites/manifest.json`, 또는 Cargo generated
target 블록을 새로 추가·수정하면 계속 실패한다. 이 보정은 CI 재생성으로 인해
삭제를 추가로 오인하는 경우만 허용한다.

## 검증 결과

- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`: 16 passed.
- `node scripts/rust-test-suite-manifest.mjs --prepare && node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel`: 통과.

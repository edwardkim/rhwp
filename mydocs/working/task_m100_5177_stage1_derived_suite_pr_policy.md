# Task M100 #5177 Stage 1 - PR 파생 Rust suite 산출물 분리

## 목적

새 회귀 테스트 PR이 `tests/cases/` 원본 외에 generated harness, suite manifest, Cargo generated test
target 블록까지 커밋해 독립 PR 간 충돌을 만드는 문제를 제거한다.

## 결정

- 기여 PR은 `tests/cases/**` 원본만 제출한다.
- `tests/generated/**`, `tests/suites/manifest.json`, `Cargo.toml`의 generated test target 블록은
  검토·CI checkout에서만 만드는 파생 산출물로 취급한다.
- `--prepare`는 삭제·이름변경 동기화와 신규 source 자동 배정을 한 번에 수행한다.
- CI는 base와 PR HEAD의 커밋된 파생 산출물 차이를 거부하고, archive 컴파일 직전에 `--prepare`를 실행한다.

## 변경 범위

- suite 생성기와 계약 테스트
- CI lint 및 nextest archive workflow
- 기여·개발·PR review 가이드

## 기대 결과

한 PR이 병합되어도 다른 PR의 generated harness·manifest 충돌이 발생하지 않으며, CI는 현재 PR source에서
결정론적으로 만든 suite로 전체 회귀 archive를 생성한다.

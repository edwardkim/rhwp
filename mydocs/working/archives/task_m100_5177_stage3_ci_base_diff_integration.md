# Task M100 #5177 Stage 3 - CI PR-base 파생 산출물 게이트 통합 검증

## 목적

CI의 `--check --base-ref`가 순수 경로 판정만 하는 것이 아니라, 실제 Git PR base와 HEAD의 차이를 읽어
수동 커밋된 generated manifest와 Cargo generated target block을 거부하는지 검증한다.

## 방법

- 최소 Rust suite fixture를 임시 Git 저장소에 base commit으로 만든다.
- manifest 또는 Cargo generated target block만 바꾼 후 head commit을 만든다.
- `validateRepository(..., { baseRef: "HEAD~1" })`가 해당 파생 산출물 오류를 보고하는지 확인한다.

## 기대 결과

CI에서 수행하는 PR-base 게이트가 실제 commit diff에 대해 동작하며, checkout 안에서 `--prepare`가 만든
uncommitted 파생 파일은 오탐으로 취급하지 않는다.

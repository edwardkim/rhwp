# 변경 범위별 검증

명령과 필수 범위의 정본은 [CONTRIBUTING.md](../../../../CONTRIBUTING.md)다.
메인터너 검토에서는 [local_validation.md](../../../../mydocs/manual/pr_review/local_validation.md)의
4.3과 기존 증거 재사용 절차를 함께 적용한다. 두 역할의 생략 조건을 혼용하지 않는다.

## Rust

- Rust source 변경은 fmt, native/WASM/workspace-all-targets의 세 Clippy 단계와
  범위표의 focused/전체 회귀를 따른다. native Clippy 한 줄이나 관련 테스트만으로 전체 게이트를 대신하지 않는다.
- Rust test/baseline helper, renderer/layout, Studio 혼합 변경은 각각 정본의 추가 게이트를 적용한다.
  문서만 바꾼 경우에 Rust 전체 검증을 무조건 강제하지 않는다.
- 제출 source SHA를 고정한 review worktree에서 파생 integration suite를 준비한다.
  이 절차는 메인터너만의 권한이 아니라 CONTRIBUTING의 기여자 검증 경로이기도 하다.
- 파생 suite 준비는 `node scripts/rust-test-suite-manifest.mjs --prepare`를 사용한다.
  이어지는 순차 lint/build/test와 manifest `--check`는 정본의 순서와 인자를 따른다.
- `tests/cases/` 원본만 제출하며 파생 suite/manifest와 진단 inventory를 stage하지 않는다.
  Cargo target registry 갱신은 별도 메인터너 절차이며 일반 기여에서 `--sync-cargo-targets`를 쓰지 않는다.
- source-side test 변경은 `node scripts/rust-unit-test-tiers.mjs --check`를 따른다.
  진단용 `--generate`를 제출 필수 단계로 바꾸지 않는다.
- 동일 Cargo target을 쓰는 빌드/검증은 동시에 실행하지 않는다. 출력 공백을 실패로 간주하지 않는다.

## 의미 있는 테스트 결과

수정 전 실패와 수정 후 독립적인 기대값 충족을 확인한다.
fixture 누락의 조용한 return, 잘못된 필터로 0건 실행, 중단된 작업은 통과가 아니다.
시간 부족으로 필수 전체 회귀를 focused test로 대체하지 않는다.

스킬 변경은 현재 skill router/계약과 변경 범위에 맞는 검사를 선정한다.
고정된 generated suite 번호나 근거 없는 동일 검사 3회 반복을 영구 게이트로 복제하지 않는다.
검사를 실행하지 않았거나 실행 권한이 제한되면 그 사실과 미완료 범위를 기록하고 통과를 주장하지 않는다.

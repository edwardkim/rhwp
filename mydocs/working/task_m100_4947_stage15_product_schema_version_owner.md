# task_m100_4947 stage 15: 제품 스키마 버전 소유권 복원

## 발견

스테이지 14 전체 nextest 실행에서 6,518개 실행 대상 중 1건이 실패했다.

```text
schema_registry_contract::capabilities_schema_registry_matches_constants
left:  "0.1.0"
right: "0.8.4"
```

`schema_registry` 구현을 `rhwp-contracts`로 분리하면서 기존 `env!("CARGO_PKG_VERSION")`이 루트 제품이 아니라 내부 크레이트의 버전을 읽게 된 것이 원인이다.

## 계약

- 스키마 축 상수와 레지스트리 JSON 조립은 `rhwp-contracts`가 소유한다.
- 공개 `crateVersion`의 소유자는 루트 `rhwp` 제품 크레이트다.
- 내부 크레이트 버전을 제품 버전과 억지로 맞추지 않는다.
- `rhwp::schema_registry` 공개 경로와 기존 함수 시그니처는 유지한다.

## 변경

- `rhwp-contracts`에 제품 버전을 인자로 받는 `registry_value_with_crate_version`을 추가했다.
- 루트 `src/schema_registry.rs` 파사드가 자신의 `CARGO_PKG_VERSION`을 조립 함수에 전달한다.
- `src/lib.rs`는 내부 모듈을 직접 재수출하는 대신 제품 파사드를 공개한다.
- 내부 크레이트를 직접 사용할 때의 `crate_version()`과 `registry_value()` 동작은 그대로 보존한다.

## 검증 계획

- 실패 계약 단독 재실행
- 스키마 레지스트리 및 내부 크레이트 단위 테스트
- 전체 nextest 회귀 재실행
- workspace clippy, release build, 문서 테스트 및 정책 게이트

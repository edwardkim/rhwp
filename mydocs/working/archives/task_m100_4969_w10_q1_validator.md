# Task M100 #4969 W10-Q1 — bounded request validator

## 판정

exact source shaping 요청의 첫 제품 경계를 `src/renderer/shaping.rs`에 추가했다. 이 경계는 아직 glyph를
생성하거나 layout·paint를 바꾸지 않는다. 요청을 실제 shaper로 넘기기 전에 source provenance, payload 상한,
direction과 writing mode, script·language·feature tag, variation axis와 vertical table capability를 검사한다.

17개 구조화 reason을 `requested`, `unsupported`, `malformed`, `bounded-limit`, `non-portable` disposition에
대응시켰다. 존재하지 않는 axis는 `unsupported`, 존재하지만 NaN·중복·범위 밖인 axis는 `malformed`다. 자동
clamp나 family-name fallback은 없다.

## 보호 불변식

- renderer 적용과 backend replay 허용은 0건이다.
- 32 MiB font, 4,096 code point, feature 64, axis 16 상한을 shaping 전에 적용한다.
- non-portable source는 parse·hash 전에 거부한다.
- SHA-256 문자열은 64자 사전 할당 버퍼에 기록해 font 크기에 비례한 임시 문자열 할당을 만들지 않는다.
- vertical 요청은 `vhea`와 `vmtx`가 함께 없으면 거부한다.
- variation은 `fvar` 실제 axis와 범위를 feature detection한다.

## 검증

- focused integration source: 4 pass, 0 fail
- `cargo check --locked --lib`: pass
- `cargo clippy --locked --lib -- -D warnings`: pass
- `cargo check --locked --lib --target wasm32-unknown-unknown`: pass
- 제품 출력·W9 layout mutation: 0

새 테스트 원본은 `tests/cases/issue_4969_shaping_request_contract.rs`만 추가했다. generated suite·manifest와
Cargo target은 만들거나 stage하지 않았다.

## 다음 절편

Q1의 다음 결손은 검증된 요청의 canonical identity다. feature의 적용 순서를 보존하고 variation axis vector의
정렬·float bit identity를 고정한 뒤, bounded rustybuzz glyph id·cluster·advance·offset 출력 계약과 연결한다.
이 단계에서도 renderer 적용은 열지 않는다.

**후속 상태**: 위 절편은 완료됐다. 현재 판정은
[`task_m100_4969_w10_q1_identity_output.md`](task_m100_4969_w10_q1_identity_output.md)를 따른다.

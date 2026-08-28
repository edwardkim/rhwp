# Task M100 #4969 W10-Q1 — canonical identity·bounded glyph output

## 판정

검증을 통과한 exact-source 요청을 canonical settings identity와 rustybuzz glyph oracle에 연결했다. 이 절편은
layout·line break·paint에 결과를 적용하지 않는다. 따라서 제품 조판 변화는 0이며, Q2에서 사용할 입력·출력
계약만 고정한다.

canonical identity는 source SHA-256·byte 수·face index, direction·writing mode, script·language,
feature·variation을 길이 구분 인코딩한 뒤 SHA-256으로 고정한다. variation axis는 tag 순으로 정렬하고 `-0.0`은
`+0.0`으로 정규화하되 나머지 값은 `f32` bit identity를 보존한다. OpenType feature는 적용 순서가 의미를 가질
수 있으므로 입력 순서를 보존한다.

원문은 identity와 직렬화 출력에 넣지 않는다. 이 hash는 **shaping 설정 identity**이지 원문을 생략한 완전한
output cache key가 아니다. Q2에서 실제 layout cache에 연결할 때는 기존 run/source range 경계와 함께 사용해야
하며, 이 settings hash만으로 서로 다른 문자열의 glyph 결과를 재사용해서는 안 된다.

## glyph output 계약

성공 결과는 `glyphId`, UTF-8 byte offset인 `clusterUtf8`, x/y advance와 x/y offset만 보존한다. 4,096 glyph를
넘는 결과는 glyph vector를 공개하지 않고 `bounded-limit/glyph-limit-exceeded`로 닫는다. validation을 통과하지
못한 요청도 기존 disposition·reason을 그대로 전달하며 빈 glyph 결과를 반환한다.

cluster overflow는 별도 손상 fixture로 만들 수 있는 입력 상태가 아니다. Rust `str`은 유효한 UTF-8이고 입력
상한 4,096 code point에서 최대 byte 길이는 16,384다. 가장 넓은 4-byte scalar 4,096개를 실제 shaping한 경계
fixture의 마지막 cluster는 16,380이었다. 따라서 `u32` cluster offset overflow는 이 validator 안에서 도달
불가능하며, 경계 fixture와 산술 상한으로 대체했다.

공개 oracle 재현 결과는 다음과 같다.

| fixture | 핵심 관찰 | 결과 |
| --- | --- | --- |
| Noto `office`, `liga=0` | glyph 6개, UTF-8 cluster 0..5 | 일치 |
| Noto `office`, `liga=1` | glyph 4개, cluster `[0,1,4,5]` | 일치 |
| Source Han `ᄒᆞᆫ글` | glyph 4개 중 3개가 cluster 9 공유 | 일치 |
| Happiness 400/900 | glyph ID 동일, advance 상이 | 일치 |

따라서 GSUB 이후 glyph와 문자를 1:1로 가정할 수 없고, variation instance가 identity에서 빠지면 동일 glyph
ID에 잘못된 advance를 재사용한다는 Q1 가설이 실행 가능한 계약으로 확인됐다.

## 보호 불변식

- exact portable source·face index를 검증하기 전에는 shaping하지 않는다.
- font 32 MiB, text 4,096 code point, feature 64, axis 16, output glyph 4,096 상한을 유지한다.
- axis 중복·NaN·Infinity·범위 밖 값은 clamp하지 않는다.
- backend reshaping과 renderer application은 아직 열지 않는다.
- trace 직렬화에는 font bytes·원문·로컬 경로를 넣지 않는다.
- 새 integration source는 `tests/cases/` 원본 하나뿐이며 generated suite·manifest·Cargo target은 만들거나
  stage하지 않았다.

## 검증

- focused integration source: 9 pass, 0 fail
- canonical settings SHA-256 oracle:
  `fc9df3bfa6a6d8f2f59728372603bafd8147ad12be7a7981ab3ee7c530615f3a`
- `cargo check --locked --lib`: pass
- `cargo clippy --locked --lib -- -D warnings`: pass
- `cargo check --locked --lib --target wasm32-unknown-unknown`: pass
- `git diff --check`: pass
- 제품 layout·paint mutation: 0

## 다음 절편

Q1 종료 전에 기존 `ShapeKey`, `FontInstanceKey`, diagnostics/trace schema가 이 계약을 손실 없이 담는지 대사한다.
실제 계보 결손이 확인된 경우에만 schema 필드를 추가한다. 충분하면 새 제품 필드 없이 Q1을 종료하고 Q2
horizontal GSUB/GPOS 공통 shaping 연결 계획을 제시한다.

**후속 상태**: 대사를 완료했다. 성공 transport는 충분하고 rejected-attempt public trace에는 결손이 있어 Q2
필수 인계 조건으로 동결했다. 현재 판정은
[`task_m100_4969_w10_q1_schema_audit.md`](task_m100_4969_w10_q1_schema_audit.md)를 따른다.

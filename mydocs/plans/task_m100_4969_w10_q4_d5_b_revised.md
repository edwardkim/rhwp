# 수정 수행계획 — Task M100 #4969 W10-Q4-D5-B exact-source 준비 비용 정정

- **상위 계획**: [`task_m100_4969_w10_q4_d5.md`](task_m100_4969_w10_q4_d5.md)
- **D5-A checkpoint**: `e31c5c15d`
- **기계 판독 계획**:
  [`w10_q4_d5_b_correction_plan.json`](../tech/investigations/issue-4969/w10_q4_d5_b_correction_plan.json)
- **상태**: 원인·수정 범위 자동 승인
- **작성일**: 2026-08-31 KST
- **제품 변경**: 내부 exact-source 준비 경로만 정정

## 1. 수정이 필요한 이유

D5-B의 9개 독립 process A/B/A에서 correctness·resource hard gate는 모두 통과했지만 Rust B 경로는 A bracket으로
설명할 수 없는 반복 비용을 보였다.

- warm layout: B p50 1,142,878,416 ns / 64회, A bracket p50 평균 598,739 ns
- layer build: B p50 223,300,253 ns / 64회, A bracket p50 평균 132,002 ns
- actual CanvasKit draw: B p50 11,317,080 ns / 64회, A bracket p50 평균 11,016,064 ns
- actual pixel BLAKE3: A1/B/A2 9개 process 전부 동일

따라서 glyph replay나 CanvasKit draw가 원인이 아니다. 호출 계보를 대사한 결과 immutable registry source가 이미
등록 시 SHA-256으로 식별됐는데도 세로 layout 한 번마다 다음 작업을 반복한다.

1. `resolve_exact_font_source`가 전체 source SHA-256을 다시 계산한다.
2. `shape_bounded_request`의 validation이 같은 SHA-256과 face parse를 다시 수행한다.
3. vertical geometry가 동일 source face를 다시 parse한다.
4. layer publication이 매 build마다 전체 source BLAKE3와 face metadata를 다시 계산한다.

이는 font bytes나 resource 수가 누적되는 누수가 아니라, immutable source의 준비 결과를 owner certificate에 전달하지
않아 생긴 재계산이다.

## 2. 정정 경계

### D5-B1 — registry-owned source의 검증된 소비

- registry가 스스로 만든 exact handle과 immutable `Arc<[u8]>`의 일치만 허용하는 내부 accessor를 추가한다.
- 외부 provider의 source를 받는 기존 `resolve_exact_font_source` 신뢰 경계는 바꾸지 않는다.
- vertical context만 registry-owned accessor를 사용한다. key에 없는 forged/stale handle은 계속 fail-closed한다.

### D5-B2 — verified shaping·geometry 단일 준비

- 같은 `ttf_parser::Face`와 등록 시 확정된 SHA-256을 validation·geometry가 공유한다.
- `canonicalize_verified_shaping_request`와 `shape_canonical_request_with_face`를 사용해 전체 source 재해시를 없앤다.
- text/glyph/variation/vertical-metric 상한과 rejection enum은 그대로 유지한다.

### D5-B3 — certificate-bound portable metadata

- layout owner가 인증서를 만들 때 BLAKE3 resource digest와 face의 glyph count·weight·width·italic을 한 번 준비한다.
- paint publication은 인증서의 immutable metadata를 소비하되 source byte 길이, face index, units-per-em, digest 기반
  resource conflict 검사를 계속 수행한다.
- page마다 blob/face 각 1개, 64 MiB/64 source 상한과 atomic rollback을 유지한다.

## 3. 검증과 재계측

1. 기존 #4969 atomic integration 원본에 registry-owned accessor의 forged/stale rejection과 metadata identity receipt를
   추가한다. 새 integration source와 product `#[cfg(test)]` seam은 만들지 않는다.
2. focused #4969 계보, malformed/oversized/fallback, horizontal shaping 회귀를 실행한다.
3. release A/B/A를 다시 9 process 실행한다. 구조 hash/counter가 정정 전과 같고 B 비용 신호가 줄었는지 원자료와
   p50·p95를 다시 기록한다.
4. Studio actual CanvasKit은 product draw 경로가 변하지 않으므로 focused correctness 1회로 재확인하고, 기존 9회
   pixel/draw 계측은 그대로 인용한다.
5. 정정 source checkpoint 뒤 causal WASM A/B와 D5-C final gate로 진행한다.

wall-time 절대 SLA는 새로 만들지 않는다. 개선 여부는 같은 하네스의 정정 전/후 effect size로 공개하며 correctness와
resource 상한만 hard gate로 유지한다.

## 4. 보호 불변식

1. 지원 tuple, public API/schema, serialized output과 backend selector를 바꾸지 않는다.
2. no-source·malformed·stale generation·forged handle은 legacy fallback을 유지한다.
3. source SHA-256은 등록 신원이며 portable resource BLAKE3와 혼용하지 않는다.
4. raw font bytes는 `Arc`로만 공유하고 복제·경로 노출·직렬화하지 않는다.
5. layout 또는 backend에서 재-shape·재측정하지 않는다.
6. generated integration suite·manifest·Cargo marker는 제출하지 않는다.

## 5. 승인과 중단 조건

메인테이너의 2026-08-31 자동 승인 지시에 따라 이 수정 수행계획과 D5-B1~B3 진입을 자동 승인한다. 다음 중 하나가
발생하면 정정을 확대하지 않고 `blocked` 후보로 보고한다.

- 기존 output/hash/pixel 또는 fallback 결과 변화
- 외부 provider 신뢰 경계 완화 필요
- source/resource 상한 완화 필요
- Q4-D allowlist 밖 제품 변경 필요
- 9-run 원자료 불완전 또는 구조 counter mismatch

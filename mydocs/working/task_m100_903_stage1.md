# Task m100 #903 Stage 1 - embedded BinData RED 테스트

## 1. 목표

Stage 0에서 확인한 1차 원인 후보를 테스트로 고정한다.

```text
HWPX 원본: BinData 5개가 Embedding + loaded bytes
한컴 정답 HWP: BinData 5개가 Embedding + loaded bytes
rhwp HWP 저장 후 재로드: BinDataContent 0개
```

즉 `samples/hwpx/hwpx-h-01.hwpx`를 `export_hwp_with_adapter()`로 HWP 저장한 뒤 재로드했을 때,
한컴 정답 HWP처럼 embedded BinData 5개가 살아남아야 한다.

## 2. 추가한 RED 테스트

파일:

```text
tests/hwpx_to_hwp_adapter.rs
```

테스트:

```text
task903_hwpx_h_01_embedded_bindata_survives_hwp_save_reload
```

검증 흐름:

1. `samples/hwpx/hancom-hwp/hwpx-h-01.hwp`를 로드한다.
2. 정답 HWP가 BinData record 5개, loaded BinDataContent 5개, 각 record `Embedding`임을 확인한다.
3. `samples/hwpx/hwpx-h-01.hwpx`를 로드한다.
4. HWPX 원본이 image payload 5개를 로드했는지 확인한다.
5. `export_hwp_with_adapter()`로 HWP bytes를 만든다.
6. 저장본을 다시 `DocumentCore::from_bytes()`로 로드한다.
7. 재로드한 HWP가 정답 HWP처럼 embedded BinData 5개를 유지하는지 확인한다.

## 3. 실행 결과

명령:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_embedded_bindata_survives_hwp_save_reload -- --nocapture
```

결과:

```text
FAILED
```

실패 지점:

```text
assertion `left == right` failed: rhwp exported hwp: loaded BinDataContent count
  left: 0
 right: 5
```

해석:

```text
한컴 정답 HWP는 테스트 기준을 통과한다.
HWPX 원본도 payload 5개를 가지고 있다.
그러나 rhwp 저장본을 HWP로 재로드하면 BinDataContent가 0개가 된다.
```

따라서 Stage 0의 원인 후보는 테스트로 재현됐다.

## 4. ir-diff 보조 진단

`ir-diff`도 함께 확인했다.

### 원본 HWPX vs 한컴 정답 HWP

```text
cargo run --bin rhwp -- ir-diff samples/hwpx/hwpx-h-01.hwpx samples/hwpx/hancom-hwp/hwpx-h-01.hwp --summary
```

요약:

```text
=== 비교 완료: 차이 133 건 ===
65건 char_shapes count
23건 indent
7건 cc
5건 ml
5건 type
```

### 원본 HWPX vs rhwp adapter 저장본

```text
cargo run --bin rhwp -- ir-diff samples/hwpx/hwpx-h-01.hwpx output/poc/hwpx2hwp/task903/stage0/hwpx-h-01_adapter.hwp --summary
```

요약:

```text
=== 비교 완료: 차이 225 건 ===
106건 vpos
56건 char_shapes count
24건 tbl outer_margin
9건 tbl horz_rel
9건 tbl vert_rel
8건 tbl wrap
```

### 한컴 정답 HWP vs rhwp adapter 저장본

```text
cargo run --bin rhwp -- ir-diff samples/hwpx/hancom-hwp/hwpx-h-01.hwp output/poc/hwpx2hwp/task903/stage0/hwpx-h-01_adapter.hwp --summary
```

요약:

```text
=== 비교 완료: 차이 286 건 ===
106건 vpos
56건 id
26건 tbl outer_margin
23건 indent
9건 char_shapes count
9건 tbl horz_rel
9건 tbl vert_rel
8건 tbl wrap
```

판정:

```text
ir-diff는 한컴 정답 HWP와 rhwp 저장본 사이의 구조 차이를 보는 보조 도구로 유용하다.
다만 현재 핵심 RED인 BinDataContent 소실은 ir-diff summary에 직접 드러나지 않는다.
따라서 Stage 2의 1차 게이트는 embedded BinData 생존 테스트로 두고,
ir-diff는 수정 후 추가 구조 회귀 확인용으로 사용한다.
```

## 5. Stage 2 제안

최소 수정 후보:

```text
HWPX 출처 embedded BinData를 HWP로 저장하기 전에 attr/type/status를 HWP 직렬화 규약에 맞게 정규화한다.
```

현재 관찰:

```text
HWPX parser: data_type=Embedding, storage_id, extension 설정
HWPX parser: attr는 기본값 0으로 남음
HWP serializer: attr를 먼저 기록
HWP parser: attr & 0x000F == 0 이면 Link로 해석
```

정상 삽입 경로 기준:

```text
attr = 0x0101
data_type = Embedding
status = Success
compression = Default
```

Stage 2에서는 이 정규화를 어디에 넣을지 먼저 결정한다.

선호 후보:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

이유:

- HWPX 원본 IR 파싱 결과 자체를 바꾸기보다 HWP 저장 어댑터의 책임으로 국한한다.
- `export_hwp_with_adapter()` 경로에서만 적용된다.
- 기존 #854/#888 흐름과 맞다.

Stage 2 승인 후 최소 수정과 GREEN 테스트를 진행한다.

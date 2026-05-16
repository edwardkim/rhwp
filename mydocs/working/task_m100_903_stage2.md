# Task m100 #903 Stage 2 - embedded BinData 저장 정규화

## 1. 목표

Stage 1 RED 테스트를 GREEN으로 만든다.

문제:

```text
HWPX 원본/한컴 정답 HWP: embedded BinData 5개가 loaded
rhwp adapter 저장본 재로드: BinDataContent 0개
```

원인:

```text
HWPX 파서가 BinData의 data_type=Embedding, storage_id, extension은 채우지만 attr가 0으로 남는다.
HWP serializer는 attr를 먼저 기록한다.
HWP parser/Hancom은 attr & 0x000F == 0 을 Link로 해석한다.
```

## 2. 수정

파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

추가:

```text
normalize_embedded_bindata_for_hwp()
```

동작:

```text
loaded BinDataContent가 존재하는 embedded BinData에 한해
attr = 0x0101
data_type = Embedding
compression = Default
status = Success
raw_data = None
```

의미:

```text
attr low nibble 1: Embedding
attr status bits 8..9 = 1: Success
```

수정 위치는 HWPX→HWP 어댑터 내부로 한정했다. HWP 원본 저장 경로는 `convert_if_hwpx_source()`에서 계속 no-op이다.

## 3. 테스트

### #903 RED 테스트

명령:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_embedded_bindata_survives_hwp_save_reload -- --nocapture
```

결과:

```text
ok
1 passed
```

### HWPX→HWP 어댑터 통합 테스트 전체

명령:

```text
cargo test --test hwpx_to_hwp_adapter
```

결과:

```text
ok
32 passed
```

## 4. Stage 2 산출물

작업지시자 확인용 HWP:

```text
output/poc/hwpx2hwp/task903/stage2/hwpx-h-01_adapter.hwp
```

생성 결과:

```text
bytes: 680960
pages: 9
bindata_records: 5
bindata_content: 5
```

`rhwp info` 확인:

```text
BinData:
  [0] Embedding (ID: 1, ext: jpg, loaded: 3072 bytes)
  [1] Embedding (ID: 2, ext: png, loaded: 17181 bytes)
  [2] Embedding (ID: 3, ext: jpg, loaded: 26127 bytes)
  [3] Embedding (ID: 4, ext: jpg, loaded: 295915 bytes)
  [4] Embedding (ID: 5, ext: jpg, loaded: 80421 bytes)
```

## 5. ir-diff 보조 확인

명령:

```text
cargo run --bin rhwp -- ir-diff samples/hwpx/hancom-hwp/hwpx-h-01.hwp output/poc/hwpx2hwp/task903/stage2/hwpx-h-01_adapter.hwp --summary
```

결과:

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
BinData 손실 문제는 해결됐다.
다만 한컴 정답 HWP와 rhwp 저장본 사이의 레이아웃/표 관련 IR 차이는 남아 있다.
Stage 2 범위는 embedded image payload 생존 보장까지로 제한한다.
```

## 6. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 확인한다.

```text
output/poc/hwpx2hwp/task903/stage2/hwpx-h-01_adapter.hwp
```

판정 항목:

- 한컴 에디터에서 파일 손상 판정이 사라지는지
- rhwp-studio에서 다시 열리는지
- 앞쪽 표/그림이 출력되는지
- 9페이지 구성이 유지되는지
- 한컴 정답 HWP 대비 눈에 띄는 레이아웃 차이가 어디서 시작되는지

## 7. 다음 후보

시각 판정 결과에 따라 다음 중 하나로 분기한다.

```text
1. 파일 손상 판정이 사라짐:
   남은 차이는 ir-diff의 vpos/table/wrap/outer_margin 계열로 좁혀 Stage 3 진단

2. 파일 손상 판정이 여전함:
   BinData 외에 group picture / shape / table attr 쪽 손상 후보를 정답 HWP와 비교

3. 한컴은 열리나 그림 일부가 빠짐:
   grouped picture 내부 image_attr / natural size / crop 계열을 별도 Stage로 분리
```

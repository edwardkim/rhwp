# Task m100 #903 Stage 34 작업 기록

## 1. 목적

Stage33에서 `01_shape_common_attr_only.hwp`는 Stage30 positive control과 `ir-diff --summary` 기준 0건 차이였지만,
한컴 에디터에서는 여전히 파일 읽기 오류였다.

Stage34는 IR 비교를 중단하고, 같은 IR처럼 보이는 두 HWP 파일의 CFB/stream/record payload 차이를 확인한다.

## 2. 비교 대상

정상 positive:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

실패 candidate:

```text
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/01_shape_common_attr_only.hwp
```

## 3. 산출물

진단 리포트:

```text
output/poc/hwpx2hwp/task903/stage34_stream_compare/stage34_stream_compare.md
```

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage34_compare_streams_for_ir_equal_files -- --nocapture
```

결과:

```text
test task903_stage34_compare_streams_for_ir_equal_files ... ok
```

## 4. 파일/스트림 요약

| role | bytes | flags | compressed |
|---|---:|---:|---|
| Stage30 positive | 375808 | `0x00000001` | true |
| Stage33 failing | 374272 | `0x00000001` | true |

CFB stream 목록:

```text
positive streams: 9
failing streams: 9
union: 9
```

동일한 stream:

```text
/FileHeader
/BinData/BIN0001.jpg
/BinData/BIN0002.png
/BinData/BIN0003.jpg
/BinData/BIN0004.jpg
/BinData/BIN0005.jpg
```

다른 stream:

```text
/DocInfo
/BodyText/Section0
/BodyText/Section1
```

## 5. 중요한 관찰

### 5.1 CFB 컨테이너/stream 누락 문제는 아니다

두 파일은 같은 stream 목록을 갖는다.
`FileHeader`도 동일하고 압축 플래그도 동일하다.

```text
FileHeader: same bytes
flags: 0x00000001
compressed: true
```

따라서 Stage33 실패는 CFB stream 누락, FileHeader flags, compressed bit 문제가 아니다.

### 5.2 BinData 실제 이미지 바이트는 동일하다

모든 `/BinData/BINxxxx.ext` stream은 바이트 단위로 동일했다.

```text
/BinData/BIN0001.jpg same
/BinData/BIN0002.png same
/BinData/BIN0003.jpg same
/BinData/BIN0004.jpg same
/BinData/BIN0005.jpg same
```

따라서 rhwp-studio의 이미지 렌더링 실패는 실제 이미지 stream 손상이 아니라,
DocInfo의 `HWPTAG_BIN_DATA` metadata 또는 BodyText의 그림 참조 payload 문제로 보는 것이 맞다.

### 5.3 DocInfo는 record 수는 같지만 payload가 다르다

압축 해제 후:

| stream | positive bytes | failing bytes | positive records | failing records |
|---|---:|---:|---:|---:|
| `/DocInfo` | 26474 | 26474 | 523 | 523 |

차이:

```text
DocInfo record count는 같다.
하지만 BIN_DATA 5개와 PARA_SHAPE 다수의 data payload가 다르다.
```

주요 차이:

```text
records 2~6: BIN_DATA data byte0 = positive 0x01, failing 0x00
records 380~: PARA_SHAPE data payload 다수 차이
```

해석:

```text
1. HWPX -> HWP 경로에서 BinData metadata가 HWP 저장 기대값과 다르다.
2. Stage30에서 정상화된 ParaShape는 ir-diff가 보던 일부 필드보다 더 넓은 payload 차이를 갖는다.
```

### 5.4 BodyText는 record 수는 같지만 compact payload로 재생성되고 있다

압축 해제 후:

| stream | positive bytes | failing bytes | positive records | failing records |
|---|---:|---:|---:|---:|
| `/BodyText/Section0` | 225296 | 216725 | 7879 | 7879 |
| `/BodyText/Section1` | 3994 | 3928 | 88 | 88 |

대표 차이:

```text
Section0 record 4:  CTRL_HEADER 47 bytes vs 28 bytes
Section0 record 15: LIST_HEADER 65 bytes vs 34 bytes
Section0 record 16: PARA_HEADER 24 bytes vs 22 bytes
Section0 record 20: CTRL_HEADER 246 bytes vs 46 bytes
Section0 record 21: SHAPE_COMPONENT same size but data differs
Section0 record 22: SHAPE_PICTURE same size but data differs
```

해석:

```text
Stage33 failing은 모델에서 재직렬화된 compact payload를 쓴다.
Stage30 positive는 한컴 정답지 기반 graft 단계의 raw-tail/확장 payload가 남아 있다.
ir-diff는 이 raw-tail/확장 payload를 비교하지 않으므로 0건으로 보였지만, 한컴은 이 차이를 읽기 단계에서 민감하게 본다.
```

## 6. Stage33 결론 보정

Stage33에서 “IR diff 0”이라는 말은 정확하지만 충분하지 않았다.

정확한 표현은 다음이다.

```text
현재 ir-diff가 비교하는 IR 필드 기준으로는 0건이다.
하지만 HWP record payload 기준으로는 DocInfo/BodyText가 다르다.
```

따라서 Stage34 이후부터는 다음을 별도 축으로 둔다.

```text
1. 모델 IR 필드 차이
2. HWP record payload 차이
3. raw_data/raw_extra 보존 또는 합성 차이
```

## 7. 다음 단계 후보

Stage35에서는 무작정 광범위 probe를 만들지 않는다.
Stage34에서 실제로 갈라진 축만 분리한다.

우선순위:

```text
1. DocInfo BIN_DATA metadata 보정
   - 실제 BinData stream은 같지만 HWPTAG_BIN_DATA record payload가 다르다.
   - rhwp-studio 이미지 렌더링 실패와 직접 연결될 가능성이 높다.

2. DocInfo ParaShape payload 보정
   - Stage30의 "ParaShape" 정상화는 ir-diff 비교 범위보다 넓은 payload를 포함한다.
   - HWPX paraPr/margin 파싱만으로 충분한지 재검증해야 한다.

3. BodyText raw-tail/확장 payload 보존 또는 합성
   - CTRL_HEADER, LIST_HEADER, PARA_HEADER의 compact 재직렬화가 positive와 다르다.
   - 특히 LIST_HEADER 65/47 vs 34, CTRL_HEADER 47/246 vs 28/46 차이는 한컴 읽기 오류 후보이다.
```

Stage35의 첫 probe는 다음처럼 좁히는 것이 합리적이다.

```text
A. Stage33 failing + positive DocInfo BIN_DATA record payload만 graft
B. Stage33 failing + positive DocInfo ParaShape payload만 graft
C. Stage33 failing + A+B
D. Stage33 failing + positive BodyText Section0/1 header-tail class payload 일부
```

단, D는 구현 가능성이 낮은 reference graft이므로 A/B/C를 먼저 판정한다.


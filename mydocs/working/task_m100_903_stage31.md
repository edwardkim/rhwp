# Task m100 #903 Stage 31 Restart

## 1. 단계 목적

Stage30에서 확정한 두 구현 항목만 실제 코드 경로에 반영한 뒤,
`hwpx-h-01.hwpx` 전체 저장 결과를 다시 검증한다.

고정 구현 항목:

```text
1. HWPX -> HWP adapter에서 DocProperties.section_count를 실제 section 개수로 보정
2. HWPX header parser에서 paraPr/margin 자식 요소형 값을 ParaShape margin 필드로 매핑
```

이번 Stage31 restart는 table/object record 직렬화 문제를 해결하는 단계가 아니다.
먼저 Stage30 구현 기준선이 소스와 산출물에 정확히 반영되었는지 확인한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage31 restart 산출물:

```text
output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp
```

## 3. 내부 검증

Targeted tests:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01 -- --nocapture
cargo test --test hwpx_to_hwp_adapter task903_stage31_restart_generate_impl_verify -- --nocapture
```

검증 포인트:

```text
- DocProperties.section_count == document.sections.len()
- DocProperties.raw_data 제거
- DocInfo raw_stream_dirty 처리
- ParaShape margin child 값 파싱
- 정답 HWP와 주요 ParaShape margin 필드 일치
- rhwp-studio 재로드 기준 9페이지 유지
```

결과:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01 -- --nocapture
=> ok. 2 passed

cargo test --test hwpx_to_hwp_adapter task903_stage31_restart_generate_impl_verify -- --nocapture
=> ok. 1 passed

generated output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp
bytes=680960
pages=9
```

작업지시자 1차 판정:

```text
한컴 에디터: 파일 읽기 오류
파일 크기: 665K
```

Stage30 조건 재확인:

```text
Stage30 공통 기준선은 저장 시 HWP 압축 플래그를 켠다.
Stage31 restart 1차 산출물은 HWPX 파서가 만든 FileHeader를 그대로 사용하여
FileHeader.flags = 0, compressed = false 상태였다.
```

따라서 Stage31 restart 구현을 보정했다.

```text
HWPX -> HWP adapter에서 FileHeader.compressed = true
HWPX -> HWP adapter에서 FileHeader.flags compressed bit = 0x01
```

재검증:

```text
cargo test --test hwpx_to_hwp_adapter task903 -- --nocapture
=> ok. 3 passed

generated output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp
bytes=374272
pages=9
```

파일 크기 비교:

```text
samples/hwpx/hwpx-h-01.hwpx                         470K
samples/hwpx/hancom-hwp/hwpx-h-01.hwp               469K
output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp 366K
```

## 4. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp
```

판정 항목:

```text
- 한컴 에디터 파일 읽기 오류/파일손상 여부
- 한컴 에디터에서 9페이지 마지막 페이지가 출력되는지
- 표/셀 배치가 정상인지
- rhwp-studio에서 9페이지로 재로드되는지
```

판정 기록:

| 파일 | 한컴 판정 유형 | 한컴 출력 페이지 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| hwpx-h-01.hwp | 파일 읽기 오류 | - | - | - | - | 압축 헤더 보정 후 재생성. 366K |

## 5. 해석 기준

가능한 판정 해석:

```text
1. 한컴 정상 + 9페이지 + 표/셀 배치 정상
   => Stage30 구현 기준선으로 #903의 핵심 문제가 닫힘.

2. 한컴 정상 + 9페이지 + 표/셀 배치 비정상
   => ParaShape margin 파싱 또는 직렬화 반영을 재검토.

3. 한컴 정상 + 8페이지
   => DocProperties.section_count 보정 또는 DocInfo 재직렬화 반영을 재검토.

4. 한컴 파일 읽기 오류
   => Stage30 구현 항목과 압축 헤더 보정은 유지하되,
      다음 stage에서 별도 호환성 축으로 분석.
      table/object record 직렬화 문제로 즉시 확정하지 않는다.
```

## 6. 비범위

이번 Stage31 restart에서 다루지 않는 것:

```text
- serializer/control.rs의 table/object 호환성 추정 수정
- Stage32 이후 probe 대량 생성
- embedded BinData 정규화
- XML entity 텍스트 보존
- 꼬리말 페이지수 빨간색 문제
```

위 항목들은 필요 시 Stage31 판정 이후 별도 stage로 분리한다.

## 7. Stage31 판정 후 해석

압축 헤더 보정 후 파일 크기는 정상 범위로 내려갔다.

```text
Stage30 05_section_count_para_shapes_no_raw.hwp  367K
Stage31 restart hwpx-h-01.hwp                    366K
한컴 정답 hwpx-h-01.hwp                          469K
```

하지만 한컴 에디터 판정은 여전히 파일 읽기 오류다.
따라서 Stage31의 읽기 오류는 단순한 비압축/파일 크기 문제가 아니다.

한컴 정상인 Stage30 `05_section_count_para_shapes_no_raw`와
한컴 읽기 오류인 Stage31 restart 산출물을 `ir-diff --summary`로 비교했다.

```text
cargo run --bin rhwp -- ir-diff \
  output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp \
  output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp \
  --summary

50건  vpos
26건  tbl outer_margin
18건  id
 9건  tbl horz_rel
 9건  tbl vert_rel
 8건  tbl wrap
 5건  tbl page_break
 4건  cc
 4건  char_offsets len
 4건  pos
 4건  text
 3건  char_shapes count
 1건  shape horz_rel
 1건  shape vert_rel
 1건  shape wrap
```

한컴 정답 HWP와 Stage31 restart 산출물 비교도 같은 축을 가리킨다.

```text
106건 vpos
 56건 id
 26건 tbl outer_margin
  9건 char_shapes count
  9건 tbl horz_rel
  9건 tbl vert_rel
  8건 tbl wrap
  5건 tbl page_break
```

Stage31 결론:

```text
1. section_count 보정은 유지한다.
2. ParaShape margin child 파싱은 유지한다.
3. FileHeader compressed 보정은 유지한다.
4. 파일 읽기 오류의 다음 후보는 Stage30 정상 기준선에 있었지만
   Stage31 실제 adapter 경로에 아직 반영되지 않은 배치/표 컨트롤 축이다.
```

다음 Stage32는 Stage30 `05`를 positive control, Stage31 restart를 negative control로 놓고
잔여 차이 중 어떤 축이 한컴 파일 읽기 오류를 좌우하는지 최소 variant로 분리한다.

# Task m100 #903 Stage 30

## 1. 단계 목적

Stage 29에서 문제가 두 축으로 분리되었다.

```text
마지막 페이지 출력: DocProperties만으로 정상화
표/셀 배치: ParaShape만으로 정상화
```

Stage 30은 실제 구현에 들어가기 전에 더 작은 최소 조건을 확인한다.

```text
1. DocProperties 전체가 아니라 section_count 필드만으로 9페이지가 살아나는지
2. ParaShape를 raw_data 없이 재직렬화해도 셀 배치가 정상화되는지
3. section_count + ParaShape 조합만으로 완전 정상화되는지
```

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 30 산출물:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 공통 기준선

Stage 30 공통 기준선은 Stage 27의 `09_final_region_full_plus_section1_full_para0` 상태다.

추가로 저장 시 HWP 압축 플래그를 켠다.

모든 variant는 DocInfo를 재직렬화한다.

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_section_count_only | `DocProperties.section_count = sections.len()`만 적용 |
| 02_doc_properties_values_no_raw | 정답 HWP DocProperties 값 복사, `raw_data = None` |
| 03_para_shapes_no_raw_only | 정답 HWP ParaShape 복사, 각 ParaShape `raw_data = None` |
| 04_section_count_para_shapes_raw | `01` + 정답 HWP ParaShape raw_data 포함 복사 |
| 05_section_count_para_shapes_no_raw | `01` + 정답 HWP ParaShape 복사, raw_data 제거 |
| 06_section_count_layout_bundle_raw | `01` + BorderFill/CharShape/ParaShape/Style raw_data 포함 복사 |
| 07_section_count_layout_bundle_no_raw | `01` + BorderFill/CharShape/ParaShape/Style 복사, raw_data 제거 |
| 08_section_count_reference_model_all_no_raw | `01` + 정답 DocInfo 모델 전체 복사, 모든 raw_data 제거 |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/01_section_count_only.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/02_doc_properties_values_no_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/03_para_shapes_no_raw_only.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/04_section_count_para_shapes_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/06_section_count_layout_bundle_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/07_section_count_layout_bundle_no_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/08_section_count_reference_model_all_no_raw.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage30_generate_minimal_docinfo_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 59 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/01_section_count_only.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/02_doc_properties_values_no_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/03_para_shapes_no_raw_only.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/04_section_count_para_shapes_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/06_section_count_layout_bundle_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/07_section_count_layout_bundle_no_raw.hwp
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/08_section_count_reference_model_all_no_raw.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 페이지 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_section_count_only | 정상 | 정상 | 정상 | 일부 비정상 | 정상 | 꼬리말 페이지수 빨간색 (비정상, 기존 결함) |
| 02_doc_properties_values_no_raw | 정상 | 정상 | 정상 | 일부 비정상 | 정상 | 꼬리말 페이지수 빨간색 (비정상, 기존 결함) |
| 03_para_shapes_no_raw_only | 정상 | 8페이지 | 미출력 | 정상 | 정상 | 꼬리말 페이지수 빨간색 (비정상, 기존 결함) |
| 04_section_count_para_shapes_raw | 정상 | 정상 | 정상 | 정상 | 정상 | 꼬리말 페이지수 빨간색 (비정상, 기존 결함) |
| 05_section_count_para_shapes_no_raw | 정상 | 정상 | 정상 | 정상 | 정상 | 꼬리말 페이지수 빨간색 (비정상, 기존 결함) |
| 06_section_count_layout_bundle_raw | 정상 | 정상 | 정상 | 정상 | 정상 | 꼬리말 페이지수 빨간색 (비정상, 기존 결함) |
| 07_section_count_layout_bundle_no_raw | 정상 | 정상 | 정상 | 정상 | 정상 | 꼬리말 페이지수 빨간색 (비정상, 기존 결함) |
| 08_section_count_reference_model_all_no_raw | 정상 | 정상 | 정상 | 정상 | 정상 | 꼬리말 페이지수 빨간색 (비정상, 기존 결함) |

판정 포인트:

```text
- 01이 9페이지를 출력하는지: section_count 단독 원인 확인
- 03이 셀 배치를 정상화하는지: ParaShape 필드 재직렬화만으로 충분한지 확인
- 04와 05가 모두 정상인지: raw_data가 없어도 section_count + ParaShape 조합이 충분한지 확인
- 06/07/08은 ParaShape 단독으로 부족할 경우의 보조 판정
```

## 8. 현재 가설

현 시점의 구현 후보는 다음 두 가지다.

```text
1. HWPX -> HWP adapter에서 DocProperties.section_count를 실제 section 개수로 보정한다.
2. HWPX header 파서에서 ParaShape margin 계열 값을 누락 없이 파싱한다.
```

Stage 30 판정 후 `section_count`와 `ParaShape` 중 실제 코드 변경 범위를 확정한다.

## 9. 판정 해석

Stage 30 판정으로 원인이 두 필드군으로 확정되었다.

```text
마지막 9페이지 미출력: DocProperties.section_count 누락
표/셀 세로 배치 비정상: HWPX ParaShape margin 계열 값 누락
```

특히 `01_section_count_only`가 마지막 페이지를 회복했고, `03_para_shapes_no_raw_only`가
마지막 페이지는 회복하지 못했지만 표/셀 배치를 정상화했다. `05_section_count_para_shapes_no_raw`
이 정상 판정을 받았으므로, 구현 후보는 다음 최소 변경으로 좁힌다.

```text
1. HWPX -> HWP 저장 어댑터에서 DocProperties.section_count를 실제 section 개수로 보정한다.
2. HWPX header 파서에서 paraPr/margin의 자식 요소형 값을 ParaShape로 매핑한다.
```

꼬리말 페이지수 빨간색 현상은 처음부터 존재한 비정상 상태다. 정상 기준은 검정색이다.

## 10. Stage30 재시작 기준

Stage31 이후 탐색은 한동안 판단 기준에서 내린다.

다시 시작할 기준은 Stage30 판정으로 확정된 두 항목이다.

```text
1. 마지막 9페이지 미출력
   - 원인: DocProperties.section_count 누락
   - 구현 방향: HWPX -> HWP 저장 어댑터에서 실제 section 개수로 보정

2. 표/셀 세로 배치 비정상
   - 원인: HWPX ParaShape margin 계열 값 누락
   - 구현 방향: HWPX header 파서에서 paraPr/margin 자식 요소형 값을 ParaShape로 매핑
```

재시작 원칙:

```text
- Stage30의 결론은 유효한 기준선으로 고정한다.
- Stage31~36에서 생성한 추가 probe는 보류 자료로만 둔다.
- 다음 작업은 Stage30의 두 구현 항목이 실제 코드에 정확히 반영되었는지부터 다시 검증한다.
- 검증 산출물은 새 output 경로에 만들되, Stage30 결론과 섞지 않는다.
```

## 11. 결과 분석 기술 보고서

### 11.1 요약

Stage 30은 `samples/hwpx/hwpx-h-01.hwpx`의 HWP 저장 결과에서 관찰된 두 가지 현상을 분리했다.

```text
현상 A: 마지막 9페이지가 출력되지 않음
현상 B: 표/셀의 세로 배치가 비정상임
```

판정 결과, 두 현상은 서로 다른 원인에 의해 발생한다.

```text
현상 A의 직접 원인: DocProperties.section_count 누락 또는 오기록
현상 B의 직접 원인: HWPX ParaShape margin 계열 값 파싱 누락
```

따라서 Stage 30 기준의 최소 구현 범위는 다음 두 항목이다.

```text
1. HWPX -> HWP 저장 전 DocProperties.section_count를 실제 section 개수로 보정한다.
2. HWPX header 파서에서 paraPr/margin 자식 요소형 값을 ParaShape 필드로 매핑한다.
```

### 11.2 실험 설계의 핵심

Stage 30은 DocInfo 전체를 무작정 정답 HWP로 덮어쓰는 방식이 아니라, 후보 필드를 단계적으로 분리했다.

분리 축:

```text
- DocProperties.section_count 단독
- DocProperties 전체
- ParaShape 단독
- section_count + ParaShape
- section_count + layout bundle
- section_count + reference DocInfo 전체
```

이 설계의 목적은 다음과 같다.

```text
- 마지막 페이지 문제와 표/셀 배치 문제가 같은 원인인지 확인한다.
- DocInfo 전체 복사가 필요한지, 특정 필드 보정만으로 충분한지 확인한다.
- raw_data가 없어도 재직렬화 모델만으로 한컴 호환 결과가 나오는지 확인한다.
```

### 11.3 관찰 결과

`01_section_count_only`의 판정:

```text
한컴 정상
마지막 페이지 정상 출력
표/셀 배치는 일부 비정상
```

해석:

```text
section_count 단독 보정만으로 마지막 페이지 출력 문제는 해결된다.
하지만 표/셀 배치 문제는 해결되지 않는다.
```

`03_para_shapes_no_raw_only`의 판정:

```text
한컴 정상
마지막 페이지는 8페이지까지만 출력
표/셀 배치는 정상
```

해석:

```text
ParaShape 보정만으로 표/셀 배치 문제는 해결된다.
하지만 마지막 페이지 출력 문제는 해결되지 않는다.
```

`05_section_count_para_shapes_no_raw`의 판정:

```text
한컴 정상
마지막 페이지 정상 출력
표/셀 배치 정상
```

해석:

```text
section_count + ParaShape 조합만으로 Stage30의 두 핵심 현상이 동시에 해결된다.
ParaShape raw_data를 정답 HWP에서 그대로 가져올 필요는 없다.
구조화된 ParaShape 필드를 올바르게 채운 뒤 재직렬화하면 충분하다.
```

### 11.4 논리적 결론

Stage 30 판정표에서 다음 논리가 성립한다.

```text
01 정상 출력 + 일부 배치 비정상
=> section_count는 페이지 수 문제의 충분조건이지만 배치 문제의 충분조건은 아니다.

03 8페이지 출력 + 배치 정상
=> ParaShape는 배치 문제의 충분조건이지만 페이지 수 문제의 충분조건은 아니다.

05 정상 출력 + 배치 정상
=> section_count와 ParaShape를 함께 보정하면 두 현상이 모두 해결된다.
```

따라서 Stage30 기준 구현 결론은 다음처럼 고정한다.

```text
DocProperties.section_count:
  HWPX 파싱 결과 또는 변환 과정에서 실제 section 수와 일치하도록 보정해야 한다.

ParaShape margin:
  HWPX의 paraPr/margin이 속성형이 아니라 자식 요소형으로 표현되는 경우에도
  indent, margin_left, margin_right, spacing_before, spacing_after 등으로 매핑해야 한다.
```

### 11.5 비범위와 별도 결함

꼬리말 페이지수 빨간색 현상은 모든 Stage30 variant에서 공통으로 남았다.

```text
정상 기준: 검정색
현재 관찰: 빨간색
```

이 현상은 `section_count` 및 `ParaShape margin` 판정축과 독립적이다.

근거:

```text
- section_count 단독 variant에서도 발생
- ParaShape 단독 variant에서도 발생
- section_count + ParaShape 정상 조합에서도 발생
- reference DocInfo 전체 no raw variant에서도 발생
```

따라서 꼬리말 페이지수 색상 문제는 Stage30의 구현 범위에 포함하지 않고 별도 결함으로 분리한다.

### 11.6 구현 검증 기준

Stage30 결과를 코드에 반영한 뒤에는 다음 조건을 먼저 확인한다.

```text
1. HWPX -> HWP adapter 실행 후 DocProperties.section_count == document.sections.len()
2. DocProperties.raw_data가 남아 보정값 직렬화를 가로막지 않음
3. HWPX paraPr/margin 자식 요소형 값이 ParaShape margin 필드에 반영됨
4. 정답 HWP와 주요 ParaShape margin 필드가 일치함
5. 저장 산출물이 rhwp-studio 기준 9페이지로 재로드됨
```

한컴 에디터 시각 판정의 기대 결과:

```text
- 마지막 9페이지 출력
- 표/셀 세로 배치 정상
- 파일 읽기 오류 여부는 Stage30의 직접 판정축이 아니며,
  별도 문제로 관찰될 경우 별도 stage에서 분리 분석한다.
```

### 11.7 Stage31 파일 읽기 오류와의 관계

Stage31에서 만든 실제 adapter 산출물이 한컴 에디터에서 `파일 읽기 오류`를 낸 것은 Stage30 결론과 모순되지 않는다.

이유는 Stage30의 공통 기준선이 clean adapter 산출물이 아니었기 때문이다.

Stage30의 공통 기준선:

```text
Stage27 09_final_region_full_plus_section1_full_para0
```

이 기준선에는 Stage27까지의 탐색 과정에서 이미 여러 BodyText table/object record materialization이 들어가 있었다.

즉 Stage30이 검증한 명제는 다음이다.

```text
Stage27 baseline 수준의 BodyText/table/object 구조가 갖춰져 있을 때,
DocProperties.section_count와 ParaShape margin을 보정하면
마지막 페이지 출력과 표/셀 배치 문제가 해결된다.
```

Stage30이 검증하지 않은 명제:

```text
clean HWPX -> HWP adapter 산출물에
section_count와 ParaShape만 보정하면
한컴 에디터가 파일을 읽을 수 있다.
```

Stage31은 바로 이 검증하지 않은 명제를 실제 구현 경로에서 확인한 단계였다.

Stage31 결과:

```text
section_count 보정 적용
ParaShape margin 파싱 적용
하지만 한컴 에디터는 파일 읽기 오류
```

따라서 Stage31 파일 읽기 오류의 의미는 다음과 같다.

```text
Stage30에서 찾은 두 원인:
  - 페이지 수 문제
  - 표/셀 배치 문제
는 유효하다.

하지만 clean adapter 경로에는 별도의 문제:
  - BodyText table/object record tuple의 한컴 호환 직렬화 부족
가 추가로 남아 있다.
```

정리:

```text
Stage30 결론은 페이지 수/표 배치 문제의 원인 분석이다.
Stage31 파일 읽기 오류는 clean adapter 저장 경로의 별도 호환성 문제다.
두 문제는 같은 파일에서 연속으로 관찰되었지만, 원인 축은 다르다.
```

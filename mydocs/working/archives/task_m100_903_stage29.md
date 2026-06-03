# Task m100 #903 Stage 29

## 1. 단계 목적

Stage 28 결과로 마지막 페이지 누락과 셀 배치 이상은 DocInfo 계열 문제로 좁혀졌다.

하지만 Stage 28의 `03_reference_docinfo`는 정답 HWP의 `DocInfo.raw_stream`까지 함께 복사했을 가능성이 높다.

따라서 Stage 29는 다음을 분리한다.

```text
1. 정답 DocInfo 모델을 raw_stream 없이 재직렬화해도 한컴에서 정상화되는지
2. 정상화된다면 DocInfo 내부 어느 필드 묶음이 결정적인지
3. 정상화되지 않는다면 raw stream 또는 미모델링 extra record/layout 문제가 맞는지
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

Stage 29 산출물:

```text
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 공통 기준선

Stage 29 공통 기준선은 Stage 27의 `09_final_region_full_plus_section1_full_para0` 상태다.

추가로 저장 시 HWP 압축 플래그를 켠다.

모든 variant는 `DocInfo.raw_stream = None`, `DocInfo.raw_stream_dirty = true`를 설정하여 DocInfo를 재직렬화한다.

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_reference_model_all_reserialized | 정답 HWP의 DocInfo 모델 전체 + DocProperties를 복사한 뒤 raw_stream 없이 재직렬화 |
| 02_doc_properties_only | DocProperties만 정답 HWP에서 복사 |
| 03_font_faces_only | font_faces만 정답 HWP에서 복사 |
| 04_border_fills_only | border_fills만 정답 HWP에서 복사 |
| 05_char_shapes_only | char_shapes만 정답 HWP에서 복사 |
| 06_para_shapes_only | para_shapes만 정답 HWP에서 복사 |
| 07_styles_only | styles만 정답 HWP에서 복사 |
| 08_layout_bundle | border_fills + char_shapes + para_shapes + styles 복사 |
| 09_tabs_numbering_bullets | tab_defs + numberings + bullets + bullet/memo count 복사 |
| 10_bin_data_list_only | bin_data_list만 정답 HWP에서 복사 |
| 11_extra_records_only | extra_records만 정답 HWP에서 복사 |
| 12_counts_extra_records_only | bullet_count + memo_shape_count + extra_records 복사 |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/01_reference_model_all_reserialized.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/02_doc_properties_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/03_font_faces_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/04_border_fills_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/05_char_shapes_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/06_para_shapes_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/07_styles_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/08_layout_bundle.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/09_tabs_numbering_bullets.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/10_bin_data_list_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/11_extra_records_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/12_counts_extra_records_only.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage29_generate_docinfo_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 58 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/01_reference_model_all_reserialized.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/02_doc_properties_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/03_font_faces_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/04_border_fills_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/05_char_shapes_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/06_para_shapes_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/07_styles_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/08_layout_bundle.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/09_tabs_numbering_bullets.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/10_bin_data_list_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/11_extra_records_only.hwp
output/poc/hwpx2hwp/task903/stage29_docinfo_probe/12_counts_extra_records_only.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 페이지 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_reference_model_all_reserialized | 정상 | 정상 | 정상 | 정상 | 정상 |  |
| 02_doc_properties_only | 정상 | 정상 | 정상 | 일부 비정상 | 정상 |  |
| 03_font_faces_only | 정상 | 8페이지 | 미출력 | 일부 비정상 | 정상 |  |
| 04_border_fills_only | 정상 | 8페이지 | 미출력 | 일부 비정상 | 정상 |  |
| 05_char_shapes_only | 정상 | 8페이지 | 미출력 | 일부 비정상 | 정상 |  |
| 06_para_shapes_only | 정상 | 8페이지 | 미출력 | 정상 | 정상 |  |
| 07_styles_only | 정상 | 8페이지 | 미출력 | 일부 비정상 | 정상 |  |
| 08_layout_bundle | 정상 | 8페이지 | 미출력 | 정상 | 정상 |  |
| 09_tabs_numbering_bullets | 정상 | 8페이지 | 미출력 | 일부 비정상 | 정상 |  |
| 10_bin_data_list_only | 정상 | 8페이지 | 미출력 | 일부 비정상 | 정상 |  |
| 11_extra_records_only | 정상 | 8페이지 | 미출력 | 일부 비정상 | 정상 |  |
| 12_counts_extra_records_only | 정상 | 8페이지 | 미출력 | 일부 비정상 | 정상 |  |

판정 포인트:

```text
- 01이 성공하는지: 성공하면 raw stream이 아니라 모델 필드 값 문제
- 01이 실패하고 Stage 28의 07만 성공하면 raw DocInfo layout/미모델링 레코드 문제
- 단일 필드 variant 중 성공하는 것이 있는지
- 08 layout_bundle이 성공하면 레이아웃 참조 테이블 묶음 문제
- 11/12가 성공하면 extra_records 또는 ID_MAPPINGS 보존 count 문제
```

## 8. 현재 가설

Stage 28에서 DocInfo raw graft가 완전히 정상화했다.

Stage 29는 DocInfo 모델 재직렬화가 그 효과를 재현할 수 있는지 확인한다.

만약 `01_reference_model_all_reserialized`가 실패하면, 현재 serializer의 DocInfo record 순서/미모델링 레코드 보존/ID_MAPPINGS 세부 필드가 한컴 호환성에 부족하다는 뜻이다.

## 9. 판정 후 해석

Stage 29로 문제가 두 축으로 분리되었다.

```text
1. 마지막 페이지 출력:
   - 02_doc_properties_only 성공
   - 따라서 DocProperties가 직접 원인이다.
   - HWPX parser는 section_count를 1로 기본 설정한다.
   - 실제 문서는 2개 section이므로, 한컴은 DOCUMENT_PROPERTIES의 section_count를 신뢰하여 1개 section만 표시한 것으로 보인다.

2. 표/셀 배치:
   - 06_para_shapes_only 성공
   - 08_layout_bundle 성공
   - 따라서 표/셀의 세로 배치 이상은 ParaShape 계열이 직접 원인이다.
```

즉 마지막 페이지 누락은 `DocProperties.section_count`, 셀 배치 이상은 `ParaShape` 쪽으로 보는 것이 합리적이다.

단, Stage 29의 `02_doc_properties_only`와 `06_para_shapes_only`는 정답 HWP 객체를 복사했기 때문에 `raw_data`도 함께 복사되었을 수 있다.

Stage 30에서는 다음을 확인한다.

```text
- section_count 필드 값만 고쳐도 9페이지가 출력되는지
- ParaShape를 raw_data 없이 필드 재직렬화해도 셀 배치가 정상화되는지
- section_count + ParaShape 조합만으로 01_reference_model_all_reserialized와 같은 효과가 나는지
```

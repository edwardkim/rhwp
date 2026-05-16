# Task m100 #903 Stage 28

## 1. 단계 목적

Stage 27에서도 모든 variant가 한컴에서 정상 열림 상태를 유지했지만 8페이지만 출력했다.

따라서 마지막 페이지 누락은 paragraph/table payload 단위 문제가 아니라 HWP container 또는 BodyText stream 레벨 문제일 가능성이 높아졌다.

Stage 28은 다음을 직접 probe한다.

```text
1. 생성 HWP를 압축 HWP로 저장하면 마지막 페이지가 출력되는지
2. 정답 HWP의 FileHeader/DocInfo를 적용하면 마지막 페이지가 출력되는지
3. 정답 HWP의 BodyText/Section0, BodyText/Section1 raw stream을 graft하면 마지막 페이지가 출력되는지
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

Stage 28 산출물:

```text
output/poc/hwpx2hwp/task903/stage28_container_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 관찰

정답 HWP:

```text
버전: 5.1.0.1
압축: 예
구역 수: 2
페이지 수: 9
```

Stage 27 생성 HWP:

```text
버전: 5.1.0.0
압축: 아니오
구역 수: 2
페이지 수: 9
```

rhwp 내부 parser는 Stage 27 생성 HWP에서 2개 구역과 9페이지를 읽는다.

그러나 한컴은 8페이지만 표시한다.

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_compressed_header_only | Stage 27 `09` 기준선에서 FileHeader 압축 플래그를 켜고 stream 압축 저장 |
| 02_reference_file_header | Stage 27 `09` 기준선에 정답 HWP FileHeader 적용 |
| 03_reference_docinfo | Stage 27 `09` 기준선에 정답 HWP DocInfo 적용 + 압축 저장 |
| 04_reference_file_header_docinfo | Stage 27 `09` 기준선에 정답 HWP FileHeader + DocInfo 적용 |
| 05_raw_graft_bodytext_section1 | 압축 기준선에 정답 HWP `/BodyText/Section1` raw stream graft |
| 06_raw_graft_bodytext_section0 | 압축 기준선에 정답 HWP `/BodyText/Section0` raw stream graft |
| 07_raw_graft_docinfo | 압축 기준선에 정답 HWP `/DocInfo` raw stream graft |
| 08_raw_graft_bodytext_section0_section1 | 압축 기준선에 정답 HWP `/BodyText/Section0`, `/BodyText/Section1` raw stream graft |
| 09_raw_graft_docinfo_bodytext | 압축 기준선에 정답 HWP `/DocInfo`, `/BodyText/Section0`, `/BodyText/Section1` raw stream graft |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage28_container_probe/01_compressed_header_only.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/02_reference_file_header.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/03_reference_docinfo.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/04_reference_file_header_docinfo.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/05_raw_graft_bodytext_section1.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/06_raw_graft_bodytext_section0.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/07_raw_graft_docinfo.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/08_raw_graft_bodytext_section0_section1.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/09_raw_graft_docinfo_bodytext.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage28_generate_container_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 57 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage28_container_probe/01_compressed_header_only.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/02_reference_file_header.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/03_reference_docinfo.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/04_reference_file_header_docinfo.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/05_raw_graft_bodytext_section1.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/06_raw_graft_bodytext_section0.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/07_raw_graft_docinfo.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/08_raw_graft_bodytext_section0_section1.hwp
output/poc/hwpx2hwp/task903/stage28_container_probe/09_raw_graft_docinfo_bodytext.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 페이지 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_compressed_header_only | 정상 | 8페이지 까지만 출력 | 미출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 02_reference_file_header | 정상 | 8페이지 까지만 출력 | 미출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 03_reference_docinfo | 정상 | 정상 | 정상 | 정상 | 정상 | 셀내 컬럼 배치도 정상 |
| 04_reference_file_header_docinfo | 정상 | 정상 | 정상 | 정상 | 정상 | 셀내 컬럼 배치도 정상 |
| 05_raw_graft_bodytext_section1 | 정상 | 8페이지 까지만 출력 | 미출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 06_raw_graft_bodytext_section0 | 정상 | 8페이지 까지만 출력 | 미출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 07_raw_graft_docinfo | 정상 | 정상 | 정상 | 정상 | 정상 | 셀내 컬럼 배치도 정상 |
| 08_raw_graft_bodytext_section0_section1 | 정상 | 8페이지 까지만 출력 | 미출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 09_raw_graft_docinfo_bodytext | 정상 | 정상 | 정상 | 정상 | 정상 | 셀내 컬럼 배치도 정상 |

판정 포인트:

```text
- 한컴에서 9페이지 마지막 참고 표가 출력되는지
- 01 또는 02에서 살아나면 compression/FileHeader 계열 문제
- 03 또는 04에서 살아나면 DocInfo 계열 문제
- 05에서 살아나면 Section1 stream 직렬화 문제
- 06에서 살아나면 Section0 마지막 boundary/stream 직렬화 문제
- 08 또는 09에서만 살아나면 BodyText stream 간 경계/조합 문제
```

## 8. 현재 가설

Stage 27까지의 결과로 보아 IR에 section 1 내용은 존재한다.

한컴이 마지막 페이지를 표시하지 않는다면 다음 중 하나일 가능성이 높다.

```text
1. 생성 HWP의 비압축 multi-section BodyText를 한컴이 마지막 section까지 해석하지 못한다.
2. Section0 또는 Section1 stream의 레코드 경계가 한컴 호환 조건과 다르다.
3. DocInfo/FileHeader의 특정 플래그 또는 raw stream layout이 section 표시 조건에 영향을 준다.
```

## 9. 판정 후 해석

Stage 28 결과로 원인은 DocInfo 계열로 좁혀졌다.

```text
- 01 압축만 적용: 실패
- 02 FileHeader만 적용: 실패
- 05 BodyText/Section1 raw graft: 실패
- 06 BodyText/Section0 raw graft: 실패
- 08 BodyText/Section0+1 raw graft: 실패
- 03 reference DocInfo 적용: 성공
- 04 reference FileHeader+DocInfo 적용: 성공
- 07 raw DocInfo graft: 성공
- 09 raw DocInfo+BodyText graft: 성공
```

따라서 마지막 페이지 누락과 셀 배치 이상은 BodyText stream 문제가 아니라 DocInfo stream 문제다.

단, `03_reference_docinfo`는 모델의 `DocInfo`와 `DocProperties`를 정답 HWP에서 복사했지만 `DocInfo.raw_stream`도 함께 보존되었을 가능성이 높다.

따라서 아직은 raw DocInfo stream 자체가 필요한지, 모델 필드 값만 맞추면 되는지 확정할 수 없다.

Stage 29는 DocInfo 내부 요소를 세분화한다.

후보:

```text
1. 정답 DocInfo 모델을 raw_stream 없이 재직렬화해도 성공하는지
2. DocProperties
3. FontFace
4. BorderFill
5. CharShape
6. ParaShape
7. Style
8. TabDef/Numbering/Bullet
9. ExtraRecords / raw_stream layout
```

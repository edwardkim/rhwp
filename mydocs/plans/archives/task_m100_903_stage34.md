# Task m100 #903 Stage 34 계획

## 1. 목적

Stage33에서 다음 사실이 확인되었다.

```text
Stage33 01_shape_common_attr_only.hwp
  - Stage30 positive control과 전체 IR diff 0건
  - 한컴 에디터에서는 파일 읽기 오류
  - rhwp-studio에서는 열리지만 이미지 렌더링 실패
```

따라서 Stage34는 IR 모델 필드 비교를 중단하고,
같은 IR을 가진 두 HWP 파일의 CFB/stream/BinData 저장 차이를 찾는 단계로 전환한다.

## 2. 비교 대상

정상 positive:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

실패 candidate:

```text
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/01_shape_common_attr_only.hwp
```

## 3. 분석 항목

다음 항목을 파일 단위/stream 단위로 비교한다.

```text
1. 전체 파일 크기와 hash
2. CFB stream 목록
3. 각 stream raw size/hash
4. DocInfo, BodyText/SectionN의 압축 해제 후 size/hash
5. BinData stream raw size/hash
6. 같은 stream의 첫 번째 byte diff offset
```

## 4. 산출물

진단 리포트:

```text
output/poc/hwpx2hwp/task903/stage34_stream_compare/stage34_stream_compare.md
```

작업 기록:

```text
mydocs/working/task_m100_903_stage34.md
```

## 5. 판정 기준

```text
- stream 목록이 다르면 누락/추가 stream을 우선 본다.
- stream 목록은 같지만 raw hash가 다르면 해당 stream을 최소 후보로 둔다.
- raw hash가 달라도 decompressed hash가 같으면 CFB/압축/저장 방식 문제로 본다.
- BinData 차이가 있으면 rhwp-studio 이미지 렌더링 실패와 연결해서 우선순위를 올린다.
- DocInfo/BodyText decompressed 차이가 있으면 Stage30의 IR diff 0과 모순되는지 재확인한다.
```

## 6. 하지 않을 것

```text
- 새 HWP variant 생성
- adapter/serializer 구현 변경
- Stage30 결론 폐기
```


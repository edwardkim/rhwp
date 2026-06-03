# Task m100 #903 Stage 21

## 1. 단계 목적

Stage 20에서 문단 `0:28` 담당자 표의 `TABLE record + encoded zone tail`이 들어가면 한컴 출력 경계가 이동했다.

하지만 판정 유형은 `파일손상`에서 `파일 읽기 오류`로 바뀌었고, 출력 위치는 `2페이지 마지막 개체 묶기 전`이었다.

Stage 21은 다음 경계인 문단 `0:29` 로고/배너 묶음 그림을 대상으로 한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 21 산출물:

```text
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:28 "※ 2024년 해외직접투자 ..." + 담당자 표, 2행 x 6열
문단 0:29 로고/배너 묶음 그림, child picture 3개
문단 0:30 별첨 제목 표, 1행 x 3열, 쪽나누기
```

Stage 21 공통 기준선은 Stage 20의 `06_notice_table_full_object_with_tail` 상태다.

즉 다음이 이미 포함된다.

```text
문단 0:28 paragraph record
문단 0:28 담당자 표 full object with encoded zone tail
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_logo_group_para_record | 문단 `0:29` paragraph record |
| 02_logo_group_full_object_without_para_record | 문단 `0:29` 묶음 그림 full object, paragraph record 제외 |
| 03_logo_group_common_attr | `01` + 묶음 그림 common attr |
| 04_logo_group_shape_attr | `01` + 묶음 그림 shape attr |
| 05_logo_group_common_shape_child_shape_attrs | `01` + 묶음 그림 common/shape attr + child picture shape attrs |
| 06_logo_group_common_shape_child_full_pictures | `01` + 묶음 그림 common/shape attr + child picture full objects |
| 07_logo_group_full_tuple | `01` + 묶음 그림 full object |
| 08_logo_group_full_tuple_plus_attachment_title_table | `07` + 문단 `0:30` 별첨 제목 paragraph/table full object |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/01_logo_group_para_record.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/02_logo_group_full_object_without_para_record.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/03_logo_group_common_attr.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/04_logo_group_shape_attr.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/05_logo_group_common_shape_child_shape_attrs.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/06_logo_group_common_shape_child_full_pictures.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/07_logo_group_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/08_logo_group_full_tuple_plus_attachment_title_table.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage21_generate_logo_group_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 50 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/01_logo_group_para_record.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/02_logo_group_full_object_without_para_record.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/03_logo_group_common_attr.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/04_logo_group_shape_attr.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/05_logo_group_common_shape_child_shape_attrs.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/06_logo_group_common_shape_child_full_pictures.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/07_logo_group_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage21_logo_group_probe/08_logo_group_full_tuple_plus_attachment_title_table.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 묶음 그림 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_logo_group_para_record | 파일 읽기 오류 | 2페이지 마지막 개체 묶기 전까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 02_logo_group_full_object_without_para_record | 파일손상 | 3페이지 `□ 국가별 동향(상위 5개)`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 03_logo_group_common_attr | 파일 읽기 오류 | 2페이지 마지막 개체 묶기 전까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 04_logo_group_shape_attr | 파일 읽기 오류 | 2페이지 마지막 개체 묶기 전까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 05_logo_group_common_shape_child_shape_attrs | 파일손상 | 3페이지 `□ 국가별 동향(상위 5개)`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 2페이지 개체 묶음 미출력 |  |
| 06_logo_group_common_shape_child_full_pictures | 파일손상 | 3페이지 `□ 국가별 동향(상위 5개)`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 07_logo_group_full_tuple | 파일손상 | 3페이지 `□ 국가별 동향(상위 5개)`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 08_logo_group_full_tuple_plus_attachment_title_table | 파일손상 | 3페이지 `□ 국가별 동향(상위 5개)`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |

판정 포인트:

```text
- 판정 유형이 파일 읽기 오류에서 파일손상 또는 정상 열림으로 바뀌는지
- 문단 0:29 로고/배너 묶음 그림이 출력되는지
- 문단 0:30 별첨 제목 표까지 출력 경계가 이동하는지
- 08에서 별첨 제목 표가 출력되는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 판정 해석

### 8.1 묶음 그림 경계 통과 조건

문단 `0:29`의 paragraph record, group common attr, group shape attr 단독으로는 경계를 넘지 못했다.

```text
01_logo_group_para_record
03_logo_group_common_attr
04_logo_group_shape_attr
```

이 세 variant는 모두 `파일 읽기 오류`이며 출력 위치도 `2페이지 마지막 개체 묶기 전`에 머문다.

반대로 다음 variant들은 `파일 읽기 오류` 경계를 통과했다.

```text
02_logo_group_full_object_without_para_record
05_logo_group_common_shape_child_shape_attrs
06_logo_group_common_shape_child_full_pictures
07_logo_group_full_tuple
08_logo_group_full_tuple_plus_attachment_title_table
```

이들은 판정 유형이 `파일손상`으로 바뀌고, 한컴 출력 경계가 `3페이지 □ 국가별 동향(상위 5개)`까지 이동했다.
즉 문단 `0:29` 묶음 그림은 paragraph record보다 group object/child picture payload 쪽이 핵심이다.

### 8.2 부분 payload의 위험

`05_logo_group_common_shape_child_shape_attrs`는 한컴 경계를 이동시키지만 rhwp-studio에서 2페이지 개체 묶음이 미출력된다.

```text
05 = group common/shape attr + child picture shape attrs
```

이는 Stage 19의 `07`과 같은 종류의 신호다.
한컴 경계 이동에는 유효하지만 rhwp-studio 렌더링이 깨지므로 production 기준선으로 쓰면 안 된다.

기준선 후보는 rhwp-studio가 정상인 다음 variant다.

```text
06_logo_group_common_shape_child_full_pictures
07_logo_group_full_tuple
08_logo_group_full_tuple_plus_attachment_title_table
```

### 8.3 다음 경계

Stage 21에서 한컴 출력 경계는 `3페이지 □ 국가별 동향(상위 5개)`까지 이동했다.

정답 HWP 기준으로 관련 문단은 다음 부근이다.

```text
문단 0:39 "□ 업종별 동향(상위 5개)"
문단 0:41 업종별 표
문단 0:43 "□ 국가별 동향(상위 5개)"
문단 0:44 국가별 표
```

다음 손상 경계는 문단 `0:43` 또는 그 직후 문단 `0:44` 국가별 표로 본다.

## 9. 다음 단계

Stage 22는 3페이지 `□ 국가별 동향(상위 5개)` 주변을 대상으로 한다.

우선순위:

```text
1. Stage 21 `07_logo_group_full_tuple` 또는 `08_logo_group_full_tuple_plus_attachment_title_table`를 기준선으로 사용
2. 문단 0:39~0:44의 제목/표 tuple을 정답 HWP와 비교
3. `TABLE record + encoded zone tail` 패턴이 다시 반복되는지 확인
4. 파일손상 경계가 다음 페이지/다음 블록으로 이동하는지 확인
```

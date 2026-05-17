# Task m100 #903 Stage 20

## 1. 단계 목적

Stage 19 판정에서 지역별 표 이후로 한컴 출력 경계가 이동했다.

```text
05_region_table_full_object_with_tail
06_region_host_para_plus_table_full_tuple
08_region_full_tuple_plus_following_text_para
```

이 variant들은 `"※ 2024년 해외직접투자"` 직전까지 출력되었다.

Stage 20은 다음 손상 경계인 문단 `0:28`을 확인한다.
이 문단은 일반 텍스트와 담당자 표가 같은 paragraph 안에 들어 있는 복합 케이스다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 20 산출물:

```text
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:26 빈 문단
문단 0:27 빈 문단
문단 0:28 "※ 2024년 해외직접투자 ..." + 담당자 표, 2행 x 6열
문단 0:29 로고/배너 묶음 그림
문단 0:30 별첨 제목 표, 1행 x 3열, 쪽나누기
```

Stage 20 공통 기준선은 Stage 19의 `08_region_full_tuple_plus_following_text_para` 상태다.

즉 다음이 이미 포함된다.

```text
문단 0:23 지역별 표 full object with encoded zone tail
문단 0:24 빈 문단 paragraph record
문단 0:25 지역별 동향 다음 본문 paragraph record
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_pre_notice_empty_paras | 문단 `0:26`, `0:27` 빈 문단 paragraph record |
| 02_notice_para_record | `01` + 문단 `0:28` paragraph record |
| 03_notice_table_ctrl_header | `02` + 문단 `0:28` 담당자 표 CTRL_HEADER |
| 04_notice_table_record_with_tail | `02` + 문단 `0:28` 담당자 표 TABLE record + encoded zone tail |
| 05_notice_table_all_cell_headers | `02` + 문단 `0:28` 담당자 표 전체 cell LIST_HEADER/PARA_HEADER |
| 06_notice_table_full_object_with_tail | `02` + 문단 `0:28` 담당자 표 full object + encoded zone tail |
| 07_notice_table_full_object_without_para_record | `01` + 문단 `0:28` 담당자 표 full object, paragraph record 제외 |
| 08_notice_full_tuple_plus_logo_group_para_record | `06` + 문단 `0:29` 로고/배너 paragraph record |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/01_pre_notice_empty_paras.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/02_notice_para_record.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/03_notice_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/04_notice_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/05_notice_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/06_notice_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/07_notice_table_full_object_without_para_record.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/08_notice_full_tuple_plus_logo_group_para_record.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage20_generate_notice_boundary_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 49 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/01_pre_notice_empty_paras.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/02_notice_para_record.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/03_notice_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/04_notice_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/05_notice_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/06_notice_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/07_notice_table_full_object_without_para_record.hwp
output/poc/hwpx2hwp/task903/stage20_notice_boundary_probe/08_notice_full_tuple_plus_logo_group_para_record.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 셀 세로 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_pre_notice_empty_paras | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. `"※ 2024년 해외직접투자"` 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 02_notice_para_record | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. `"※ 2024년 해외직접투자"` 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 03_notice_table_ctrl_header | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. `"※ 2024년 해외직접투자"` 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 04_notice_table_record_with_tail | 파일 읽기 오류 | 2페이지 마지막 개체 묶기 전까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 05_notice_table_all_cell_headers | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. `"※ 2024년 해외직접투자"` 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 06_notice_table_full_object_with_tail | 파일 읽기 오류 | 2페이지 마지막 개체 묶기 전까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 07_notice_table_full_object_without_para_record | 파일 읽기 오류 | 2페이지 마지막 개체 묶기 전까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 08_notice_full_tuple_plus_logo_group_para_record | 파일 읽기 오류 | 2페이지 마지막 개체 묶기 전까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |

판정 포인트:

```text
- 한컴 출력 경계가 "※ 2024년 해외직접투자" 이후로 이동하는지
- 담당자 표가 출력되는지
- 문단 0:29 로고/배너 묶음 그림 또는 문단 0:30 별첨 표까지 출력되는지
- Stage 19까지의 본문/표 배치가 유지되는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 판정 해석

### 8.1 파일손상과 파일 읽기 오류 분리

Stage 20에서는 판정 유형이 두 갈래로 나뉜다.

파일손상으로 남은 variant:

```text
01_pre_notice_empty_paras
02_notice_para_record
03_notice_table_ctrl_header
05_notice_table_all_cell_headers
```

파일 읽기 오류로 바뀐 variant:

```text
04_notice_table_record_with_tail
06_notice_table_full_object_with_tail
07_notice_table_full_object_without_para_record
08_notice_full_tuple_plus_logo_group_para_record
```

이전 단계에서 확인한 것처럼 파일손상과 파일 읽기 오류는 다른 종류의 문제다.
따라서 Stage 20의 `04/06/07/08`은 "해결"이 아니라 한컴 파서가 더 뒤까지 진행한 뒤 다른 오류 유형으로 실패한 상태로 기록한다.

### 8.2 경계 이동 신호

다음 payload가 들어간 variant에서 출력 경계가 `"※ 2024년 해외직접투자"` 전에서 `2페이지 마지막 개체 묶기 전`으로 이동했다.

```text
04_notice_table_record_with_tail
06_notice_table_full_object_with_tail
07_notice_table_full_object_without_para_record
```

즉 문단 `0:28`의 담당자 표에서도 이전 stage들과 같은 패턴이 반복된다.

```text
TABLE record + encoded zone tail
```

이 payload가 한컴 파서의 표 경계를 전진시키는 핵심 신호다.

### 8.3 다음 경계

`08_notice_full_tuple_plus_logo_group_para_record`에서도 파일 읽기 오류 위치가 `2페이지 마지막 개체 묶기 전`이다.
즉 문단 `0:29`의 paragraph record만 복사해도 묶음 그림 자체는 아직 한컴이 통과하지 못한다.

다음 경계는 문단 `0:29`의 로고/배너 묶음 그림이다.

```text
문단 0:29 로고/배너 묶음 그림
```

rhwp-studio는 모든 Stage 20 variant를 정상 렌더링하므로, 다음 probe는 한컴 호환성 관점에서 묶음 그림의 raw/control payload를 좁힌다.

## 9. 다음 단계

Stage 21은 문단 `0:29` 묶음 그림을 대상으로 한다.

우선순위:

```text
1. Stage 20 `06_notice_table_full_object_with_tail` 또는 `08_notice_full_tuple_plus_logo_group_para_record`를 기준선으로 사용
2. 문단 0:29의 Shape/Group control payload를 정답 HWP와 비교
3. group common attr, shape attr, child picture common/shape attr, matrix/crop/effect payload를 단계별로 graft
4. 파일 읽기 오류가 사라지고 문단 0:30 별첨 표까지 이동하는지 확인
```

Stage 21에서는 파일손상 여부만 보지 말고, 반드시 판정 유형을 구분한다.

```text
파일손상
파일 읽기 오류
정상 열림
```

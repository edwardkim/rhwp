# Task m100 #903 Stage 12

## 1. 단계 목적

Stage 11 결과:

```text
모든 variant:
  한컴 에디터: 첫 표 출력 후 파일손상
  rhwp-studio: 정상 렌더링
```

따라서 첫 표 내부 payload를 더 맞추는 방식은 더 이상 출력 위치를 밀지 못한다.

Stage 12는 새 payload 이식 variant를 바로 만들지 않는다.
먼저 한컴이 첫 표를 출력한 직후 깨지는 record/control 경계를 찾는 진단 단계로 진행한다.

## 2. 기준 파일

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 11 기준 HWP:

```text
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/05_picture_full_plus_structural_bundle.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/06_picture_full_plus_first_table_ctrl_data_records.hwp
```

Stage 12 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/
```

작업지시자 시각 판정용 HWP를 만들기 전, 이 단계는 record dump와 경계 분석 문서를 먼저 만든다.

## 3. 문제 분류

현재 문제는 더 이상 파일 읽기/저장 오류가 아니다.

```text
파일 읽기/저장 오류:
  한컴이 문서를 렌더링하기 전 record stream을 읽다가 실패

파일손상:
  한컴이 일부 문서를 렌더링한 뒤 다음 record/control 처리에서 정합성 실패
```

Stage 11 기준선은 파일손상 단계까지 이동했으므로, 이제 다음 관심사는 "첫 표 다음에 한컴이 읽는 record"다.

## 4. Stage 12 진단 계획

### 01_record_dump_reference

정답 HWP의 BodyText record dump를 생성한다.

출력:

```text
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/reference_records.txt
```

### 02_record_dump_stage11

Stage 11 기준 파일의 BodyText record dump를 생성한다.

출력:

```text
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/stage11_05_records.txt
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/stage11_06_records.txt
```

### 03_first_table_record_span

정답 HWP와 Stage 11 파일에서 첫 표 record span을 식별한다.

기록 항목:

- 첫 표가 시작되는 `CTRL_HEADER(tbl )`
- 첫 표 내부 `TABLE`
- 첫 표 내부 `LIST_HEADER`
- 첫 표 내부 그림 `CTRL_HEADER(gso)`, `SHAPE_COMPONENT`, `SC_PICTURE`
- 첫 표가 끝난 직후의 다음 record 20개

출력:

```text
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/first_table_boundary_report.md
```

### 04_boundary_diff

첫 표 종료 직후 record sequence를 정답 HWP와 Stage 11 파일 사이에서 비교한다.

비교 항목:

- record tag
- record level
- record size
- control id
- paragraph boundary
- LIST_HEADER/PARA_HEADER/CTRL_HEADER 크기

출력:

```text
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/boundary_diff.md
```

## 5. Stage 12 이후의 후보 probe

Stage 12에서 첫 실패 경계를 잡은 뒤에만 다음 HWP variant를 만든다.

가능 후보:

```text
첫 표 직후 paragraph header 이식
첫 표 직후 control raw payload 이식
첫 표 직후 그림/묶음 control payload 이식
첫 표 직후 record span 단위 graft
```

하지만 Stage 12에서는 아직 이 후보들을 실행하지 않는다.

## 6. 내부 검증

진단 산출물이 생성되면 다음을 확인한다.

```text
reference_records.txt 존재
stage11_05_records.txt 존재
stage11_06_records.txt 존재
first_table_boundary_report.md 존재
boundary_diff.md 존재
```

## 7. 승인 요청

Stage 12는 구현/이식 전에 record boundary 진단 산출물을 먼저 생성한다.

## 8. 실행 결과

다음 dump 산출물을 생성했다.

```text
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/reference_records.txt
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/stage11_05_records.txt
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/stage11_06_records.txt
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/reference_record_headers.txt
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/stage11_05_record_headers.txt
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/stage11_06_record_headers.txt
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/first_table_boundary_report.md
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/boundary_diff.md
```

record dump line 수:

```text
reference_records.txt: 12652
stage11_05_records.txt: 12611
stage11_06_records.txt: 12611
```

## 9. 경계 분석 요약

첫 표는 정답 HWP와 Stage 11 기준 파일 모두 record `13..36` 범위에 있다.

Stage 11의 한컴 판정은 "첫 표 출력 후 파일손상"이므로, 첫 표 내부 payload를 계속 보강하는 방식은 더 이상 출력 위치를 밀지 못한다고 판단한다.

첫 표 종료 직후의 첫 차이는 최상위 문단 header 크기다.

```text
reference:
  [37] PARA_HEADER lv=0 sz=24
  [40] PARA_HEADER lv=0 sz=24
  [43] PARA_HEADER lv=0 sz=24

stage11:
  [37] PARA_HEADER lv=0 sz=22
  [40] PARA_HEADER lv=0 sz=22
  [43] PARA_HEADER lv=0 sz=22
```

그 다음 큰 차이는 두 번째 표 시작 record다.

```text
reference:
  [47] CTRL_HEADER(tbl) lv=1 sz=46
       ... 10 23 2a 08 ...
       ... 01 00 00 00 ...
       ... bc ce f2 49 ...

stage11:
  [47] CTRL_HEADER(tbl) lv=1 sz=46
       ... 10 03 2a 00 ...
       ... 00 00 00 00 ...
       ... 00 00 00 00 ...
```

두 번째 표의 child header도 compact/generated 형태와 정답지 형태가 다르다.

```text
reference:
  [48] TABLE       sz=24
  [49] LIST_HEADER sz=65
  [50] PARA_HEADER sz=24

stage11:
  [48] TABLE       sz=22
  [49] LIST_HEADER sz=34
  [50] PARA_HEADER sz=22
```

## 10. 결론

Stage 12 기준으로 다음 실패 경계는 첫 표 내부가 아니라 다음 구간으로 본다.

```text
첫 표 종료 직후 최상위 문단 37..46
두 번째 표 시작 record 47
두 번째 표 child header 48..50
```

다음 Stage 13에서는 이 범위를 분리해서 이식 probe를 만든다.

권장 variant:

```text
01: post-first-table top-level PARA_HEADER 37/40/43 only
02: next table CTRL_HEADER 47 only
03: next table child header 48/49/50 only
04: next table record span 47..53
05: post-first-table records 37..46 + next table span 47..53
```

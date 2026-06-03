# Task M100-993 Stage 17 계획서

## 1. 목적

Stage 15의 통과 기준 파일을 adapter 산출물로 재현한다.

기준 파일:

```text
output/poc/hwpx2hwp/task993/stage15_first_para_text_marker_contract/02_oracle_para_text_guard.hwp
```

이 파일은 다음 두 조건을 동시에 만족한다.

```text
1. 1페이지 첫 문단의 고스트 선이 제거된다.
2. 2페이지 "인원 현황" 표가 정상 높이로 출력된다.
```

Stage 16은 TAB extension byte 정합성만 확인했으며, 2페이지 표 높이 문제를 해결하지 못했으므로
최종 후보로 보지 않는다.

## 2. 선행 판정

Stage 10 판정:

```text
03, 06, 07, 09가 개선
공통 positive 축 = pi=22 인원 현황 표의 OracleListExtra
```

Stage 15 판정:

```text
02_oracle_para_text_guard.hwp만 통과
```

따라서 Stage 17의 구현 축은 다음이다.

```text
1. 첫 문단 PARA_TEXT marker/TAB extension materialization은 Stage 16 결과를 유지한다.
2. Stage 9에서 전역 적용했던 cell LIST_HEADER width_ref bit 0 보강을 제한한다.
3. raw_list_extra 13바이트는 유지하되, width_ref bit 0은 고열 수 micro-grid 표에만 적용한다.
```

## 3. 구현 방향

수정 대상:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

규칙:

```text
1. 모든 셀에 raw_list_extra 13바이트를 materialize한다.
2. table.col_count >= 30 인 micro-grid 표에만 cell.list_header_width_ref bit 0을 세운다.
3. 일반 데이터 표에서는 bit 0을 세우지 않는다.
```

근거:

```text
tb-org-02 / 조직도 표:
  10x65 micro-grid 표이며 width_ref bit 0 + raw_list_extra가 셀 내부 줄나눔 개선에 필요했다.

mel-001 pi=22 인원 현황 표:
  8x12 데이터 표이며 width_ref bit 0을 전역으로 세우면 한컴이 병합 셀 높이를 과도하게 계산했다.
  oracle LIST_HEADER는 raw_list_extra는 가지지만 width_ref bit 0은 세우지 않는다.
```

## 4. 생성 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/
```

판정 파일:

| file | 한컴 판정 유형 | 1페이지 고스트 선 | 1x1 표 배경 | 셀 대각선 | 인원 현황 표 높이 | 조직도 셀 줄나눔 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/mel-001.hwp` |  |  |  |  |  |  |  |  | Stage 15 기준 재현 후보 |
| `output/poc/hwpx2hwp/task993/stage17_stage15_reproduction/tb-org-02.hwp` |  | - | - |  | - |  |  |  | 조직도 guard |

## 5. 성공 기준

```text
1. mel-001.hwp가 한컴 에디터에서 파일손상 없이 열린다.
2. 1페이지 첫 문단 고스트 선이 없다.
3. 2페이지 첫 1x1 표 gradient 배경이 정상이다.
4. 셀 대각선이 유지된다.
5. 2페이지 인원 현황 표의 병합 셀 높이가 과도하게 커지지 않는다.
6. tb-org-02 조직도 셀 줄나눔이 Stage 9 개선 상태를 유지한다.
```


# Task M100-993 Stage 9 완료 보고서

## 1. 목적

Stage 8에서 개선이 확인된 `07_all_org_cells_list_width_ref_plus_extra` 축을 실제
HWPX -> HWP adapter 후보로 반영했다.

이번 단계의 목적은 셀 높이를 직접 보정하거나 문자열을 다시 나누는 것이 아니라, 한컴 에디터가
셀 내부 줄나눔 폭을 정답지와 같은 방식으로 산정하도록 HWP5 셀 `LIST_HEADER` contract를
materialize하는 것이다.

## 2. 구현 범위

수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

추가한 보강 규칙:

```text
대상:
  HWPX/HWP3 출처 문서를 HWP5로 저장할 때의 모든 table cell

보강:
  1. cell.list_header_width_ref bit 0을 세운다.
  2. cell.raw_list_extra가 비어 있으면 13바이트를 생성한다.
     - bytes 0..4  = cell.width, u32 little endian
     - bytes 4..13 = 0

보존:
  raw_list_extra가 이미 있는 셀은 덮어쓰지 않는다.
```

보고용 카운터도 추가했다.

```text
AdapterReport.cells_list_header_contract_materialized
```

## 3. 비포함 범위

Stage 8의 `08`에 포함된 셀 내부 `PARA_HEADER` tail 보강은 이번 단계에 포함하지 않았다.

```text
이번 단계 포함:
  LIST_HEADER width_ref bit 0
  LIST_HEADER raw_list_extra 13 bytes

이번 단계 제외:
  셀 내부 PARA_HEADER tail 합성
  셀 높이 직접 보정
  문자열 재분할
```

## 4. 생성 결과

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/
```

생성 파일:

| file | size | 목적 |
|---|---:|---|
| `output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/tb-org-02.hwp` | 7,680 bytes | 조직도 셀 줄나눔 target |
| `output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/mel-001.hwp` | 162,304 bytes | mel-001 guard |

보조 inventory:

```text
output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/tb-org-02_inventory.jsonl
output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/mel-001_inventory.jsonl
```

## 5. 로드 확인

`tb-org-02.hwp`:

```text
HWP 5.1
compressed: yes
sections: 1
pages: 1
주요 table: 10x65, cells=240
```

`mel-001.hwp`:

```text
HWP 5.1
compressed: yes
sections: 1
pages: 20
tables: 44
조직도 성격 table 포함
```

## 6. 계약 확인

`tb-org-02_inventory.jsonl` 기준으로 조직도 table cell의 `LIST_HEADER`는 다음 형태로 저장된다.

```text
LIST_HEADER size = 47
payload bytes 6..7 = 01 00
raw_list_extra 13 bytes 포함
raw_list_extra first u32 = cell.width
```

이는 Stage 8에서 개선이 확인된 `07` 축과 같은 방향이다.

## 7. 실행 검증

```text
cargo check
cargo test cell_list_header_contract_materializes_width_ref_and_extra
```

결과:

```text
success
```

참고:

```text
cargo test 실행 중 기존 warning이 출력되었으나, 이번 Stage 9 테스트는 통과했다.
- duplicate #[test] warning
- unused parens warning
- 일부 non_snake_case test name warning
- 일부 unused Result warning
```

## 8. 한컴 판정 요청

| file | 한컴 판정 유형 | 조직도 셀 줄나눔 | 1x1 표 배경 | 표/셀 배치 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/tb-org-02.hwp` | 성공 | 개선 | - | 개선 | 성공 | 성공 | target |
| `output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/mel-001.hwp` | 성공 | 개선 | 성공 | 일부 실패 | 성공 | 성공 | guard |

작업지시자 판정:

```text
두 파일 모두 조직도 셀 처리는 개선되었다.

잔존 문제:
  mel-001.hwpx의 2. 인원현황 다음 표에서 병합된 아래쪽 셀 높이가 한컴 에디터에서 과도하게
  렌더링된다. rhwp-studio에서는 정상이다.

결과:
  rhwp-studio에서는 2페이지에 포함되는 표가 한컴 에디터에서는 다음 페이지로 분리된다.
```

## 9. 판정 기준

`tb-org-02.hwp`에서 확인할 핵심:

```text
1. 조직도 셀 텍스트가 한글자씩 줄나눔되지 않고 2글자씩 줄나눔되는지
2. 조직도 표 높이가 정답 PDF/HWP와 같은 방향으로 줄어드는지
3. 셀 세로 정렬이 Stage 5 통과 상태를 유지하는지
```

`mel-001.hwp`에서 확인할 핵심:

```text
1. 2페이지 첫 1x1 표 gradient 배경이 회귀하지 않는지
2. 파일손상 없이 열리는지
3. 표/셀 배치가 Stage 6 대비 악화되지 않는지
```

## 10. 다음 판단

이번 Stage 9는 Stage 8의 `07` 축만 반영했다.

```text
Stage 9 판정:
  LIST_HEADER materialization은 조직도 셀 줄나눔 개선에 효과가 있다.
  production 후보로 유지한다.

잔존 문제:
  mel-001의 2. 인원현황 다음 표에서 한컴 에디터가 병합된 아래쪽 셀 높이를 과도하게 산정한다.
```

## 11. mel-001 잔존 문제 위치

`dump-pages` 기준으로 문제 표는 다음 위치다.

```text
file: output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/mel-001.hwp
page: 2
paragraph: section0 pi=22
table: 8x12, cells=87
위치: "󰊲  인원 현황(’25.11.30. 기준)" 바로 다음 표
```

rhwp 기준 페이지 2 배치:

```text
FullParagraph  pi=21  "󰊲  인원 현황(’25.11.30. 기준)"
Table          pi=22  8x12  630.4x146.1px
FullParagraph  pi=23  "  * 별정직, 전문경력관..."
FullParagraph  pi=24  "󰊳  예산 현황"
Table          pi=25  12x5
```

정답 HWP와 Stage 9 생성 HWP를 `dump`로 비교하면 `pi=22`의 표 크기, 셀 높이, 병합 구조,
주요 문단/line segment 값은 IR 수준에서 거의 같은 형태로 보인다. 따라서 이 문제는 rhwp IR
렌더링 차이가 아니라, 한컴 에디터가 읽는 HWP5 raw record contract 차이일 가능성이 높다.

## 12. 다음 단계

Stage 10은 `mel-001`의 `section0 pi=22` 표만 대상으로 좁힌다.

```text
1. 정답 HWP와 Stage 9 생성 HWP의 pi=22 table record bundle을 raw payload 단위로 비교한다.
2. 병합된 아래쪽 셀 후보를 중심으로 LIST_HEADER, TABLE, CELL, PARA_HEADER, PARA_LINE_SEG 차이를
   분리한다.
3. Stage 9의 모든 셀 LIST_HEADER extra materialization이 pi=22 표에 과보강으로 작동했는지 확인한다.
4. 필요한 경우 pi=22 표를 대상으로 다음 후보를 생성한다.
   - raw_list_extra 조건 축소
   - 병합 셀만 oracle payload 적용
   - 셀 내부 PARA_HEADER tail 후보 적용
```

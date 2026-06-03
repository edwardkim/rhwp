# Task m100 #949 Stage 10 작업 보고서: hwpx-h-03 성공 축 재검증

## 1. 목적

Stage 9에서 `hwpx-h-01`에 성공했던 TABLE/CTRL_HEADER(Table) probe 축을
`hwpx-h-03` current baseline에도 적용했다.

핵심 질문:

```text
hwpx-h-01에서 성공한 04_ctrl_common_attr_only 또는 08_all_table_axes가
hwpx-h-03의 한컴 파일손상 판정도 제거하는가?
```

## 2. 입력

```text
oracle:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated baseline:
output/poc/hwpx2hwp/task903/stage58_table_ctrl_attr_margin_probe/01_current_baseline_hwpx-h-03.hwp
```

Stage 58 기록상 `01_current_baseline | hwpx-h-03`은 한컴 에디터에서 파일손상,
2페이지까지만 출력, 이미지/표 배치 실패였다.

## 3. 생성 명령

```bash
./target/debug/rhwp hwp5-table-probe \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task903/stage58_table_ctrl_attr_margin_probe/01_current_baseline_hwpx-h-03.hwp \
  --out-dir output/poc/hwpx2hwp/task949/stage10/hwpx-h-03
```

보조 보고서:

```bash
./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task903/stage58_table_ctrl_attr_margin_probe/01_current_baseline_hwpx-h-03.hwp \
  --align lcs \
  --report table-probe-plan \
  --out output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/table_probe_plan.md
```

## 4. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/
```

| file | bytes | rhwp reload | outer margin | ctrl attr | table attr | table tail |
|---|---:|---|---:|---:|---:|---:|
| `01_ctrl_outer_margin_only.hwp` | 38400 | ok, pages=9 | 26 | 0 | 0 | 0 |
| `02_table_attr_only.hwp` | 38400 | ok, pages=9 | 0 | 0 | 19 | 0 |
| `03_table_tail_only.hwp` | 38400 | ok, pages=9 | 0 | 0 | 0 | 26 |
| `04_ctrl_common_attr_only.hwp` | 38400 | ok, pages=9 | 0 | 9 | 0 | 0 |
| `05_outer_margin_table_attr.hwp` | 38400 | ok, pages=9 | 26 | 0 | 19 | 0 |
| `06_outer_margin_table_tail.hwp` | 38400 | ok, pages=9 | 26 | 0 | 0 | 26 |
| `07_table_attr_tail.hwp` | 38400 | ok, pages=9 | 0 | 0 | 19 | 26 |
| `08_all_table_axes.hwp` | 38400 | ok, pages=9 | 26 | 9 | 19 | 26 |

생성 보고서:

```text
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/stage9_generation.md
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/table_probe_plan.md
```

주의: `hwp5-table-probe`의 현재 보고서 파일명은 `stage9_generation.md`로 고정되어 있다.
이번 산출 위치는 Stage 10이므로 문맥상 Stage 10 결과로 취급한다.

## 5. 작업지시자 판정 대상

우선 판정 대상:

```text
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/04_ctrl_common_attr_only.hwp
output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp
```

전체 판정표:

| case | 한컴 에디터 판정 | 이미지 출력 | 표/셀배치 여부 | rhwp-studio | 비고 |
|---|---|---|---|---|---|
| `hwpx-h-03/01_ctrl_outer_margin_only.hwp` | 파일손상 |  |  |  |  |
| `hwpx-h-03/02_table_attr_only.hwp` | 파일손상 |  |  |  |  |
| `hwpx-h-03/03_table_tail_only.hwp` | 파일손상 |  |  |  |  |
| `hwpx-h-03/04_ctrl_common_attr_only.hwp` | 파일손상 |  |  |  | hwpx-h-01 성공 축 |
| `hwpx-h-03/05_outer_margin_table_attr.hwp` | 파일손상 |  |  |  |  |
| `hwpx-h-03/06_outer_margin_table_tail.hwp` | 파일손상 |  |  |  |  |
| `hwpx-h-03/07_table_attr_tail.hwp` | 파일손상 |  |  |  |  |
| `hwpx-h-03/08_all_table_axes.hwp` | 파일손상 |  |  |  | hwpx-h-01 성공 축 |

## 6. 해석 기준

```text
- 04 또는 08에서 파일손상이 사라지면, h-03의 초기 파일손상도 TABLE CTRL_HEADER/common attr 축과 관련이 있을 가능성이 커진다.
- 04 또는 08에서도 파일손상이 유지되면, h-03의 파일손상은 Stage 9의 h-01 성공 축만으로는 설명되지 않는다.
- 이 probe는 HWPX 저장기 구현 결과가 아니라 BodyText payload graft 파일이다.
```

## 7. 판정 결과 해석

작업지시자 판정 결과, `hwpx-h-03`의 8개 probe는 모두 한컴 에디터에서 파일손상 판정을
받았다.

따라서 Stage 9에서 `hwpx-h-01`을 정상화한 TABLE/CTRL_HEADER(Table) 축은
`hwpx-h-03` 파일손상 조건을 제거하지 못한다. 이 결과는 다음을 의미한다.

```text
- hwpx-h-01의 표 배치/파일 정상화 조건을 hwpx-h-03에 일반화하면 안 된다.
- hwpx-h-03의 파일손상 원인은 Stage 10의 네 축
  (ctrl_outer_margin, ctrl_common_attr, table_attr, table_tail) 밖에 있다.
- 다음 h-03 조사는 TABLE 축 보정의 확대가 아니라, h-03 정답 HWP와 generated HWP의
  record contract 위반 위치를 별도로 추적해야 한다.
```

## 8. 파일손상 원인 분석

이 단계의 질문은 "기록"이 아니라 "`hwpx-h-03`에서 왜 파일손상이 발생하는가"다.
현재까지 확인한 사실은 다음과 같다.

### 8.1 TABLE 축은 직접 원인이 아니다

`08_all_table_axes.hwp`는 `ctrl_outer_margin`, `ctrl_common_attr`, `table_attr`, `table_tail`
네 축을 모두 정답 HWP 값으로 graft한 파일이다. 그 결과 `hwp5-inventory-diff --report hints`
에서 TABLE 후보는 사라졌다.

```text
Table Candidates: 후보 없음
```

그런데도 한컴 에디터는 파일손상 판정을 내렸다. 따라서 `hwpx-h-03`의 파일손상 직접 원인은
Stage 9/10에서 다룬 TABLE 축이 아니다.

### 8.2 h01 성공 파일에도 남아 있는 차이는 원인에서 제외한다

비교 기준으로 `hwpx-h-01/08_all_table_axes.hwp`를 함께 보았다. 이 파일은 한컴 에디터에서
성공 판정을 받은 파일이다. 그런데 이 성공 파일에도 다음 차이는 여전히 남아 있다.

```text
- SectionDef CTRL_HEADER size/payload 차이
- GenShape CTRL_HEADER payload 차이
- SHAPE_COMPONENT / SHAPE_PICTURE payload 차이
- BodyText CTRL_DATA 누락
- DocInfo DOC_DATA / FORBIDDEN_CHAR / COMPATIBLE_DOCUMENT / LAYOUT_COMPATIBILITY / TRACKCHANGE 누락
```

따라서 위 항목들은 단독으로는 한컴 파일손상 원인이라고 확정할 수 없다. 이 항목을 그대로
원인으로 지목하면 h01 성공 케이스와 모순된다.

### 8.3 h03 고유 후보: DocInfo MEMO_SHAPE 누락

`hwpx-h-03` 정답 HWP에는 `DocInfo#525 MEMO_SHAPE` record가 존재한다.
Stage 10 생성 파일에는 이 record가 없다.

```text
oracle:
DocInfo#525 MEMO_SHAPE size=22

generated:
MEMO_SHAPE 없음
```

반면 `hwpx-h-01/08_all_table_axes.hwp` 성공 비교에서는 `MEMO_SHAPE` 누락이 나타나지 않는다.
즉 현재까지의 비교에서 `MEMO_SHAPE` 누락은 `hwpx-h-03` 실패 쪽에만 보이는 고유 차이다.

또한 `ID_MAPPINGS` record 크기도 정답과 생성 파일이 다르다.

```text
oracle:    ID_MAPPINGS size=72
generated: ID_MAPPINGS size=64
```

`MEMO_SHAPE`는 DocInfo count/reference table과 함께 해석되는 record이므로,
`MEMO_SHAPE` record 자체와 `ID_MAPPINGS` count가 함께 맞지 않으면 한컴 에디터가
DocInfo 계약 위반으로 판단할 가능성이 높다.

### 8.4 보조 후보: 2페이지 도형 묶음 내부 CTRL_DATA 누락

`hwpx-h-03` 정답 HWP의 2페이지 도형 묶음 주변에는 다음 record가 있다.

```text
CTRL_HEADER(GenShape)
SHAPE_COMPONENT
CTRL_DATA
SHAPE_PICTURE
```

Stage 10 생성 파일에서는 같은 위치의 `CTRL_DATA`가 빠지고, 바로 `SHAPE_PICTURE`로 이어진다.

```text
CTRL_HEADER(GenShape)
SHAPE_COMPONENT
SHAPE_PICTURE
```

다만 `hwpx-h-01` 성공 파일에도 BodyText `CTRL_DATA` 누락은 존재한다. 따라서
`CTRL_DATA` 누락만으로 파일손상을 설명할 수는 없다. 현재 해석은 다음과 같다.

```text
1순위 후보: DocInfo ID_MAPPINGS / MEMO_SHAPE 계약 불일치
2순위 후보: h03의 2페이지 도형 묶음 내부 CTRL_DATA + SHAPE payload 계약 불일치
제외된 후보: Stage 10 TABLE 축 단독 원인
```

즉 지금 단계에서 가장 정확한 답은 다음이다.

```text
hwpx-h-03은 TABLE payload 보정 이후에도 한컴 에디터가 요구하는
DocInfo MEMO_SHAPE/ID_MAPPINGS 계약을 만족하지 못한다. 이 차이는 h01 성공 파일에는 없는
h03 고유 차이다. 따라서 h03 파일손상은 TABLE 축 문제가 아니라 DocInfo memo/count 계열
또는 이와 결합된 2페이지 도형 묶음 contract 문제로 봐야 한다.
```

확정 원인으로 닫으려면 다음 probe에서 `MEMO_SHAPE + ID_MAPPINGS`만 보정한 파일과
`2페이지 도형 묶음 CTRL_DATA`만 보정한 파일을 분리 판정해야 한다.

## 9. 검증

```text
hwp5-table-probe: 8개 HWP 생성 통과
hwp5-inventory-diff --report table-probe-plan: 생성 통과
rhwp reload: 8개 모두 pages=9
```

# Task m100 #949 Stage 19 작업 기록

## 1. 목적

Stage 18에서 `hwpx-h-01`, `hwpx-h-02`는 table-axis 계약 반영 후 한컴 에디터와
rhwp-studio 모두 성공했다. 반면 `hwpx-h-03`은 TABLE field diff가 0인데도 한컴에서
2페이지 부근 파일손상 판정을 받았다.

이 단계의 목적은 `hwpx-h-03`을 대상으로 표 계약이 아닌 별도 계약 위반을 정답 HWP와 비교해
분리하고, 확인 가능한 단일 후보만 만든다.

## 2. 입력

```text
source HWPX: samples/hwpx/hwpx-h-03.hwpx
oracle HWP:  samples/hwpx/hancom-hwp/hwpx-h-03.hwp
generated:   output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-03.hwp
```

성공 대조군:

```text
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-02.hwp
```

## 3. Stage 18 판정에서 고정된 사실

```text
hwpx-h-01: 한컴 성공, 이미지 성공, 표/셀 배치 성공, 마지막 페이지 성공, rhwp-studio 성공
hwpx-h-02: 한컴 성공, 이미지 성공, 표/셀 배치 성공, 마지막 페이지 성공, rhwp-studio 성공
hwpx-h-03: 한컴 파일손상, 이미지 성공, 표/셀 배치 성공, 2페이지까지만 출력,
           rhwp-studio 1페이지 페이지네이션 실패
```

정적 비교:

```text
hwpx-h-03 TABLE field diff count = 0
```

따라서 `hwpx-h-03`의 파일손상 원인을 TABLE record field 축으로 단정하지 않는다.

## 4. Stage 19 진단 결과

진단 산출물:

```text
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/
```

주요 파일:

```text
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/docinfo_contract.md
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/bodytext_control_graph.md
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/record_hints.md
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/success_h01/docinfo_contract.md
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/success_h02/docinfo_contract.md
```

확인된 고유 차이:

```text
hwpx-h-03 HWPX에는 hh:memoProperties / hh:memoPr가 존재한다.
hwpx-h-01, hwpx-h-02에는 같은 memoProperties가 없다.
```

`hwpx-h-03` 정답 HWP의 DocInfo에는 다음 계약이 있다.

```text
MEMO_SHAPE records = 1
ID_MAPPINGS memo_shape_count(index 15) = 1
ID_MAPPINGS payload size = 72 bytes
```

Stage 18 생성본은 다음 상태였다.

```text
MEMO_SHAPE records = 0
ID_MAPPINGS memo_shape_count(index 15) = 0
ID_MAPPINGS payload size = 64 bytes
```

따라서 이 단계의 좁은 구현 후보는 `HWPX memoProperties -> HWP DocInfo MEMO_SHAPE` 계약이다.

## 5. 구현 내용

수정 경로:

```text
src/parser/hwpx/header.rs
src/serializer/doc_info.rs
src/serializer/doc_info/tests.rs
```

구현한 계약:

```text
1. header.xml의 hh:memoPr를 파싱한다.
2. memoPr 속성을 HWP MEMO_SHAPE payload로 변환한다.
3. 변환된 MEMO_SHAPE를 DocInfo extra_records에 level=1로 추가한다.
4. doc_info.memo_shape_count를 MEMO_SHAPE 개수와 동기화한다.
5. HWPX에서 새로 직렬화하는 ID_MAPPINGS를 18개 u32, 즉 72바이트로 생성한다.
```

`hwpx-h-03`에서 확인한 `memoPr`:

```xml
<hh:memoPr id="1" width="15591" lineWidth="5" lineType="DASH"
  lineColor="#A9A9A9" fillColor="#FDFCC6" activeColor="#C0DBFB"
  memoType="NOMAL"/>
```

정답 MEMO_SHAPE payload:

```text
e7 3c 00 00 03 05 a9 a9 a9 00 fd fc c6 00 c0 db fb 00 00 00 00 00
```

## 6. 후보 파일

판정 대상:

```text
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/hwpx-h-03-stage19-candidate.hwp
```

참고용 재생성 파일:

```text
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/regression-hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/regression-hwpx-h-02.hwp
```

rhwp 재로드 정보:

```text
hwpx-h-03-stage19-candidate.hwp: sections=2, pages=9
regression-hwpx-h-01.hwp:        sections=2, pages=9
regression-hwpx-h-02.hwp:        sections=2, pages=10
```

## 7. 후보 정적 비교 결과

후보 분석 산출물:

```text
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/candidate_contract_v2/
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/candidate_hints_v2.md
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/candidate_docinfo_bundles_v2.md
```

DocInfo 계약은 닫혔다.

```text
MEMO_SHAPE records: oracle=1, generated=1
ID_MAPPINGS memo_shape_count(index 15): oracle=1, generated=1
ID_MAPPINGS payload size: oracle=72, generated=72
unknown_count_16: oracle=0, generated=0
unknown_count_17: oracle=0, generated=0
```

남은 정적 차이:

```text
BodyText record tag count는 정답지와 생성본이 일치한다.
남은 차이는 주로 GenShape CTRL_HEADER payload와 SectionDef CTRL_HEADER payload다.
특히 GenShape#1은 CTRL_HEADER size가 oracle=60, generated=46으로 다르다.
```

이 차이는 Stage 19 후보가 한컴에서 여전히 실패할 경우 다음 단계에서 다룰 후보군이다.
하지만 이번 단계에서는 먼저 고유하게 확인된 DocInfo memo 계약을 닫은 후보를 한컴에서 판정한다.

## 8. 판정표

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/hwpx-h-03-stage19-candidate.hwp` | 파일손상 | 성공 | 성공 | - | 실패(2페이지까지만) | 1페이지 페이지네이션 실패 | Stage 18과 동일. `memoProperties -> MEMO_SHAPE`, ID_MAPPINGS 72B |

## 8.1 판정 해석

작업지시자 판정 결과, Stage 19 후보는 Stage 18과 동일하게 실패했다.

따라서 다음과 같이 정리한다.

```text
1. HWPX memoProperties -> HWP MEMO_SHAPE 계약은 정적 비교상 닫혔다.
2. 하지만 한컴 파일손상과 rhwp-studio 페이지네이션 실패는 개선되지 않았다.
3. 그러므로 MEMO_SHAPE 누락은 정합성 결함이지만, 이번 파일손상의 직접 원인은 아니다.
4. 다음 직접 후보는 BodyText의 GenShape control contract다.
```

현재 남은 가장 강한 단서는 다음이다.

```text
BodyText.Section0#824
oracle:    CTRL_HEADER GenShape size=60
generated: CTRL_HEADER GenShape size=46
```

이 레코드는 `GenShape#1`이며, 2페이지 이미지 개체 묶기 주변 control graph에 해당한다.
Stage 20은 이 `GenShape#1`의 HWPX source tree, oracle HWP record payload, generated HWP record
payload를 먼저 비교한다.

## 9. 실행한 검증

```text
cargo check --quiet
cargo build --quiet
cargo test --quiet test_parse_hwpx_memo_shape_record
cargo test --quiet test_serialize_id_mappings_uses_modern_count_table_size
cargo test --quiet table_axis_materializes_hancom_record_contract
cargo test --quiet picture_href_ctrl_data
```

비고:

```text
cargo test --quiet table_axis_materializes_hancom_record_contract picture_href_ctrl_data
```

위처럼 테스트 필터 2개를 한 번에 넘긴 호출은 Cargo 사용법 오류로 실패했다. 이후 두 테스트를
각각 다시 실행해 통과를 확인했다.

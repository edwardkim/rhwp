# Task M100-1110 Stage 1 작업 기록

## 1. 목적

`samples/hwpx/exam_social.hwpx`를 HWP로 저장했을 때 한컴 에디터에서 파일손상 판정이 나는 원인을
정답 HWP와 비교하여 추적한다.

## 2. 대상

```text
source HWPX:
  samples/hwpx/exam_social.hwpx

oracle HWP:
  samples/exam_social.hwp

기존 저장본:
  saved/111exam_social.hwp

1차 판정 대상:
  output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/exam_social_stage1_autonum_range.hwp

2차 판정 대상:
  output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/exam_social_stage1_autonum_range_narrow.hwp
```

## 3. 1차 확인

현재 코드로 새로 저장한 파일과 기존 `saved/111exam_social.hwp`는 동일했다.

```text
saved/111exam_social.hwp
output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/exam_social_fresh.hwp
```

따라서 #1108에서 구현한 HWPX 렌더링 보강은 HWP 저장 파일의 binary record contract에는 아직
변화를 만들지 못한 상태였다.

## 4. 정답지와의 손상 후보

`hwp5-anchor-trace`로 `사회·문화` 주변을 비교했다.

정답 HWP의 문제 지점:

```text
PARA_TEXT:
  <0x0012:AutoNumber:0x61746e6f><0x001f>(사회·문화)<PARA_END>

PARA_RANGE_TAG:
  start=15, end=16, tag=0x01000023
  payload = 0f 00 00 00 10 00 00 00 23 00 00 01
```

기존 저장본의 문제 지점:

```text
PARA_TEXT:
  <0x0012:AutoNumber:0x61746e6f> (사회·문화)<PARA_END>

문제:
  1. HWP fixed-width space control code 0x001F가 아니라 Unicode U+2007로 저장됨
  2. 해당 문단의 PARA_RANGE_TAG가 누락됨
```

이 차이는 한컴이 `AutoNumber` 뒤의 고정폭 빈칸과 이어지는 문단 텍스트를 해석하는 HWP5 contract
차이로 본다.

## 5. 구현

구현 내용:

```text
1. HWPX -> HWP 저장 adapter에서 다음 형태의 문단을 감지한다.
   - AutoNumber 컨트롤 포함
   - text가 " " + U+2007 + visible text 형태

2. 정답 HWP와 같은 PARA_RANGE_TAG를 materialize한다.
   - start = 마지막 visible char offset
   - end = start + 마지막 visible char UTF-16 길이
   - tag = 0x01000023

3. HWP BodyText serializer에서 U+2007을 HWP5 fixed-width blank control code인 0x001F로 직렬화한다.
```

수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
src/serializer/body_text.rs
```

## 6. 1차 결과

1차 저장 파일:

```text
output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/exam_social_stage1_autonum_range.hwp
```

`rhwp info` 결과:

```text
로드 성공
구역 수 = 2
페이지 수 = 4
크기 = 426,496 bytes
```

정답 HWP와 신규 저장본의 anchor trace:

```text
output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/stage1_autonum_range_anchor_social.md
```

핵심 지점:

```text
PARA_TEXT hash = 4fb83b2eca6dc4d4
PARA_RANGE_TAG hash = de1451294020541a
PARA_RANGE_TAG payload = 0f 00 00 00 10 00 00 00 23 00 00 01
```

해당 `PARA_TEXT`와 `PARA_RANGE_TAG`는 정답 HWP와 일치한다.

하지만 작업지시자 한컴 판정 결과:

```text
비정상 종료
```

해석:

```text
1. AutoNumber 주변 PARA_RANGE_TAG 후보는 필요한 축일 수 있다.
2. 그러나 1차 구현은 U+2007을 HWP5 0x001F로 전역 직렬화했다.
3. 전역 변환이 다른 문단의 fixed-width space까지 건드려 한컴 비정상 종료를 유발했을 가능성이 있다.
```

따라서 1차 후보는 성공 후보에서 제외한다.

## 6.1 2차 후보 — 전역 변환 제거

2차 구현은 `U+2007 -> 0x001F` 변환 범위를 다음 문단 형태로 제한한다.

```text
조건:
  - AutoNumber 컨트롤 포함
  - text가 " " + U+2007 로 시작
  - tag=0x01000023 PARA_RANGE_TAG가 materialize된 문단
```

일반 U+2007은 기존처럼 Unicode code point `0x2007`로 저장한다.

2차 저장 파일:

```text
output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/exam_social_stage1_autonum_range_narrow.hwp
```

`rhwp info` 결과:

```text
로드 성공
구역 수 = 2
페이지 수 = 4
크기 = 426,496 bytes
```

정답 HWP와 2차 후보의 핵심 anchor:

```text
output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/stage1_autonum_range_narrow_anchor_social.md
```

핵심 지점:

```text
PARA_TEXT:
  <0x0012:AutoNumber:0x61746e6f><0x001f>(사회·문화)<PARA_END>

PARA_RANGE_TAG:
  payload = 0f 00 00 00 10 00 00 00 23 00 00 01
```

일반 `사회탐구 영역` 문단은 2차 후보에서 다시 U+2007로 저장된다.

## 7. 남은 diff

LCS inventory diff:

```text
output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/stage1_autonum_range_vs_oracle_hints_lcs.md
```

변화:

```text
이전 fresh diff:
  missing = 6
  changed = 531
  missing PARA_RANGE_TAG 존재

Stage 1 1차 후보 diff:
  missing = 5
  changed = 513
  missing PARA_RANGE_TAG 제거됨

Stage 1 2차 후보 diff:
  missing = 5
  changed = 530
  missing PARA_RANGE_TAG 제거됨
  전역 U+2007 변환은 제거됨
```

남은 missing은 DocInfo 보조 record 계열이다.

```text
DOC_DATA
FORBIDDEN_CHAR
COMPATIBLE_DOCUMENT
LAYOUT_COMPATIBILITY
TRACKCHANGE
```

## 8. 검증

```text
cargo fmt --check
cargo check
cargo build --bin rhwp
cargo test fixed_width_space
cargo test autonum_fwspace_materializes_hancom_range_tag_once
```

결과:

```text
success
```

## 9. 한컴 판정 요청

작업지시자 판정 대상:

```text
output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/exam_social_stage1_autonum_range.hwp
```

판정표:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/exam_social_stage1_autonum_range.hwp` | 비정상 종료 |  |  |  |  | 1차 후보, 전역 U+2007 변환 포함 |
| `output/poc/hwpx2hwp/task1110/stage1_save_side_gap_apply/exam_social_stage1_autonum_range_narrow.hwp` | 비정상 종료 |  |  |  |  | 2차 후보, AutoNumber 문단 한정 변환 |

## 10. Stage 2 — DocInfo bundle 후보

작업지시자 판정 결과, 2차 후보도 한컴 에디터에서 비정상 종료했다.

따라서 AutoNumber/fixed-width space 축은 정답지와 맞춰야 할 계약이지만, 이번 비정상 종료의 충분조건은
아닌 것으로 본다. 다음 후보는 Stage 1 diff에 남아 있던 DocInfo 누락 record 축이다.

`exam_social.hwpx` header에는 다음 XML이 존재한다.

```xml
<hh:compatibleDocument targetProgram="HWP201X">
  <hh:layoutCompatibility/>
</hh:compatibleDocument>
<hh:docOption>
  <hh:linkinfo path="" pageInherit="0" footnoteInherit="0"/>
</hh:docOption>
<hh:trackchageConfig flags="56"/>
```

기존 HWPX parser는 `pageInherit=0`, `footnoteInherit=0`일 때 `linkinfo`를 무시했다.
그러나 정답 HWP에는 다음 DocInfo bundle이 존재한다.

```text
DOC_DATA
FORBIDDEN_CHAR
COMPATIBLE_DOCUMENT
LAYOUT_COMPATIBILITY
TRACKCHANGE
```

Stage 2 후보는 `linkinfo` 값이 false여도 요소가 존재하면 HWP5 DocInfo bundle을 materialize한다.

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage2_docinfo_bundle_candidate/exam_social_stage2_docinfo_bundle.hwp
```

rhwp 재로드:

```text
로드 성공
구역 수 = 2
페이지 수 = 4
크기 = 427,008 bytes
```

정답지 대비 missing diff:

```text
output/poc/hwpx2hwp/task1110/stage2_docinfo_bundle_candidate/stage2_vs_oracle_missing_hints.md
```

요약:

```text
missing = 0
extra = 0
```

즉 Stage 1까지 남아 있던 DocInfo missing record는 제거되었다.

AutoNumber 지점도 Stage 1 2차 후보와 동일하게 정답지와 맞는다.

```text
output/poc/hwpx2hwp/task1110/stage2_docinfo_bundle_candidate/stage2_anchor_social.md
```

핵심:

```text
PARA_TEXT:
  <0x0012:AutoNumber:0x61746e6f><0x001f>(사회·문화)<PARA_END>

PARA_RANGE_TAG:
  payload = 0f 00 00 00 10 00 00 00 23 00 00 01
```

Stage 2 판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage2_docinfo_bundle_candidate/exam_social_stage2_docinfo_bundle.hwp` | 파일손상 |  |  |  |  | DocInfo missing 제거 후보 |

## 11. Stage 3 — TABLE attr projection 후보

Stage 2는 정답 HWP 대비 missing record를 제거했지만 한컴 에디터에서는 여전히 파일손상 판정이었다.
따라서 남은 문제는 단순 record 누락이 아니라 기존 record payload contract 차이로 본다.

`saved/111exam_social.hwp`와 정답 HWP를 `table-fields`로 비교한 결과, 반복적으로 다음 차이가
확인되었다.

```text
generated TABLE attr = 0x00000004 / 0x00000006 / 0x04000006
oracle TABLE attr    = 0x04000004 / 0x04000006 / 0x06000004 / 0x06000006
```

특징:

```text
1. rows/cols/cell_spacing/in_margin/row_count_hint/col_count_hint/tail은 같은데 TABLE attr만 다른 표가 많다.
2. 정답 HWP는 table padding이 0인 표에도 0x04000000 또는 0x06000000 상위 비트를 저장한다.
3. 현재 adapter는 table padding이 non-zero일 때만 0x04000000을 materialize한다.
4. exam_social.hwpx는 repeatHeader=true, pageBreak=CELL/NONE, 1x10/3x3 선택지 표가 반복되는 문서다.
```

따라서 Stage 3는 구현 규칙을 곧바로 확정하지 않고, 정답 HWP의 TABLE 관련 payload 일부를 generated
HWP에 graft한 판정용 후보로 TABLE attr 축이 파일손상의 충분조건인지 확인한다.

생성 명령:

```text
target/debug/rhwp hwp5-table-probe \
  samples/exam_social.hwp \
  output/poc/hwpx2hwp/task1110/stage2_docinfo_bundle_candidate/exam_social_stage2_docinfo_bundle.hwp \
  --out-dir output/poc/hwpx2hwp/task1110/stage3_table_attr_projection
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/01_ctrl_outer_margin_only.hwp
output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/02_table_attr_only.hwp
output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/03_table_tail_only.hwp
output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/04_ctrl_common_attr_only.hwp
output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/05_outer_margin_table_attr.hwp
output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/06_outer_margin_table_tail.hwp
output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/07_table_attr_tail.hwp
output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/08_all_table_axes.hwp
```

요약:

```text
02_table_attr_only.hwp:
  TABLE attr 13개를 정답지 payload로 graft

04_ctrl_common_attr_only.hwp:
  table CTRL_HEADER common_attr 6개를 정답지 payload로 graft

08_all_table_axes.hwp:
  TABLE attr 13개 + CTRL_HEADER common_attr 6개를 함께 graft
```

Stage 3 판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/02_table_attr_only.hwp` |  |  |  |  |  | TABLE attr 단독 graft |
| `output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/04_ctrl_common_attr_only.hwp` |  |  |  |  |  | table CTRL_HEADER common_attr 단독 graft |
| `output/poc/hwpx2hwp/task1110/stage3_table_attr_projection/08_all_table_axes.hwp` |  |  |  |  |  | TABLE attr + CTRL common_attr graft |

판정 해석 기준:

```text
1. 02가 열리면 TABLE attr 상위 비트가 파일손상 핵심 축이다.
2. 04만 열리면 table CTRL_HEADER common_attr의 noAdjust/flow bit가 핵심 축이다.
3. 08만 열리면 TABLE attr와 CTRL common_attr의 결합 contract가 필요하다.
4. 셋 모두 실패하면 table attr 외 다른 payload 축으로 이동한다.
```

작업지시자 판정:

| case | 한컴 판정 유형 |
|---|---|
| 01 | 파일손상 |
| 02 | 파일손상 |
| 03 | 파일손상 |
| 04 | 파일손상 |
| 05 | 파일손상 |
| 06 | 파일손상 |
| 07 | 파일손상 |
| 08 | 파일손상 |

해석:

```text
1. TABLE attr, TABLE tail, table CTRL_HEADER common_attr를 정답지 payload로 graft해도 모두 실패했다.
2. 따라서 exam_social 전체 파일손상의 충분조건은 table axis가 아니다.
3. 같은 방식의 table probe를 반복하지 않고, 문제 범위를 첫 페이지 분리본으로 축소한다.
```

## 12. Stage 4 — 첫 페이지 분리본 sentinel

작업지시자가 문제 범위를 단순화하기 위해 첫 페이지만 분리한 샘플을 제공했다.

```text
source HWPX:
  samples/hwpx/exam_social-p1.hwpx

oracle HWP:
  samples/exam_social-p1.hwp
```

현재 adapter로 생성한 판정 파일:

```text
output/poc/hwpx2hwp/task1110/stage4_p1_sentinel/exam_social-p1-current.hwp
```

rhwp 재로드:

```text
로드 성공
구역 수 = 1
페이지 수 = 1
크기 = 113,152 bytes
```

정답 HWP:

```text
구역 수 = 1
페이지 수 = 1
크기 = 117,248 bytes
```

정답지와 현재 생성본의 inventory diff:

```text
output/poc/hwpx2hwp/task1110/stage4_p1_sentinel/p1_vs_oracle_hints.md
output/poc/hwpx2hwp/task1110/stage4_p1_sentinel/p1_vs_oracle_docinfo_bundles.md
output/poc/hwpx2hwp/task1110/stage4_p1_sentinel/p1_vs_oracle_table_fields.md
```

요약:

```text
missing = 0
extra = 1
changed = 190
```

주요 차이:

```text
1. DocInfo BIN_DATA attr
   oracle    = 0x0001
   generated = 0x0101

2. DocInfo FACE_NAME payload
   oracle은 대체 글꼴/속성 payload를 포함한다.
   generated는 face name 문자열 중심의 짧은 payload만 저장한다.

3. TABLE attr
   p1 oracle은 0x00000004 / 0x00000006 / 0x0000000e 계열이 많다.
   generated는 일부 표에 0x04000000 상위 비트를 materialize한다.
```

주의:

```text
전체 exam_social 정답지는 TABLE attr 상위 비트를 많이 가진 반면,
첫 페이지 분리본 정답지는 같은 구간에서도 낮은 TABLE attr를 가진다.
따라서 TABLE attr 상위 비트는 문서/저장 경로별로 달라질 수 있으며,
파일손상 원인으로 단정하면 안 된다.
```

Stage 4 판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage4_p1_sentinel/exam_social-p1-current.hwp` |  |  |  |  |  | 첫 페이지 분리본 current adapter |

판정 해석:

```text
1. p1도 파일손상이면 원인은 문서 초반 공통 DocInfo/body contract다.
2. p1이 정상 열리면 전체 파일의 후반 구역, 대형 이미지, 또는 section boundary contract로 이동한다.
```

작업지시자 판정:

```text
파일손상
```

해석:

```text
1. 전체 exam_social의 후반 구역이 없어도 파일손상이 재현된다.
2. 따라서 2구역, 3~4페이지, 대형 이미지 5/6만으로 설명할 수 없다.
3. 첫 페이지의 공통 DocInfo/body record contract에서 원인을 찾아야 한다.
```

## 13. Stage 5 — p1 TABLE projection

p1 current와 p1 oracle의 table field diff에서는 TABLE attr 방향이 전체 문서와 다르게 나타난다.

```text
p1 oracle TABLE attr:
  0x00000004 / 0x00000006 / 0x0000000e

p1 generated TABLE attr:
  0x04000004 / 0x04000006 / 0x0400000e 일부 포함
```

전체 exam_social oracle은 TABLE attr 상위 비트를 많이 포함하지만, p1 oracle은 같은 초반 표 구간에서
상위 비트를 포함하지 않는다. 즉 TABLE attr materialization은 문서 저장 경로/샘플 단위로 다르게
나타나며, 전체 문서의 관찰을 p1에 그대로 적용하면 안 된다.

생성 명령:

```text
target/debug/rhwp hwp5-table-probe \
  samples/exam_social-p1.hwp \
  output/poc/hwpx2hwp/task1110/stage4_p1_sentinel/exam_social-p1-current.hwp \
  --out-dir output/poc/hwpx2hwp/task1110/stage5_p1_table_projection
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/01_ctrl_outer_margin_only.hwp
output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/02_table_attr_only.hwp
output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/03_table_tail_only.hwp
output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/04_ctrl_common_attr_only.hwp
output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/05_outer_margin_table_attr.hwp
output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/06_outer_margin_table_tail.hwp
output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/07_table_attr_tail.hwp
output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/08_all_table_axes.hwp
```

우선 판정 대상:

| file | 한컴 판정 유형 | 비고 |
|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/02_table_attr_only.hwp` |  | p1 TABLE attr만 oracle로 graft |
| `output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/04_ctrl_common_attr_only.hwp` |  | p1 table CTRL_HEADER common_attr만 oracle로 graft |
| `output/poc/hwpx2hwp/task1110/stage5_p1_table_projection/08_all_table_axes.hwp` |  | p1 TABLE attr + CTRL common_attr graft |

판정 해석:

```text
1. 02 또는 08이 열리면 p1 손상은 TABLE attr 상위 비트가 핵심이다.
2. 04만 열리면 table CTRL_HEADER common_attr가 핵심이다.
3. 셋 모두 파일손상이면 p1 원인은 TABLE 축이 아니며 DocInfo BIN_DATA/FACE_NAME payload로 이동한다.
```

## 14. Stage 6 — p1 머리말/꼬리말 CTRL_HEADER attr 확인

작업지시자 지시에 따라 첫 페이지 분리본에서 머리말/꼬리말 축을 먼저 확인했다.

Stage 4 p1 current와 정답지의 control bundle 비교에서 Header/Footer CTRL_HEADER 차이가 있었다.

```text
Footer CTRL_HEADER:
  oracle    = 74 6f 6f 66 74 00 3a 00 04 00 00 00
  generated = 74 6f 6f 66 00 00 00 00 04 00 00 00

Header(Odd) CTRL_HEADER:
  oracle    = 64 61 65 68 76 00 3a 00 02 00 00 00
  generated = 64 61 65 68 02 00 00 00 02 00 00 00

Header(Even) CTRL_HEADER:
  oracle/generated 모두 64 61 65 68 01 00 00 00 03 00 00 00
```

해석:

```text
1. HWPX에서 들어온 Header/Footer raw attr가 비어 있는 경우가 있다.
2. 한컴 HWP 저장 결과는 Odd Header와 Both Footer에 HWP5 raw attr를 materialize한다.
3. Even Header는 정답지에서도 낮은 apply bit만 유지하므로 보강하지 않는다.
```

구현 후보:

```text
Odd Header  -> raw_attr = 0x003a0076
Both Footer -> raw_attr = 0x003a0074
Even Header -> 기존 값 유지
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage6_p1_header_footer_trace/exam_social-p1-header-footer-attr.hwp
```

검증 결과:

```text
cargo fmt --check
cargo test header_footer_ctrl_attr_materializes_hancom_save_contract --lib
cargo check
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage6_p1_header_footer_trace/exam_social-p1-header-footer-attr.hwp
```

정답지 대비 diff:

```text
output/poc/hwpx2hwp/task1110/stage6_p1_header_footer_trace/p1_header_footer_attr_ctrl_bundles.md
output/poc/hwpx2hwp/task1110/stage6_p1_header_footer_trace/p1_header_footer_attr_hints.md
```

요약:

```text
matched = 852
missing = 0
extra = 1
changed = 188
```

Header/Footer CTRL_HEADER는 changed 후보에서 사라졌다.
남은 CTRL_HEADER 후보는 SectionDef 1건, Table 2건이다.

Stage 6 판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage6_p1_header_footer_trace/exam_social-p1-header-footer-attr.hwp` | 파일손상 |  |  |  |  | p1 Header/Footer attr materialize 후보 |

판정 해석:

```text
1. 이 파일이 열리면 p1 파일손상 원인은 Header/Footer CTRL_HEADER attr 축이다.
2. 여전히 파일손상이면 Header/Footer 축은 제거된 것으로 보고 SectionDef, DocInfo, Table 축으로 이동한다.
```

작업지시자 판정:

```text
한컴 에디터: 파일손상
rhwp-studio: 정상
```

해석:

```text
1. Header/Footer CTRL_HEADER attr 차이는 정답지와 맞췄다.
2. 하지만 한컴 파일손상은 유지되었다.
3. 따라서 Header/Footer attr는 필요한 보강일 수 있으나 p1 파일손상의 충분조건은 아니다.
4. Stage 6 diff에 남은 SectionDef/page_control 축을 다음 후보로 확인한다.
```

## 15. Stage 7 — p1 SectionDef/Page control 후보

Stage 6 후보와 정답지의 `SectionDef` 주변 차이를 비교했다.

주요 차이:

```text
SectionDef flags:
  oracle    = 0x20000000
  generated = 0x40000000

SectionDef tail:
  oracle    = 대표Language 0 + 17 byte zero
  generated = 대표Language 0 + 0x0001 marker + 15 byte zero

EndNote FOOTNOTE_SHAPE:
  oracle    separator_length = 0x2ff8, separator_margin_top = 224
  generated separator_length = 0,      separator_margin_top = -1
```

원인:

```text
1. HWPX secPr@masterPageCnt=1을 단순히 flags bit 30-31에 count=1로 저장하면 0x40000000이 된다.
   하지만 한컴이 exam_social-p1.hwpx를 HWP5로 저장한 정답지는 단일 Both 바탕쪽에서 0x20000000을 쓴다.

2. 기존 adapter는 바탕쪽이 있으면 SectionDef tail에 항상 0x0001 marker를 넣었다.
   exam_kor처럼 masterPageCnt=3인 경우에는 이 패턴이 맞지만,
   exam_social-p1처럼 단일 Both 바탕쪽인 경우 정답지는 marker 없이 zero tail이다.

3. HWPX endNotePr noteLine length="14692344"는 i16 범위를 넘는다.
   한컴 HWP5 저장본은 이 값을 버리지 않고 low word(0x2ff8)를 FOOTNOTE_SHAPE separator_length에 저장한다.
```

구현 후보:

```text
1. 단일 master page이고 flags가 HWPX count=1 패턴(0x40000000)이면
   한컴 저장본 패턴인 0x20000000으로 보정한다.

2. SectionDef tail은 master_pages.len() > 1인 경우에만 0x0001 marker를 넣고,
   단일 master page에서는 17 byte zero tail로 저장한다.

3. endNotePr noteLine length가 i16 범위를 넘는 양수이면 HWP5 저장본처럼 low word를 보존한다.
   해당 endNote separator_margin_top은 224로 materialize한다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage7_p1_sectiondef_page_contract/exam_social-p1-sectiondef-page.hwp
```

검증 결과:

```text
cargo fmt --check
cargo test single_master_page_flags_materialize_hancom_save_contract --lib
cargo test section_def_master_page_tail_marker_depends_on_master_page_count --lib
cargo test test_parse_endnote_long_note_line_keeps_hwp5_low_word --lib
cargo test header_footer_ctrl_attr_materializes_hancom_save_contract --lib
cargo check
cargo build --bin rhwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage7_p1_sectiondef_page_contract/exam_social-p1-sectiondef-page.hwp
```

결과:

```text
success
rhwp reload ok, sections=1, pages=1
```

정답지 대비 diff:

```text
output/poc/hwpx2hwp/task1110/stage7_p1_sectiondef_page_contract/p1_sectiondef_page_ctrl_bundles.md
output/poc/hwpx2hwp/task1110/stage7_p1_sectiondef_page_contract/p1_sectiondef_page_hints.md
```

요약:

```text
Stage 6:
  changed = 188
  ctrl_header SectionDef = 1
  page_control = 1

Stage 7:
  changed = 186
  ctrl_header SectionDef = 0
  page_control = 1
  남은 ctrl_header 후보 = Table 2건
```

Stage 7 판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage7_p1_sectiondef_page_contract/exam_social-p1-sectiondef-page.hwp` | 성공 |  |  |  | 성공 | p1 SectionDef/Page control 후보 |

판정 해석:

```text
1. 이 파일이 열리면 p1 파일손상 원인은 SectionDef flags/tail 또는 endNote FOOTNOTE_SHAPE 축이다.
2. 여전히 파일손상이면 SectionDef 축도 제거하고, 남은 p1 후보인 Table CTRL_HEADER reserved bit 축으로 이동한다.
```

작업지시자 판정:

```text
한컴 에디터에서 정상적으로 열림
```

해석:

```text
1. p1 파일손상은 Header/Footer attr 단독으로는 풀리지 않았고,
   SectionDef flags/tail + endNote FOOTNOTE_SHAPE 보강 후 정상으로 전환되었다.
2. 따라서 p1 손상 핵심 축은 SectionDef/Page control contract로 확정한다.
3. 같은 구현을 전체 exam_social.hwpx에 적용해 전체 문서 손상 해소 여부를 확인한다.
```

## 16. Stage 8 — 전체 exam_social 적용 후보

Stage 7에서 p1 파일손상 해소가 확인되었으므로, 같은 구현으로 전체 `exam_social.hwpx`를 다시 저장했다.

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage8_full_sectiondef_page_contract/exam_social-stage8.hwp
```

rhwp 재로드:

```text
구역 수 = 2
페이지 수 = 4
크기 = 427,008 bytes
```

diff 산출물:

```text
output/poc/hwpx2hwp/task1110/stage8_full_sectiondef_page_contract/full_stage8_hints.md
output/poc/hwpx2hwp/task1110/stage8_full_sectiondef_page_contract/full_stage8_ctrl_bundles.md
```

Stage 8 판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage8_full_sectiondef_page_contract/exam_social-stage8.hwp` | 파일손상 |  |  |  |  | full target |

판정 해석:

```text
1. 전체 파일도 열리면 #1110 파일손상은 Stage 7의 SectionDef/Page control 보강으로 해결된 것으로 본다.
2. 전체 파일만 실패하면 p1 이후 구역/대형 이미지/후반 표 축을 별도 stage로 분리한다.
```

작업지시자 판정:

```text
파일손상
```

## 17. Stage 9 — 2개 바탕쪽 SectionDef 계약 후보

Stage 8은 전체 문서에서 여전히 파일손상 판정이 났다. `samples/exam_social.hwp` 정답지와 Stage 8
생성본을 비교하면, `Section1`의 `SectionDef` payload가 아직 다르다.

핵심 차이:

```text
oracle    Section1 SectionDef flags = 0xC0000000
generated Section1 SectionDef flags = 0x80000000

oracle    SectionDef tail marker = 0
generated SectionDef tail marker = 1
```

`exam_social.hwpx`의 두 번째 구역은 바탕쪽이 2개이다.

```text
masterPageCnt=2
masterpage1 = Both
masterpage2 = Odd
```

반면 이전 Stage 7 구현은 `masterPageCnt > 1`이면 tail marker를 1로 넣었다. 이는 `exam_kor`처럼
바탕쪽이 3개인 케이스에는 맞았지만, `exam_social`의 2개 바탕쪽 케이스에는 맞지 않았다.

Stage 9 구현 후보:

```text
1. masterPageCnt=2이고 flags high bits가 0x80000000이면 0xC0000000으로 materialize한다.
2. SectionDef tail marker는 masterPageCnt >= 3일 때만 1로 저장한다.
3. masterPageCnt=1 또는 2에서는 tail marker를 0으로 저장한다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage9_full_two_masterpage_sectiondef_contract/exam_social-stage9.hwp
output/poc/hwpx2hwp/task1110/stage9_full_two_masterpage_sectiondef_contract/exam_social-p1-stage9.hwp
```

검증:

```text
cargo fmt --check
cargo check
cargo build --bin rhwp
cargo test single_master_page_flags_materialize_hancom_save_contract --lib
cargo test two_master_page_flags_materialize_hancom_save_contract --lib
cargo test section_def_master_page_tail_marker_depends_on_master_page_count --lib
```

결과:

```text
success
rhwp reload ok, full sections=2, pages=4
rhwp reload ok, p1 sections=1, pages=1
```

diff 산출물:

```text
output/poc/hwpx2hwp/task1110/stage9_full_two_masterpage_sectiondef_contract/full_stage9_hints.md
output/poc/hwpx2hwp/task1110/stage9_full_two_masterpage_sectiondef_contract/full_stage9_ctrl_bundles.md
```

Stage 9 diff 요약:

```text
changed = 530
missing = 0
extra = 0
ctrl_header SectionDef = 0
ctrl_header Header = 1
ctrl_header Footer = 1
ctrl_header Table = 6
page_control = 2
```

판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage9_full_two_masterpage_sectiondef_contract/exam_social-stage9.hwp` | 파일손상 |  |  |  |  | full target |
| `output/poc/hwpx2hwp/task1110/stage9_full_two_masterpage_sectiondef_contract/exam_social-p1-stage9.hwp` |  |  |  |  |  | p1 guard |

작업지시자 판정:

```text
파일손상
```

해석:

```text
1. SectionDef 축은 정답지와 맞아졌지만 파일손상이 남았다.
2. Stage 9에서 남은 고위험 CTRL_HEADER 차이는 Header/Footer attr 보강과 Table CTRL_HEADER 6건이다.
3. p1 Stage 7 성공은 Header/Footer attr 보강이 필수였다는 뜻이 아니라, SectionDef/Page control 축이
   손상 원인이었다는 뜻에 가깝다.
4. full 정답지는 Header/Footer CTRL_HEADER attr를 low attr 값으로 저장하므로, Stage 10에서
   Header/Footer attr 보강을 제거해 본다.
```

## 18. Stage 10 — Header/Footer attr 보강 제거 후보

Stage 9의 full diff에서 다음 차이가 남았다.

```text
Footer oracle    attr = 0x00000000
Footer generated attr = 0x003a0074

Header oracle    attr = 0x00000002
Header generated attr = 0x003a0076
```

따라서 이전에 넣은 Header/Footer attr materialization은 full `exam_social` 정답지와 맞지 않는
잘못된 후보로 정리하고 제거했다.

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/exam_social-stage10.hwp
output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/exam_social-p1-stage10.hwp
```

검증:

```text
cargo fmt --check
cargo check
cargo build --bin rhwp
cargo test single_master_page_flags_materialize_hancom_save_contract --lib
cargo test two_master_page_flags_materialize_hancom_save_contract --lib
cargo test section_def_master_page_tail_marker_depends_on_master_page_count --lib
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/exam_social-stage10.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/exam_social-p1-stage10.hwp
```

결과:

```text
success
rhwp reload ok, full sections=2, pages=4
rhwp reload ok, p1 sections=1, pages=1
```

diff 산출물:

```text
output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/full_stage10_hints.md
output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/full_stage10_ctrl_bundles.md
```

Stage 10 diff 요약:

```text
changed = 528
missing = 0
extra = 0
ctrl_header Header = 0
ctrl_header Footer = 0
ctrl_header SectionDef = 0
ctrl_header Table = 6
page_control = 2
```

판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/exam_social-stage10.hwp` |  |  |  |  |  | full target |
| `output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/exam_social-p1-stage10.hwp` |  |  |  |  |  | p1 guard |

다음 판단:

```text
1. Stage 10이 열리면 full 파일손상 원인은 Header/Footer attr 과 SectionDef 2개 바탕쪽 계약의 조합으로 본다.
2. Stage 10도 파일손상이면 남은 고위험 후보는 Table CTRL_HEADER 6건과 Page control 2건이다.
```

작업지시자 판정:

```text
한컴편집기에서 잘 열림.
머리말 내 텍스트 스타일은 정답지와 다름.
```

해석:

```text
1. Stage 10으로 파일손상/비정상 종료 축은 해소되었다.
2. 남은 문제는 머리말 내부 텍스트의 HWP5 paragraph/text/char-shape contract 정합성이다.
3. Header/Footer attr 보강 제거는 유지한다.
```

## 19. Stage 11 — 머리말/꼬리말 fixed-width space 제어문자 후보

정답지와 Stage 10 저장본의 `사회탐구` anchor trace를 비교했다.

핵심 차이:

```text
oracle    : 사회탐구<0x001f>영역
generated : 사회탐구 영역
```

한컴 HWPX→HWP 저장 결과는 머리말/꼬리말 내부 fixed-width space를 literal U+2007이 아니라
HWP5 fixed blank control `0x001f`로 저장한다. 이때 `PARA_HEADER.control_mask`의 bit 31도
같이 설정된다.

구현 후보:

```text
1. HWPX→HWP adapter에 paragraph context를 추가한다.
2. Header/Footer 내부 문단, 그리고 그 안의 table/textbox 중첩 문단까지 HeaderFooter context를 전파한다.
3. HeaderFooter context에서 U+2007이 있는 문단은 control_mask bit 31을 세운다.
4. serializer는 control_mask bit 31이 있는 U+2007을 HWP5 0x001f로 저장한다.
5. 본문 일반 U+2007은 기존처럼 literal U+2007로 유지한다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage11_header_fixed_width_space_contract/exam_social-stage11.hwp
output/poc/hwpx2hwp/task1110/stage11_header_fixed_width_space_contract/exam_social-p1-stage11.hwp
```

검증:

```text
cargo fmt --check
cargo build --bin rhwp
cargo test header_footer_fwspace_marks_hwp5_fixed_blank_control --lib
cargo test test_control_mask_fixed_width_space_serializes_as_hwp_control_code --lib
```

anchor trace 결과:

```text
generated: 사회탐구<0x001f>영역
PARA_HEADER/control_mask hash matched for the header table paragraph.
PARA_TEXT hash matched for the header table paragraph.
```

남은 차이:

```text
AutoNumber + fixed-width space + "(사회·문화)" 문단의 PARA_CHAR_SHAPE 시작 위치가
generated = 0,2,9
oracle    = 0,9,16
```

## 20. Stage 12 — AutoNumber fixed-width space char-shape offset 보정

Stage 11에서 fixed-width space의 저장 코드는 정답지와 맞았지만, AutoNumber 뒤 문자 모양 경계가
HWPX 논리 문자 기준으로 남아 있었다.

문제 구조:

```text
HWPX logical positions:
  placeholder space(1) + fixed-width space(1) + visible text...

HWP5 serialized positions:
  AutoNumber extended control(8) + fixed blank control(1) + visible text...
```

따라서 `AutoNumber + fixed-width space` 문단에서 `start_pos >= 2`인 `PARA_CHAR_SHAPE`
시작 위치는 `+7` 보정이 필요하다.

구현 후보:

```text
1. AutoNumber가 있고 text가 " \u{2007}"로 시작하는 문단을 대상으로 한다.
2. char shape start_pos가 2..8 사이에 있으면 아직 HWP5 위치로 보정되지 않은 것으로 판단한다.
3. start_pos >= 2인 char shape boundary를 +7 한다.
4. 2차 실행 시 다시 보정하지 않도록 idempotent 조건을 둔다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage12_header_autonum_charshape_contract/exam_social-stage12.hwp
output/poc/hwpx2hwp/task1110/stage12_header_autonum_charshape_contract/exam_social-p1-stage12.hwp
```

검증:

```text
cargo fmt --check
cargo build --bin rhwp
cargo test autonum_fwspace_materializes_char_shape_offsets_once --lib
cargo test header_footer_fwspace_marks_hwp5_fixed_blank_control --lib
target/debug/rhwp hwp5-anchor-trace output/poc/hwpx2hwp/task1110/stage12_header_autonum_charshape_contract/exam_social-stage12.hwp --needle 사회탐구 --section 0 --window 4 --out output/poc/hwpx2hwp/task1110/stage12_header_autonum_charshape_contract/generated_social_header_anchor.md
```

anchor trace 결과:

```text
사회탐구<0x001f>영역:
  PARA_HEADER hash matched
  PARA_TEXT hash matched
  PARA_CHAR_SHAPE hash matched

AutoNumber<0x001f>(사회·문화):
  PARA_HEADER hash matched
  PARA_TEXT hash matched
  PARA_CHAR_SHAPE hash matched
```

판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 문단번호 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage12_header_autonum_charshape_contract/exam_social-stage12.hwp` |  |  |  |  |  | full target |
| `output/poc/hwpx2hwp/task1110/stage12_header_autonum_charshape_contract/exam_social-p1-stage12.hwp` |  |  |  |  |  | p1 guard |

## 21. Stage 13 — 바탕쪽 AutoNumber placeholder space 제거

작업지시자 판정:

```text
머리글, 꼬리말 스타일은 적용됨.
하지만 3페이지부터 머리글 영역이 한컴 에디터에서 비정상적으로 출력됨.
```

Stage 12 저장본과 정답 HWP를 section1 기준으로 비교했다.

핵심 관찰:

```text
1. SectionDef/PAGE_DEF/PAGE_BORDER_FILL는 정답지와 주요 head 값이 일치한다.
2. 3페이지는 section1의 홀수쪽 바탕쪽이 적용되는 지점이다.
3. section1 바탕쪽의 페이지 번호 표 셀에서 AutoNumber-only 문단이 정답지와 다르다.
```

정답 HWP:

```text
PARA_HEADER char_count = 9
PARA_TEXT   = AutoNumber control + PARA_END
```

Stage 12 저장본:

```text
PARA_HEADER char_count = 10
PARA_TEXT   = U+0020 space + AutoNumber control + PARA_END
```

원본 HWPX `Contents/masterpage1.xml`, `Contents/masterpage2.xml` 확인 결과 해당 문단은 다음 구조다.

```xml
<hp:ctrl>
  <hp:autoNum num="1" numType="PAGE">...</hp:autoNum>
</hp:ctrl>
<hp:t/>
```

즉 XML에는 실제 공백 문자가 없고, 기존 HWPX 파서가 AutoNumber placeholder로 만든 공백이 HWP
직렬화까지 살아남은 것이다. 한컴 HWPX→HWP 저장 결과는 바탕쪽의 이 문단을 AutoNumber-only로 저장한다.

구현 후보:

```text
1. paragraph context에 MasterPage를 추가한다.
2. SectionDef.master_pages 내부 문단에는 MasterPage context를 전파한다.
3. MasterPage context에서 다음 조건의 문단만 핀셋 보정한다.
   - text == " "
   - char_offsets == [0]
   - controls == [AutoNumber]
4. 해당 문단의 text/char_offsets를 비우고 char_count=9로 유지한다.
5. 본문/머리말/꼬리말 일반 AutoNumber 문단은 변경하지 않는다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage13_masterpage_autonum_placeholder/exam_social-stage13.hwp
output/poc/hwpx2hwp/task1110/stage13_masterpage_autonum_placeholder/exam_social-p1-stage13.hwp
```

검증:

```text
cargo fmt --check
cargo check
cargo build --bin rhwp
cargo test master_page_autonum_removes_parser_placeholder_space --lib
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage13_masterpage_autonum_placeholder/exam_social-stage13.hwp
target/debug/rhwp hwp5-inventory output/poc/hwpx2hwp/task1110/stage13_masterpage_autonum_placeholder/exam_social-stage13.hwp --section 1 --out output/poc/hwpx2hwp/task1110/stage13_masterpage_autonum_placeholder/generated_section1_inventory.md
```

section1 바탕쪽 AutoNumber 문단 비교 결과:

```text
BodyText.Section1#23 PARA_HEADER:
  char_count = 9, hash matched with oracle

BodyText.Section1#24 PARA_TEXT:
  AutoNumber control + PARA_END, hash matched with oracle

BodyText.Section1#40 PARA_HEADER:
  char_count = 9, hash matched with oracle

BodyText.Section1#41 PARA_TEXT:
  AutoNumber control + PARA_END, hash matched with oracle
```

판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 3페이지 머리글 영역 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage13_masterpage_autonum_placeholder/exam_social-stage13.hwp` |  |  |  |  |  | full target |
| `output/poc/hwpx2hwp/task1110/stage13_masterpage_autonum_placeholder/exam_social-p1-stage13.hwp` |  |  |  |  |  | p1 guard |

## 22. Stage 14 — pagePr gutterType -> HWP5 PAGE_DEF binding attr

작업지시자 추가 판정:

```text
rhwp-studio는 3페이지도 정답지처럼 정상적으로 머리말 출력됨.
한컴편집기만 3페이지 머리말 영역을 다르게 해석함.
```

따라서 렌더링/IR 문제가 아니라 HWP5 저장 payload 계약 문제로 분리한다.

Stage 13 저장본과 정답 HWP의 section1 `PAGE_DEF`를 비교했다.

```text
PAGE_DEF head32:
  정답지와 저장본이 동일

PAGE_DEF payload hash:
  정답지   blake3:0b2e7cee3cef5fae...
  Stage13  blake3:21d23c421c15c616...
```

head32가 같고 `PAGE_DEF` size가 40바이트이므로 차이는 마지막 8바이트인 `gutter` 또는 `attr`
영역에 있다. `rhwp info` 기준 `gutter=0`은 같으므로, 남은 차이는 `PAGE_DEF.attr`이다.

원본 HWPX section1:

```xml
<hp:pagePr landscape="WIDELY" width="77102" height="111685" gutterType="LEFT_RIGHT">
```

기존 HWPX 파서는 `gutterType`을 읽지 않아 `PageDef.attr`의 제책 방법 비트를 0으로 저장했다.
한컴 HWP5 정답지는 `LEFT_RIGHT`를 맞쪽 편집 binding bit로 저장한다.

구현:

```text
HWPX pagePr@gutterType -> HWP5 PAGE_DEF attr binding bits

LEFT_ONLY   -> SingleSided  -> attr bits 1..2 = 0
LEFT_RIGHT  -> DuplexSided  -> attr bits 1..2 = 1
TOP_BOTTOM  -> TopFlip      -> attr bits 1..2 = 2
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage14_page_def_gutter_type_binding/exam_social-stage14.hwp
output/poc/hwpx2hwp/task1110/stage14_page_def_gutter_type_binding/exam_social-p1-stage14.hwp
```

Stage 14 확인:

```text
BodyText.Section1#6 PAGE_DEF payload hash:
  generated = blake3:0b2e7cee3cef5fae...
  oracle    = blake3:0b2e7cee3cef5fae...
```

즉 section1 `PAGE_DEF`는 정답 HWP와 payload hash가 일치한다.

검증:

```text
cargo fmt
cargo fmt --check
cargo check
cargo build --bin rhwp
cargo test test_parse_page_pr_gutter_type_materializes_hwp5_binding_attr --lib
git diff --check
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage14_page_def_gutter_type_binding/exam_social-stage14.hwp
```

판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 3페이지 머리글 영역 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage14_page_def_gutter_type_binding/exam_social-stage14.hwp` |  |  |  |  |  | full target |
| `output/poc/hwpx2hwp/task1110/stage14_page_def_gutter_type_binding/exam_social-p1-stage14.hwp` |  |  |  |  |  | p1 guard |

## 23. Stage 15/16 — section1 Odd masterpage line shape contract

작업지시자 판정:

```text
Stage14도 3페이지만 머리글 영역이 크게 출력됨.
특징: 3페이지 머리글 내 표와 글상자의 높이가 한컴편집기에서 더 크게 해석됨.
4페이지는 다시 정상 높이.
```

Section1의 3페이지/4페이지 차이는 바탕쪽 적용 유형 차이로 보았다. 정답 HWP와 Stage14 저장본의
Section1 masterpage bundle을 비교한 결과, 3페이지에 적용되는 Odd masterpage 구간에서
다음 record들은 모두 일치했다.

```text
BodyText.Section1#32 LIST_HEADER
BodyText.Section1#33 PARA_HEADER
BodyText.Section1#34 PARA_TEXT
BodyText.Section1#35 PARA_CHAR_SHAPE
BodyText.Section1#36 PARA_LINE_SEG
BodyText.Section1#37 CTRL_HEADER(Table)
BodyText.Section1#38 TABLE
BodyText.Section1#39 LIST_HEADER
BodyText.Section1#40..48 table cell paragraphs
BodyText.Section1#49 CTRL_HEADER(GenShape)
BodyText.Section1#51 SHAPE_LINE
```

차이가 남은 것은 `BodyText.Section1#50 SHAPE_COMPONENT` 하나였다.

Stage14 `SHAPE_COMPONENT#50` 비교:

```text
oracle    hash = ab3dc6586271d27c
generated hash = 37d64f6369c31b5a
```

디코드 결과:

```text
공통 치수/중심:
  offset = 0,0
  original = 100 x 100
  current  = 1 x 92409
  rotation_center = 0,46204
  flip = 0x00080000

차이 1: rendering scale matrix
  oracle    sx=0.01, sy=924.09
  generated sx=0.009999999776482582, sy=924.0900268554688

차이 2: lineShape border attr
  oracle    attr=0xd1000041
  generated attr=0xc0000001
```

lineShape attr 차이는 공식 스펙의 테두리 선 정보 속성으로 설명된다.

```text
style=SOLID                 -> bit 0..5 = 1
endCap=FLAT                 -> bit 6..9 = 1
headSz=MEDIUM_MEDIUM        -> bit 22..25 = 4
tailSz=MEDIUM_MEDIUM        -> bit 26..29 = 4
headfill=1, tailfill=1      -> bit 30, bit 31
```

따라서 HWPX `lineShape` 파서에서 `endCap`, `headSz`, `tailSz`를 HWP5 `ShapeBorderLine.attr`
로 materialize하도록 보강했다.

rendering matrix는 기존 Stage36의 f32 양자화 규칙을 전체 취소하지 않고, 바탕쪽 line shape에
한해 `curSz/orgSz`에서 계산되는 exact size ratio가 raw matrix와 충분히 가까울 때만 보정한다.

```text
current_width / original_width   = 1 / 100     = 0.01
current_height / original_height = 92409 / 100 = 924.09
```

Stage16 확인:

```text
BodyText.Section1#18 SHAPE_COMPONENT:
  generated hash = 9be1d362a6eb302f
  oracle hash    = 9be1d362a6eb302f

BodyText.Section1#50 SHAPE_COMPONENT:
  generated hash = ab3dc6586271d27c
  oracle hash    = ab3dc6586271d27c
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage16_masterpage_line_rendering_ratio/exam_social-stage16.hwp
output/poc/hwpx2hwp/task1110/stage16_masterpage_line_rendering_ratio/exam_social-p1-stage16.hwp
```

검증:

```text
cargo fmt
cargo check
cargo build --bin rhwp
cargo test document_core::converters::hwpx_to_hwp::tests::master_page_line_rendering_uses_exact_size_ratio --lib
cargo test parser::hwpx::section::tests::test_parse_hwpx_masterpage_line_materializes_shape_common_attr --lib
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage16_masterpage_line_rendering_ratio/exam_social-stage16.hwp
target/debug/rhwp hwp5-anchor-trace output/poc/hwpx2hwp/task1110/stage16_masterpage_line_rendering_ratio/exam_social-stage16.hwp --needle 32 --section 1 --window 10 --out output/poc/hwpx2hwp/task1110/stage16_masterpage_line_rendering_ratio/stage16_s1_32_trace.md
```

판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 3페이지 머리글 영역 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage16_masterpage_line_rendering_ratio/exam_social-stage16.hwp` |  |  |  |  |  | full target |
| `output/poc/hwpx2hwp/task1110/stage16_masterpage_line_rendering_ratio/exam_social-p1-stage16.hwp` |  |  |  |  |  | p1 guard |

작업지시자 판정:

```text
개선되지 않음.
```

따라서 Stage16에서 정답지와 맞춘 바탕쪽 line shape의 `SHAPE_COMPONENT` payload는 남은
3페이지 머리말/바탕쪽 영역 확대 현상의 직접 원인이 아닌 것으로 분리한다.

## 24. Stage 17 — 후속 section 첫 문단 break flag 정정

Stage16 이후에도 남은 Section1 초입의 차이를 다시 확인했다.

정답 HWP와 Stage16 생성본의 Section1 첫 `PARA_HEADER`는 다음 1바이트만 달랐다.

```text
record: BodyText.Section1#0 PARA_HEADER
offset: payload byte 11

oracle    = 0x03
generated = 0x04
```

해석:

```text
0x01 = 구역 나누기
0x02 = 다단 나누기
0x04 = 쪽 나누기
0x08 = 단 나누기

oracle 0x03 = 구역 + 다단
generated 0x04 = 쪽 나누기
```

`exam_social.hwpx`의 Section1 첫 문단은 `secPr`와 `colPr`를 포함하지만 `pageBreak="1"`이
아니다. 따라서 HWP5 저장 시 `0x03`이어야 한다. 기존 adapter는 후속 section 첫 문단이라는
이유로 `raw_break_type`을 `0x04`로 강제하고 있었다.

이번 수정:

```text
1. HWPX parser에서 hp:p의 pageBreak/columnBreak attr과 secPr/colPr 존재 여부를 bitwise 합성한다.
2. pageBreak 없는 secPr+colPr 문단은 0x03으로 저장한다.
3. pageBreak 있는 secPr+colPr 문단은 0x07으로 저장한다.
4. adapter의 후속 section 0x04 강제는 제거하고, raw_break_type이 비어 있을 때만 0x01 fallback을 적용한다.
```

Stage17 확인:

```text
exam_social Section1#0 PARA_HEADER:
  generated hash = 4cdb65f0d610f075
  oracle hash    = 4cdb65f0d610f075
  break_type     = 0x03

exam_kor guard Section1#0 PARA_HEADER:
  generated hash = f8a69c73b95c3020
  oracle hash    = f8a69c73b95c3020
  break_type     = 0x07
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_social-stage17.hwp
output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_social-p1-stage17.hwp
output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_kor-guard-stage17.hwp
```

검증:

```text
cargo fmt
cargo build --bin rhwp
cargo test parser::hwpx::section::tests::test_parse_section_col_pr_break_type_without_page_break --lib
cargo test parser::hwpx::section::tests::test_parse_section_col_pr_break_type_with_page_break --lib
cargo test document_core::converters::hwpx_to_hwp::tests::following_section_first_paragraph_break_type_preserves_materialized_flags --lib
cargo test document_core::converters::hwpx_to_hwp::tests::following_section_first_paragraph_break_type_fills_missing_section_flag --lib
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_social-stage17.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_social-p1-stage17.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_kor-guard-stage17.hwp
```

판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 3페이지 머리글 영역 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_social-stage17.hwp` |  |  |  |  |  | full target |
| `output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_social-p1-stage17.hwp` |  |  |  |  |  | p1 guard |
| `output/poc/hwpx2hwp/task1110/stage17_section_break_flags/exam_kor-guard-stage17.hwp` |  |  |  |  |  | #1099 guard |

작업지시자 판정:

```text
여전히 3페이지 머리말 내 글상자와 표의 높이가 머리말 영역보다 크게 출력된다.
이 문서의 특징은 1페이지의 머리말 크기가 3페이지의 머리말 크기로 그대로 유지되는 듯한
한컴 에디터 표시 버그다.
```

Stage17에서 Section1 첫 `PARA_HEADER`와 3페이지에 적용되는 masterpage bundle은 정답지와 맞췄으나
한컴 에디터 표시 차이는 남았다. 따라서 다음 축은 masterpage record 자체가 아니라, 한컴 HWP5
조판기가 masterpage 내부 문단/표 높이를 해석할 때 참조하는 DocInfo 글꼴 계약으로 이동한다.

## 25. Stage 18 — HWPX font typeInfo/default FACE_NAME 계약 보존

정답 HWP와 Stage17 생성본의 Section1 masterpage record bundle은 동일하지만, DocInfo `FACE_NAME`
payload가 크게 달랐다.

HWPX `header.xml`에는 각 `<hh:font>` 아래에 `<hh:typeInfo>`가 존재한다.

```xml
<hh:font face="굴림" type="TTF">
  <hh:typeInfo familyType="FCAT_GOTHIC" weight="6" proportion="0" .../>
</hh:font>
```

기존 parser/serializer는 이를 HWP5 `FACE_NAME`의 type info/default font 영역으로 저장하지 않았다.
한컴 에디터는 같은 masterpage record라도 `FACE_NAME`의 type info/default font 계약이 다르면
머리말 내부 텍스트/표 높이를 다르게 해석할 수 있다.

이번 수정:

```text
1. Font IR에 HWP5 FACE_NAME type_info 10바이트를 보존한다.
2. HWPX <hh:font type="TTF|HFT">를 FACE_NAME attr 하위 type bit로 저장한다.
3. HWPX <hh:typeInfo>를 HWP5 FACE_NAME type info 영역으로 저장한다.
4. 한컴 HWPX→HWP 저장 결과에서 반복 확인되는 기본 영문 글꼴명을 보존한다.
5. HFT 계열은 이름으로 serif 값을 추정하지 않고, 정답지와 같이 type_info[1]=0으로 둔다.
```

정답지 대조:

```text
FACE_NAME #8 굴림:
  oracle    size=29 hash=daa3cd6101a92c30...
  generated size=29 hash=daa3cd6101a92c30...

FACE_NAME #9 한컴바탕:
  oracle    size=53 hash=813bcdbba5ebd29d...
  generated size=53 hash=813bcdbba5ebd29d...

FACE_NAME #10 함초롬바탕:
  oracle    size=45 hash=6c353a0a6bb3d8b6...
  generated size=45 hash=6c353a0a6bb3d8b6...
```

HFT 계열 일부 기본 영문 글꼴명은 아직 정답지와 byte-perfect까지 맞추지 못했지만, Stage17의
짧은 FACE_NAME payload 문제는 해소했다.

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage18_facename_typeinfo_contract/exam_social-stage18.hwp
output/poc/hwpx2hwp/task1110/stage18_facename_typeinfo_contract/exam_social-p1-stage18.hwp
```

비교 산출물:

```text
output/poc/hwpx2hwp/task1110/stage18_facename_typeinfo_contract/docinfo_stage18_hints.md
```

검증:

```text
cargo fmt
cargo test face_name_with_type_info -- --nocapture
cargo test test_parse_hwpx_font_type_info_and_hwp5_default_name -- --exact --nocapture
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage18_facename_typeinfo_contract/exam_social-stage18.hwp
target/debug/rhwp hwp5-inventory-diff samples/exam_social.hwp output/poc/hwpx2hwp/task1110/stage18_facename_typeinfo_contract/exam_social-stage18.hwp --align index --report hints --focus docinfo --out output/poc/hwpx2hwp/task1110/stage18_facename_typeinfo_contract/docinfo_stage18_hints.md
```

판정 대상:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 머리말/꼬리말 | 3페이지 머리글 영역 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage18_facename_typeinfo_contract/exam_social-stage18.hwp` |  |  |  |  |  | full target |
| `output/poc/hwpx2hwp/task1110/stage18_facename_typeinfo_contract/exam_social-p1-stage18.hwp` |  |  |  |  |  | p1 guard |

## 26. 정정 — 한컴2020 손상/변조 경고는 Stage18 신규 회귀가 아니다

작업지시자 확인 결과, 다음 파일도 한컴2020에서 동일하게 “문서가 손상되었거나 변조되었을
가능성이 있다”는 판정을 받았다.

```text
output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/exam_social-stage10.hwp
output/poc/hwpx2hwp/task1110/stage10_full_header_footer_attr_revert/exam_social-p1-stage10.hwp
```

따라서 Stage18의 `FACE_NAME typeInfo/default` 보존 변경만을 한컴2020 경고 원인으로 단정하면
안 된다. 현재 판정 축은 다음처럼 분리한다.

```text
1. 한컴2010 열림 여부
2. 한컴2020 손상/변조 가능성 경고 여부
3. 3페이지 머리말 영역 과대 렌더링 여부
```

Stage18은 DocInfo 글꼴 계약 차이를 줄인 단계이지만, 한컴2020 경고는 Stage10 p1에서도 재현되는
공통 p1/container 계약 문제로 재분류한다.

## 27. Stage 19 — 한컴2020 p1 경고 축 분리 probe

Stage10 p1과 Stage18 p1 모두 한컴2020 경고가 있으므로, `exam_social-p1` 기준으로 FileHeader,
DocInfo, BodyText, container metadata 축을 정답지 raw stream으로 분리 치환한 후보를 만들었다.

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage19_p1_hancom2020_warning_probe/01_stage18_current.hwp
output/poc/hwpx2hwp/task1110/stage19_p1_hancom2020_warning_probe/02_oracle_file_header.hwp
output/poc/hwpx2hwp/task1110/stage19_p1_hancom2020_warning_probe/03_oracle_docinfo.hwp
output/poc/hwpx2hwp/task1110/stage19_p1_hancom2020_warning_probe/04_oracle_bodytext.hwp
output/poc/hwpx2hwp/task1110/stage19_p1_hancom2020_warning_probe/05_oracle_docinfo_bodytext.hwp
output/poc/hwpx2hwp/task1110/stage19_p1_hancom2020_warning_probe/06_oracle_file_header_docinfo_bodytext.hwp
output/poc/hwpx2hwp/task1110/stage19_p1_hancom2020_warning_probe/07_oracle_container_meta.hwp
```

생성 리포트:

```text
output/poc/hwpx2hwp/task1110/stage19_p1_hancom2020_warning_probe/stage19_generation.md
```

판정표:

| file | 한컴2020 판정 | 한컴2010 판정 | p1 렌더링 | 비고 |
|---|---|---|---|---|
| `01_stage18_current.hwp` |  |  |  | Stage18 p1 baseline |
| `02_oracle_file_header.hwp` |  |  |  | FileHeader만 정답지 raw 사용 |
| `03_oracle_docinfo.hwp` |  |  |  | DocInfo/DocProperties만 정답지 raw/model 사용 |
| `04_oracle_bodytext.hwp` |  |  |  | BodyText Section0만 정답지 raw 사용 |
| `05_oracle_docinfo_bodytext.hwp` |  |  |  | DocInfo + BodyText Section0 정답지 사용 |
| `06_oracle_file_header_docinfo_bodytext.hwp` |  |  |  | FileHeader + DocInfo + BodyText Section0 정답지 사용 |
| `07_oracle_container_meta.hwp` |  |  |  | FileHeader + preview/extra stream 정답지 사용 |

검증:

```text
cargo test task1110_stage19_generate_p1_hancom2020_warning_probe -- --ignored --nocapture
```

결과:

```text
success
```

모든 Stage19 후보는 rhwp 기준 1페이지로 재로드된다.

작업지시자 한컴2020 판정:

| file | 한컴2020 판정 | 해석 |
|---|---|---|
| `01_stage18_current.hwp` | 파일변조 | baseline 재현 |
| `02_oracle_file_header.hwp` | 파일변조 | FileHeader 단독 원인 아님 |
| `03_oracle_docinfo.hwp` | 파일변조 | DocInfo 단독 원인 아님 |
| `04_oracle_bodytext.hwp` | 정상 | Section0 BodyText raw contract가 핵심 축 |
| `05_oracle_docinfo_bodytext.hwp` | 정상 | 정상화 원인은 BodyText 축으로 본다 |
| `06_oracle_file_header_docinfo_bodytext.hwp` | 정상 | 정상화 원인은 BodyText 축으로 본다 |
| `07_oracle_container_meta.hwp` | 파일변조 | preview/extra stream 단독 원인 아님 |

결론:

```text
한컴2020의 p1 손상/변조 가능성 판정은 FileHeader, DocInfo, preview/extra stream 문제가 아니라
BodyText/Section0 내부 record contract 문제다.
```

## 28. Stage 20 — Section0 record tag 단위 분리

Stage19에서 `04_oracle_bodytext.hwp`가 정상 판정을 받았으므로, 다음은 Section0 내부 record tag를
정답지 값으로 graft해 한컴2020 판정이 정상화되는 최소 축을 찾는다.

첫 실행에서 `PARA_HEADER` record의 tag 순서는 같지만 일부 record level이 정답지와 다르다는 점이
확인되었다.

```text
record index = 38
tag = PARA_HEADER
generated level = 3
oracle level = 2
```

따라서 Stage20 probe는 tag가 같은 record에 한해 level을 포함한 record header 전체를 정답지 값으로
치환하는 방식으로 진행한다. 이 level 차이 자체가 한컴2020 판정의 후보 축이다.

생성 대상:

```text
output/poc/hwpx2hwp/task1110/stage20_p1_section0_tag_probe/
```

판정표:

| file | 한컴2020 판정 | p1 렌더링 | 비고 |
|---|---|---|---|
| `01_para_header.hwp` |  |  | PARA_HEADER |
| `02_para_text.hwp` |  |  | PARA_TEXT |
| `03_para_char_shape.hwp` |  |  | PARA_CHAR_SHAPE |
| `04_para_line_seg.hwp` |  |  | PARA_LINE_SEG |
| `05_para_range_tag.hwp` |  |  | PARA_RANGE_TAG |
| `06_ctrl_header.hwp` |  |  | CTRL_HEADER |
| `07_list_header.hwp` |  |  | LIST_HEADER |
| `08_page_records.hwp` |  |  | PAGE_DEF/FOOTNOTE_SHAPE/PAGE_BORDER_FILL |
| `09_shape_records.hwp` |  |  | SHAPE/CTRL_DATA records |
| `10_table_records.hwp` |  |  | TABLE |
| `11_para_core.hwp` |  |  | PARA_HEADER/TEXT/CHAR/LINE/RANGE |
| `12_control_envelope.hwp` |  |  | control/list/page/shape/table |
| `13_text_layout_table.hwp` |  |  | paragraph core + TABLE |
| `14_all_section0_tags.hwp` |  |  | all Section0 tags |

작업지시자 한컴2020 판정:

| file | 한컴2020 판정 | 해석 |
|---|---|---|
| `11_para_core.hwp` | 정상 | 문단 core record 묶음이 정상화 축 |
| `13_text_layout_table.hwp` | 정상 | TABLE은 추가되어도 정상, 핵심은 문단 core |
| `14_all_section0_tags.hwp` | 정상 | Section0 전체 치환은 정상 |

결론:

```text
한컴2020 경고 원인은 TABLE/control/shape 계열이 아니라,
PARA_HEADER, PARA_TEXT, PARA_CHAR_SHAPE, PARA_LINE_SEG, PARA_RANGE_TAG 사이의 상호 contract다.
단일 tag 치환 후보가 정상으로 보고되지 않았으므로 다음 단계는 para core record의 2개/3개 조합으로
최소 정상화 조합을 찾는다.
```

## 29. Stage 21 — para core 조합 probe

Stage20에서 `11_para_core.hwp`가 정상 판정을 받았으므로, 다음은 문단 core record 조합을 더 잘게
나눈다.

생성 대상:

```text
output/poc/hwpx2hwp/task1110/stage21_p1_para_core_combo_probe/
```

판정표:

| file | 한컴2020 판정 | p1 렌더링 | 비고 |
|---|---|---|---|
| `01_header_text.hwp` |  |  | PARA_HEADER + PARA_TEXT |
| `02_header_char.hwp` |  |  | PARA_HEADER + PARA_CHAR_SHAPE |
| `03_header_line.hwp` |  |  | PARA_HEADER + PARA_LINE_SEG |
| `04_header_range.hwp` |  |  | PARA_HEADER + PARA_RANGE_TAG |
| `05_text_char.hwp` |  |  | PARA_TEXT + PARA_CHAR_SHAPE |
| `06_text_line.hwp` |  |  | PARA_TEXT + PARA_LINE_SEG |
| `07_text_range.hwp` |  |  | PARA_TEXT + PARA_RANGE_TAG |
| `08_char_line.hwp` |  |  | PARA_CHAR_SHAPE + PARA_LINE_SEG |
| `09_char_range.hwp` |  |  | PARA_CHAR_SHAPE + PARA_RANGE_TAG |
| `10_line_range.hwp` |  |  | PARA_LINE_SEG + PARA_RANGE_TAG |
| `11_header_text_char.hwp` |  |  | PARA_HEADER + PARA_TEXT + PARA_CHAR_SHAPE |
| `12_header_text_line.hwp` |  |  | PARA_HEADER + PARA_TEXT + PARA_LINE_SEG |
| `13_header_text_range.hwp` |  |  | PARA_HEADER + PARA_TEXT + PARA_RANGE_TAG |
| `14_header_char_line.hwp` |  |  | PARA_HEADER + PARA_CHAR_SHAPE + PARA_LINE_SEG |
| `15_header_char_range.hwp` |  |  | PARA_HEADER + PARA_CHAR_SHAPE + PARA_RANGE_TAG |
| `16_header_line_range.hwp` |  |  | PARA_HEADER + PARA_LINE_SEG + PARA_RANGE_TAG |
| `17_text_char_line.hwp` |  |  | PARA_TEXT + PARA_CHAR_SHAPE + PARA_LINE_SEG |
| `18_text_char_range.hwp` |  |  | PARA_TEXT + PARA_CHAR_SHAPE + PARA_RANGE_TAG |
| `19_text_line_range.hwp` |  |  | PARA_TEXT + PARA_LINE_SEG + PARA_RANGE_TAG |
| `20_char_line_range.hwp` |  |  | PARA_CHAR_SHAPE + PARA_LINE_SEG + PARA_RANGE_TAG |

작업지시자 한컴2020 판정:

| file | 한컴2020 판정 | 해석 |
|---|---|---|
| `01_header_text.hwp` | 정상 | 최소 정상화 후보 |
| `11_header_text_char.hwp` | 정상 | `PARA_CHAR_SHAPE`는 추가되어도 정상 |
| `12_header_text_line.hwp` | 정상 | `PARA_LINE_SEG`는 추가되어도 정상 |
| `13_header_text_range.hwp` | 정상 | `PARA_RANGE_TAG`는 추가되어도 정상 |

결론:

```text
한컴2020 경고의 최소 축은 PARA_HEADER + PARA_TEXT 쌍이다.
PARA_HEADER 단독 또는 PARA_TEXT 단독이 아니라, 두 record가 level/char_count/text payload 관점에서
서로 일관된 쌍으로 저장되어야 한다.
```

다음 단계는 `exam_social-p1`의 `PARA_HEADER`와 `PARA_TEXT` 차이를 record index 단위로 해부하고,
정답지와 다른 필드 중 어느 값을 저장기에 반영해야 하는지 결정한다.

## 30. Stage 22 — Header/Footer 내부 문단 level 계약 구현 후보

Stage21에서 `PARA_HEADER + PARA_TEXT` 치환 후보가 한컴2020 정상 판정을 받았다. Stage20/21 diff를
확인한 결과 가장 명확한 구조 차이는 header/footer control 내부의 `LIST_HEADER` 아래 문단 레벨이었다.

정답지:

```text
CTRL_HEADER(Header/Footer)
  LIST_HEADER
  PARA_HEADER
    PARA_TEXT
    PARA_CHAR_SHAPE
    PARA_LINE_SEG
```

기존 저장기:

```text
CTRL_HEADER(Header/Footer)
  LIST_HEADER
    PARA_HEADER
      PARA_TEXT
      PARA_CHAR_SHAPE
      PARA_LINE_SEG
```

따라서 `serialize_header_footer_list_header_with_paragraphs()`에서 header/footer 내부 문단을
`LIST_HEADER`보다 한 단계 깊게 쓰지 않고, `LIST_HEADER`와 같은 level로 쓰도록 수정했다.

수정 파일:

```text
src/serializer/control.rs
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-p1-stage22.hwp
output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-stage22.hwp
```

검증:

```text
cargo fmt
cargo build --bin rhwp
target/debug/rhwp convert samples/hwpx/exam_social-p1.hwpx output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-p1-stage22.hwp
target/debug/rhwp convert samples/hwpx/exam_social.hwpx output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-stage22.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-p1-stage22.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-stage22.hwp
target/debug/rhwp hwp5-inventory-diff samples/exam_social-p1.hwp output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-p1-stage22.hwp --align index --report diff --section 0 --out output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/p1_stage22_diff.md
```

rhwp 재로드:

```text
exam_social-p1-stage22.hwp: ok, sections=1, pages=1
exam_social-stage22.hwp: ok, sections=2, pages=4
```

diff 요약:

```text
Stage21 baseline p1 diff:
  Section0 scope_changed = 48

Stage22 candidate p1 diff:
  Section0 scope_changed = 0
  전체 scope_changed = 4 (DocInfo record alignment only)
```

판정표:

| file | 한컴2020 판정 | 한컴2010 판정 | p1/full 렌더링 | 비고 |
|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-p1-stage22.hwp` | 문서 손상 |  |  | p1 guard |
| `output/poc/hwpx2hwp/task1110/stage22_header_footer_para_level_candidate/exam_social-stage22.hwp` | 문서 손상 |  |  | full target |

해석:

```text
1. Stage22는 Section0 scope_changed를 제거했지만 한컴2020 손상 판정은 해소하지 못했다.
2. 따라서 손상 판정의 남은 원인은 header/footer 내부 문단 level이 아니라 PARA_HEADER/PARA_TEXT payload다.
3. Stage21의 01_header_text.hwp가 정상 판정이었으므로, 다음 단계는 payload가 다른 PARA_HEADER/PARA_TEXT
   쌍을 record index 단위로 분리한다.
```

## 31. Stage 23 — PARA_HEADER/PARA_TEXT payload 쌍 단위 분리 probe

Stage22 후보를 기준으로 Section0 scope level은 정답지와 맞춘 상태다. 이 상태에서 아직 payload가 다른
`PARA_HEADER/PARA_TEXT` 쌍을 정답지 record로 치환해 한컴2020 손상 판정의 최소 문단 쌍을 찾는다.

생성 대상:

```text
output/poc/hwpx2hwp/task1110/stage23_p1_header_text_payload_pair_probe/
```

생성 파일:

```text
01_pair_0_1.hwp
02_pair_67_68.hwp
03_pair_72_73.hwp
04_pair_90_91.hwp
05_pair_134_135.hwp
06_pair_147_148.hwp
07_pair_154_155.hwp
08_pair_169_170.hwp
09_pair_176_177.hwp
10_front_pairs.hwp
11_late_pairs.hwp
12_all_changed_pairs.hwp
```

검증:

```text
cargo test task1110_stage23_generate_p1_header_text_payload_pair_probe -- --ignored --nocapture
```

결과:

```text
success
모든 후보 rhwp reload pages=1
```

판정표:

| file | 한컴2020 판정 | p1 렌더링 | 비고 |
|---|---|---|---|
| `01_pair_0_1.hwp` |  |  | records 0/1 |
| `02_pair_67_68.hwp` |  |  | records 67/68 |
| `03_pair_72_73.hwp` |  |  | records 72/73 |
| `04_pair_90_91.hwp` |  |  | records 90/91 |
| `05_pair_134_135.hwp` |  |  | records 134/135 |
| `06_pair_147_148.hwp` |  |  | records 147/148 |
| `07_pair_154_155.hwp` |  |  | records 154/155 |
| `08_pair_169_170.hwp` |  |  | records 169/170 |
| `09_pair_176_177.hwp` |  |  | records 176/177 |
| `10_front_pairs.hwp` |  |  | records 0/1, 67/68, 72/73, 90/91 |
| `11_late_pairs.hwp` |  |  | records 134/135, 147/148, 154/155, 169/170, 176/177 |
| `12_all_changed_pairs.hwp` |  |  | all changed PARA_HEADER/TEXT pairs |

작업지시자 판정:

```text
01, 10, 12 : 정상판정
```

해석:

```text
1. 10/12가 정상인 것은 01이 포함되어 있기 때문으로 본다.
2. 01_pair_0_1 단독 정상 판정이므로, 한컴2020 손상/변조 판정의 핵심은 Section0 첫 문단의
   PARA_HEADER/PARA_TEXT 쌍이다.
3. 따라서 추가 probe를 반복하지 않고, 첫 문단 raw payload의 의미 차이를 소스 레벨에서 고친다.
```

정답지와 Stage22 생성본의 첫 문단 차이:

```text
oracle:
  control_mask = 0x00210804
  PARA_TEXT = SectionDef, ColumnDef, NewNumber(0x0015), Footer, Table, Header, Header, PageHide(0x0015), text

generated stage22:
  control_mask = 0x00250804
  PARA_TEXT = SectionDef, ColumnDef, NewNumber(0x0012), Footer, Table, Header, Header, text, PageHide(0x0015)
```

핵심 차이:

```text
1. HWPX newNum을 HWP5 저장 시 0x0012 자동번호 계열로 저장했다.
2. HWPX newNum이 문단 텍스트 흐름의 inline marker를 만들지 않아, 뒤따르는 PageHide가 visible text 뒤로 밀렸다.
3. 이 때문에 control_mask에는 불필요한 0x0012 bit가 추가되고, 한컴2020은 첫 문단 control chain을
   손상/변조로 판정했다.
```

## 32. Stage 24 — newNum/PageHide 첫 문단 control chain 계약 수정

수정 방향:

```text
1. HWPX parser에서 hp:newNum도 pageHiding처럼 8 UTF-16 code unit inline marker를 갖도록 한다.
2. 단, autoNum과 달리 visible placeholder space는 만들지 않는다.
3. HWP5 저장 시 Control::NewNumber는 0x0015 + nwno로 저장한다.
4. AutoNumber(0x0012 + atno)의 기존 placeholder 동작은 유지한다.
```

수정 파일:

```text
src/parser/hwpx/section.rs
src/serializer/body_text.rs
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-p1-stage24.hwp
output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-stage24.hwp
```

검증:

```text
cargo fmt --check
cargo test -q serializer::body_text::tests::test_control_char_code
cargo build --bin rhwp
target/debug/rhwp convert samples/hwpx/exam_social-p1.hwpx output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-p1-stage24.hwp
target/debug/rhwp convert samples/hwpx/exam_social.hwpx output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-stage24.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-p1-stage24.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-stage24.hwp
target/debug/rhwp hwp5-anchor-trace output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-p1-stage24.hwp --needle 밑줄 --section 0 --window 2 --out output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/generated_first_para_trace.md
```

검증 결과:

```text
cargo fmt --check: success
cargo test serializer::body_text::tests::test_control_char_code: success
cargo build --bin rhwp: success
exam_social-p1-stage24.hwp: rhwp reload ok, pages=1
exam_social-stage24.hwp: rhwp reload ok, pages=4
```

첫 문단 raw trace:

```text
PARA_HEADER hash = d5fe7a0e57ed1320
PARA_TEXT   hash = 1f0617019b1f706a

정답지 samples/exam_social-p1.hwp 의 첫 문단 PARA_HEADER/PARA_TEXT와 동일하다.
```

판정표:

| file | 한컴2020 판정 | 한컴2010 판정 | p1/full 렌더링 | 비고 |
|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-p1-stage24.hwp` | 정상 |  |  | p1 guard |
| `output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/exam_social-stage24.hwp` | 정상 |  |  | full target |

작업지시자 판정:

```text
이제 2 파일모두 파일변조 손상 현상은 제거되었습니다.
```

결론:

```text
1. 한컴2020의 손상/변조 판정 원인은 HWPX newNum을 HWP5 PARA_TEXT에서 0x0012 자동번호 계열로
   저장한 것과, newNum inline marker 누락으로 PageHide가 visible text 뒤로 밀린 것이다.
2. NewNumber를 0x0015 + nwno로 저장하고, HWPX parser에서 newNum을 비가시 inline control marker로
   보존하면 p1/full 모두 손상/변조 판정이 제거된다.
3. Stage24 구현 후보는 task1110의 핵심 파일손상 축을 해결했다.
```

## 33. Stage 25 — 3페이지 머리말 쪽번호 앞 공백 원인 정리

작업지시자 관찰:

```text
3페이지 머리글 글상자 내용이 현재 {space}3 이다.
한컴편집기에서 space를 삭제하면 정상 조판된다.
```

원인 확인:

```text
1. generated HWP의 PARA_TEXT에는 실제 " 3" 문자열이 없다.
2. 쪽번호 자동번호 문단의 PARA_TEXT는 정답지와 동일하게 AutoNumber control만 가진다.
3. 차이는 해당 문단이 참조하는 DocInfo PARA_SHAPE에 있었다.
```

정답지와 Stage24 생성본의 차이:

```text
paragraph para_shape_id = 58
DocInfo PARA_SHAPE record = DocInfo#254

oracle    DocInfo#254 head32 = 80 00 00 00 ...
generated DocInfo#254 head32 = 00 00 00 00 ...
```

HWPX 원본의 `hh:paraPr id="58"`:

```xml
<hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="KEEP_WORD" .../>
```

해석:

```text
1. HWPX parser가 breakNonLatinWord를 HWP5 ParaShape attr1 bit 7로 반영하지 않았다.
2. 이 때문에 한컴 에디터가 바탕쪽/머리말의 AutoNumber-only 문단을 정답지와 다르게 해석했다.
3. 사용자가 본 {space}3은 literal PARA_TEXT 공백이 아니라, AutoNumber 문단의 문단 모양 계약 차이로
   한컴이 표시/조판한 자리표시 폭으로 본다.
```

수정:

```text
src/parser/hwpx/header.rs

breakSetting.breakNonLatinWord="KEEP_WORD"  -> ParaShape.attr1 bit 7 set
breakSetting.breakNonLatinWord="BREAK_WORD" -> ParaShape.attr1 bit 7 clear
```

주의:

```text
이전 이슈에서 문제가 되었던 "모든 ParaShape에 bit 7을 강제로 set"하는 방식은 사용하지 않는다.
HWPX paraPr의 명시 속성만 반영한다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage25_break_non_latin_parashape/exam_social-p1-stage25.hwp
output/poc/hwpx2hwp/task1110/stage25_break_non_latin_parashape/exam_social-stage25.hwp
```

검증:

```text
cargo fmt --check
cargo test -q parser::hwpx::header::tests::test_parse_hwpx_para_shape_break_non_latin_word_bit
cargo test -q document_core::converters::hwpx_to_hwp::tests::master_page_autonum_removes_parser_placeholder_space
cargo build --bin rhwp
target/debug/rhwp convert samples/hwpx/exam_social-p1.hwpx output/poc/hwpx2hwp/task1110/stage25_break_non_latin_parashape/exam_social-p1-stage25.hwp
target/debug/rhwp convert samples/hwpx/exam_social.hwpx output/poc/hwpx2hwp/task1110/stage25_break_non_latin_parashape/exam_social-stage25.hwp
target/debug/rhwp hwp5-inventory output/poc/hwpx2hwp/task1110/stage25_break_non_latin_parashape/exam_social-stage25.hwp --section 1 --format md --out /tmp/generated_stage25_s1_inventory.md
target/debug/rhwp hwp5-anchor-trace output/poc/hwpx2hwp/task1110/stage25_break_non_latin_parashape/exam_social-stage25.hwp --needle " 3" --section 1 --window 18 --out /tmp/generated_stage25_s1_space3.md
```

검증 결과:

```text
cargo fmt --check: success
cargo test parser::hwpx::header::tests::test_parse_hwpx_para_shape_break_non_latin_word_bit: success
cargo test document_core::converters::hwpx_to_hwp::tests::master_page_autonum_removes_parser_placeholder_space: success
cargo build --bin rhwp: success
exam_social-p1-stage25.hwp: generated
exam_social-stage25.hwp: generated
" 3" anchor trace hits: 0
```

Stage25 생성본의 DocInfo#254:

```text
generated DocInfo#254 head32 = 80 00 00 00 ...
oracle    DocInfo#254 head32 = 80 00 00 00 ...
```

판정표:

| file | 한컴2020 판정 | 3페이지 머리글 쪽번호 | 3페이지 머리글 영역 | 비고 |
|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage25_break_non_latin_parashape/exam_social-p1-stage25.hwp` |  |  |  | p1 guard |
| `output/poc/hwpx2hwp/task1110/stage25_break_non_latin_parashape/exam_social-stage25.hwp` |  |  |  | full target |

## 34. Stage 26 — PARA_SHAPE payload 58바이트 계약

작업지시자 정정:

```text
개선되지 않았다.
글상자안의 페이지번호가 다음 줄로 넘어가서 높이가 커진 것이다.
```

Stage25 이후 남은 차이:

```text
1. Section1의 page number 주변 BodyText record bundle은 정답지와 동일하다.
2. Stage25에서 breakNonLatinWord bit도 정답지와 일치했다.
3. 그러나 해당 문단이 참조하는 DocInfo PARA_SHAPE payload 길이는 여전히 달랐다.
```

비교:

| record | oracle | Stage25 generated |
|---|---:|---:|
| `DocInfo#254` | 58 bytes | 54 bytes |
| `DocInfo#255` | 58 bytes | 54 bytes |

공식 스펙 `표 43: 문단 모양`은 전체 길이를 54바이트로 적지만, 한컴이 HWPX를 HWP로 내보낸
정답지들은 `PARA_SHAPE`를 58바이트로 저장한다. 이 4바이트 tail이 없으면 한컴 편집기가
masterpage/header 글상자 내부 쪽번호 문단의 줄나눔 폭을 다르게 해석할 수 있다.

수정:

```text
src/serializer/doc_info.rs

HWPX에서 새로 구성한 PARA_SHAPE를 직렬화할 때 말미 4바이트 zero tail을 materialize한다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage26_parashape_payload58/exam_social-p1-stage26.hwp
output/poc/hwpx2hwp/task1110/stage26_parashape_payload58/exam_social-stage26.hwp
```

검증:

```text
cargo fmt --check
cargo test -q serializer::doc_info::tests::test_serialize_para_shape_roundtrip
cargo test -q parser::hwpx::header::tests::test_parse_hwpx_para_shape_break_non_latin_word_bit
cargo test -q document_core::converters::hwpx_to_hwp::tests::master_page_autonum_removes_parser_placeholder_space
cargo build --bin rhwp
target/debug/rhwp convert samples/hwpx/exam_social-p1.hwpx output/poc/hwpx2hwp/task1110/stage26_parashape_payload58/exam_social-p1-stage26.hwp
target/debug/rhwp convert samples/hwpx/exam_social.hwpx output/poc/hwpx2hwp/task1110/stage26_parashape_payload58/exam_social-stage26.hwp
target/debug/rhwp hwp5-inventory output/poc/hwpx2hwp/task1110/stage26_parashape_payload58/exam_social-stage26.hwp --section 1 --format jsonl
target/debug/rhwp hwp5-anchor-trace output/poc/hwpx2hwp/task1110/stage26_parashape_payload58/exam_social-stage26.hwp --needle " 3" --section 1 --window 18 --out /tmp/generated_stage26_s1_space3.md
```

검증 결과:

```text
cargo fmt --check: success
cargo test serializer::doc_info::tests::test_serialize_para_shape_roundtrip: success
cargo test parser::hwpx::header::tests::test_parse_hwpx_para_shape_break_non_latin_word_bit: success
cargo test document_core::converters::hwpx_to_hwp::tests::master_page_autonum_removes_parser_placeholder_space: success
cargo build --bin rhwp: success
" 3" anchor trace hits: 0
```

Stage26 생성본의 `DocInfo#254/#255`는 정답지와 payload hash까지 일치한다.

```text
DocInfo#254:
  oracle    size=58 hash=140f6ed648185788...
  generated size=58 hash=140f6ed648185788...

DocInfo#255:
  oracle    size=58 hash=dad61cc3927b4dcf...
  generated size=58 hash=dad61cc3927b4dcf...
```

판정표:

| file | 한컴2020 판정 | 3페이지 머리글 쪽번호 | 3페이지 머리글 영역 | 비고 |
|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage26_parashape_payload58/exam_social-p1-stage26.hwp` |  |  |  | p1 guard |
| `output/poc/hwpx2hwp/task1110/stage26_parashape_payload58/exam_social-stage26.hwp` |  |  |  | full target |

## Stage27: rhwp-studio 정상/한컴 비정상 분리

작업지시자 판정:

```text
Stage26도 3페이지 머리말 글상자 안의 페이지 번호가 다음 줄로 넘어가며 실패.
하지만 rhwp-studio에서는 정상 처리된다.
```

해석:

```text
1. rhwp-studio가 정상이라는 것은 IR 렌더링 계산 자체가 주원인이 아니라는 뜻이다.
2. 한컴 에디터가 HWP5 저장 전용 record contract를 더 엄격하게 해석하는 지점으로 분리한다.
3. 따라서 머리말 글상자 자체를 계속 조정하지 않고, 정답 HWP와 생성 HWP의 binary record diff에서
   한컴만 민감하게 반응할 가능성이 큰 저장 필드를 추적한다.
```

Stage26 생성본과 정답 HWP를 `section=1`, `needle=32` 기준으로 비교한 결과, 직접적인 master page
쪽번호 주변 bundle은 정답지와 동일했다.

반복적으로 남은 차이는 본문 표 셀 `LIST_HEADER`의 `list_header_width_ref` high bits였다.

```text
oracle:
  ... 00 05 ...

generated:
  ... 00 00 ...
```

이 값은 rhwp-studio 렌더러가 직접 의존하지 않더라도, 한컴 HWP5 로더가 셀 내부 줄나눔 폭/조판 상태를
초기화하는 데 사용하는 저장 전용 contract일 가능성이 있다.

수정 후보:

```text
src/document_core/converters/hwpx_to_hwp.rs

본문 표 셀 중 폭이 충분하고 문단/컨트롤 내용을 가진 셀에 한해
LIST_HEADER width_ref high-bit contract 0x0500을 materialize한다.

단, MasterPage/HeaderFooter 내부 셀은 정답지와 이미 일치하므로 적용하지 않는다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage27_cell_list_header_high_ref/exam_social-p1-stage27.hwp
output/poc/hwpx2hwp/task1110/stage27_cell_list_header_high_ref/exam_social-stage27.hwp
```

검증:

```text
cargo fmt --check
cargo test -q document_core::converters::hwpx_to_hwp::tests::cell_list_header_contract_materializes_width_ref_and_extra
cargo test -q document_core::converters::hwpx_to_hwp::tests::cell_list_header_contract_materializes_hancom_text_width_ref_high_bits
cargo test -q document_core::converters::hwpx_to_hwp::tests::master_page_autonum_removes_parser_placeholder_space
cargo build --bin rhwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage27_cell_list_header_high_ref/exam_social-p1-stage27.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage27_cell_list_header_high_ref/exam_social-stage27.hwp
```

결과:

```text
cargo fmt --check: success
targeted cargo tests: success
cargo build --bin rhwp: success
exam_social-p1-stage27.hwp: rhwp reload success, sections=1, pages=1
exam_social-stage27.hwp: rhwp reload success, sections=2, pages=4
```

정답 HWP와 Stage27 생성본의 `section=1`, `needle=32` anchor trace는 source path line을 제외하면
동일하다.

판정표:

| file | 한컴2020 판정 | 3페이지 머리글 쪽번호 | 3페이지 머리글 영역 | 비고 |
|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage27_cell_list_header_high_ref/exam_social-p1-stage27.hwp` |  |  |  | p1 guard |
| `output/poc/hwpx2hwp/task1110/stage27_cell_list_header_high_ref/exam_social-stage27.hwp` |  |  |  | full target |

작업지시자 판정:

```text
Stage27도 실패.
한컴 편집기에서 3페이지 머리말 글상자 안의 쪽번호 3 앞에서 Backspace로 빈 공간을 지우면 정상 배치된다.
```

정정:

```text
Stage27의 LIST_HEADER width_ref high-bit 후보는 원인 후보에서 제외한다.
정답지와 생성본의 해당 master page AutoNumber 문단 자체는 byte 수준으로 동일했으므로,
글상자 내부 쪽번호 문단만 조정하는 방식은 문제를 설명하지 못한다.
```

## Stage28: 본문 fixed-width space HWP5 control contract 재검증

정답 HWP와 Stage27 생성본을 다시 `section=1` inventory로 비교했다.

확인된 차이:

```text
oracle:
  일부 본문 PARA_TEXT의 fixed-width space가 HWP5 control code 0x001f로 저장됨

generated:
  같은 위치가 Unicode U+2007(0x2007) literal code point로 저장됨
```

예:

```text
BodyText.Section1#1768:
  oracle/stage28 = 1f 00 2a 00 20 00 1f 00 ...

BodyText.Section1#1912:
  oracle/stage28 = 2a 00 1f 00 ... 1f 00 3d 00 1f 00 ...

BodyText.Section1#1955:
  oracle/stage28 = 2a 00 1f 00 ... 1f 00 3d 00 1f 00 ...
```

구현 후보:

```text
1. 실패한 Stage27 LIST_HEADER high-bit 후보를 제거한다.
2. HWPX 본문/header/footer 문단의 U+2007 fixed-width space를 HWP5 fixed blank control mask로
   materialize한다.
3. MasterPage AutoNumber-only placeholder 문단은 기존 전용 계약을 유지하고, 본 일반 변환에서 제외한다.
```

생성 파일:

```text
output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-p1-stage28.hwp
output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-stage28.hwp
```

검증:

```text
cargo fmt --check
cargo test hwp5_save_fwspace_marks_fixed_blank_control --lib
cargo build --bin rhwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-p1-stage28.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-stage28.hwp
target/debug/rhwp hwp5-inventory-diff samples/exam_social.hwp \
  output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-stage28.hwp \
  --align lcs --section 1 --report hints --focus all --format md \
  --out output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/stage28_vs_oracle_s1_hints.md
```

결과:

```text
cargo fmt --check: success
cargo test hwp5_save_fwspace_marks_fixed_blank_control --lib: success
cargo build --bin rhwp: success
exam_social-p1-stage28.hwp: rhwp reload success, sections=1, pages=1
exam_social-stage28.hwp: rhwp reload success, sections=2, pages=4
```

diff 변화:

```text
Stage28 section=1 diff:
  missing = 0
  extra = 0
  changed = 291

Stage27에서 남아 있던 본문 U+2007/0x001f 차이는 Stage28에서 정답지 형태로 정렬됨.
```

판정표:

| file | 한컴2020 판정 | 3페이지 머리글 쪽번호 | 3페이지 머리글 영역 | 비고 |
|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-p1-stage28.hwp` |  |  |  | p1 guard |
| `output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-stage28.hwp` |  |  |  | full target |

작업지시자 판정:

```text
Stage28도 실패.
```

정리:

```text
1. #1110은 현재 진행된 파일손상/문서 변조 판정 제거 범위까지 성공으로 판정한다.
2. 3페이지 홀수쪽 머리말 글상자 내부 쪽번호 줄바꿈/높이 증가 문제는 별도 이슈로 분리한다.
3. 후속 이슈에서는 글상자 좌표 보정보다, 한컴 에디터가 의미 있게 해석하는 storage-only
   record/attribute 차이를 정답 HWP와 비교해 추적한다.
```

최종 보고서:

```text
mydocs/report/task_m100_1110_report.md
```

후속 이슈:

```text
https://github.com/edwardkim/rhwp/issues/1113
```

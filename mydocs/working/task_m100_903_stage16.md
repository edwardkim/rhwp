# Task m100 #903 Stage 16

## 1. 단계 목적

Stage 15 진단 결론:

```text
Stage 14 02는 한컴 출력 경계를 뒤로 밀었지만 정상 baseline은 아니다.
문단 0:10 차트 표의 host paragraph + table object tuple이 정답 raw 구조와 다르다.
```

특히 Stage 14 `02_chart_table_record_only`는 이름과 달리 저장된 HWP raw record가 다음 상태였다.

```text
정답:
  [103] TABLE       sz=30
  [104] LIST_HEADER sz=47
  [105] PARA_HEADER sz=24

Stage 14 02:
  [103] TABLE       sz=28
  [104] LIST_HEADER sz=34
  [105] PARA_HEADER sz=22
```

원인 후보:

```text
TABLE record의 zone-count tail 2바이트가 serializer에서 사라짐
cell LIST_HEADER/PARA_HEADER raw tail이 compact 상태로 남음
host paragraph record와 table object tuple 조합이 불완전함
```

Stage 16은 문단 `0:10` 차트 표의 tuple 정합성을 분리하기 위한 probe를 생성한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 16 산출물:

```text
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/
```

작업지시자 시각 판정용 HWP는 반드시 `output/` 아래에 생성한다.

## 3. 구현 핵심

Stage 16 probe 생성기에는 다음 보조 함수를 추가했다.

```text
task903_table_record_tail_with_zone_count
task903_copy_table_record_payload_with_encoded_tail
task903_materialize_chart_table_record_with_encoded_tail_from_reference
task903_materialize_chart_table_full_object_with_encoded_tail_from_reference
task903_materialize_top_level_para_record_from_reference
```

`task903_table_record_tail_with_zone_count`는 HWP `TABLE` record에서 borderFillId 뒤에 오는 tail을 명시적으로 재구성한다.

```text
UINT16 nZones
TableZone[nZones]
raw_table_record_extra
```

차트 표의 경우 `nZones=0`이지만 이 2바이트가 있어야 record size가 정답처럼 `sz=30`이 된다.

## 4. Variant

모든 variant는 Stage 13 `03_next_table_child_headers` 기준선을 재현한 뒤, 문단 `0:10` 차트 표 주변만 additive하게 적용한다.

| variant | 적용 payload |
|---|---|
| 01_chart_host_para_raw_headers | 문단 `0:10` host paragraph record만 정답에서 복사 |
| 02_chart_ctrl_table_raw_pair | 차트 표 `CTRL_HEADER` + `TABLE` record with encoded zone tail |
| 03_chart_table_all_cell_headers_raw | 차트 표 `TABLE` record with encoded zone tail + 전체 cell `LIST_HEADER/PARA_HEADER` |
| 04_chart_host_para_plus_chart_full_raw_tuple | 문단 `0:10` host paragraph + 차트 표 full object + encoded zone tail |
| 05_chart_title_text_angle_brackets_only | 문단 `0:13` 제목 paragraph record만 정답에서 복사 |
| 06_chart_tuple_plus_following_title_text | variant 04 + 문단 `0:13` 제목 paragraph record |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/01_chart_host_para_raw_headers.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/02_chart_ctrl_table_raw_pair.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/03_chart_table_all_cell_headers_raw.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/04_chart_host_para_plus_chart_full_raw_tuple.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/05_chart_title_text_angle_brackets_only.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/06_chart_tuple_plus_following_title_text.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage16_generate_chart_tuple_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 44 filtered out
```

전체 adapter test:

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
```

결과:

```text
test result: ok. 45 passed; 0 failed
```

모든 Stage 16 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. Raw Record 확인

Stage 16 `02_chart_ctrl_table_raw_pair`:

```text
[102] CTRL_HEADER(tbl) lv=1 sz=46 ... 10 23 2a 08 ...
[103] TABLE            lv=2 sz=30
[104] LIST_HEADER      lv=2 sz=34
[105] PARA_HEADER      lv=2 sz=22
```

해석:

```text
CTRL_HEADER와 TABLE record size는 정답 수준으로 맞췄고,
cell LIST_HEADER/PARA_HEADER는 아직 compact 상태로 남긴다.
```

Stage 16 `04_chart_host_para_plus_chart_full_raw_tuple`:

```text
[102] CTRL_HEADER(tbl) lv=1 sz=46 ... 10 23 2a 08 ...
[103] TABLE            lv=2 sz=30
[104] LIST_HEADER      lv=2 sz=47
[105] PARA_HEADER      lv=2 sz=24
```

해석:

```text
차트 표 시작 tuple은 정답 record size에 맞춰졌다.
```

## 8. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/01_chart_host_para_raw_headers.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/02_chart_ctrl_table_raw_pair.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/03_chart_table_all_cell_headers_raw.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/04_chart_host_para_plus_chart_full_raw_tuple.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/05_chart_title_text_angle_brackets_only.hwp
output/poc/hwpx2hwp/task903/stage16_chart_tuple_probe/06_chart_tuple_plus_following_title_text.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 셀 세로 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_chart_host_para_raw_headers | 파일손상 | 이전 Stage와 동일 |  |  |  |
| 02_chart_ctrl_table_raw_pair | 파일손상 | 이전 Stage와 동일 |  |  |  |
| 03_chart_table_all_cell_headers_raw | 파일손상 | 이전 Stage와 동일 |  |  |  |
| 04_chart_host_para_plus_chart_full_raw_tuple | 파일손상 | 이전 Stage와 동일 |  |  |  |
| 05_chart_title_text_angle_brackets_only | 파일손상 | 이전 Stage와 동일 |  |  |  |
| 06_chart_tuple_plus_following_title_text | 파일손상 | 이전 Stage와 동일 |  |  | 그동안 보이지 않던 `<`, `>`가 출력됨 |

판정 포인트:

```text
- 한컴 파일손상 위치가 Stage 14 02보다 뒤로 이동하는지
- "업종별 동향" 이후 다음 표가 출력되는지
- Stage 14 02에서 관찰된 셀 안 텍스트 세로 배치 상승이 사라지는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 9. 판정 해석

Stage 16의 모든 variant는 한컴 에디터에서 파일손상 판정을 받았다.

중요한 변화는 `06_chart_tuple_plus_following_title_text`에서만 관찰됐다.

```text
이전에는 보이지 않던 '<', '>'가 출력됨
```

해석:

```text
문단 0:13 제목 paragraph record 복사는 실제로 적용됐다.
하지만 파일손상 경계 자체는 사라지지 않았다.
```

따라서 Stage 15에서 세운 "문단 0:10 차트 표 tuple 불완전성만으로 다음 표 진입 시 깨진다"는 가설은 약해졌다.

현재 더 강한 가설은 다음이다.

```text
차트 표 tuple과 제목 문단을 맞춰도 한컴은 다음 표 진입 시 파일손상을 낸다.
즉 다음 실패 원인은 문단 0:14의 4행×6열 표 자체의 raw tuple일 가능성이 높다.
```

이미 Stage 15에서 문단 `0:14` 표 raw 차이가 확인됐다.

```text
정답:
  attr=0x0000000c
  raw=[10, 23, 2A, 08, ...]

Stage 14/16 계열:
  attr=0x00000004 또는 compact/generated 상태
  raw=[10, 03, 2A, 00, ...]
```

따라서 다음 단계는 문단 `0:14`의 `업종별 동향` 표 tuple을 대상으로 해야 한다.

## 10. Stage 17 제안

## 10-A. 추가 확인: HWPX XML entity 텍스트 보존

작업지시자 판정에서 `06_chart_tuple_plus_following_title_text`만 `<`, `>`가 보인 것은,
Stage 16 variant 06만 한컴 정답 HWP의 paragraph record를 복사했기 때문이다.

원본 HWPX `Contents/section0.xml`에는 제목이 다음처럼 entity로 들어 있다.

```xml
<hp:t>&lt; </hp:t>
...
<hp:t>&gt;</hp:t>
```

기존 HWPX 파서는 `<hp:t>` 내부에서 `Event::Text`만 누적하고,
quick-xml이 `&lt;`, `&gt;`, `&amp;` 등을 `Event::GeneralRef`로 내보내는 경우를 버리고 있었다.

수정:

```text
src/parser/hwpx/section.rs
- decode_xml_general_ref 추가
- read_text_content_with_tabs에서 Event::GeneralRef를 텍스트에 반영
- field/compose/ruby/form 텍스트 리더에도 동일 처리 추가
```

회귀 테스트:

```text
cargo test test_parse_text_preserves_xml_general_refs -- --nocapture
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_preserves_xml_entity_text -- --nocapture
```

결과:

```text
test result: ok
```

수정 후 원본 HWPX IR:

```text
문단 0:9  "< 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) >"
문단 0:13 "< 업종별 동향(억 달러, %) >"
```

Stage 16 판정 파일도 재생성했다.

```text
cargo test --test hwpx_to_hwp_adapter task903_stage16_generate_chart_tuple_probe_variants -- --nocapture
```

재생성 후 `01`, `05`, `06` 모두 문단 `0:13`에서 `< 업종별 동향(억 달러, %) >`를 보존한다.

따라서 `<`, `>` 누락은 #903 안에서 같이 처리된 별도 원인 하나로 정리한다.
한컴 파일손상 원인은 여전히 문단 `0:14` 업종별 동향 표 raw tuple 쪽으로 남는다.

Stage 17은 Stage 16 `06_chart_tuple_plus_following_title_text`를 기준선으로 두고, 문단 `0:14`의 4행×6열 표 payload를 additive하게 적용한다.

후보 variant:

```text
01_industry_table_ctrl_header
02_industry_table_record_with_tail
03_industry_table_all_cell_headers
04_industry_table_full_object_with_tail
05_industry_host_para_plus_table_full_tuple
06_industry_tuple_plus_next_boundary
```

판정 기준:

```text
- 한컴 파일손상 위치가 문단 0:14 이후로 이동하는지
- 4행×6열 업종별 동향 표가 출력되는지
- Stage 16 06에서 보였던 '<', '>' 출력은 유지되는지
- rhwp-studio 정상 렌더링을 유지하는지
```

# Task m100 #903 Stage 52 결과

## 1. 목적

Stage51에서 셀 텍스트 클리핑의 직접 원인이 `ParaShape.attr1 bits 20..21`로
확정되었다. Stage52에서는 이 비트가 HWPX의 어떤 속성에서 와야 하는지 확인하고,
해당 매핑을 파서에 반영한 후보 HWP를 생성했다.

## 2. 분석 대상

```text
HWPX 원본: samples/hwpx/hwpx-h-01.hwpx
정답 HWP: samples/hwpx/hancom-hwp/hwpx-h-01.hwp
positive: output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
baseline: output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp
```

## 3. 분석 결과

HWPX `header.xml`의 `paraPr`는 총 85개다.

```text
align.vertical=BASELINE: 74개
align.vertical=CENTER: 11개
autoSpacing eAsianEng/eAsianNum: 전체 0/0
```

`align.vertical=CENTER`인 paraPr id는 다음과 같다.

```text
1, 3, 7, 11, 41, 44, 56, 62, 63, 71, 73
```

정답 HWP에서 `ParaShape.attr1 bits 20..21 == 2`인 ParaShape id도 위 목록과
정확히 일치했다. 따라서 Stage52의 매핑 결론은 다음이다.

```text
HWPX align.vertical=BASELINE -> HWP ParaShape.attr1 bits 20..21 = 0
HWPX align.vertical=CENTER   -> HWP ParaShape.attr1 bits 20..21 = 2
```

`autoSpacing`은 본 샘플에서 전부 `0/0`이므로 원인이 아니며, 기존처럼 attr1
20/21 비트에 쓰면 문단 세로 정렬 비트를 오염시킨다.

## 4. 구현 변경

대상 파일:

```text
src/parser/hwpx/header.rs
```

변경 내용:

```text
1. <align vertical="...">를 파싱해 ParaShape.attr1 bits 20..21에 반영
2. BASELINE=0, TOP=1, CENTER=2, BOTTOM=3 매핑 추가
3. <autoSpacing>이 attr1 bits 20..21을 건드리던 기존 코드를 제거
```

검증용 테스트와 리포트 생성은 다음 파일에 추가했다.

```text
tests/hwpx_to_hwp_adapter.rs
```

## 5. 생성 산출물

```text
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/para_pr_inventory.md
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/parashape_vertical_bits.md
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/stage52_generation.md
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/hwpx-h-01.hwp
```

후보 HWP:

```text
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/hwpx-h-01.hwp
```

생성 결과:

```text
bytes: 374272
hash: 342d8a4bcc385e0b
rhwp reload: ok, pages=9
```

## 6. 실행한 검증

```bash
cargo test -q parser::hwpx::header::tests::test_parse_vertical_alignment_bits
cargo test --test hwpx_to_hwp_adapter task903_stage52_generate_vertical_attr_mapping_probe -- --nocapture
cargo test --test hwpx_to_hwp_adapter task903_stage52_export_hwpx_h_01_after_parser_fix -- --nocapture
```

결과:

```text
모두 통과
```

## 7. 작업지시자 판정 요청

| 파일 | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/hwpx-h-01.hwp` | 성공 | 실패 | 실패 | 실패 | 성공 | 이미지 출력않되고 표 엉망 배치 |  |

## 8. 판정 포인트

```text
1. 한컴 에디터에서 파일 읽기 오류/파일손상 없이 열리는지
2. 이미지가 그림경로 찾기 대화창 없이 출력되는지
3. 표/셀 배치가 정상인지
4. 셀 텍스트가 셀 위쪽에서 클리핑되지 않는지
5. 마지막 9페이지가 출력되는지
6. rhwp-studio 재열기에서도 정상인지
```

## 9. 판정 해석

Stage52 구현은 한컴 파일 열기와 마지막 페이지 출력 조건은 만족했다.

```text
해결:
- 한컴 파일 열기 성공
- 마지막 페이지 출력 성공

미해결:
- 이미지 출력 실패
- 표/셀 배치 실패
- 셀 텍스트 클리핑 실패
- rhwp-studio에서도 이미지/표 배치 실패
```

따라서 `align.vertical -> ParaShape.attr1 bits 20..21` 매핑은 필요한 변경이지만,
이번 샘플의 최종 성공 조건에는 충분하지 않다. 다음 단계에서는 Stage46~49에서 이미
분리했던 다음 축들이 실제 구현 경로에 반영되어 있는지 재점검해야 한다.

```text
1. DocInfo BIN_DATA metadata/raw_data 보존 또는 합성
2. 표 관련 CTRL_HEADER/LIST_HEADER/PARA_HEADER payload 보존 또는 합성
3. ParaShape attr1 vertical bits가 raw_stream 재사용/재직렬화 경로에서 실제 최종 HWP에 반영되는지
```

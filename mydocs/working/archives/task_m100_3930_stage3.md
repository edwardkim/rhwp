# Task M100 #3930 Stage 3 - Hancom PDF 불일치 raw record 격리

- 이슈: [#3930](https://github.com/edwardkim/rhwp/issues/3930)
- 브랜치: `fix/issue-3930-save-layout-input`
- 시작 기준: `47216ca6b` (`fix(hwp): 한컴 단일 바탕쪽 저장 계약 보존`)
- 기록일: 2026-08-05 KST
- 상태: 조사 시작

## 목표

HWP 2020 MCP `PrintToPDFEx`를 표준으로 삼아, 원본 HWPX의 Hancom PDF와 rhwp 저장 HWP의
Hancom PDF가 달라지는 110쪽의 원인을 HWP5 raw record 수준에서 격리한다. 페이지 수·용지·timeout은
이미 일치하므로, 내부 rhwp page count만으로 결론을 내리지 않는다.

## 시작 증적

- 실제 MCP 후보 PDF: 383쪽, 556 x 754pt, `PrintToPDFEx`, `PrintMethod=0`, validation `ok`.
- 원본 PDF와 후보 PDF의 96dpi 전수 raster: 383쪽 중 273쪽 동일, 110쪽 불일치.
- 첫 불일치: p4. 대표 후반 불일치: p304, p383.
- Stage 2는 단일 Odd master HWP 계약, PICTURE common attr bit 28, SC_PICTURE extra를 보정했으나
  시각 불일치를 모두 해소하지 못했다.

## 계획

1. p4/p304/p383에 대응하는 HWP section·master page·개체를 원본 HWPX, Hancom 직접 HWP,
   rhwp 저장 HWP에서 식별한다.
2. `rhwp dump`와 raw HWP record inventory를 사용해 LIST_HEADER, SECTION_DEF, SHAPE_COMPONENT,
   SC_PICTURE, border/fill 및 object attr 차이를 최소 범위로 대조한다.
3. 표준 PDF raster와 개체 차이를 연결할 수 있는 단일 원인을 찾은 뒤에만 보정한다.
4. 보정 뒤 실제 HWP 2020 MCP PDF를 다시 생성해 전 페이지 동일성 수치를 갱신한다.

## 성공 기준

- p4/p304/p383 불일치 각각이 같은 raw record 원인인지 또는 독립 원인인지 재현 가능하게 분류한다.
- 코드 보정이 필요하면 focused regression과 실제 MCP PDF 비교를 함께 통과한다.
- 원인 미확정이면 관찰값·기각 가설·다음 최소 실험을 남기고 추측 보정을 하지 않는다.

## 테스트 결과

분석 명령과 결과를 실행 직후 이 문서에 기록한다.

## 1차 raw record 분석

### 전체 비교의 한계와 대표 쪽 판정

- HWP 2020 MCP 표준 PDF의 `pdftotext -layout` 비교에서 p4, p15, p53, p304, p383의 본문
  텍스트는 모두 일치했다. 따라서 110쪽 불일치는 문자 손실이나 페이지 순서 차이가 아니다.
- 96dpi raster는 p4에서 하단 1px 선만 다르고, p53에서는 `<참고 자료>` 상자의 외곽선과
  배경 레이어가 다르다. 즉 시각 차이의 우선 축은 표/도형 HWP5 저장 속성이다.
- `dump-records`의 section 0 record 수·tag 순서는 기준 HWP와 rhwp HWP가 같으며, 전면 raw
  index diff는 DocInfo ID 재배치 때문에 직접 원인 판별에 적합하지 않았다.

### p53 참고 상자 raw anchor

`hwp5-anchor-trace --needle '한국어 어문 규범' --section 2`로 같은 표를 대조했다.

- 기준 한컴 HWP: table record attr `0x00000006`.
- rhwp 저장 HWP: table record attr `0x04000006`.
- 두 파일의 표 본문, GenShape 그림 control, `SHAPE_COMPONENT`, `SC_PICTURE` payload는 이
  anchor에서 동일하다. 차이는 HWPX parser가 non-zero table padding을 근거로 합성한
  `HWPTAG_TABLE` bit 26이다.
- 기준 HWP의 해당 표도 non-zero padding을 보유하므로, `padding != 0 -> bit 26` 규칙은
  한컴 2020 저장 계약으로 입증되지 않는다.

원본 HWPX의 같은 `<hp:tbl>`은 `pageBreak="CELL"`, `repeatHeader="1"`,
`noAdjust="0"`, `<hp:inMargin left="1133" right="1133" top="141" bottom="141"/>`다.
즉 non-zero `inMargin` 자체는 bit 26의 충분조건이 아니다. parser는 `noAdjust="1"`을
저위 bit 3 (`0x08`)에 보존하므로, bit 26의 조건을 단순 여백 대신 별도 저장 계약으로
분리할 필요가 있다.

### 다음 최소 실험

소스를 먼저 바꾸지 않는다. 기존 `hwp5-table-probe`로 기준 HWP의 `TABLE` attr만 rhwp 저장본에
이식한 `02_table_attr_only.hwp`를 만든 뒤, HWP 2020 MCP PDF의 p53 및 전수 raster를 비교한다.
이 실험이 p53 외곽선과 110쪽 수치를 개선할 때에만 HWPX table attr materializer를 보정한다.

첫 foreground `convert` 호출은 결과 파일을 쓰지 못한 채 Streamable HTTP 대기 상태에 남아
해당 local process만 종료했다. 이 foreground 결과를 판단 증적으로 사용하지 않는다. 같은
입력은 MCP 표준 async lifecycle (`start -> status -> download`)로 다시 시작했으며, job ID와
진행 수치는 hwp-convert의 `task_local_013_stage2.md`에 기록했다. 완료 후 checksum을 검증한
PDF만 이 stage의 raster 비교 입력으로 사용한다.

### table attr 단일 변경 MCP 결과

async job은 620초에 `succeeded`로 끝났고, `PrintToPDFEx`, `PrintMethod=0`, 383쪽,
`validation=ok`를 반환했다. `download`는 약 6.5초에 `20,569,645` byte PDF를 저장했고,
client/server SHA-256은 `bfb5b8d7...b8452e5`로 일치했다.

동일 96dpi raster 비교 결과:

- 기준/변경본 모두 383쪽, byte-identical 273쪽, changed 110쪽.
- p53 pixel diff: 기존 rhwp 저장본 대비 `18,647`, table attr 변경본 대비 `18,647`.
- 기존 rhwp 저장본과 table attr 변경본의 p53 pixel diff는 `0`이다.

따라서 `HWPTAG_TABLE` bit 26은 이 문서의 Hancom PDF 차이를 유발하지 않는다. 이 가설은
기각하며 `materialize_table_record_attr`를 변경하지 않는다. 다음 조사는 p4/p304/p383의
표 이외 control·shape·master page raw payload를 대상으로 한다.

### 대표 불일치 raster 범위

기존 기준/후보 raster의 changed pixel bounding box를 계산했다.

| 쪽 | changed pixels | bounding box | 관찰 |
| --- | ---: | --- | --- |
| p4 | 903 | x=0..738, y=887..888 | 페이지 하단 2px 수평선 계열 |
| p53 | 35,009 | x=98..628, y=148..598 | 참고 상자 외곽선/배경 |
| p304 | 108,392 | x=113..641, y=254..881 | 반복 표·채움 영역 |
| p383 | 3,137 | x=172..569, y=744..833 | 하단 표/도형 영역 |

110쪽은 하나의 text layout 문제가 아니라 적어도 하단선, 표/채움, 반복 표의 여러 저장 계약이
겹친 결과다. 다음 실험은 가장 영향이 큰 p304의 raw anchor를 먼저 식별한다.

### p304 기준 PDF 위치 보정

첫 `pdftotext` 실행은 기준 PDF를 Stage 2 candidate 디렉터리에서 찾도록 적어 I/O error로
끝났다. 실제 기준 PDF는 `output/task3930-stage1/mcp/`에 있다. 이 실패는 문서나 HWP 저장
결함이 아니며, 다음 명령에서 기준 경로를 보정해 p304 text anchor를 추출한다.

### p304 anchor trace 범위 보정

기준/후보 HWP에 `hwp5-anchor-trace --needle '다수의 수신자에게'`를 실행했으나, 도구의 기본
section은 `0`이고 두 파일 모두 hit `0`이었다. p304 본문은 다른 section에 있으므로, 다음 trace는
section `0..13`을 명시 순회한다. 이 결과는 raw 저장 차이가 아니라 trace 범위 제한을 확인한
것이다.

### p304 section 10 raw anchor

기준 PDF p304의 `46. 다수의 수신자에게...`는 section `10` hit 1에 해당한다. 기준/후보
`hwp5-anchor-trace`를 같은 window로 저장해 diff한 결과, 일반 본문 `PARA_LINE_SEG`는 같지만
질문/답변 표 셀의 `LIST_HEADER` attr가 반복적으로 다르다.

- 기준: `01 00 00 00 20 00 00 00 ...` (`0x00000020`)
- 후보: `01 00 00 00 20 00 00 04 ...` (`0x04000020`)

p304 raster의 차이 bbox(x=113..641, y=254..881)는 이 표의 Q 아이콘·회색 질문행·셀 영역과
일치한다. table record bit 26만 바꾼 실험은 효과가 없었지만, cell `LIST_HEADER` bit 26은
별도 조판 계약 후보다. 다음 단계에서 `materialize_cell_list_header_contract`가 이 bit를
합성하는 조건과 기준 HWP의 셀 margin을 대조한다.

### p304 LIST_HEADER field 경계 정정

앞 절의 `0x04000020` 해석은 field 경계를 잘못 묶은 것이다. `LIST_HEADER` 앞부분은
`u16 paragraph_count | u32 list_attr | u16 width_ref`이므로:

- 기준 `01 00 | 00 00 20 00 | 00 00`: list_attr `0x00200000`, width_ref `0x0000`
- 후보 `01 00 | 00 00 20 00 | 00 04`: list_attr `0x00200000`, width_ref `0x0400`

`src/serializer/control.rs::serialize_cell`은 `cell.list_header_width_ref == 0`일 때
`0x0400`을 무조건 기록한다. p304 기준 HWP는 같은 normal table cell에 `0x0000`을 쓰므로
이 fallback이 실제 불일치 원인 후보이다. 다음 최소 실험은 fallback만 제거한 저장본의
Hancom PDF 비교다.

### width_ref fallback 구현 계획

수정 대상은 `src/serializer/control.rs`의 `serialize_cell` 한 곳이다.

1. `width_ref=0`을 `0x0400`으로 치환하지 않고 원값 그대로 직렬화한다.
2. 기존 normal-cell·명시 width-ref 단위 test를 `0` 보존과 non-zero 보존으로 분리한다.
3. focused Rust test와 HWP 재파싱으로 저장 구조를 확인한다.
4. 같은 383쪽 HWP를 실제 HWP 2020 MCP async lifecycle로 PDF 변환해 p304와 전수 raster를
   비교한다.

성공 기준은 p304의 질문/답변 표가 기준과 개선되고 전체 110쪽 수치가 감소하는 것이다. 개선이
없으면 이 보정을 되돌리지 않고 관찰 결과만 기록한다.

### width_ref fallback 구현 및 로컬 검증

`0x0400`은 새 표를 한컴에서 삽입했을 때의 기본값이지만, HWPX에서 변환한 기존 표 셀의
`0x0000`도 유효한 원본 저장값이다. serializer가 두 경우를 구분하지 못한 것이 원인이므로
다음처럼 책임을 분리했다.

- `src/serializer/control.rs`: `list_header_width_ref`를 있는 값 그대로 기록한다. 0을 0x0400으로
  바꾸는 전역 fallback을 제거했다.
- `src/model/table.rs::Cell::new_empty`: 새 표 생성 경로에서만 `0x0400`을 명시적으로 초기화한다.
  기존 HWP/HWPX 셀은 이 생성 경로를 거치지 않으므로 원값을 보존한다.
- `src/serializer/control/tests.rs`: 기존 셀의 `0` 보존과 `Cell::new_empty`의 명시 `0x0400` 보존을
  각각 검증하는 test를 추가했다.

실행 결과:

- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `serializer::control::tests::test_roundtrip_table`: 통과 (`width_ref=0` 보존)
- `serializer::control::tests::test_roundtrip_new_empty_cell_preserves_hancom_width_ref`: 통과
- `document_core::converters::hwpx_to_hwp::tests::cell_list_header_contract_keeps_width_ref_clear_for_normal_tables`: 통과
- `cargo test --test issue_1623_cellzone_diagonal`: 19/19 통과
- `cargo build --bin rhwp`: 통과

루트 파일시스템 여유 공간이 0이어서 최초 focused Rust compile은 `No space left on device`로
중단됐다. 실행 중인 프로세스가 없음을 확인한 뒤 8월 3일 CI artifact 임시 ZIP(936MB)과
Cargo 설치 임시 디렉터리(282MB)만 정리했고, 기존 `target`과 Hancom cache는 보존했다. 이후
공유 `target/debug` 캐시를 재사용해 위 검증을 완료했다.

새 저장본:

```text
output/task3930-stage3/width-ref-zero/2025-행정업무운영-편람-rhwp-width-ref-zero.hwp
```

이 파일의 section 10 `다수의 수신자에게` anchor에서 p304 질문/답변 셀의 `LIST_HEADER`가
`01 00 | 00 00 20 00 | 00 00`으로 저장됐다. 즉 기존 후보의 `width_ref=0x0400`은 사라졌고
기준 한컴 HWP와 같은 field 값이다.

### 실제 Hancom PDF 검증 진행

실제 HWP 2020 MCP async lifecycle을 시작했다.

- job ID: `4ac78717-3106-4823-96ee-8ee788ea22d0`
- 입력: 위 `width-ref-zero` HWP
- target: PDF, timeout: 3600초
- 출력명: `2025-행정업무운영-편람-rhwp-width-ref-zero-hancom2020.pdf`
- 시작 직후 상태: `running/converting`, 경과 8초, 생성 중 PDF 716,800 byte

완료 뒤 `download`의 SHA-256 검증, PDF page count, 96dpi 전수 raster 비교를 추가한다.

### width_ref fallback 실제 Hancom PDF 결과: 기각

async job `4ac78717-3106-4823-96ee-8ee788ea22d0`은 558초에 성공했다.

- `PrintToPDFEx`, `PrintMethod=0`, `run_status=0`, `validation=ok`
- 편집기/PDF 모두 383쪽, 556 x 754pt
- 결과 크기 20,569,645 byte
- 서버/클라이언트 SHA-256: `207ea1b6857cc264988eecba1a1ee9447588213f05b05468c9e098511195a848`
- p304 PDF 본문은 기준과 같이 질문 46~48 및 쪽번호 296을 포함한다.

결과 PDF를 96dpi로 383쪽 전수 raster화해 기준 원본 HWPX Hancom PDF와 비교했다.

| 비교 | byte-identical | pixel changed pages | p4 | p53 | p304 | p383 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 기존 후보 vs 기준 | 273 | 109 | 718 | 10,774 | 3,485 | 132 |
| width_ref=0 후보 vs 기준 | 273 | 109 | 718 | 10,774 | 3,485 | 132 |
| width_ref=0 후보 vs 기존 후보 | 383 | 0 | 0 | 0 | 0 | 0 |

여기서 pixel 수치는 `pixelmatch(threshold=0.1, includeAA=false)` 결과다. 새 후보와 기존 후보는
PNG 파일 바이트까지 383/383 동일했다. 즉 p304의 `width_ref=0x0400 -> 0x0000` raw 차이는
Hancom 2020 `PrintToPDFEx` 조판 결과에 영향을 주지 않는다.

따라서 다음 변경은 **유지하지 않는다**.

- serializer의 전역 width_ref fallback 제거
- `Cell::new_empty`의 명시 0x0400 초기화
- 그에 딸린 단위 test

모두 작업 트리에서 되돌렸다. 이 가설은 저장 raw 차이의 존재만 확인했으며, #3930의 110쪽
시각 불일치 원인은 아니다. 다음 최소 조사는 p4 하단선과 p53/p383의 도형·바탕쪽 control payload로
진행한다.

### 도형 control contract probe

기존 `hwp5-contract-probe`로 Hancom 직접 저장 HWP와 rhwp 후보 HWP를 비교해 7개 probe를
생성했다. 생성 보고서에서 `MEMO_SHAPE`와 누락 `CTRL_DATA` 이식 수는 모두 0이었다. 유일한
차이는 `ID_MAPPINGS` 1건이므로, 불필요한 7개 PDF 변환 대신 `01_id_mappings_only.hwp`만 실제
MCP로 검증한다.

- job ID: `45632723-8f92-4090-9c0f-a4970aa803a2`
- target: PDF, timeout: 3600초
- 시작 상태: `running/converting`

완료 뒤 기준/기존 후보와의 96dpi raster를 비교한다. 이 probe는 원인 분리용 raw graft이며
serializer 구현 변경은 아니다.

### ID_MAPPINGS raw graft 결과: 한컴 거부

job `45632723-8f92-4090-9c0f-a4970aa803a2`는 시작 23초 뒤 `failed`로 끝났다.

- `run_status=139`, `validation=fail`, `validation_detail=invalid_output`
- PDF 0 byte, timeout 아님
- `MEMO_SHAPE`와 `CTRL_DATA` 축은 애초 이식 건수 0

따라서 ID_MAPPINGS 1건을 단독으로 graft하면 BodyText/DocInfo 참조 일관성이 깨져 Hancom 2020이
종료한다. 이 probe는 정상 PDF를 만들지 못했으므로 시각 불일치 원인이나 serializer 보정 근거로
사용하지 않는다. 다음 실험은 raw ID_MAPPINGS 교체가 아니라, rhwp가 HWPX `BinData`·도형 참조
ID를 재배치하는 정확한 경로를 parser/serializer 수준에서 추적해야 한다.

### DocInfo inventory 출력 규약 확인

`hwp5-inventory-diff`를 `--align lcs --report diff --focus docinfo --format md --out
output/task3930-stage3/bin-data-order/docinfo_inventory_diff.md`로 실행했다.

- exit code는 0이었지만 지정한 파일은 생성되지 않았다.
- 따라서 이 실행은 DocInfo 차이 판단 근거로 사용하지 않는다. 해당 진단 명령의 `--out`이
  파일 경로인지 출력 디렉터리인지 구현을 확인한 뒤, 같은 입력을 올바른 규약으로 다시 실행한다.

### BinData 순서와 metadata 분리 분석

`hwp5-inventory-diff`는 positional 인자보다 옵션을 앞에 둔 호출에서 `--out` 파일을 정상 생성했다.
그 보고서와 CFB/ZIP structured parser 분석을 함께 수행했다.

- HWPX manifest/ZIP, 한컴 직접 HWP CFB, rhwp 저장 HWP CFB는 모두 BinData 항목이 `435`개다.
- 한컴 직접 HWP와 rhwp 저장 HWP의 `BinData/BIN0001..BIN01B3` decoded payload SHA-256은 ID별
  `435/435` 모두 같다. 따라서 현재 `materialize_hwp5_bin_data_order`의 순서/참조 remap은 이
  fixture에서 한컴 결과와 같다.
- 반면 모든 `HWPTAG_BIN_DATA` record가 다르다. 한컴 직접 HWP의 embedded image는 모두
  `attr=0x0001`, `status=NotAccessed`이고 rhwp 저장 HWP는 모두 `attr=0x0101`,
  `status=Success`이다. storage_id와 extension은 435개 모두 같다.

따라서 raw ID_MAPPINGS graft가 망가뜨린 것은 ID table 자체가 아니라, 그와 함께 관찰된 BinData
metadata 계약이다. 다음 보정은 순서/바이트를 바꾸지 않고 HWPX-origin embedded BinData의 상태만
한컴 직접 저장값으로 materialize한다.

### BinData metadata 보정 구현 계획

수정 대상은 `src/document_core/converters/hwpx_to_hwp.rs`다.

1. HWPX-origin `BinDataType::Embedding`의 `attr`/`status`를 storage 동반 여부와 무관하게
   한컴 직접 HWP와 같은 `0x0001`/`NotAccessed`로 저장한다.
2. `BinDataType::Storage`의 기존 `0x0002`/`NotAccessed` 계약은 유지한다.
3. embedded 및 storage 혼합 입력을 만드는 focused unit test로 두 계약을 고정한다.
4. `cargo fmt --check`, focused test, HWP 저장 후 CFB raw record 확인을 거친다.
5. 실제 HWP 2020 MCP async PDF(383쪽)를 다시 받아 96dpi 전수 raster 수치를 기존 후보와
   기준 한컴 PDF에 비교한다.

성공 기준은 `BIN_DATA.attr` 435개가 한컴 직접 HWP와 일치하고, 기준 대비 changed page/p53/p304
pixel 수가 줄어드는 것이다. raster가 byte-identical이면 이 metadata는 raw contract 차이지만
조판 원인이 아니므로 변경을 되돌린다.

### BinData metadata 구현: 형식 검사

`src/document_core/converters/hwpx_to_hwp.rs`에서 embedded BinData의 저장값을
`0x0001`/`NotAccessed`로 통일하고, OLE storage의 `0x0002`/`NotAccessed` 계약은 그대로
유지했다. embedded-only 및 embedded+storage focused unit test를 추가했다.

- `cargo fmt --check`: 통과.

`cargo test --lib document_core::converters::hwpx_to_hwp::tests::hwpx_bin_data -- --nocapture`도
통과했다(2 passed, 0 failed). embedded-only와 embedded+storage input 모두 한컴 NotAccessed
계약으로 저장되는 것을 확인했다.

`cargo build --bin rhwp`도 통과했다.

`target/debug/rhwp convert 'samples/2025 행정업무운영 편람(최종).hwpx'
output/task3930-stage3/bin-data-not-accessed/2025-행정업무운영-편람-rhwp-bin-data-not-accessed.hwp`
로 383쪽 원본을 다시 저장했다(8,870KB).

CFB `DocInfo` stream을 structured parser로 검사한 결과 `HWPTAG_BIN_DATA` 435개가 모두
`attr=0x0001`, storage_id `1..435`로 저장됐다. 한컴 직접 HWP의 435개 metadata와 일치한다.

다음은 이 저장본을 HWP 2020 MCP의 표준 async lifecycle로 PDF 변환하고, 성공 PDF만 내려받아
기준 한컴 PDF와 96dpi 전수 raster 비교한다.

async job `8adec68d-0523-4c69-ae73-a13b781f0c9a`를 timeout `3600`초로 시작했다. 시작 응답은
`queued`(queue position 1), output bytes 0이다. terminal result 및 checksum을 확인하기 전까지는
이 metadata 보정의 효과를 결론내리지 않는다.

104초 status polling은 `running/converting`, output bytes `3,563,520`, delivery
`not_requested`를 반환했다. 출력 크기와 last activity가 증가해 실제 Hancom 인쇄가 진행 중임을
확인했다.

186초 status polling도 `running/converting`, output bytes `6,340,608`, delivery
`not_requested`를 반환했다. 출력 파일이 계속 증가하므로 timeout이나 전송 오류가 아닌 정상 인쇄
진행 상태다.

303초 status polling은 `running/converting`, output bytes `11,677,696`, delivery
`not_requested`를 반환했다. 186초 대비 약 5.3MB 증가했고 마지막 activity도 갱신됐다.

473초 status polling도 `running/converting`, output bytes `18,190,336`, delivery
`not_requested`를 반환했다. 변환은 종료 직전까지 출력이 증가하는 정상 경로다.

job은 581초에 `succeeded/completed`했다.

- `run_status=0`, `validation=ok`, timeout false
- `PrintToPDFEx`, `PrintMethod=0`, 편집기/PDF 모두 383쪽
- PDF 20,569,645 bytes, server SHA-256
  `21538f9cf0485a4c2eb17a3559a00aaff144e24664533b5442dd8270a2d65b06`
- delivery는 아직 `not_requested`

다음 `download`에서 client local checksum과 response delivery 완료를 확인한 뒤 같은 96dpi 전수
raster 비교를 수행한다.

`download`가 exit code 0으로 완료됐다.

- client local PDF: `20,569,645` bytes
- client SHA-256과 server SHA-256 모두
  `21538f9cf0485a4c2eb17a3559a00aaff144e24664533b5442dd8270a2d65b06`
- 한컴 validation, 출력 action, page count는 terminal status와 동일하게 정상이다.

다음 단계에서 기준 HWPX Hancom PDF, 기존 rhwp 후보 PDF, NotAccessed 후보 PDF의 96dpi
383쪽 raster를 비교한다.

`pdftoppm -r 96 -png`가 NotAccessed 후보 PDF의 raster `383`장을 모두 생성했다. 다음 비교는
기존 기준 HWPX Hancom PDF raster와 기존 rhwp 후보 raster를 그대로 재사용해, 새 metadata만의
시각적 영향 여부를 분리한다.

### BinData NotAccessed metadata 실제 Hancom PDF 결과: 기각

NotAccessed 후보 PDF의 96dpi 383쪽 raster를 `pixelmatch(threshold=0.1, includeAA=false)`로
비교했다.

| 비교 | byte-identical | pixel changed pages | pixel total | p4 | p53 | p304 | p383 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 기준 한컴 PDF vs NotAccessed 후보 | 273 | 109 | 392,833 | 718 | 10,774 | 3,485 | 132 |
| 기존 rhwp 후보 vs NotAccessed 후보 | 383 | 0 | 0 | 0 | 0 | 0 | 0 |

두 rhwp 후보는 PNG 파일 바이트까지 383/383 동일했다. 따라서 HWPX `BIN_DATA`의
`0x0101/Success -> 0x0001/NotAccessed` 변경은 한컴 2020 `PrintToPDFEx` 출력에 영향을 주지
않는다. 순서와 payload 역시 이미 동일했으므로 BinData 축은 #3930의 시각 불일치 원인이 아니다.

다음 변경은 유지하지 않는다.

- `normalize_bin_data_for_hwp`의 embedded status 규칙 변경
- `hwpx_bin_data_*` focused unit test 2건

모두 작업 트리에서 되돌린다. 생성 HWP/PDF/raster 및 raw comparison report는 기각 근거로 보존한다.

되돌린 뒤 `git diff --check`를 실행했고 통과했다. source 작업 트리에는 이 stage 문서만
untracked로 남아 있으며 serializer/adapter code diff는 없다.

### p53 참고 상자 anchor 재대조

section 2의 `한국어 어문 규범` anchor raw trace를 diff했다.

- p53 참고 상자의 Table `CTRL_HEADER`, GenShape `CTRL_HEADER`, `SHAPE_COMPONENT`,
  `SHAPE_PICTURE` payload는 기준과 후보에서 같다.
- 차이는 다시 Table record bit 26과 해당 cell `LIST_HEADER width_ref=0x0400`이며, 둘 다
  앞선 실제 MCP PDF 실험에서 시각 영향이 없었다.
- 상자 제목 `<참고 자료>`의 `PARA_CHAR_SHAPE`는 기준 ID `0x228/0x229`, 후보 ID
  `0x231/0x232`를 가리킨다. 단순 numeric ID 차이는 DocInfo char-shape 순서 재배치일 수
  있으므로, 다음에는 각 ID가 가리키는 실제 `CHAR_SHAPE` payload를 대조한다.

### p53 CharShape payload 분리

직접 HWP와 rhwp 후보의 `DocInfo`에서 p53 제목이 참조한 `CHAR_SHAPE` 원본 payload를
structured CFB parser로 추출해 `output/task3930-stage3/p53-char-shape-window.tsv`에 저장했다.

- 직접 HWP의 title style `0x0228`과 후보 `0x0231`은 font ID, 장평, 자간, 상대 크기,
  기준 크기(`0x03d4`)는 같지만 `attr` 4 byte가 각각 `fa00043c`, `02000000`으로 다르다.
- 직접 HWP의 연속 style `0x0229`와 후보 `0x0232`도 같은 계열이지만 `attr`와 shadow color가
  다르다. 직접 HWP는 `f800043c` 및 `c0c0c000`, 후보는 `00000000` 및 `b2b2b200`이다.

따라서 p53의 차이는 `PARA_CHAR_SHAPE` numeric ID 재배치만으로 설명할 수 없다. HWPX
`charPr`에서 attr으로 재구성되어야 할 semantic field가 누락되었거나, HWP direct save가 가진
기본값을 rhwp serializer가 다른 값으로 materialize한 후보이다. 다음에는 HWPX의 해당
`charPr` XML과 `parse_char_shape`/`serialize_char_shape`의 field 대응을 해석하고, raw attr을
그대로 이식하는 추측성 보정은 하지 않는다.

### CharShape sentinel probe 구현 계획

원본 HWPX `charPr id=561`은 `<hh:bold/>`, `underline NONE/SOLID`, `strikeout NONE`,
`shadow NONE/#C0C0C0`, offset `(10,10)`을 명시한다. rhwp 저장 HWP의 `0x0231`은 이를
semantic field 그대로 `attr=0x00000002`로 기록했다. 같은 의미의 한컴 직접 HWP `0x0228`은
`attr=0x3c0400fa`를 쓴다. 이 값은 비활성 underline/strikeout의 HWP5 sentinel을 포함하므로
파서의 logical boolean만으로는 재생성되지 않는다.

다음 최소 구현은 production serializer가 아니라 기존 `hwp5-contract-probe`에
`CHAR_SHAPE_DEFAULTS` 축을 추가하는 것이다.

1. `HWPTAG_CHAR_SHAPE` payload에서 font~base size 및 색/테두리/취소선 필드는 모두 같고,
   `attr`(offset 46..50)와 shadow color(64..68)만 달라도 같은 semantic style로 본다.
2. oracle에서 이 key가 단일이거나, 복수여도 교체 대상 8 byte가 동일한 경우에만 generated
   record의 그 8 byte를 치환한다. 애매한 key는 건너뛰고 개수를 report한다.
3. focused unit test로 unique mapping, ambiguous mapping, 다른 semantic payload의 미치환을
   검증한다.
4. probe HWP를 rhwp reload와 HWP 2020 MCP async PDF 변환으로 확인한다. p53 및 383쪽 raster가
   개선될 때에만 production `serialize_char_shape`의 canonical sentinel 보정을 별도 단계에서
   설계한다.

### CharShape sentinel probe 구현 및 focused 검증

`src/diagnostics/hwp5_contract_probe.rs`에 `CHAR_SHAPE_DEFAULTS` 축과
`08_char_shape_defaults_only.hwp` variant를 추가했다. HWP5 payload 중 attr(46..50)와 shadow
color(64..68)를 제외한 나머지 bytes가 같은 style만 oracle과 대응시킨다. 대응 oracle이 여러 개이고
두 교체 field가 다르면 ambiguous로 보고 치환하지 않는다.

새 unit test는 다음 세 경우를 검증한다.

- unique semantic match: attr/shadow color 8 byte만 oracle 값으로 교체
- ambiguous oracle values: generated payload를 보존
- 다른 semantic payload: generated payload를 보존

실행 결과:

- `cargo fmt`: 통과.
- `git diff --check`: 통과.
- `cargo test --lib diagnostics::hwp5_contract_probe::tests -- --nocapture`: 통과. 이후
  `cargo build --bin rhwp`까지 같은 `&&` command chain이 진행됐으므로 focused test는 성공했다.
- `cargo build --bin rhwp`: 통과. link를 포함한 build process가 종료됐으며 새 diagnostic command를
  실행할 준비가 됐다.

다음 테스트는 `08_char_shape_defaults_only.hwp`의 실제 mapping count, rhwp reload, HWP 2020 MCP
PDF 변환이다.

### CharShape-only probe 실행 경로 보정 계획

기존 `hwp5-contract-probe`는 historical contract axis 8개를 항상 모두 생성하고 각 파일을
383쪽 전체 재로드한다. 이번 Stage 3은 `CHAR_SHAPE_DEFAULTS` 한 축만 필요하지만, 첫 실행이
앞선 6개 legacy probe의 재로드에 묶여 target variant 전에 과도한 CPU 시간을 사용했다. 이 실행은
production 파일을 건드리지 않고 종료했다.

다음 보정은 diagnostic CLI에 `--only-char-shape-defaults` 허용 옵션을 추가한다.

1. 옵션을 명시하면 `CHAR_SHAPE_DEFAULTS`를 가진 variant만 생성·재로드한다.
2. 기본 invocation은 기존 8개 variant 동작을 보존한다.
3. parse unit test는 두 입력 HWP와 `--out-dir`을 유지하면서 선택 옵션이 true가 되는지 검증한다.
4. focused test와 binary build 뒤 same input으로 target HWP 하나만 생성한다.

### CharShape-only 실행 경로 구현 및 Stage 3 종료

`hwp5-contract-probe`에 `--only-char-shape-defaults`를 추가했다. 이 옵션은
`CHAR_SHAPE_DEFAULTS` axis가 없는 legacy variant를 모두 건너뛰며, 옵션이 없을 때의 기존 8개
variant 생성 규칙은 바꾸지 않는다. report에는 선택 옵션 상태도 기록한다.

첫 all-variant 실행은 `01`부터 `06`까지 생성한 뒤 legacy 조합 재로드에 묶여 target variant 전에
중단했다. 그 출력물은 diagnostic artifact일 뿐 이후 판정에 사용하지 않는다. 중단 전 production
source, 입력 HWP, MCP server에는 어떤 변경도 없었다.

추가 unit test `parse_args_selects_only_char_shape_defaults`는 두 입력·`--out-dir` 계약을 유지한
상태에서 선택 옵션이 정확히 파싱되는지 검증한다.

실행 결과:

- `cargo fmt`: 통과.
- `git diff --check`: 통과.
- `cargo test --lib diagnostics::hwp5_contract_probe::tests -- --nocapture`: 통과, 4 passed.
- `cargo build --bin rhwp`: 통과.

Stage 3는 raw CharShape sentinel을 production 동작에서 분리해 실제 PDF 영향만 판단할 수 있는
diagnostic code와 focused regression을 갖췄다. 다음 Stage 4는 `--only-char-shape-defaults` target
한 파일을 생성한 뒤 실제 HWP 2020 MCP async conversion 및 383쪽 raster 비교만 수행한다.

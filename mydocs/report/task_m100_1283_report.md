# Task #1283 진행 보고서

## 요약

HWPX 내부 OLE 차트가 렌더링되지 않던 원인은 `BinData/ole3.ole`이 ZIP 내부에 존재함에도
`content.hpf`의 `isEmbeded="0"` 때문에 외부 링크로 분류되어 bytes 로딩이 생략된 것이다.
내부 OLE 항목은 `BinDataType::Storage`로 분류하고, HWP5 Storage와 동일하게 4-byte size prefix를
제거해 기존 OLE `/Contents` 차트 렌더러로 전달하도록 수정했다.

추가로 웹 저장본 `saved/222143E433F503322BD33.hwp`에서 한컴 편집기가 파일 읽기 오류를
보이는 현상을 확인했다. 원인은 HWPX→HWP 저장 어댑터가 `Embedding | Storage` BinData 모두에
일반 embedded file 속성을 강제해 OLE Storage를 일반 embedded file로 기록한 것이다. Storage 타입은
HWP5 BIN_DATA attr 하위 타입 비트가 `2`여야 하므로 `attr=0x0002`로 materialize하도록 보정했다.

이후 새 저장본 `saved/111143E433F503322BD33.hwp`에서도 읽기 오류와 chart placeholder가
남는 것을 확인했다. 추가 원인은 HWPX에서 생성한 `OleShape`에는 `raw_tag_data`가 없는데
HWP serializer가 `SHAPE_COMPONENT_OLE` payload를 raw bytes만으로 기록해, 저장 후
OLE 컨트롤의 `bin_data_id`가 `0`으로 재파싱되는 것이었다. HWP 정답지의 26-byte OLE payload
계약(`property`, `extent_x/y`, `bin_data_id`, reserved)을 합성하도록 수정했다.

`saved/3333143E433F503322BD33.hwp`에서도 한컴 파일 읽기 오류가 남아 `hwp_chart_spec.md`와
한컴 저장본(`samples/143E433F503322BD33.hwp`)의 HWP5 레코드를 다시 대조했다. HWP5 공식 문서의
chart tag 경로와 달리 이 샘플은 실제로 `SHAPE_COMPONENT_OLE` + OLE Storage `/Contents`로
차트를 보관한다. 마지막 차이는 OLE payload가 아니라 앞선 `SHAPE_COMPONENT`였다. 한컴 저장본은
`$ole/$ole` ctrl id와 원본/현재 크기 `7200×7200`을 기록하지만, HWPX 기원 저장본은 해당 필드가
0으로 materialize되어 한컴이 OLE shape contract를 읽지 못했다. HWPX OLE 파서와 HWP serializer
양쪽에서 `$ole` ctrl id, two-ctrl-id, 원본/현재 크기 `7200×7200`을 보정하도록 수정했다.

`saved/777143E433F503322BD33.hwp`까지도 한컴 편집기에서 동일하게 파일 읽기 오류가 발생했다.
추가 대조 결과 BinData 순서와 OLE 본체는 정답지와 맞았지만, 한컴 저장본의 BodyText에는
`CTRL_DATA` 3개가 있고 HWPX 저장본에는 없었다. 2개는 문단 첫 글자 장식용 사각형 텍스트박스의
보조 ParameterSet이고, 1개는 책갈피 이름(`참조`) ParameterSet이다.

후속 수정에서는 HWPX 일반 도형 `<hp:sz protect="1">`를 `CommonObjAttr.size_protect`에 보존하고,
HWPX dropcap-like 사각형 텍스트박스 저장 시 한컴 저장본과 같은 24-byte `CTRL_DATA`를
`CTRL_HEADER` 아래와 `SHAPE_COMPONENT` 아래에 각각 materialize하도록 했다. 또한 HWPX bookmark
저장 시 이름을 `CTRL_HEADER` inline payload가 아니라 한컴 저장본과 같은 16-byte `CTRL_DATA`
ParameterSet으로 materialize하도록 수정했다.

`saved/999143E433F503322BD33.hwp`는 한컴 편집기에서 파일 열기에는 성공했지만, 차트가
차트 객체로 인식되지 않는 것으로 확인됐다. 추가 대조 결과 HWP 레코드의 OLE payload와
`CTRL_DATA` 계약은 정답지와 맞았지만, 정답 HWP의 `/BinData/BIN0002.OLE` 스트림은 raw deflate
압축된 `[4-byte size][CFB]` payload이고 `999` 후보는 같은 payload를 비압축으로 저장하고 있었다.
HWP serializer가 OLE Storage만 예외적으로 비압축 처리하던 이전 가설을 폐기하고, Storage도
문서/BinData 압축 속성에 따라 압축하도록 수정했다.

새 후보 파일은 `saved/1000143E433F503322BD33.hwp`다. 로컬 검증 기준으로 정답 HWP처럼
`/BinData/BIN0002.OLE`가 압축 저장되고, 압축 해제 후 `11776-byte` nested CFB size prefix와
CFB magic이 복원된다. 메인테이너가 한컴 편집기에서 파일 열기와 차트 객체 인식이 성공함을
확인했다. 이후 rhwp-studio에서도 동일 HWPX 문서를 HWP로 저장한 뒤 다시 여는 라운드트립이
성공했다.

## 변경 파일

- `src/parser/hwpx/mod.rs`
  - 내부 `BinData/*.ole` 판정 helper 추가
  - 내부 OLE는 `Storage` + `OLE` extension으로 등록
  - `isEmbeded="0"`이어도 내부 OLE는 ZIP에서 로딩
  - `[4-byte size][CFB bytes]` prefix 정규화 추가
- `src/document_core/converters/hwpx_to_hwp.rs`
  - HWPX→HWP 저장 전 BinData attr materialize 시 `Storage`는 하위 타입 비트 `0x0002`로 기록
- `src/serializer/cfb_writer.rs`
  - BinDataContent 대응 metadata lookup에서 `Storage`도 정식 매칭
  - 압축 문서의 OLE Storage도 `[size][CFB]` payload를 만든 뒤 raw deflate로 저장하도록 수정
- `src/serializer/control.rs`
  - HWPX 기원 OLE처럼 `raw_tag_data`가 없는 경우 `SHAPE_COMPONENT_OLE` 26-byte payload 합성
  - OLE 컨트롤의 `bin_data_id`가 저장 후 `0`으로 떨어지지 않도록 보존
  - OLE `SHAPE_COMPONENT`의 ctrl id `$ole`, two-ctrl-id, 원본/현재 크기 계약을 저장 직전 방어적으로 보정
  - HWPX dropcap-like 사각형 텍스트박스의 한컴 `CTRL_DATA` 계약 합성
  - HWPX bookmark 이름을 한컴 `CTRL_DATA` ParameterSet 방식으로 저장
- `src/parser/tags.rs`
  - `SHAPE_OLE_ID` (`$ole`) 상수 추가
- `src/parser/hwpx/section.rs`
  - `<hp:chart>`, `<hp:ole>`에서 생성한 OLE Shape에 `$ole` shape component 계약과 `7200×7200` 크기 보정
  - 일반 도형 `<hp:sz protect="1">`를 `CommonObjAttr.size_protect`에 보존
- `tests/issue_1251_ole_chart_contents.rs`
  - `samples/hwpx/143E433F503322BD33.hwpx` 회귀 테스트 추가
  - HWPX `BinData #3`가 OLE Storage로 로드되고 legacy `/Contents` 차트로 파싱되는지 검증
  - SVG 렌더 결과에 `hwp-ole-chart-rust-svg`가 포함되고 placeholder가 사라지는지 검증
  - HWPX→HWP export 후 재파싱 시 `BinData #3`가 `Storage/OLE`로 유지되고 OLE 컨트롤도
    `bin_data_id=3`, `raw_tag_data=26B`, `SHAPE_COMPONENT=$ole`, 원본/현재 크기 `7200×7200`으로 유지되는지 검증
  - HWPX→HWP export 후 한컴 저장본과 같은 shape/bookmark `CTRL_DATA` 계약이 materialize되는지 검증
  - HWPX→HWP export 후 OLE Storage stream이 압축 저장되고, 압축 해제 후 `[size][CFB]` payload가
    복원되는지 검증
- `src/serializer/cfb_writer/tests.rs`
  - 압축/비압축 문서에서 OLE Storage size prefix 복원과 BinData 압축 정책을 검증하는 단위 테스트 추가
- `tests/issue_1156_chart_column_flow.rs`
  - 기존 placeholder 기준 차트 bbox 검증을 실제 OLE chart SVG 기준도 허용하도록 갱신

## 산출물

- `output/poc/task1283/hwpx-ole-chart/143E433F503322BD33.svg`
- `output/poc/task1283/save-candidate-1000/1000143E433F503322BD33.svg`

## 검증 결과

| 항목 | 결과 |
|---|---|
| `cargo test --test issue_1251_ole_chart_contents -- --nocapture` | 통과 |
| `cargo test --test issue_1156_chart_column_flow -- --nocapture` | 통과 |
| `cargo test --lib hwpx -- --nocapture` | 통과 |
| HWPX SVG 문자열 점검 | `hwp-ole-chart-rust-svg`, `연금 재정 전망`, `적립금` 확인 |
| HWPX→HWP 변환본 재파싱 | `saved/1000143E433F503322BD33.hwp`의 `BinData #2`가 `Storage/OLE`, OLE 컨트롤 `bin_data_id=2`로 확인 |
| HWPX→HWP 변환본 dump 점검 | `SHAPE_COMPONENT`가 `$ole/$ole`, `요소: orig=7200×7200, curr=7200×7200`으로 확인 |
| HWPX→HWP 변환본 SVG 문자열 점검 | `hwp-ole-chart-rust-svg`, `data-rhwp-bin-data-id="2"`, `연금 재정 전망`, `적립금` 확인 |
| HWP5 CTRL_DATA trace | 정답지/후보 모두 `3 records / 64 bytes`, 해시 `53a38a34667ceaa0, 53a38a34667ceaa0, 8dc77320d651d0dd` 일치 |
| HWP5 OLE BinData stream 점검 | `saved/1000143E433F503322BD33.hwp`의 `/BinData/BIN0002.OLE`가 raw deflate이며, payload prefix `11776` + CFB magic 확인 |
| 한컴 편집기 판정 | `saved/1000143E433F503322BD33.hwp` 파일 열기 성공, 차트 객체 인식 성공 |
| rhwp-studio 라운드트립 판정 | HWPX 문서 로드 → HWP 저장 → 재로드 성공, 차트 렌더링 성공 |
| WASM 빌드 | `docker compose --env-file .env.docker run --rm wasm` 통과, `pkg/rhwp_bg.wasm`/`pkg/rhwp.js` 갱신 |
| `cargo check` | 통과 |
| `cargo test test_parse_rect_preserves_size_protect -- --nocapture` | 통과 |
| `cargo test ole_storage -- --nocapture` | 통과 |

## 판정

`samples/hwpx/143E433F503322BD33.hwpx`의 OLE chart Contents 렌더링 누락은 SVG/rhwp-studio
시각 판정 통과 상태다. HWP 저장 파일 읽기 오류와 한컴 편집기 차트 객체 미인식 문제도
`saved/1000143E433F503322BD33.hwp` 후보에서 해소됐다. rhwp-studio HWP 저장 라운드트립도
통과했으므로 이번 타스크는 성공 판정한다.

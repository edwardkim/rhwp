# Task #195: 차트/OLE 개체 렌더링 — 구현계획서

> 수행계획서: [task_195.md](task_195.md)
> 마일스톤: 미지정
> **스코프 확장 (2026-04-19)**: 단계 1~5 완료 후 작업지시자 요청으로 실제 데이터 렌더링까지 범위 확대. 단계 6~8 추가.

## 전체 개요

8단계로 진행한다. 각 단계는 독립적으로 커밋 가능하며, 단계 완료 후 단계별 완료보고서(`_stageN.md`)를 작성해 승인받은 뒤 다음 단계로 진행한다.

| 단계 | 제목 | 산출물 | 커밋 단위 | 상태 |
|------|------|--------|----------|------|
| 1 | 스펙 조사 및 IR 설계 | tech 문서 2건 | 문서만 | ✅ `8c660f0` |
| 2 | Model 계층: ChartShape / OleShape | model/shape.rs 확장 | model 추가 | ✅ `80558bb` |
| 3 | Parser 계층: CHART_DATA / OLE | parser 신규 파일 2건 + shape.rs 분기 | parser 완성 | ✅ `2aa5f2f` |
| 4 | Renderer 계층: placeholder SVG | renderer 분기 + 라벨 | 1차 렌더 | ✅ `081df07`, `a2511e6` |
| 5 | 기존 samples 회귀 + 보고서 | 단계별 보고서 + 최종 보고서 | 1차 마무리 | ✅ `7869b99` |
| 6 | BinData 해제 인프라 | bin_data 디코더 + DocInfo 플래그 + API | infra | 진행 예정 |
| 7 | 내부 CFB 파싱 | OlePres000 / OOXMLChartContents / Contents 추출 | parser 확장 | 진행 예정 |
| 8 | OOXML 차트 네이티브 SVG 렌더 + EMF 폴백 | 실제 차트 렌더 | 2차 마무리 | 진행 예정 |

## 단계 1: 스펙 조사 및 IR 설계

### 목적
CHART_DATA / SHAPE_COMPONENT_OLE 바이너리 스펙을 조사하고, rhwp 내부 표현(IR)을 확정한다. **코드 변경 없음**, 문서만 작성.

### 작업 항목
1. `mydocs/tech/hwp_chart_spec.md` 작성
   - HWP 5.0 파일 포맷 스펙 문서 기반 CHART_DATA 레코드 구조 정리
   - 차트 종류(막대/선/파이/영역/분산형 등) enum
   - 데이터 시리즈, 축, 레이블, 범례 필드
   - 참조: pyhwp / hwplib / HWP 스펙 PDF
2. `mydocs/tech/hwp_ole_spec.md` 작성
   - SHAPE_COMPONENT_OLE 레코드 구조
   - BinData 스트림의 OLE 컨테이너(CFB) 구조
   - 프리뷰 이미지 추출 경로(Compound File \001CompObj / \005SummaryInformation / Ole10Native 등)
3. IR 설계 표
   - `ChartShape { common, drawing, chart_type, series: Vec<DataSeries>, axes, title, legend, ... }`
   - `OleShape { common, drawing, bin_item_id, preview: Option<Vec<u8>>, preview_format: ImageFormat }`

### 검증
- 문서 2건이 HWP 5.0 공식 스펙과 pyhwp 구현과 일치하는지 자체 리뷰
- IR 설계가 기존 `ShapeObject` 패턴(Line/Rectangle/Picture)과 일관성 있는지

### 산출물 커밋
- `mydocs/tech/hwp_chart_spec.md`
- `mydocs/tech/hwp_ole_spec.md`
- `mydocs/working/task_195_stage1.md` (단계별 완료보고서)

## 단계 2: Model 계층 확장

### 목적
`ShapeObject` enum에 `Chart`, `Ole` variant를 추가하고 기본 필드와 impl 블록을 구성한다.

### 작업 항목
1. `src/model/shape/chart.rs` 신규 — `ChartShape` 구조체, `ChartType` enum, `DataSeries` 등
2. `src/model/shape/ole.rs` 신규 — `OleShape` 구조체, `OleFormat` enum
3. `src/model/shape.rs` 수정
   - `ShapeObject` enum에 `Chart(Box<ChartShape>)`, `Ole(Box<OleShape>)` 추가
   - `common()`, `common_mut()`, `drawing()`, `drawing_mut()`, `shape_attr()` 등 기존 매치 arm 확장
4. `src/model/control.rs`의 Control enum은 `Shape(Box<ShapeObject>)`로 이미 감싸므로 변경 없음 확인

### 검증
- `cargo build --release` 컴파일 성공
- `cargo test` 기존 테스트 회귀 없음 (모델 추가만으로는 깨지지 않아야 함)

### 산출물 커밋
- 위 3개 파일
- `mydocs/working/task_195_stage2.md`

## 단계 3: Parser 계층 구현

### 목적
shape.rs의 `shape_tag_id` 분기에 CHART / OLE 처리를 추가한다.

### 작업 항목
1. `src/parser/control/shape_chart.rs` 신규
   - `parse_chart_shape(common, drawing, shape_tag_data, child_records) -> ChartShape`
   - child_records에서 `HWPTAG_CHART_DATA` 탐색 후 파싱
2. `src/parser/control/shape_ole.rs` 신규
   - `parse_ole_shape(common, drawing, shape_tag_data) -> OleShape`
   - BinData 참조 ID 추출, 프리뷰 이미지는 BinData 스트림에서 별도 로드
3. `src/parser/control/shape.rs` 수정
   - `Some(tags::HWPTAG_SHAPE_COMPONENT_OLE)` 분기 추가 → `parse_ole_shape` 호출
   - child_records에 `HWPTAG_CHART_DATA`가 있으면 차트로 인식 (차트는 GSO + CHART_DATA 조합)
   - 미지 분기(`_ =>`)는 유지하되 로그/diag 출력 추가
4. 단위 테스트
   - shape_chart.rs / shape_ole.rs 각각 고정 바이트 픽스처 테스트 (픽스처는 자체 제작 파일에서 추출)

### 검증
- `cargo build --release` 성공
- `cargo test` 신규 파서 단위 테스트 통과
- 로컬 1.hwp에 대해 `rhwp dump`로 "도형" 대신 "차트" / "OLE"로 식별되는지 확인
- 기존 샘플 회귀 없음

### 산출물 커밋
- 위 3개 파일
- `mydocs/working/task_195_stage3.md`

## 단계 4: Renderer 계층 SVG 출력

### 목적
파싱된 ChartShape / OleShape를 SVG로 렌더링한다.

### 작업 항목
1. `src/renderer/layout/shape_layout.rs`에 Chart / Ole 분기 추가
2. `src/renderer/svg_chart.rs` 신규 (또는 `svg.rs` 확장)
   - 1차 범위: 막대(세로/가로) / 선 / 파이
   - 축, 레이블, 범례, 타이틀 렌더
   - 제외 범위: 3D, 복합, 보조축
3. Ole 렌더링
   - 프리뷰 이미지가 있으면 `<image>` 태그로 placeholder 출력
   - 없으면 회색 사각형 + "OLE" 텍스트
4. `cargo test` 업데이트 — 스냅샷 테스트(가능 시)

### 검증
- 로컬 1.hwp `rhwp export-svg`로 차트 영역이 막대 그래프로 출력되는지 브라우저 확인
- 축/레이블 위치 HWP 뷰어와 육안 비교
- 기존 export-svg 회귀 없음

### 산출물 커밋
- 위 2개 파일 + 기존 수정
- `mydocs/working/task_195_stage4.md`

## 단계 5: 검증 및 마무리

### 목적
자체 제작 샘플로 회귀 테스트를 고정하고 최종 보고서를 작성한다.

### 작업 항목
1. 한컴오피스로 차트(막대/선/파이) 포함 HWP 자체 제작 → `samples/chart-basic.hwp`
2. E2E 회귀 스크립트에 chart-basic.hwp 추가
3. serializer 라운드트립 확인 (읽기 → 저장 → 읽기 동일성)
4. 전체 samples/ export-svg 회귀
5. `mydocs/working/task_195_stage5.md` + `task_195_report.md` (최종 보고서)
6. `mydocs/orders/` 오늘할일 갱신
7. GitHub Issue #195 close 준비 (승인 후 close)

### 검증
- samples/chart-basic.hwp → export-svg 정상
- 전체 samples/ 회귀 통과
- 단위 테스트 전체 green

### 산출물 커밋
- `samples/chart-basic.hwp`
- 보고서 문서
- 오늘할일 갱신

## 단계 6: BinData 스트림 해제 인프라

### 목적
`BinData/BIN000N.OLE` 스트림의 압축 해제 경로를 rhwp 파서에 통합하여, 이후 단계가 decompressed bytes를 얻을 수 있게 한다.

### 사전 조사 결과 (1.hwp 실측)
- DocInfo의 `BinDataItem` 레코드에 `compressed` 플래그 존재
- 1.hwp는 `BinData/BIN0001.OLE` 30KB → zlib raw deflate 해제 → 384KB
- 해제 후 선두 4바이트(size 추정: `00 de 05 00`)를 건너뛰면 표준 CFB 매직(`d0cf11e0`)

### 작업 항목
1. DocInfo BinDataItem 플래그 파싱 확인
   - `src/parser/doc_info/` 내 BinDataItem 파싱 위치 확인 → compressed 플래그 필드 존재 여부 검증
   - 없으면 추가
2. BinData 스트림 접근 헬퍼 신규 `src/parser/bin_data.rs`
   - `fn get_bin_data_raw(document, bin_id) -> Option<&[u8]>` — 원본 스트림 바이트
   - `fn get_bin_data_decompressed(document, bin_id) -> Option<Vec<u8>>` — compressed 플래그 보고 자동 해제
   - OLE 확장자 처리: 해제 후 선두 4바이트(size prefix) 스킵 옵션
3. zlib raw deflate 해제: `flate2` crate 재사용 (이미 종속성 확인)
4. 단위 테스트
   - 가상 BinData 스트림(소형 deflate 데이터)으로 decompress 검증
   - 압축 플래그 off일 때 원본 그대로 반환 확인

### 검증
- `cargo build --release` 성공
- `cargo test` 신규 테스트 통과
- 1.hwp 로드 후 `get_bin_data_decompressed(256)` 호출 → 384KB 반환 확인 (통합 테스트)

### 산출물 커밋
- src/parser/bin_data.rs
- src/parser/doc_info/bin_data_item.rs (필요 시 수정)
- src/parser/mod.rs (pub 선언)
- mydocs/working/task_195_stage6.md

## 단계 7: 내부 CFB 파싱 (OLE 프리뷰 / OOXML 차트 추출)

### 목적
해제된 BinData 바이트에서 표준 CFB 컨테이너를 파싱하여, OLE 프리뷰 이미지와 OOXML 차트 XML 등 내부 스트림을 꺼낸다.

### 사전 조사 결과 (1.hwp BIN0001 실측)
- CFB 내부 스트림:
  - `\x02OlePres000` (361KB) — OLE Presentation Stream, 내부에 EMF 바이트
  - `Contents` (10KB) — 내부 원본 데이터
  - `OOXMLChartContents` (8KB) — 직접 OOXML XML (ZIP 아님)
- OlePres000 헤더 구조 (offset 0~): clipboard_format(4) / tgtDevSize(4) / tgtDev / aspect(4) / lindex(4) / advf(4) / reserved(4) / width(4) / height(4) / size(4) / bytes[size]
- 1.hwp의 OlePres000에서 EMR_HEADER는 offset 12부터 시작 (간략 헤더 스킵 후 바로 EMF)

### 작업 항목
1. `cfb` crate 재사용 (이미 `Cargo.toml` 존재 확인)
2. 신규 `src/parser/ole_container.rs`
   ```rust
   pub struct OleContainer {
       pub preview_emf:   Option<Vec<u8>>,  // \x02OlePres000에서 추출한 EMF 바이트
       pub ooxml_chart:   Option<Vec<u8>>,  // OOXMLChartContents 원본
       pub raw_contents:  Option<Vec<u8>>,  // Contents 원본
   }
   pub fn parse_ole_container(decompressed: &[u8]) -> Option<OleContainer>;
   ```
3. OlePres000 헤더 파서
   - 선두 4×7 = 28바이트 헤더 스킵 후 size/bytes
   - EMR_HEADER 매직(offset 0~3: `0x00000001`, offset 40~43: `" EMF"`) 검증
   - 검증 실패 시 raw 전체 반환(하위 호환)
4. OleShape 확장
   - `preview: Option<OlePreview>` 필드를 단계 7에서 채움 (BinDataItem + parse_ole_container 호출)
   - 파싱 시점은 GSO 파서가 아니라 **렌더 시점에 lazy load**로 하여 CPU 낭비 방지
5. 단위 테스트
   - 자체 제작한 미니 CFB 바이트로 stream 추출 검증
   - OlePres000 헤더 스킵 후 EMF 바이트 추출 검증

### 검증
- 1.hwp의 BIN0001.OLE, BIN0002.OLE 각각에 대해 `parse_ole_container` 호출 → `ooxml_chart.is_some()` + `preview_emf.is_some()` 확인
- 통합 테스트: 1.hwp 로드 후 각 OleShape의 bin_data_id로 container 추출

### 산출물 커밋
- src/parser/ole_container.rs
- src/model/shape.rs (OleShape.preview 활용)
- mydocs/working/task_195_stage7.md

## 단계 8: 실제 차트 렌더링

### 목적
OLE 컨테이너에서 얻은 OOXML 차트 XML을 네이티브 SVG 차트로 렌더링한다. OOXML 부재 시 EMF를 외부 파일로 저장하고 `<image>` 참조로 폴백한다.

### 작업 항목

#### 8-A. OOXML 차트 파서 (신규 `src/ooxml_chart/`)
1. 파일 구조
   ```
   src/ooxml_chart/
     ├── mod.rs         (공개 API + 데이터 모델)
     ├── parser.rs      (XML 파싱)
     └── renderer.rs    (SVG 변환)
   ```
2. 데이터 모델
   ```rust
   pub struct OoxmlChart {
       pub chart_type: OoxmlChartType,  // Bar/Line/Pie
       pub series: Vec<OoxmlSeries>,
       pub categories: Vec<String>,
       pub title: Option<String>,
   }
   pub struct OoxmlSeries {
       pub name: String,
       pub values: Vec<f64>,
       pub color: Option<u32>,
   }
   pub enum OoxmlChartType { Bar { horizontal: bool }, Line, Pie }
   ```
3. XML 파서: `quick-xml` crate 사용 (이미 쓰이는 경우 재사용, 아니면 추가)
4. 1차 파싱 범위
   - `c:barChart` → Bar (barDir=bar → horizontal)
   - `c:lineChart` → Line
   - `c:pieChart` → Pie
   - 시리즈: `c:ser` 안의 `c:val/c:numRef/c:numCache/c:pt/c:v` (숫자) + `c:tx/c:strRef/c:strCache/c:pt/c:v` (시리즈명)
   - 카테고리: `c:cat/c:strRef/c:strCache/c:pt/c:v`
   - 제목: `c:title/c:tx/c:rich/a:t` (여러 개면 concat)
5. 범위 외: 3D, 산점도, 영역, 복합 차트, 보조축 → `chart_type = Unknown`으로 폴백

#### 8-B. OOXML 차트 SVG 렌더러
1. 입력: `OoxmlChart` + bounding box (render_x, render_y, render_w, render_h)
2. 출력: SVG 문자열 조각 (RenderNodeType::RawSvgFragment 신규 variant 또는 기존 Group에 추가)
3. 구성:
   - 배경: 흰색 + 옅은 테두리
   - 플롯 영역: 85% × 70% (제목/레전드 공간 확보)
   - 축: 계산된 min/max 기반 grid + tick
   - 바 차트: `<rect>` 반복, 시리즈별 색상
   - 선 차트: `<polyline>` 시리즈별
   - 파이 차트: `<path>` arc 커맨드
   - 범례: 상단 또는 우측
   - 제목: 상단 중앙
4. 색상 팔레트: OOXML에 색 지정 없으면 기본 팔레트 (7색 순환)

#### 8-C. Chart/Ole 렌더 분기 통합
1. `src/renderer/layout/shape_layout.rs`의 OLE arm 수정
   ```rust
   ShapeObject::Ole(ole) => {
       if let Some(container) = get_ole_container(ole.bin_data_id, ...) {
           if let Some(chart) = container.parse_as_ooxml_chart() {
               // 8-B 네이티브 SVG 렌더
           } else if let Some(emf) = container.preview_emf {
               // 8-D 폴백: EMF 외부 파일 + <image>
           } else {
               // 기존 placeholder 유지
           }
       } else {
           // 기존 placeholder 유지
       }
   }
   ```
2. Chart arm도 동일 패턴 (CHART_DATA + OOXML 폴백 가능성 대비)

#### 8-D. EMF 폴백 (OOXML 부재 시)
1. 옵션: 옵션 플래그 `--embed-ole-preview` 추가
   - ON: EMF 바이트를 `output/ole_{bin_id}.emf`로 저장 + `<image href>`
   - OFF(기본): placeholder 유지
2. EMF → SVG 네이티브 변환은 범위 외 (별도 이슈)

### 검증
- 1.hwp → export-svg → 페이지 3, 4의 OLE 위치에 **실제 막대 차트 렌더링** (2 시리즈, 색상 구분, 카테고리 레이블)
- PNG 렌더 후 육안 비교 (작업지시자 확인)
- 단위 테스트: OOXML XML 픽스처로 chart_type/series 파싱 검증
- 회귀: 기존 samples/ export-svg 크래시 없음
- 성능: 1.hwp export-svg 전체 소요 시간 이전 대비 30% 이내 증가 허용

### 산출물 커밋
- src/ooxml_chart/mod.rs, parser.rs, renderer.rs
- src/renderer/layout/shape_layout.rs (Ole/Chart arm 확장)
- src/renderer/render_tree.rs (RawSvgFragment 등 필요 시)
- mydocs/working/task_195_stage8.md
- mydocs/working/task_195_report.md (최종 보고서 갱신)

## 공통 규칙

- 각 단계 커밋 메시지: `Task #195: <단계 제목>`
- 단계별 완료 후 승인 없이 다음 단계 진행 금지
- 단계별 완료보고서에는 **실제 수정 파일 목록**, **테스트 결과**, **미해결 이슈** 포함
- 단계 5 완료 후 `local/task195` → `local/devel` merge는 작업지시자 승인 후 수행

## 리스크 및 대응

| 리스크 | 대응 |
|--------|------|
| CHART_DATA 스펙 일부 필드 미문서화 | pyhwp 구현 참조, 미지 필드는 raw 보존하여 라운드트립 유지 |
| OLE 프리뷰 이미지 포맷이 WMF/EMF | rhwp의 기존 WMF 파서(`src/wmf/`) 재사용, EMF는 별도 이슈 |
| 차트 종류가 많아 1차 범위 초과 | 막대/선/파이만 우선 구현, 나머지는 별도 이슈로 분리 |
| 자체 제작 샘플의 차트 종류 제한 | 단계 5에서 별도 이슈로 이월 |
| OOXML 차트 스펙 방대 | barChart/lineChart/pieChart 기본 경로만 구현, 고급 기능 별도 이슈 |
| OOXMLChartContents 부재 파일 | 단계 8-D EMF 폴백 경로 또는 placeholder 유지 |
| BinData 압축 플래그 다양성 | DocInfo BinDataItem의 `compressed` 비트만 지원. 암호화/외부 파일 경로는 범위 외 |
| 파싱 성능 (대용량 1.hwp 384KB 해제) | 렌더 시점 lazy load로 import 비용 최소화 |

## 승인 요청 항목 (스코프 확장본)

1. 단계 6~8 분할이 적절한지 (인프라 → CFB 추출 → 렌더)
2. OOXML 차트를 **1차 경로**로, EMF를 **폴백 경로**로 두는 결정이 맞는지
3. OOXML 차트 1차 범위를 `barChart / lineChart / pieChart`로 제한하는 것이 맞는지 (그 외 → Unknown → EMF 폴백 또는 placeholder)
4. EMF → SVG 네이티브 변환을 **본 태스크에서 제외**하고 별도 이슈로 분리하는 것이 맞는지
5. 승인 시 **단계 6(BinData 해제 인프라)** 착수

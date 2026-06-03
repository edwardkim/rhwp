# 구현 계획서 — Task #164

**이슈**: [#164](https://github.com/edwardkim/rhwp/issues/164)
**타이틀**: HWPX Serializer 구현 — Document IR → HWPX(ZIP+XML) 저장
**마일스톤**: M100
**작성일**: 2026-04-16
**브랜치**: `feature/task164-hwpx-serializer`

---

## 수정/신규 대상

| 파일 | 단계 | 비고 |
|------|------|------|
| `src/serializer/hwpx/mod.rs` | 1 | 신규 — `serialize_hwpx` 엔트리 |
| `src/serializer/hwpx/writer.rs` | 1 | 신규 — ZIP 컨테이너 |
| `src/serializer/hwpx/content.rs` | 1 | 신규 — `Contents/content.hpf` |
| `src/serializer/hwpx/header.rs` | 1, 2 | 신규 — `Contents/header.xml` |
| `src/serializer/hwpx/section.rs` | 1~4 | 신규 — `Contents/section*.xml` |
| `src/serializer/hwpx/bin_data.rs` | 4 | 신규 — `BinData/*` 배치 |
| `src/serializer/hwpx/utils.rs` | 1 | 신규 — XML escape / quick-xml 헬퍼 |
| `src/serializer/mod.rs` | 1 | 수정 — `HwpxSerializer` 구현체 + `serialize_document` 경로 |
| `src/main.rs` | 5 | 수정 — `export-hwpx` 서브커맨드 |
| `tests/hwpx_roundtrip.rs` | 5 | 신규 — 라운드트립 통합 테스트 |
| `CHANGELOG.md` | 5 | 수정 |

---

## 공통 설계

### 엔트리 API

`src/serializer/hwpx/mod.rs`:
```rust
use crate::model::document::Document;
use crate::serializer::SerializeError;

pub fn serialize_hwpx(doc: &Document) -> Result<Vec<u8>, SerializeError>;
```

`src/serializer/mod.rs`:
```rust
pub mod hwpx;
pub use hwpx::serialize_hwpx;

pub struct HwpxSerializer;

impl DocumentSerializer for HwpxSerializer {
    fn serialize(&self, doc: &Document) -> Result<Vec<u8>, SerializeError> {
        serialize_hwpx(doc)
    }
}
```

`SerializeError`에는 이미 HWP용 variant가 있으므로 필요 시 `XmlError(String)`, `ZipError(String)` variant를 추가한다.

### ZIP 구조

```
mimetype                          ← STORED, "application/hwp+zip"
META-INF/container.xml
META-INF/manifest.xml
Contents/content.hpf              ← OPF manifest + spine
Contents/header.xml               ← DocInfo/DocProperties
Contents/section0.xml             ← 본문 섹션 0
Contents/section1.xml (있다면)
BinData/image{N}.{ext}            ← 이미지
```

(정확한 목록은 `reader.rs`에서 인식하는 파일 목록 기준으로 1단계에서 조사하여 보강)

### XML 출력 규칙

- quick-xml `Writer` 사용 (이미 deps)
- XML 선언: `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>`
- 네임스페이스: 파서가 기대하는 접두사 그대로 미러링 (`ha`, `hh`, `hp`, `hp10`, `hs`, `hc` 등 — 파서 코드에서 무시하고 로컬 태그로 비교하므로 prefix 선택은 유연)
- 결정적 출력: 속성 순서 고정, 들여쓰기 없음 (바이트 diff 최소화)
- ZIP 옵션: 압축 레벨 `Deflated::default()`, mtime 고정값(`1980-01-01`)

### 에러 처리

- `quick_xml::Error`, `zip::result::ZipError` → `SerializeError::XmlError` / `ZipError` 매핑
- 파서 미지원 요소 발견 시 `log::warn!` + 건너뜀 (빌드 중단 없음)

---

## 구현 단계

### 1단계: 모듈 스켈레톤 + 빈 HWPX

#### 원인/현황

`src/serializer/hwpx/` 미존재. `HwpxSerializer`도 없다. 빈 Document를 HWPX로 쓸 방법 자체가 없어 라운드트립 검증 베이스를 만들 수 없다.

#### 구체적 수정 내용

**1-1. ZIP 컨테이너 (`writer.rs`)**

```rust
use std::io::{Cursor, Write};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

pub struct HwpxZipWriter {
    inner: ZipWriter<Cursor<Vec<u8>>>,
}

impl HwpxZipWriter {
    pub fn new() -> Self { ... }
    pub fn write_stored(&mut self, name: &str, data: &[u8]) -> Result<(), SerializeError>;
    pub fn write_deflated(&mut self, name: &str, data: &[u8]) -> Result<(), SerializeError>;
    pub fn finish(self) -> Result<Vec<u8>, SerializeError>;
}
```

`mimetype`은 반드시 **맨 처음**, **STORED(무압축)**, extra fields 없음.

**1-2. content.hpf 생성 (`content.rs`)**

파서 `content.rs`의 역방향. 최소 형태:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/" version="" unique-identifier="">
  <opf:manifest>
    <opf:item id="header" href="Contents/header.xml" media-type="application/xml"/>
    <opf:item id="section0" href="Contents/section0.xml" media-type="application/xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="header" linear="yes"/>
    <opf:itemref idref="section0" linear="yes"/>
  </opf:spine>
</opf:package>
```

```rust
pub fn write_content_hpf(
    section_hrefs: &[String],
    bin_data_items: &[BinDataEntry],
) -> Result<Vec<u8>, SerializeError>;
```

**1-3. header.xml 최소 버전 (`header.rs`)**

`template/empty.hwp` 파싱 결과를 기준으로 최소 폰트/문자모양/문단모양만 출력. 1단계에서는 완전성보다 **오픈 가능한 최소 구조** 우선.

**1-4. section0.xml 최소 버전 (`section.rs`)**

```xml
<hs:sec xmlns:hs="..." xmlns:hp="...">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr>...</hp:secPr>
      <hp:ctrl><hp:colPr .../></hp:ctrl>
    </hp:run>
    <hp:linesegarray>
      <hp:lineseg textpos="0" vertpos="0" vertsize="1000" textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="42520"/>
    </hp:linesegarray>
  </hp:p>
</hs:sec>
```

`SectionDef`/`ColumnDef`의 속성(용지 크기/여백/단 구성)은 `Section` IR에서 추출.

**1-5. `mod.rs` 엔트리**

```rust
pub fn serialize_hwpx(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    let mut z = HwpxZipWriter::new();
    z.write_stored("mimetype", b"application/hwp+zip")?;
    z.write_deflated("META-INF/container.xml", &write_container()?)?;
    let header_xml = header::write_header(doc)?;
    let section_hrefs: Vec<String> = (0..doc.sections.len())
        .map(|i| format!("Contents/section{}.xml", i)).collect();
    z.write_deflated("Contents/header.xml", &header_xml)?;
    for (i, sec) in doc.sections.iter().enumerate() {
        let xml = section::write_section(sec, doc, i)?;
        z.write_deflated(&section_hrefs[i], &xml)?;
    }
    let content_hpf = content::write_content_hpf(&section_hrefs, &[])?;
    z.write_deflated("Contents/content.hpf", &content_hpf)?;
    z.finish()
}
```

#### 단위 테스트

- `tests::serialize_empty_doc_parses_back` — `Document::default()` 직렬화 → `parse_hwpx()` 재파싱 성공
- `tests::mimetype_first_and_stored` — ZIP 엔트리 0번이 `mimetype`, STORED인지 확인

#### 수동 검증

- 한컴 오피스(가능 시)에서 오픈 → 빈 1쪽 문서 표시

---

### 2단계: 본문 문단·텍스트·lineSegArray

#### 수정 파일

- `section.rs` (확장)
- `header.rs` (확장 — charShapes/paraShapes/fonts/borderFills 완전화)
- `utils.rs` (제어문자 인코딩 헬퍼)

#### 핵심 구현

**2-1. 문단(`<hp:p>`) 직렬화**

```rust
fn write_paragraph(
    w: &mut Writer<impl Write>,
    p: &Paragraph,
    para_index: usize,
) -> Result<(), SerializeError>;
```

속성: `id`, `paraPrIDRef`, `styleIDRef`, `pageBreak`, `columnBreak`, `merged`.

**2-2. 런(`<hp:run>`) 직렬화**

`Paragraph.text` + `char_shapes`(start_pos→char_shape_id) 조합으로 런 경계를 재구성. 같은 `char_shape_id`가 연속되는 구간을 하나의 `<hp:run>`으로 묶는다.

제어문자 분기:
- Tab(0x09) → `<hp:tab/>` (확장 데이터 있으면 `width`/`leader` 속성)
- LineBreak(0x0A) → `<hp:lineBreak/>`
- 확장 컨트롤(SectionDef, ColumnDef, Table, Picture 등) → `<hp:ctrl>` 래퍼 + 자식 요소
- 일반 문자 → `<hp:t>` 내부 텍스트 (XML 이스케이프)

**2-3. lineSegArray**

```rust
fn write_linesegarray(w: &mut Writer<impl Write>, p: &Paragraph) -> Result<(), SerializeError>;
```

각 LINE_SEG 항목 → `<hp:lineseg textpos vertpos=0 vertsize horzpos horzsize textheight baseline spacing tag>` (HWPX 관례에 맞춰 **vpos=0 고정**).

**2-4. header.xml 완전화**

- `<hh:fontfaces>` — Font IR 전체 출력 (typeface/fontRefList)
- `<hh:charProperties>` — CharShape 전체 (fontRef/baseSize/color/italic/bold/underline/strikeout 등)
- `<hh:paraProperties>` — ParaShape 전체 (align/indent/margin/lineSpacing/tabDef 참조)
- `<hh:borderFills>`, `<hh:tabProperties>`, `<hh:numberings>`, `<hh:bullets>`, `<hh:styles>`
- `<hh:compatibleDocument>` 기본값 (targetProgram=HWP)

#### 단위 테스트

- `tests/hwpx_text_roundtrip.rs` — 텍스트 전용 `Document` 3종(한글/영문/혼합) 라운드트립 IR diff 0
- `template/empty.hwp` 파싱 → HwpxSerializer → parse_hwpx → ParaShape/CharShape/LineSeg 동등성

---

### 3단계: 표(Table) 직렬화

#### 수정 파일

- `section.rs` (확장)

#### 핵심 구현

**3-1. `<hp:tbl>` 속성 역매핑**

| XML attr | IR 소스 |
|----------|---------|
| `rowCnt` | `table.row_count` |
| `colCnt` | `table.col_count` |
| `cellSpacing` | `table.cell_spacing` |
| `borderFillIDRef` | `table.border_fill_id` |
| `textWrap` | `table.common.text_wrap` (not `table.attr`!) |
| `pageBreak` | `table.common` 플래그 |
| `repeatHeader` | `table.common` 플래그 |

**3-2. 자식 요소**

- `<hp:sz width height/>` ← `table.common.width/height`
- `<hp:pos treatAsChar vertRelTo horzRelTo vertOffset horzOffset/>` ← `table.common`
- `<hp:outMargin left right top bottom/>` ← `table.common.outer_margin`
- `<hp:inMargin left right top bottom/>` ← `table.inner_margin`
- `<hp:rowSz><hp:rowSize val="..."/>...</hp:rowSz>` ← `table.row_sizes[]`

**3-3. 셀 `<hp:tr><hp:tc>`**

- `<hp:tc>` 속성: `name`, `header`, `hasMargin="true"` (IR의 `cell.apply_inner_margin==true`일 때만 출력, 기본값 false는 생략)
- `<hp:cellAddr colAddr rowAddr/>`
- `<hp:cellSpan colSpan rowSpan/>`
- `<hp:cellSz width height/>`
- `<hp:cellMargin left right top bottom/>` (hasMargin=true일 때만)
- `<hp:subList>` → 셀 내부 문단 (2단계 로직 재사용)

**3-4. 주의사항 (IR 차이 문서 §1, §4)**

- `table.raw_ctrl_data`는 사용하지 않는다.
- `table.attr` 비트 연산 금지 — `table.common.text_wrap` 사용.
- 셀의 `apply_inner_margin`이 false면 `<cellMargin>` 요소 자체 생략.

#### 단위 테스트

- 단순 1x1 / 2x3 표 라운드트립
- `samples/hwpx/` 중 표 포함 파일 IR diff 0

---

### 4단계: 그림(Picture) + BinData

#### 수정 파일

- `section.rs` (확장)
- `bin_data.rs` (신규)
- `content.rs` (확장 — manifest에 BinData 추가)

#### 핵심 구현

**4-1. `<hp:pic>` 직렬화**

```
<hp:pic id="..." numberingType="..." textWrap="...">
  <hp:sz width height/>
  <hp:pos .../>
  <hp:outMargin .../>
  <hp:img binaryItemIDRef="image{N}" bright="0" contrast="0" effect="RealPic"/>
  <hp:imgRect>...</hp:imgRect>
  <hp:imgClip/>
  <hp:inMargin/>
  <hp:lineShape .../>
  <hp:fillBrush/>
</hp:pic>
```

- `image{N}`: `BinData.storage_id` 기반 1부터 증가
- MVP에서는 파서가 읽는 최소 속성만 출력. 밝기/대비 등 확장 속성은 기본값.

**4-2. BinData 엔트리**

```rust
pub struct BinDataEntry {
    pub id: String,         // "image1"
    pub href: String,       // "BinData/image1.png"
    pub media_type: String, // "image/png"
    pub data: Vec<u8>,
}

pub fn collect_bin_data(doc: &Document) -> Vec<BinDataEntry>;
```

확장자→MIME:
```
png → image/png, jpg/jpeg → image/jpeg, gif → image/gif,
bmp → image/bmp, wmf → application/x-msmetafile,
emf → image/x-emf, svg → image/svg+xml
```

**4-3. content.hpf manifest 확장**

```xml
<opf:item id="image1" href="BinData/image1.png" media-type="image/png"/>
```

`isEmbeded="1"` 속성 추가 (파서가 무시하므로 호환용).

#### 단위 테스트

- 이미지 1장 포함 HWPX 라운드트립: ZIP 엔트리 수·바이트, `bin_data_content` IR 동등
- 여러 이미지 + 여러 MIME 혼합

---

### 5단계: 라운드트립 테스트 + CLI + 보고서

#### 수정 파일

- `tests/hwpx_roundtrip.rs` (신규)
- `src/main.rs` (확장)
- `mydocs/report/task_m100_164_report.md` (신규)
- `CHANGELOG.md`

#### 핵심 구현

**5-1. 라운드트립 테스트**

```rust
#[test]
fn roundtrip_samples() {
    for path in glob("samples/hwpx/*.hwpx") {
        let original = read(path);
        let doc1 = parse_hwpx(&original).unwrap();
        let out = serialize_hwpx(&doc1).unwrap();
        let doc2 = parse_hwpx(&out).unwrap();
        assert_ir_equivalent(&doc1, &doc2, path);
    }
}
```

`assert_ir_equivalent`는 IR 차이점 문서에 따라 **파서 지원 범위**만 비교:
- 본문 텍스트, char_shapes, para_shapes, line_segs
- 표(common/inner_margin/row_sizes/cells)
- 그림(bin_data_content + Picture IR)
- 미구현 요소(각주/미주/머리말/꼬리말/도형/필드)는 비교 제외 (경고 로그만)

**5-2. CLI**

```
rhwp export-hwpx input.hwpx -o output.hwpx
```

HWP → HWPX 변환은 이번 이슈 범위 외이므로, 입력도 HWPX만 허용하고 HWP 입력은 `SerializeError::UnsupportedInput`로 명시 거부.

**5-3. 최종 보고서**

`mydocs/report/task_m100_164_report.md`:
- 단계별 완료 현황
- 라운드트립 통과율 (샘플별)
- 미구현/후속 이슈 후보 목록
- 코드 diff 규모

---

## 검증 계획

### 자동 검증

```bash
cargo fmt --check
cargo clippy --all-targets
cargo test                                  # 전체
cargo test --test hwpx_roundtrip            # 5단계 전용
```

### 수동 검증

- 주요 샘플 3~5개를 한컴 오피스에서 오픈 → 정상 표시 확인
- 한컴 오피스에서 저장한 HWPX와 본 serializer 출력의 unzip 결과 diff 비교 (구조 관찰용)

### WASM 호환 확인

```bash
docker compose --env-file .env.docker run --rm wasm
```

`zip` + `quick-xml` 빌드 통과 확인.

---

## 예상 diff 규모

| 단계 | 신규 LOC | 수정 LOC | 테스트 LOC |
|------|---------|---------|-----------|
| 1 | ~400 | ~30 | ~50 |
| 2 | ~600 | ~50 | ~150 |
| 3 | ~300 | ~20 | ~100 |
| 4 | ~250 | ~40 | ~100 |
| 5 | ~100 | ~80 | ~200 |
| **합계** | **~1,650** | **~220** | **~600** |

---

## 위험 및 완화

| 위험 | 완화 |
|------|------|
| HWPX 네임스페이스 prefix 호환 | 파서가 로컬 태그로 매칭 — prefix 자유도 있음. 한컴 오피스 오픈 검증으로 최종 확인. |
| `<hp:ctrl>` 내부 구조가 컨트롤마다 상이 | 파서 `section.rs`의 분기 맵핑을 그대로 미러. |
| lineSegArray `vpos=0` → 일부 뷰어가 배치 계산 실패 | 한컴 오피스는 재계산하므로 문제 없음. 다른 HWPX 뷰어 호환은 범위 외. |
| WASM 빌드에서 `zip` Deflate 플래그 | `Cargo.toml`에 이미 `features=["deflate"]` 설정. 1단계에서 WASM 빌드 실행해 사전 확인. |

---

## 승인 요청

위 구현계획서를 검토 후 승인해주시면 **1단계 구현**을 시작하겠습니다.

각 단계는 독립 커밋으로 쌓고, 단계 완료마다 `task_m100_164_stage{N}.md` 완료보고서 작성 후 다시 승인 요청드립니다.

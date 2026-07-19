---
kind: investigation
status: historical
canonical: mydocs/tech/archive/README.md
last_verified: 2026-07-17
---

# hwpers 프로젝트 분석 — rhwp 관점의 경쟁 분석

## 1. 개요

| 항목 | hwpers | rhwp |
|------|--------|------|
| 저장소 | github.com/Indosaram/hwpers | (비공개) |
| 언어 | Rust (edition 2021) | Rust (edition 2021) |
| 라이선스 | MIT / Apache-2.0 | 미정 |
| 현재 버전 | v0.5.0 (2026-01-19) | - |
| crates.io 공개 | O (`cargo add hwpers`) | X |
| 총 코드 규모 | ~13,400줄 (Rust) | ~30,000줄+ (Rust + JS) |
| 테스트 | 104개 (15개 테스트 파일) | 414개 |
| 최초 커밋 | 2025-01-20 | 2025년 초 |
| 총 커밋 수 | 20 | 66 |

hwpers는 crates.io에 공개된 Rust HWP 파서/라이터 라이브러리이다.
"Indosaram"이라는 개발자가 Claude Code를 사용하여 개발한 것으로 추정된다
(CLAUDE.md 존재, 커밋 메시지 패턴, 코드 생성 패턴).

---

## 2. 버전 이력

| 버전 | 날짜 | 주요 변경 |
|------|------|----------|
| v0.1.0 | 2025-01-20 | 초기 구조, 기본 HWP 읽기 |
| v0.2.0 | 2025-01-28 | 파서 완성, SVG 렌더링, 레이아웃 엔진 |
| v0.3.0 | 2025-01 | Writer 기능 추가 (부분) |
| v0.3.1 | 2025-01-29 | Writer 완성 (표, 리스트, 이미지, 하이퍼링크 등) |
| v0.4.0 | 2025-12-29 | **Writer 호환성 수정** — 한컴에서 열리지 않던 치명적 버그 수정 (주장) |
| v0.5.0 | 2026-01-19 | HWPX 지원, 배포용 문서 복호화, 미리보기 추출 |

**주목할 점**: v0.3.x에서 Writer를 "완성"이라고 했으나, v0.4.0에서야 한컴오피스에서
정상적으로 열리도록 수정했다고 주장한다. 그러나 **v0.5.0으로 실제 검증한 결과,
생성한 5개 HWP 파일 모두 한컴오피스에서 "파일이 손상되었습니다" 오류가 발생했다.**
즉, v0.1.0부터 v0.5.0까지 **Writer로 생성한 HWP는 한컴오피스에서 단 한 번도 정상 동작하지 않았을 가능성**이 높다.

---

## 3. 아키텍처 비교

### 3-1. 계층 구조

```
hwpers                                    rhwp
──────────────────────                    ──────────────────────
reader/  (CFB 읽기)                       parser/  (CFB 읽기 + 레코드 해석)
  cfb.rs (cfb crate 위임)                   cfb_reader.rs (자체 구현)
  stream.rs                                 record.rs, byte_reader.rs
                                            header.rs, doc_info.rs, body_text.rs
                                            control.rs, bin_data.rs, crypto.rs

parser/  (레코드 파싱)                     model/  (문서 IR)
  header.rs, doc_info.rs                    document.rs, paragraph.rs
  body_text.rs, record.rs                   control.rs, table.rs, style.rs
                                            page.rs, shape.rs, image.rs 등

model/  (문서 모델)                        renderer/  (렌더링 엔진)
  23개 파일                                  pagination.rs, layout.rs
  document.rs (596줄)                        composer.rs, height_measurer.rs
                                            style_resolver.rs, render_tree.rs
                                            svg.rs, html.rs, canvas.rs
                                            web_canvas.rs, scheduler.rs

render/  (렌더링)                          serializer/  (HWP 직렬화)
  layout.rs (380줄)                          cfb_writer.rs (자체 구현)
  renderer.rs (310줄)                        header.rs, doc_info.rs, body_text.rs
                                            control.rs, record_writer.rs

writer/  (HWP 생성)                        wasm_api.rs  (78개 WASM API)
  mod.rs (1,995줄)                         web/  (에디터 UI)
  serializer.rs (1,007줄)                    editor.js, text_selection.js
  style.rs (980줄)                           format_toolbar.js
                                            editor.html, editor.css
hwpx/  (HWPX 지원)
  reader.rs, writer.rs (1,559줄)
  xml_types.rs

crypto/  (배포용 문서)
preview/  (미리보기)
```

### 3-2. 핵심 설계 차이

| 관점 | hwpers | rhwp |
|------|--------|------|
| **CFB 처리** | `cfb` crate 위임 | 자체 CFB 파서/라이터 구현 |
| **직렬화 방식** | 새 문서 생성 (from scratch) | 왕복 보존 (읽기→수정→저장) |
| **Writer 접근** | 고수준 API (add_paragraph 등) | 저수준 모델 직접 조작 |
| **렌더링 엔진** | 기초적 (텍스트 너비 추정) | 정교한 (WASM+Canvas 측정) |
| **WASM 지원** | 없음 | 78개 API, 브라우저 동작 |
| **편집 기능** | 없음 (생성만 가능) | 삽입/삭제/서식/리플로우 |
| **HWPX 지원** | O (v0.5.0) | X |

---

## 4. 기능 상세 비교

### 4-1. 읽기 (Reader/Parser)

| 기능 | hwpers | rhwp |
|------|--------|------|
| HWP 5.0 파싱 | O | O |
| OLE CFB 읽기 | O (cfb crate) | O (자체 구현) |
| zlib 압축 해제 | O | O |
| 배포용 문서 복호화 | O (v0.5.0) | O |
| DocInfo 해석 | O | O |
| BodyText 해석 | O | O |
| 표 파싱 | O | O (중첩 표 포함) |
| 이미지/BinData | O | O |
| 하이퍼링크 | O | O |
| 머리글/꼬리글 | O (모델만) | O (모델만) |
| 각주/미주 | 모델 존재 | 모델 존재 |
| 미리보기 추출 | O (PrvText, PrvImage, Summary) | X |
| HWPX 읽기 | O (v0.5.0) | X |
| 텍스트 추출 | O (extract_text) | O |

**평가**: 읽기 기능은 양쪽 모두 HWP 5.0 기본 구조를 처리한다.
hwpers는 HWPX와 미리보기 추출이 추가로 있고, rhwp는 중첩 표 등 복잡 구조 처리가 더 견고하다.

### 4-2. 쓰기 (Writer/Serializer)

| 기능 | hwpers | rhwp |
|------|--------|------|
| **접근 방식** | **새 문서 생성** (빈 문서→콘텐츠 추가) | **기존 문서 수정** (읽기→편집→저장) |
| 빈 문서 생성 | O (HwpWriter::new) | O (create_empty) |
| 텍스트 추가 | O (add_paragraph) | O (insert_text) |
| 서식 적용 | O (TextStyle API) | O (apply_char_format) |
| 표 생성 | O (TableBuilder) | O (모델 직접 조작) |
| 이미지 삽입 | O (add_image) | 모델 존재, API 미완성 |
| 하이퍼링크 | O (add_paragraph_with_hyperlinks) | X |
| 리스트/번호 | O (bullet/numbered/korean) | X |
| 텍스트 박스 | O (add_text_box) | X |
| 머리글/꼬리글 | O (add_header/footer) | X |
| 페이지 레이아웃 | O (set_custom_page_size) | 기본 |
| 문서 메타데이터 | O (title, author 등) | X |
| HWPX 쓰기 | O (HwpxWriter) | X |
| **기존 문서 수정** | **X (새 문서만)** | **O (핵심 기능)** |
| **왕복 보존** | **X** | **O (미지원 레코드 보존)** |
| 한컴 호환성 | **X (v0.5.0 실제 검증 실패)** | O (검증 완료) |
| 압축 지원 | X (쓰기 시) | O |

**핵심 차이**: hwpers의 Writer는 "새 문서 생성" 전용이다.
기존 HWP를 읽어서 수정하는 기능은 없다. 반면 rhwp는 "읽기→수정→저장" 왕복이 핵심이다.

**hwpers 표 직렬화의 치명적 결함 (2026-02-10 검증)**:

hwpers의 `TableBuilder`는 메모리 모델(`table_data`)을 올바르게 구성하지만,
**serializer가 표 컨트롤 레코드를 전혀 직렬화하지 않는다.**

```rust
// serializer.rs:374-432 — write_content_paragraph()
// 모든 문단에 대해 동일하게 처리:
para_header.write_u32::<LittleEndian>(0)?; // controlMask = 0 (항상!)
// → ctrl_header, table_data, picture_data, text_box_data 모두 무시
// → HWPTAG_CTRL_HEADER("tbl "), HWPTAG_TABLE, HWPTAG_LIST_HEADER 레코드 없음
// → 셀 텍스트만 일반 문단으로 평탄화되어 출력
```

**테스트도 이 결함을 은폐하고 있다**:
- `table_test.rs`: 메모리 모델만 검증 (`table_para.table_data.as_ref()`)
- `advanced_table_test.rs`: 동일 — 저장 후 검증 없음
- `serialization_test.rs`: 표 직렬화 테스트 자체가 없음
- `writer_test.rs` roundtrip: `is_ok()` 결과를 `println!`으로 출력만, `assert!` 없음

결과적으로 hwpers의 표 API(`add_table`, `add_simple_table`, `TableBuilder`)는
**메모리에만 존재하는 dead code**이며, 생성된 HWP 파일에는 표가 포함되지 않는다.

추가적인 모델 수준의 한계:

| 한계 | 설명 |
|------|------|
| 열 너비 고정 (50mm) | `col_width = 5000u32` 하드코딩, 개별 지정 불가 |
| 행 높이 고정 (10mm) | `row_height = 1000u32` 하드코딩, 자동 조절 불가 |
| 셀당 문단 1개 | 셀 내 다중 문단, 리스트, 서식 혼합 불가 |
| 셀 내 서식 없음 | 헤더 행만 굵게 가능, 개별 셀 폰트/색상 불가 |
| 셀 배경색 불가 | BorderFill에 테두리만, 채우기 없음 |
| 중첩 표 불가 | 셀 안에 표 삽입 불가 |

### 4-3. 렌더링 엔진

| 기능 | hwpers | rhwp |
|------|--------|------|
| SVG 출력 | O (기초적) | O (정교) |
| HTML 출력 | X | O |
| Canvas 2D | X | O (WASM) |
| 페이지네이션 | O (기본) | O (고급) |
| 레이아웃 엔진 | 380줄, 추정 기반 | 수천 줄, 실측 기반 |
| 텍스트 너비 계산 | `char_count * font_size / 2` (추정) | Canvas measureText (정확) |
| 표 렌더링 | X (텍스트만) | O (경계선, 병합셀, 중첩) |
| 이미지 렌더링 | 모델만 (SVG에 미포함) | O (Base64 임베딩) |
| 도형 렌더링 | X | O |
| 줄바꿈 | `text.lines()` (줄바꿈 문자만) | LineSeg + 자동 리플로우 |
| 언어별 폰트 | X | O (7개 카테고리) |
| 다단 레이아웃 | X | X |

**평가**: 렌더링 엔진은 **rhwp가 압도적**이다.

hwpers의 텍스트 너비 계산:
```rust
// hwpers - render/layout.rs:370
fn calculate_text_width(&self, text: &str, char_shape: Option<&CharShape>) -> i32 {
    let char_count = text.chars().count() as i32;
    let avg_char_width = font_size / 2; // Rough approximation
    char_count * avg_char_width
}
```
모든 문자를 동일 너비로 취급하며, 한글/영문 구분 없이 `font_size / 2`로 근사한다.
실제 문서 렌더링에서는 텍스트 위치가 크게 어긋날 수밖에 없다.

rhwp는 Canvas `measureText()` API로 실제 폰트 메트릭을 측정하며,
7개 언어 카테고리별 폰트/자간/장평을 분리 적용한다.

### 4-4. 웹/WASM 지원

| 기능 | hwpers | rhwp |
|------|--------|------|
| WASM 빌드 | X | O |
| 브라우저 뷰어 | X | O |
| 브라우저 에디터 | X | O |
| 캐럿/선택 | X | O |
| 서식 툴바 | X | O |
| 텍스트 입력/삭제 | X | O |
| WASM API 수 | 0 | 78 |

hwpers는 네이티브 Rust 라이브러리로만 사용 가능하며,
웹 환경 지원은 전혀 없다.

### 4-5. hwpers에만 있는 기능

| 기능 | 설명 | rhwp 대응 필요성 |
|------|------|----------------|
| HWPX 읽기/쓰기 | HWP의 XML 기반 포맷 | 낮음 (시장에서 .hwp가 주류) |
| 미리보기 추출 | PrvText, PrvImage, SummaryInfo | 낮음 (뷰어/변환 용도) |
| 고수준 Writer API | add_paragraph, add_table 등 | **참고 가치** (Python API 설계 시) |
| 리스트/번호 매기기 생성 | 불릿, 번호, 한국어 리스트 | 중간 (HTML→HWP 시 필요) |
| 텍스트 박스 생성 | 위치/스타일 지정 텍스트 박스 | 낮음 |
| 하이퍼링크 생성 | URL, email, 파일, 북마크 | 중간 (HTML→HWP 시 필요) |
| 문서 메타데이터 쓰기 | 제목, 저자, 키워드 등 | 낮음 |

---

## 5. 코드 품질 비교

### 5-1. Writer 호환성 — 실제 검증 결과

hwpers의 Writer는 v0.3.0(2025-01)에서 "완성"으로 발표되었으나,
실제로 한컴오피스에서 열리지 않는 치명적 버그가 있었다.
v0.4.0(2025-12-29)에서 수정했다고 CHANGELOG에 기록되어 있다:

```
- FileHeader: version 5.0.3.4, compression disabled, reserved[4]=0x04
- Scripts streams: uncompressed raw data matching hwplib format
- BodyText: section/column definition paragraph 누락
  → secd, cold 컨트롤 문자 추가
  → PAGE_DEF, FOOTNOTE_SHAPE, PAGE_BORDER_FILL 레코드 추가
  → lastInList 플래그 처리
- PARA_LINE_SEG 추가
```

#### 실제 검증 (2026-02-10)

hwpers v0.5.0으로 5개 HWP 테스트 파일을 생성하여 한컴오피스에서 열어본 결과:

| 테스트 파일 | 내용 | 한컴오피스 결과 |
|------------|------|----------------|
| hwpers_test1_basic.hwp | 기본 텍스트 (한/영/숫자) | **"파일이 손상되었습니다"** |
| hwpers_test2_styled.hwp | 서식 (굵게/기울임/밑줄/색상/크기) | **"파일이 손상되었습니다"** |
| hwpers_test3_table.hwp | 기본 표 (3x3, 4x2) | **"파일이 손상되었습니다"** |
| hwpers_test4_complex_table.hwp | 셀 병합 (가로/세로) | **"파일이 손상되었습니다"** |
| hwpers_test5_comprehensive.hwp | 종합 (페이지설정/머리글/표/리스트) | **"파일이 손상되었습니다"** |

**5개 파일 모두 한컴오피스에서 열리지 않았다.**

이는 v0.4.0의 "호환성 수정"이 실제로는 문제를 해결하지 못했거나,
v0.5.0에서 새로운 회귀 버그가 도입되었을 가능성을 시사한다.

**결론**: hwpers의 Writer는 **현재 시점에서 실용적으로 사용 불가능**하다.
crates.io에 공개된 상태로 이 수준의 호환성 문제가 존재한다는 것은,
HWP 바이너리 포맷 직렬화가 얼마나 까다로운지를 극명하게 보여준다.

rhwp는 "기존 문서 읽기→수정→저장" 접근이므로 이런 구조적 오류가 발생하기 어렵다.
(원본 문서의 구조를 그대로 보존하며 변경분만 적용)

### 5-2. 렌더링 정확도

hwpers의 레이아웃 엔진은 문자 너비를 `font_size / 2`로 일률 추정하고,
줄바꿈은 `text.lines()` (명시적 줄바꿈 문자만)으로 처리한다.
이는 실제 문서에서 텍스트 위치가 크게 어긋나며, 자동 줄바꿈이 작동하지 않는다.

rhwp는 Canvas measureText로 실측하고, LineSeg 기반 + 자동 리플로우를 지원한다.

### 5-3. 직렬화 전략

| | hwpers | rhwp |
|---|--------|------|
| CFB 생성 | `cfb` crate의 `CompoundFile::create_with_version()` | 자체 mini_cfb 라이터 |
| 레코드 직렬화 | 필요한 레코드만 생성 | 원본 레코드 보존 + 변경분 적용 |
| 미지원 레코드 | 생략 (정보 손실) | 바이트 단위 보존 |
| 압축 | 읽기만 지원, 쓰기 시 비압축 | 읽기/쓰기 모두 지원 |

rhwp의 "왕복 보존" 접근은 기존 HWP 파일의 복잡한 구조를
이해하지 못하는 레코드까지 포함하여 완전히 보존한다.
hwpers는 새 문서만 생성하므로, 기존 문서의 수정/보존이 불가능하다.

---

## 6. 전략적 시사점

### 6-1. hwpers가 rhwp의 위협이 되는가?

**현재 시점에서 직접적 위협은 없다.** 이유:

1. **Writer가 동작하지 않는다**: hwpers v0.5.0으로 생성한 HWP 파일이 한컴오피스에서
   모두 "파일이 손상되었습니다"로 표시된다. **Writer 기능이 실질적으로 사용 불가능**하다.

2. **목표가 다르다**: hwpers는 "Rust에서 HWP 읽기/쓰기 라이브러리",
   rhwp는 "AI Agent가 HWP를 조작하는 도구". 시장 포지셔닝이 다르다.

3. **핵심 기능 부재**: hwpers에는 WASM, 웹 에디터, 기존 문서 수정, 정교한 렌더링이 없다.
   AI Agent 도구로 사용하려면 이 모든 것이 필요하다.

4. **렌더링 품질**: hwpers의 렌더링은 POC 수준이다.
   실제 문서를 정확하게 표시하거나 PDF로 변환하는 용도로는 부족하다.

5. **편집 불가**: hwpers는 새 문서 생성만 가능하고, 기존 문서를 수정할 수 없다.
   AI Agent의 핵심 시나리오인 "기존 양식에 데이터 채워넣기"가 불가능하다.

### 6-2. hwpers에서 참고할 점

1. **고수준 Writer API 설계**:
   `add_paragraph()`, `add_heading()`, `add_table()`, `add_image()` 등의
   고수준 API 패턴은 rhwp의 Python 바인딩 설계 시 참고할 가치가 있다.

2. **HWPX 지원**:
   hwpers가 HWPX 읽기/쓰기를 구현한 점은 향후 rhwp에서도 고려할 수 있다.
   다만 현재 시장에서 .hwp(바이너리)가 압도적 주류이므로 우선순위는 낮다.

3. **crates.io 공개 선례**:
   hwpers가 이미 crates.io에 공개되어 있으므로, rhwp가 Rust crate로
   공개할 때 네이밍 차별화가 필요하다.

4. **Writer 호환성 교훈**:
   hwpers가 v0.4.0에서 한컴 호환성을 수정했다고 주장했으나,
   v0.5.0에서도 여전히 한컴오피스에서 열리지 않는 사실은
   HWP Writer의 검증이 얼마나 까다로운지를 보여준다.
   **"from scratch" 방식으로 HWP를 생성하는 것은 구조적으로 매우 어렵다.**
   rhwp의 "왕복 보존" 전략이 이 문제를 구조적으로 회피한다.

### 6-3. rhwp의 차별화 포인트 재확인

hwpers와의 비교를 통해 rhwp의 고유 강점이 더욱 명확해진다:

| rhwp 고유 강점 | hwpers에 없는 이유 |
|---------------|-------------------|
| WASM + 웹 에디터 | 네이티브 전용 설계 |
| 기존 문서 수정 (왕복 보존) | 새 문서 생성만 지원 |
| 정교한 렌더링 (Canvas 실측) | 추정 기반 렌더링 |
| 78개 WASM API | WASM 미지원 |
| 414개 단위 테스트 | 104개 |
| 언어별 폰트 분기 | 단일 폰트만 사용 |
| 서식 편집 (굵게/색상/정렬) | 편집 기능 없음 |
| AI Agent 도구 목표 | 범용 라이브러리 목표 |

### 6-4. 시장 포지셔닝

```
                    렌더링 정확도
                    ▲
                    │
              rhwp  │  ★ (WASM+Canvas 실측, 표/이미지/도형)
                    │
                    │
                    │
             hwpers │  ● (추정 기반, 텍스트만)
                    │
                    └──────────────────────────────► 문서 조작 능력
                         읽기    생성    수정    AI Agent 통합
                         전용    전용    가능    (MCP/PyPI)

                    hwpers: 읽기+생성
                    rhwp:   읽기+생성+수정+렌더링+AI통합
```

---

## 7. AI 코드 생성의 맹점 — hwpers 사례 분석

hwpers는 Claude Code로 개발된 프로젝트이다. 이를 뒷받침하는 증거:

| 증거 | 내용 |
|------|------|
| `CLAUDE.md` 존재 | Claude Code 전용 프로젝트 지시 파일 |
| 커밋 메시지 패턴 | 일관된 구조화 형식, AI 생성 특유의 체계성 |
| 코드 생성 패턴 | API 설계는 정교하나, 바이너리 호환성 검증 부재 |
| 코드 Co-Authored-By | Claude 관련 서명 존재 가능성 |

### 7-1. AI 코드 생성이 잘 한 것

- **API 설계**: `HwpWriter`, `TableBuilder`, `TextStyle` 등 fluent API 패턴이 깔끔
- **코드 구조**: 모듈 분리, 타입 시스템 활용, 에러 처리 등 Rust 관용구 준수
- **문서화**: README, CHANGELOG, 인라인 문서가 체계적
- **테스트 수**: 104개 테스트, 15개 테스트 파일

### 7-2. AI 코드 생성이 놓친 것

| 문제 | 상세 |
|------|------|
| **바이너리 호환성 미검증** | 생성된 HWP를 한컴오피스에서 열어보는 검증이 불가능 |
| **직렬화 구현 누락** | 메모리 모델은 완벽하나 serializer가 표/이미지/텍스트박스 컨트롤 무시 |
| **테스트의 맹점** | 104개 테스트가 메모리 모델만 검증, 실제 출력 바이너리 미검증 |
| **CHANGELOG 과장** | v0.4.0 "한컴 호환성 수정 완료" 기재, 실제로는 여전히 손상 |
| **dead code 방치** | `TableBuilder`, `add_image`, `add_text_box` 등이 직렬화되지 않는 dead code |
| **roundtrip 테스트 부실** | `writer_test.rs`에서 `is_ok()` 결과를 `println!`으로만 출력, `assert!` 없음 |

### 7-3. 근본 원인

AI 코드 생성 도구는 다음을 잘 수행한다:
- 타입 시스템, API 설계, 코드 구조화
- 단위 테스트 작성 (메모리 내 상태 검증)
- 문서화, 주석, CHANGELOG 작성

그러나 다음을 수행할 수 없다:
- **외부 프로그램과의 통합 검증** (한컴오피스에서 파일 열기)
- **바이너리 포맷의 실제 호환성 확인** (HWP 스펙의 미묘한 요구사항)
- **end-to-end 검증** (생성→저장→외부 프로그램 로드→시각적 확인)

hwpers의 사례는 **"테스트가 통과한다 ≠ 실제로 동작한다"**를 보여주는 교과서적 사례이다.
104개 테스트가 모두 통과하지만, 한컴오피스에서는 단 하나의 파일도 열리지 않는다.

### 7-4. rhwp에 대한 교훈

1. **실제 한컴오피스 검증 필수**: 코드 변경 시 반드시 한컴오피스에서 열어 확인
2. **왕복 보존 전략의 우위**: "from scratch" 생성은 AI가 놓치기 쉬운 바이너리 세부사항이 많음.
   기존 문서의 구조를 보존하며 변경분만 적용하는 접근이 구조적으로 안전
3. **end-to-end 테스트 강화**: 단위 테스트 외에 실제 파일 생성→파싱 왕복 검증 필수
4. **작업지시자의 수동 검증**: AI가 자동화할 수 없는 "한컴에서 열기" 검증은
   사람이 최종 관문 역할을 해야 함

---

## 8. 결론

hwpers는 HWP 파일을 Rust로 다루는 최초의 공개 crate로서 의미가 있으나,
rhwp와는 **목표, 아키텍처, 완성도 모두에서 근본적으로 다른 프로젝트**이다.

- hwpers = **"Rust에서 HWP 읽기/쓰기"** (범용 라이브러리, 실질적으로 미완성)
- rhwp = **"AI Agent가 HWP를 조작하는 도구"** (AI 시대 특화, 실동작 검증 완료)

hwpers의 존재는 rhwp의 전략에 큰 영향을 미치지 않으며,
오히려 "HWP를 프로그래매틱하게 다루려는 수요가 실재한다"는 시장 신호로 해석할 수 있다.

hwpers는 AI 코드 생성 도구에만 의존하여 개발된 프로젝트로,
**"테스트 통과 = 실동작"이라는 착각**에 빠진 전형적 사례이다.
v0.1.0(2025-01)부터 v0.5.0(2026-01)까지 약 1년간 개발했으나,
한컴오피스에서 정상적으로 열리는 HWP 파일을 한 번도 생성하지 못했을 가능성이 높다.

rhwp가 집중해야 할 것은 hwpers와의 경쟁이 아니라,
**"AI Agent 도구"라는 고유 포지션에서 3월 말까지 제품을 완성하는 것**이다.

---

*작성: 2026-02-10*
*분석 대상: hwpers v0.5.0 (github.com/Indosaram/hwpers)*

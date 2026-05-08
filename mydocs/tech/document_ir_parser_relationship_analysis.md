# Document IR ↔ 각 파서 관계 분석 보고서

## 분석 일시

2026-05-04, PR #589 검토 중 작업지시자 요청 (IR 표준 부재 본질 검증)

## 1. Document IR 구조 (`src/model/`)

### 1.1 모델 파일 구성

| 파일 | LOC | 책임 |
|------|----|------|
| `document.rs` | 431 | Document, Section, FileHeader, DocInfo, DocProperties |
| `paragraph.rs` | 956 | **Paragraph, LineSeg, CharShapeRef, RangeTag** (본 분석 핵심) |
| `style.rs` | 756 | CharShape, ParaShape, BorderFill, Font, TabDef, Numbering, Bullet |
| `table.rs` | 1007 | Table, Cell, BorderFill 등 |
| `shape.rs` | 758 | ShapeObject, CommonObjAttr, TextWrap, VertRelTo |
| `control.rs` | 432 | Control enum (Picture/Table/Shape/Equation/...) |
| `image.rs` | 150 | Picture |
| `bin_data.rs` | 98 | BinData, BinDataContent |
| `event.rs` | 141 | Hyperlink/Bookmark 등 |
| `header_footer.rs` | 87 | MasterPage |
| `page.rs` | 235 | PageDef |
| `footnote.rs` | 127 | FootnoteShape |
| `path.rs` | 60 | Vector path |
| `mod.rs` | 86 | HwpVersion, HwpUnit (`pub type HwpUnit = u32`) |

### 1.2 IR 가 표현하는 본질

`Document` IR 은 **HWP5 형식 기반으로 설계**되었으며 (`CLAUDE.md` 명시), HWPX 는 같은 의미의 XML 포맷, HWP3 는 고전 포맷이지만 동일 IR 로 변환.

```rust
// src/model/document.rs:24
pub struct Document {
    pub header: FileHeader,                  // HWP5 FileHeader 스트림 기반
    pub doc_properties: DocProperties,
    pub doc_info: DocInfo,                   // HWP5 DocInfo 스트림 기반
    pub sections: Vec<Section>,              // HWP5 BodyText/SectionN 기반
    pub preview: Option<Preview>,
    pub bin_data_content: Vec<BinDataContent>,
    pub extra_streams: Vec<(String, Vec<u8>)>, // HWP5 라운드트립 보존
}
```

### 1.3 `LineSeg` 표준 정의 (현재)

```rust
// src/model/paragraph.rs:133
pub struct LineSeg {
    pub text_start: u32,        // "텍스트 시작 위치"  (단위 모호)
    pub vertical_pos: i32,      // "줄의 세로 위치"    (단위/원점 모호)
    pub line_height: i32,       // "줄의 높이"        (HWPUNIT 추정)
    pub text_height: i32,       // "텍스트 부분의 높이" (HWPUNIT 추정)
    pub baseline_distance: i32, // "베이스라인까지 거리" (HWPUNIT 추정)
    pub line_spacing: i32,      // "줄간격"          (HWPUNIT 추정)
    pub column_start: i32,      // "컬럼에서의 시작 위치" (단위/scope 모호)
    pub segment_width: i32,     // "세그먼트 폭"      (단위 모호)
    pub tag: u32,               // "태그 플래그"
}
```

**doc 주석의 본질적 부족**:
- 단위 (HWPUNIT vs px) 미명시
- 원점/scope (문단 시작 vs 페이지 시작 vs body 시작) 미명시
- "0" 의 의미 미정의 (wrap 없음 vs 미설정)

### 1.4 `Paragraph.wrap_precomputed` (본 세션 보완6 추가)

```rust
// src/model/paragraph.rs:52
/// LineSeg cs/sw가 파서에 의해 사전 계산된 wrap zone 문단.
/// true: 모든 LineSeg의 vertical_pos=0이며 일부 cs>0으로 wrap zone이 이미 인코딩됨.
/// → layout 엔진이 WrapAroundPara 흡수 없이 FullParagraph path로 렌더링해야 한다.
pub wrap_precomputed: bool,
```

**doc 주석 자체가 IR 표준 위배**: "모든 LineSeg의 vertical_pos=0" 는 HWP3 인코딩 특성. IR 필드 doc 에 파서별 인코딩 디테일 누설 — **HWP3 휴리스틱이 IR 표준에 새어 들어간 증거**.

---

## 2. 파서별 LineSeg 인코딩 매트릭스

### 2.1 HWP5 파서 (`src/parser/body_text.rs`)

```rust
// src/parser/body_text.rs:422
fn parse_para_line_seg(data: &[u8]) -> Vec<LineSeg> {
    // 36바이트 바이너리 레코드 → LineSeg 직접 매핑
    LineSeg {
        text_start: r.read_u32(),       // HWP5 PARA_LINE_SEG offset 0
        vertical_pos: r.read_i32(),     // offset 4 — HWPUNIT 누적값
        line_height: r.read_i32(),      // offset 8 — HWPUNIT
        text_height: r.read_i32(),      // offset 12 — HWPUNIT
        baseline_distance: r.read_i32(),// offset 16 — HWPUNIT
        line_spacing: r.read_i32(),     // offset 20 — HWPUNIT
        column_start: r.read_i32(),     // offset 24 — HWPUNIT, body_left 기준
        segment_width: r.read_i32(),    // offset 28 — HWPUNIT
        tag: r.read_u32(),              // offset 32
    }
}
```

**특성**: HWP5 바이너리 레코드를 1:1 그대로 IR 에 저장. **변형/계산 없음**. HWP5 가 IR 의 origin 인 만큼 자연스러움.

### 2.2 HWPX 파서 (`src/parser/hwpx/section.rs`)

```rust
// src/parser/hwpx/section.rs:497
fn parse_lineseg_element(e: &BytesStart) -> LineSeg {
    // <hp:lineseg> XML 속성 → LineSeg 매핑
    match attr.key.as_ref() {
        b"textpos"     => seg.text_start = parse_u32,        // HWPUNIT
        b"vertpos"     => seg.vertical_pos = parse_i32,      // HWPUNIT
        b"vertsize"    => seg.line_height = parse_i32,       // HWPUNIT
        b"textheight"  => seg.text_height = parse_i32,
        b"baseline"    => seg.baseline_distance = parse_i32,
        b"spacing"     => seg.line_spacing = parse_i32,
        b"horzpos"     => seg.column_start = parse_i32,      // HWPUNIT
        b"horzsize"    => seg.segment_width = parse_i32,     // HWPUNIT
        b"flags"       => seg.tag = parse_u32,
    }
}
```

**특성**: HWPX XML 속성을 1:1 IR 매핑. HWP5 와 의미 정합 (`vertpos=vertical_pos` 누적값, `horzpos=column_start` 등).

빈 문단 fallback (section.rs:360):
```rust
if para.line_segs.is_empty() {
    para.line_segs.push(LineSeg { tag: 0x00060000, ..Default::default() });
    // Default::default(): 모든 i32/u32 필드가 0
}
```

### 2.3 HWP3 파서 (`src/parser/hwp3/mod.rs`)

```rust
// src/parser/hwp3/mod.rs:1409
line_segs.push(LineSeg {
    text_start,
    vertical_pos: 0,                                    // ⚠️ 항상 0
    line_height: lh,                                    // 계산 (text_height * ratio)
    text_height: th,                                    // 계산 (linfo.line_height * 4)
    baseline_distance: bl,                              // 계산 (th * 0.85)
    line_spacing: ls,                                   // 계산
    column_start: line_cs_sw.map(|(cs,_)| cs).unwrap_or(0), // ⚠️ wrap zone 휴리스틱 (current_zone)
    segment_width: line_cs_sw.map(|(_,sw)| sw).unwrap_or(0),
    tag,
});
```

**특성**: HWP3 → HWP5 IR 변환 시 **다수의 계산/휴리스틱**:
- `vertical_pos = 0`: HWP3 LineSeg 에 vpos 개념 없음 → 항상 0
- `line_height/text_height/baseline_distance/line_spacing`: HWP3 의 line height (1바이트 * 4) 에서 비례 계산
- `column_start/segment_width`: **`current_zone` 추적** (Square wrap 그림 영역에 있는 줄에만 cs/sw 설정)

추가로 보완6/8 후처리 (mod.rs:1556~):
```rust
// Pattern A/B (multi-LineSeg): 모든 vpos=0 + 일부 cs>0 → wrap_precomputed=true
// Pattern C (single-LineSeg): cs>0 + sw>0 + 그림 없음 OR 페이지 첫 문단 → wrap_precomputed=true
```

### 2.4 비교 매트릭스

| 필드 | HWP5 파서 | HWPX 파서 | HWP3 파서 | 표준 정합 |
|------|---------|---------|---------|---------|
| `text_start` | 원본 그대로 | 원본 그대로 | utf16 변환 매핑 | ✅ HWPUNIT 단위 (UTF-16 code unit pos) |
| `vertical_pos` | 원본 (누적값) | 원본 (누적값) | **항상 0** | ❌ HWP3 만 0 |
| `line_height` | 원본 | 원본 | 계산 | ⚠️ 계산 정합성 검증 안 됨 |
| `text_height` | 원본 | 원본 | 계산 (* 4) | ⚠️ 동일 |
| `baseline_distance` | 원본 | 원본 | 계산 (* 0.85) | ⚠️ 동일 |
| `line_spacing` | 원본 | 원본 | 계산 | ⚠️ 동일 |
| `column_start` | 원본 | 원본 | 휴리스틱 (current_zone) | ⚠️ HWP3 만 추정 |
| `segment_width` | 원본 | 원본 | 휴리스틱 (current_zone) | ⚠️ HWP3 만 추정 |
| `tag` | 원본 | 원본 | 원본 (break_flag 매핑) | ⚠️ break flag 본질 차이 |
| `wrap_precomputed` (Paragraph) | **미설정** (false 유지) | **미설정** | **설정** (보완6/8 후처리) | ❌ HWP3 만 |

---

## 3. 렌더러 ↔ LineSeg 의존성

### 3.1 LineSeg 필드 사용처 통계

`src/renderer/` 전체에서 `column_start/segment_width/vertical_pos` 참조: **136 회**

| 파일 | vertical_pos | column_start/segment_width |
|------|------------|-------------------------|
| `layout.rs` | 14회 | 다수 (Task #463/489/525 등) |
| `typeset.rs` | 13회 | 다수 (wrap_around_cs/sw 등) |
| `layout/paragraph_layout.rs` | 1회 | 다수 (Task #489 effective_col_x, 보완6) |
| `composer.rs` | 0 | 0 |

### 3.2 vertical_pos 의 광범위 의존

`vertical_pos` 는 단순 "줄 y 좌표" 가 아니라 **페이지/단 분할 의사결정의 핵심 시그널**:

| 사용처 | 본질 |
|--------|------|
| `typeset.rs:415-428` Task #321 vpos-reset | "현재 first_vpos < prev last_vpos → 컬럼/페이지 reset 의도" 검출 |
| `typeset.rs:451-452` 페이지 분할 가드 | next_first_vpos vs curr_last_vpos 비교 |
| `typeset.rs:566` vpos_end 계산 | first_seg.vertical_pos + para_h_hu |
| `typeset.rs:1972` 정렬 검증 | line_segs[i].vpos < line_segs[i-1].vpos 가드 |
| `layout.rs:1352~1370` vpos_page_base | 페이지 첫 항목의 vpos 를 보정 기준점으로 사용 (Task #412) |
| `layout.rs:1438~1517` vpos correction | drift 누적 보정 (Task #332 Stage 5) |
| `layout.rs:2404` next_seg.vpos - seg.vpos | 줄 간격 계산 |
| `layout.rs:3004,3126` table_base_vpos | 표 wrap-around y 계산 |
| `layout.rs:862,875` paper_area + vpos | 절대 y 계산 |

### 3.3 HWP3 vertical_pos=0 의 광범위 영향

HWP3 파서가 모든 LineSeg vpos=0 으로 채우면 다음이 잘못 동작:

- `vpos_page_base`: 항상 0 (다른 페이지도 0) → 보정 기준점 의미 없음
- Task #321 vpos-reset 트리거: cv=0, pv=0 → 무발동
- `vpos_correction`: prev_seg.vpos==0 가드 (`if !(seg.vertical_pos == 0 && prev_pi > 0)`) → 보정 skip
- `last.vpos - first.vpos`: 항상 0 → 문단 높이 계산 시 다른 path 필요

→ HWP3 파서는 **vpos 기반 로직 전체를 우회**하기 위해 다른 휴리스틱 (line_height 누적 등) 으로 동작. 본 세션 보완6 의 `wrap_precomputed` 도 이 우회 path 의 일부.

### 3.4 column_start/segment_width 사용처

| 사용처 | 본질 |
|--------|------|
| `typeset.rs:479-491` wrap zone 흡수 매칭 | para_cs == wrap_around_cs && para_sw == wrap_around_sw |
| `typeset.rs:624-628` wrap zone 등록 (표) | 표 anchor 의 cs/sw 기록 |
| `typeset.rs:647-651` wrap zone 등록 (그림) | 그림 anchor 의 cs/sw 기록 |
| `layout.rs:1908,2076,2500` seg_width 검증 | wrap zone 너비 검사 |
| `layout.rs:2428-2429,2661-2662` wrap_cs/sw | layout_wrap_around_paras 인자 |
| `layout.rs:2774` Task #534 v2 | LINE_SEG.column_start 인라인 표/그림 처리 |
| `layout.rs:3007` Task #463 wrap_text_x | column_start 기반 paragraph 위치 |
| `paragraph_layout.rs:828-837` Task #489 effective_col_x | comp_line cs/sw → x 계산 |
| `paragraph_layout.rs:862~` 보완6 line_cs_offset | LineSeg cs/sw → x 계산 (wrap_precomputed) |

→ cs/sw 는 **wrap zone 본질을 인코딩하는 핵심 필드**. 모든 파서가 정합한 의미로 인코딩하면 렌더러가 일관 처리 가능.

---

## 4. 본질적 결함 식별

### 4.1 IR 설계 부채

| 부채 | 본질 | 영향 |
|------|------|------|
| **vertical_pos 의미 모호** | doc 주석 "줄의 세로 위치" — 단위/원점 미정의 | HWP3 만 0 으로 인코딩 → 렌더러 vpos 기반 로직 광범위 우회 |
| **column_start/segment_width 의 0 의미 모호** | 0 = wrap 없음 vs 0 = 미설정 vs 0 = 정상값 | typeset/layout 의 wrap zone 판정 로직이 휴리스틱 의존 |
| **wrap_precomputed 플래그 추가** | HWP3 휴리스틱이 IR 에 누설 | 다른 파서 (HWP5/HWPX) 가 동일 정보를 다른 방식으로 표현 → 렌더러 처리 분기 |
| **doc 주석 부족** | 단위/원점/0 의미 미문서화 | 신규 파서/포맷 확장 시 인코딩 일관성 위배 위험 |

### 4.2 파서 구현의 본질적 격차

**HWP5/HWPX**: 원본 데이터를 IR 에 1:1 매핑. **변형 없음**. HWP5 가 IR origin 이므로 자연스러움.

**HWP3**: HWP3 → HWP5 IR 로 **다수의 계산/휴리스틱 변환** 수행. 그러나 그 결과 IR 데이터가 HWP5/HWPX 와 의미 정합 안 됨:
- vpos 항상 0 → 렌더러 vpos 로직 우회 필요
- cs/sw 의 wrap zone 휴리스틱 (`current_zone` 추적) → 정확하지 않을 수 있음
- line_height 등의 계산 — HWP3 line height 단위 (1바이트 * 4) 와 HWP5 단위의 정합성 검증 부재

### 4.3 렌더러의 본질적 부담

렌더러는 IR 표준이 모호하기 때문에 **포맷별 휴리스틱을 직접 처리**:
- typeset.rs: "vertical_pos==0 가드" — HWP3 인코딩 우회용
- typeset.rs: wrap_around_cs/sw 매칭 — HWP3/HWP5/HWPX 모두 동일 구현이지만 의미 정합 검증 부재
- layout.rs: Task #321/332/412/463/489/525 등 다수 정정 — 각 정정이 IR 의 모호함을 보상

본 세션 보완6 의 `wrap_precomputed` 는 IR 모호성을 IR 에 새 플래그를 추가해서 우회한 임시 정정 — IR 표준 정정이 아님.

---

## 5. 권장 IR 표준 정정 방향

### 5.1 LineSeg 표준 정의 (제안)

```rust
/// 줄 레이아웃 정보 (HWPTAG_PARA_LINE_SEG)
///
/// **표준** (본 IR 의 정합한 의미):
/// - 모든 i32 필드 단위: HWPUNIT (1 inch = 7200 HWPUNIT)
/// - vertical_pos: 페이지 내 흐름 y 좌표, 누적 절대값, 페이지 시작 = 0
/// - column_start: 단 좌측 기준 x 오프셋. 0 = wrap 없음 (단 전체 너비 사용)
/// - segment_width: 줄 너비. column_width 와 같으면 wrap 없음, 작으면 wrap zone
///
/// **각 파서의 인코딩 책임**:
/// - HWP5: 원본 바이너리 그대로 (origin)
/// - HWPX: XML 속성 그대로 (HWP5 와 의미 정합)
/// - HWP3: HWP3 line height (* 4) → HWPUNIT 변환 + vpos 누적 계산 + cs/sw wrap zone 인코딩
pub struct LineSeg {
    pub text_start: u32,
    /// 페이지 내 흐름 y 좌표 (HWPUNIT, 누적 절대값)
    pub vertical_pos: i32,
    /// 줄 높이 (HWPUNIT)
    pub line_height: i32,
    /// 텍스트 높이 (HWPUNIT)
    pub text_height: i32,
    /// 베이스라인 거리 (HWPUNIT, 줄 시작 기준)
    pub baseline_distance: i32,
    /// 줄간격 (HWPUNIT)
    pub line_spacing: i32,
    /// 단 좌측 기준 x 오프셋 (HWPUNIT). 0 = wrap 없음
    pub column_start: i32,
    /// 줄 너비 (HWPUNIT). 단 전체 너비와 같으면 wrap 없음
    pub segment_width: i32,
    /// 태그 비트 플래그
    pub tag: u32,
}
```

### 5.2 wrap zone 판정 (렌더러 표준)

```rust
impl LineSeg {
    /// 이 줄이 wrap zone 안에 있는지 (포맷 무관)
    pub fn is_in_wrap_zone(&self, column_width_hu: i32) -> bool {
        self.column_start > 0
            || (self.segment_width > 0 && self.segment_width < column_width_hu)
    }
}
```

→ `Paragraph.wrap_precomputed` 플래그 **불필요** (모든 LineSeg 의 cs/sw 자체가 표준).

### 5.3 HWP3 파서 정합화 영역

| 영역 | 현재 | 정합 표준 |
|------|----|---------|
| `vertical_pos` | 항상 0 | 누적 계산: 첫 줄=0, 다음 줄=prev.vpos+prev.lh+prev.ls |
| `line_height/text_height/baseline_distance/line_spacing` | 계산 (단위 검증 부재) | HWPUNIT 정합 검증 |
| `column_start/segment_width` | current_zone 휴리스틱 | wrap zone 검출 표준화 (HWP5 인코더 동작 모방) |
| `wrap_precomputed` 후처리 | mod.rs:1556 | **제거** (IR 표준 위배) |

### 5.4 렌더러 정합화 영역

| 영역 | 현재 | 정합 표준 |
|------|----|---------|
| typeset.rs vpos-reset 가드 | HWP3 0 우회 휴리스틱 | 모든 포맷에서 동일 작동 (HWP3 누적 vpos 정합 후) |
| `wrap_precomputed` 분기 | typeset.rs:496, paragraph_layout.rs:862 등 | `seg.is_in_wrap_zone(col_w)` 로 통일 |
| `effective_col_x` (Task #489) | has_picture_shape_square_wrap 분기 | wrap zone 판정 표준화 통합 |

---

## 6. 정정 방식 비교

### 6.1 옵션 비교

| 옵션 | 영역 | 장점 | 단점 |
|------|------|------|------|
| **IR 표준 즉시 정정 (재PR)** | model + 모든 파서 + 렌더러 | 본질 깔끔, 부채 즉시 청산 | scope 큼 (수개 commit), 회귀 검증 광범위 |
| **본 PR (#589) 머지 + 후속 task** | 후속 task 로 분리 | 점진적 개선, 본 PR 의 page 4/8 가치 즉시 | IR 부채 잔존, 다른 컨트리뷰터 오해 위험 |
| **본 PR 정정 후 머지** | 본 PR 에 wrap_precomputed 제거 + cs/sw 표준화 | 본 PR 가치 + IR 부채 일부 해결 | HWP3 vpos 누적 계산은 별도 task 분리 필요 |

### 6.2 점진적 정정 단계 (권고)

**Stage A — 표준 문서화 + LineSeg helper**:
- `mydocs/tech/document_ir_lineseg_standard.md` 신설
- `LineSeg::is_in_wrap_zone(col_w)` helper 추가
- doc 주석 정합화 (단위/원점/0 의미 명시)

**Stage B — 렌더러 정합화 (wrap_precomputed 제거)**:
- typeset.rs, paragraph_layout.rs 의 `wrap_precomputed` 검사를 `seg.is_in_wrap_zone(col_w)` 로 교체
- `Paragraph.wrap_precomputed` 필드 제거

**Stage C — HWP3 파서 정합화**:
- HWP3 파서의 LineSeg vpos 누적 계산
- HWP3 line_height/text_height 단위 정합성 검증
- HWP3 cs/sw 인코딩 표준화

**Stage D — Task #525 본질 재검토**:
- `layout_wrap_around_paras` 함수 dead code 제거 가능 여부
- 또는 IR 기반 가드로 재구현

---

## 7. 결론

### 7.1 본질 진단

작업지시자 지적 정확:
> document IR 쪽에서 표준만 잡혀있으면 hwp3 포맷 파서에서 document IR 로 던져주기만 하면 되는 문제 아님? 현재 document IR 이 잘못되어 있는것 아님?

**본질**:
1. Document IR 의 `LineSeg` 표준이 명시적으로 정의되지 않음 (단위/원점/0 의미 등)
2. HWP5 가 IR origin 이라 HWP5/HWPX 는 자연스럽게 1:1 매핑
3. **HWP3 파서가 HWP3 → HWP5 IR 변환 시 의미 격차** 가 IR 모호성과 결합되어 vpos=0/cs=0/sw=0 등의 인코딩 차이 발생
4. 렌더러가 이를 보상하기 위해 다수의 휴리스틱/우회 로직 도입 (Task #321/332/412/463/489/525 등)
5. 본 세션 보완6 의 `wrap_precomputed` 도 IR 모호성 보상의 일부 — IR 표준 정정이 아님

### 7.2 정합한 방향

- **단기 (본 PR)**: page 4/8 결함 정정 가치 보존 (옵션 IR-3 - cs/sw 표준화) — Stage A+B 본 PR 에 포함 검토
- **중기 (후속 task)**: HWP3 파서 vpos 누적 계산 (Stage C)
- **장기 (별도 task)**: Task #525 등의 정정이 IR 표준 통일 후에도 유효한지 재검토 (Stage D)

### 7.3 제안

본 PR (#589) 정정 방향:
1. `wrap_precomputed` 필드 제거 (Stage B 일부)
2. typeset/paragraph_layout 의 `wrap_precomputed` 검사를 `is_in_wrap_zone` 로 교체
3. `LineSeg::is_in_wrap_zone(col_w)` helper 추가 (Stage A 일부)
4. 표준 문서화 후속 task 분리 (Stage A 본격 + C/D 분리)

이로써 **page 4/8 결함 정정 가치 보존 + IR 부채 일부 청산 + HWP3 vpos 표준화는 별도 task 로 안전 분리**.

---

## 참조

### 선행 분석 문서
- [mydocs/tech/hwp5_wrap_precomputed_analysis.md](mydocs/tech/hwp5_wrap_precomputed_analysis.md) — HWP5/HWPX wrap_precomputed 미적용 결함
- [mydocs/tech/document_ir_wrap_zone_standard_review.md](mydocs/tech/document_ir_wrap_zone_standard_review.md) — IR 표준 부재 본질

### 주요 파일 (본 분석 영역)

| 파일 | LOC | 역할 |
|------|----|------|
| `src/model/paragraph.rs` | 956 | LineSeg/Paragraph IR 정의 |
| `src/parser/body_text.rs` | 884 | HWP5 파서 |
| `src/parser/hwpx/section.rs` | 3181 | HWPX 파서 |
| `src/parser/hwp3/mod.rs` | 2013 | HWP3 파서 |
| `src/renderer/typeset.rs` | (다수) | 페이지/단 분할, wrap-around 흡수 |
| `src/renderer/layout.rs` | (다수) | 페이지 렌더링, vpos 보정 |
| `src/renderer/layout/paragraph_layout.rs` | (다수) | 문단 레이아웃, Task #489/보완6 |

### 관련 task
- **Task #321** — vpos-reset 기반 페이지 분할
- **Task #332 Stage 5** — vpos correction trigger 완화
- **Task #362** — wrap-around 흡수 및 페이지네이션
- **Task #412** — vpos_page_base / vpos_lazy_base 분리
- **Task #463** — wrap_text_x 계산
- **Task #489** — Picture/Shape Square wrap LINE_SEG.cs/sw 적용
- **Task #525** — Picture Square wrap 호스트 텍스트 중복 emit 정정
- **Task #460 보완6/8** (본 세션) — wrap_precomputed IR 플래그 도입

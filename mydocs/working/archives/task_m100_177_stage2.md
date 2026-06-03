# Stage 2 단계별 완료보고서: Serializer 원본 보존

- **타스크**: [#177](https://github.com/edwardkim/rhwp/issues/177)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task177`
- **일자**: 2026-04-18
- **단계**: Stage 2 / 4

## 1. 수행 범위

구현계획서의 Stage 2 — `src/serializer/hwpx/section.rs` 의 `push_lineseg` 하드코딩 로직을 제거하고 IR의 `Paragraph.line_segs` 값을 그대로 XML 출력으로 전환. 기존 하드코딩(`vertsize=1000, textheight=1000, baseline=850, spacing=600, horzsize=42520, flags=393216`)으로 **새 부정확 값을 생산하지 않도록** 변경.

## 2. 산출물

### 2.1 수정 파일: `src/serializer/hwpx/section.rs`

기존 303줄 → 약 380줄. 주요 변경:

#### (a) `render_paragraph_parts` 시그니처 변경

```rust
// Before: fn render_paragraph_parts(text: &str, vert_start: u32) -> ...
// After:  fn render_paragraph_parts(para: &Paragraph, vert_start: u32) -> ...
```

IR 전체에 접근 가능해져 `line_segs` 를 그대로 사용 가능.

#### (b) `render_lineseg_array_from_ir` 신규

```rust
fn render_lineseg_array_from_ir(segs: &[LineSeg]) -> String {
    let mut out = String::new();
    for seg in segs {
        out.push_str(&format!(
            r#"<hp:lineseg textpos="{}" vertpos="{}" vertsize="{}" textheight="{}" baseline="{}" spacing="{}" horzpos="{}" horzsize="{}" flags="{}"/>"#,
            seg.text_start, seg.vertical_pos, seg.line_height,
            seg.text_height, seg.baseline_distance, seg.line_spacing,
            seg.column_start, seg.segment_width, seg.tag,
        ));
    }
    out
}
```

**9개 속성 전부 IR 값** (`textpos`, `vertpos`, `vertsize`, `textheight`, `baseline`, `spacing`, `horzpos`, `horzsize`, `flags`). 하드코딩 제거.

#### (c) `render_lineseg_array_fallback` 유지 (호환성)

IR에 `line_segs` 가 비어있는 경우(예: `Document::default()` + 수동 Paragraph 생성)에만 기존 로직으로 fallback. 실문서는 `DocumentCore::from_bytes` 의 `reflow_zero_height_paragraphs` 로 IR이 채워지므로 fallback 경로로 가지 않음. 테스트 호환성 유지 목적.

#### (d) `render_paragraph_parts_for_text` 신규

`write_section` 이 `first_para == None` 인 빈 섹션을 처리할 때 텍스트만 받아 lineseg를 생성. Paragraph 없이도 호출 가능.

#### (e) `push_lineseg` → `push_lineseg_static` 으로 이름 변경

Fallback 전용임을 명확히 하기 위해 이름 변경. 코드 주석에 "이 함수 출력은 명세 상 정확한 값이 아닌 정적 자리표" 임을 명시.

### 2.2 신규 단위 테스트 (4개)

`src/serializer/hwpx/section.rs` 내부:

1. **`task177_lineseg_reflects_ir_values`** — IR 의 9개 필드 (vertical_pos=5000, line_height=1200, text_height=1100, baseline_distance=900, line_spacing=700, column_start=100, segment_width=50000, tag=999) 가 XML 속성에 정확히 반영
2. **`task177_multiple_linesegs_preserved_in_order`** — IR 3개 lineseg 가 순서대로 출력
3. **`task177_fallback_used_when_ir_empty`** — IR 비어있을 때 fallback 경로로 정적값(1000/850) 출력
4. **`task177_ir_lineseg_takes_precedence_over_text`** — 텍스트에 `\n` 이 2개여도 IR line_segs 가 1개면 1개만 출력 (원본 보존)

### 2.3 신규 통합 테스트 (2개)

`tests/hwpx_roundtrip_integration.rs`:

1. **`task177_lineseg_preserved_on_roundtrip_ref_text`** — ref_text.hwpx 파싱 → 직렬화 → 재파싱 시 모든 섹션·문단의 **모든 lineseg 6개 필드(text_start, vertical_pos, line_height, text_height, baseline_distance, line_spacing)** 가 원본과 동일
2. **`task177_lineseg_preserved_on_roundtrip_ref_mixed`** — ref_mixed.hwpx 의 첫 문단 lineseg 보존 확인

## 3. 검증 결과

### 3.1 단위 테스트

```
serializer::hwpx::section 관련: 9 passed, 0 failed
- 기존 5개 (hp_p_attrs, hp_run, page_break, default_paragraph, additional_paragraphs)
- 신규 4개 (#177 Stage 2)
```

### 3.2 기존 hwpx 테스트 유지

```
serializer::hwpx::*: 57 passed (기존과 동일, 회귀 없음)
특히 linesegs_emitted_per_linebreak 는 fallback 경로로 동일한 정적값 출력 → 그린
```

### 3.3 통합 테스트

```
10 passed, 0 failed
- Stage 0/1/5 기존 8개 유지
- #177 Stage 2 신규 2개 추가
```

### 3.4 전체 라이브러리

**864 passed, 0 failed, 1 ignored** — Stage 1 의 860 대비 +4. 회귀 0건.

## 4. 완료 기준 대조

구현계획서 Stage 2 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| `render_lineseg_array` 신규 함수로 교체, `push_lineseg` 제거 | ✅ | `render_lineseg_array_from_ir` + `push_lineseg_static` (fallback 전용) |
| `render_paragraph_parts` 시그니처 변경 (Paragraph 참조 받도록) | ✅ | `&Paragraph` 수용 |
| 기존 `linesegs_emitted_per_linebreak` 재작성 | ⏸ | 불필요 (fallback 경로로 동일 결과) |
| 신규 단위 테스트 3~4개 추가 | ✅ | 4개 |
| 통합 테스트 `task177_lineseg_preserved_on_roundtrip` 추가 | ✅ | ref_text + ref_mixed 2건 |
| 기존 라운드트립 테스트 유지 | ✅ | Stage 0/1/5 하네스 유지 |

### 재작성 생략 사유

`linesegs_emitted_per_linebreak` 테스트는 `Document::default()` + `Paragraph::default()` + 텍스트만 주입하는 구조로, 현재 **IR의 line_segs 가 비어있음** → 자동으로 fallback 경로로 진입 → 기존 하드코딩값(vertsize=1000/baseline=850/...) 을 그대로 출력 → 테스트가 이미 그린. 재작성하지 않아도 의미가 있음.

## 5. 주요 설계 결정

### 5.1 "IR 있으면 IR, 없으면 fallback" 이원화

완전 제거 대신 이원화한 이유:
- **IR 있을 때 (실문서 경로)**: 원본 값 100% 보존 → rhwp가 비표준을 새로 생산하지 않음 (Discussion #188 원칙)
- **IR 없을 때 (빈 문서 경로)**: 텍스트 기반 합리적 기본값 → 새 문서 생성 시나리오에서도 한컴이 열 수는 있는 형태 유지

### 5.2 9개 속성 전부 IR 사용

기존 하드코딩: `vertsize=1000`, `textheight=1000`, `baseline=850`, `spacing=600`, `horzpos=0`, `horzsize=42520`, `flags=393216` — **7개 속성 전부 정적**.

변경 후: 모두 IR 값 (`line_height`, `text_height`, `baseline_distance`, `line_spacing`, `column_start`, `segment_width`, `tag`). text_start/vertical_pos 는 원래부터 IR 사용.

### 5.3 `render_lineseg_array_fallback` 에서는 `DocumentCore::from_bytes` 경로 가정

`Paragraph::default()` 에는 `line_segs` 가 비어있지만, `from_bytes` 는 `reflow_zero_height_paragraphs` 로 IR을 채운다. 따라서:
- **실문서 저장 시**: fallback 경로 안 탐
- **단위 테스트에서 `Paragraph` 를 직접 만들 때**: fallback 경로
- **`Document::default()` 직렬화**: fallback (빈 섹션/문단이라 거의 영향 없음)

## 6. 알려진 제한

- **Paragraph.line_segs 가 stale 한 경우**: 편집 후 lineseg 가 IR에 남아 있지만 텍스트가 바뀌어 어긋난 경우, 그대로 출력되어 저장 파일이 부정확해질 수 있음. 이는 **편집 경로의 문제**이며 본 타스크 범위 외. `reflow_zero_height_paragraphs` 의 트리거 조건이나 명시적 invalidation 은 별도 이슈.
- **Stage 4 회귀 테스트**: 원본 보존 로직이 실제로 **rhwp 저장 → rhwp 재로드** 시 겹침을 해결하는지는 Stage 4의 `hwpx-02.hwpx` 검증에서 확인 예정.

## 7. 다음 단계 (Stage 3)

**Reflow on-demand + WASM API**:

- `DocumentCore::reflow_linesegs_on_demand()` 메서드 추가 — 사용자 명시 요청 시 넓은 기준(빈 line_segs + text 존재 포함)으로 reflow
- WASM API 2개 추가:
  - `getValidationWarnings() -> String` (JSON) — 경고 목록 조회
  - `reflowLinesegs() -> usize` — 사용자 명시 reflow 실행 + 처리된 문단 수 반환
- 단위 테스트 4개

완료 기준: rhwp-studio 에서 JS로 `window.hwpDoc.getValidationWarnings()` 호출 가능, `reflowLinesegs()` 로 보정 실행 가능.

## 8. 승인 요청

본 Stage 2 완료보고서 검토 후 승인 시 Stage 3 착수.

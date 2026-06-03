# Task #574 Stage 0 — 정밀 진단 + 본질 확정 보고서

**브랜치**: `local/task574`
**이슈**: https://github.com/edwardkim/rhwp/issues/574

---

## 1. 진단 도구

`examples/inspect_574.rs` 추가 — `rhwp dump` 가 truncate 하는 셀 내부 paragraph 의
char_shapes / CharShape 풀 덤프 (font_ids, base_size, bold, text_color, ratios).

```
cargo run --release --example inspect_574
```

## 2. 진단 결과

### 2.1 쪽번호 "1" 의 진짜 출처 (이슈 본문 가설 정정)

**이슈 본문 가설**: 바탕쪽 (master page) 의 별도 CharShape 미적용
**실제**: **본문 [6] 표 셀 paragraph[0] 의 Shape (사각형, InFrontOfText) TextBox** 내부 텍스트 "1"

```
본문 [6] 표 셀[0] p[0] ctrl[0] Shape(사각형) TextBox paras=1
  tb_p[0]: cc=2 text="1" char_shapes=1
    pos=0 cs_id=0 → size=3300(33.0pt) bold=false color=#000000 ratio[0]=90% font_id[0]=8
```

페이지 번호는 **본문 표 셀 paragraph 의 Shape 내부 TextBox 의 hardcoded text "1"**.
바탕쪽도, 머리말도 아님. (각 페이지마다 별도의 master page 가 있긴 하나 그 표 셀의
"1, 2, 31, 32" 와는 무관.)

**CharShape cs_id=0 의 IR 값**:
- `base_size=3300` (33pt)
- `bold=false` ← 핵심
- `text_color=#000000` (검정 — 이슈 본문의 "회색" 가설은 잘못)
- `ratios=[90, 90, ...]`
- `font_ids=[8, 9, 8, 7, 5, 9, 5]` — 한국어 font_id=8 → "HY견명조"
- `attr=0x00000000`

### 2.2 SVG 출력 (`output/svg/exam_science_diag/exam_science_001.svg:167`)

```xml
<text transform="translate(924.36,114.87) scale(0.9000,1)"
      font-family="HY견명조,..." font-size="44" font-weight="bold" fill="#000000">1</text>
```

| 속성 | IR (cs_id=0) | SVG | 정합 |
|------|------------|-----|------|
| size | 3300 HU (33pt) | font-size=44 (33pt at 96 dpi) | ✓ |
| ratio | 90% | scale(0.9) | ✓ |
| color | #000000 | fill="#000000" | ✓ |
| bold | **false** | **font-weight="bold"** | **✗ MISMATCH** |
| font_id | 8 → HY견명조 | HY견명조 | ✓ |

→ **본질**: bold 만 잘못. 색상은 IR/SVG 모두 검정 일치. (이슈 본문의 "회색" 가설 잘못)

### 2.3 본질 코드 위치 — `is_heavy_display_face`

`src/renderer/style_resolver.rs:601-613`:

```rust
pub(crate) fn is_heavy_display_face(font_family: &str) -> bool {
    let primary = font_family.split(',').next().unwrap_or(font_family)
        .trim().trim_matches('\'').trim_matches('"');
    matches!(primary,
        "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
        | "HY견고딕" | "HY견명조" | "HY견명조B"
        | "HY그래픽" | "HY그래픽M"
    )
}
```

`src/renderer/mod.rs:163-165`:

```rust
pub fn is_visually_bold(&self) -> bool {
    self.bold || crate::renderer::style_resolver::is_heavy_display_face(&self.font_family)
}
```

`src/renderer/svg.rs:249`:
```rust
if run.style.is_visually_bold() { attrs.push_str(" font-weight=\"bold\""); }
```

**작동 흐름**:
1. CharShape cs_id=0: `bold=false`, font_family="HY견명조,..."
2. ResolvedCharStyle: `bold=false`, `font_family="HY견명조,..."`
3. TextStyle: `bold=false`, `font_family="HY견명조,..."`
4. `is_visually_bold()` → `false || is_heavy_display_face("HY견명조,...")` → **`true`**
5. SVG 출력: `font-weight="bold"` 강제

**`is_heavy_display_face` 의도** (코드 주석 line 593-600):
> HY헤드라인M, HY견고딕 등 face 이름 자체가 굵은 display 폰트들은 HWP CharShape.bold=false
> 로 저장되어도 실제로는 시각적 bold 로 렌더된다. 해당 face 가 설치되지 않은 환경에서
> Malgun Gothic 등 regular weight fallback 으로 떨어지면 PDF(한컴) 출력과 시각 괴리가
> 발생하므로, 이 리스트에 포함된 face 는 SVG 에서 font-weight="bold" 를 강제해 fallback
> bold variant 로 근사 렌더한다.

→ **HY헤드라인M / HY견고딕** 처럼 face 이름 자체에 굵음 (Heavy/Heading) 의미가 있는
폰트 위주. **HY견명조** 는 한컴의 일반 두께 명조 폰트 — 잘못 분류.

### 2.4 가설 결론

| 가설 | 평가 |
|------|------|
| A — HWP CharShape 자체가 bold (rhwp 무결) | **반박**: cs_id=0 bold=false 확인 |
| B — parse_char_shape 비트 결함 | **반박**: cs_id=0 의 bold=false 가 parse 정상 (다른 CharShape 도 정합) |
| C — composed run char_shape_id 미전달 | **반박**: SVG 의 size/ratio/color 가 cs_id=0 과 정합 — char_shape_id 전달 정상 |
| D — marker char CharShape 미할당 | **반박 (해당 없음)**: 텍스트 "1" 은 literal 이며 marker 가 아님 |
| **E (신규) — `is_heavy_display_face` HY견명조 오분류** | **확정**: hardcoded list 의 HY견명조 가 CharShape.bold=false 무시 |

## 3. 수정 방향 (Stage 1 입력)

### 3.1 핵심 수정 (`src/renderer/style_resolver.rs:601-613`)

```rust
pub(crate) fn is_heavy_display_face(font_family: &str) -> bool {
    let primary = ...;
    matches!(primary,
        "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
        | "HY견고딕"
        // | "HY견명조" | "HY견명조B"   ← 제거
        | "HY그래픽" | "HY그래픽M"
    )
}
```

**근거**:
- HY헤드라인M, HY견고딕 = display 면 자체가 굵은 의도 (Heading / Kun "thick").
- HY견명조 = 한컴 명조 일반 두께. "견" 접두는 (한양 견명조) 이력이지만 weight 이 아님.
- HY견명조B = 명시 Bold variant — 단, exam_science 사례엔 bold=false 와 함께 쓰이지 않음.

→ **HY견명조** 단독 제거 (HY견명조B 보존 검토).

### 3.2 회귀 영향 평가

**exam_science.hwp 페이지 1 SVG** (현재):
- HY견명조 사용 텍스트 = 24개, 모두 `font-weight="bold"` 강제

**fix 후 예상**:
- CharShape.bold=true 인 HY견명조 텍스트 → 그대로 bold 유지
- CharShape.bold=false 인 HY견명조 텍스트 (쪽번호 "1", "제 4 교시" 등) → bold 해제

**골든 SVG 테스트 (`tests/golden_svg/`)**:
- HY견명조 사용 골든 SVG = **0건** (검색 결과 없음)
- HY헤드라인M 사용 골든 SVG = `issue-267/ktx-toc-page.svg` (영향 없음)

**광범위 sweep 필요**:
- 5개 샘플 (synam-001 / 복학원서 / exam_kor/eng/math) 전체 페이지 SVG diff
- HY견명조 사용 텍스트의 bold 변경 패턴 분석
- 변경된 영역이 CharShape.bold 와 일치하면 회귀 아님 (의도된 정정)

### 3.3 한컴 PDF 시각 검증

`samples/exam_science.pdf` 와 fix 후 SVG 비교 — 쪽번호 "1" 이 한컴 PDF 에서
non-bold 인지 확인 (이슈 본문에 따르면 그래야 함).

## 4. Stage 0 산출물

| 파일 | 변경 |
|------|------|
| `examples/inspect_574.rs` | 신규 (진단 스크립트, Stage 1 보존) |
| `mydocs/working/task_m100_574_stage0.md` | 본 보고서 |

## 5. 결정 요청

작업지시자 확인 필요 사항:

1. **본질 확정 동의 여부**: `is_heavy_display_face` 의 HY견명조 오분류 → 본질 확정
2. **Stage 1 진행 동의**: 단일 줄 수정 (HY견명조 / HY견명조B 제거) + TDD + 광범위 회귀 sweep
3. **HY견명조B 제거 여부**: B 변종은 명시 Bold variant. 동시 제거 vs HY견명조만 제거

---

승인 후 Stage 1 (구현 계획서 작성) 진행합니다.

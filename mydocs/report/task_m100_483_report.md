# Task M100 #483 최종 결과 보고서

| 항목 | 내용 |
|------|------|
| 이슈 | [#483](https://github.com/edwardkim/rhwp/issues/483) |
| 마일스톤 | M100 (v1.0.0) |
| 브랜치 | `local/task483` |

## 1. 증상

`samples/2010-01-06.hwp` 페이지 1 하단 각주 1)("경상수입 / .조세수입 / .세외수입") paragraph 간 줄 간격이 좁음. p[0]→p[1] gap이 HWP vpos delta(20.8px) 대비 **16.0px로 -4.8px 부족**.

## 2. 근본 원인 (확정)

`src/renderer/layout/picture_footnote.rs:702` 의 `layout_footnote_paragraph_with_number` 가 paragraph 마지막 line 처리에서 trailing `line_spacing` 누락:

```rust
parent.children.push(line_node);
y += line_height;     // ← line_spacing 누락
```

같은 라이브러리의 `paragraph_layout.rs:2560` `layout_composed_paragraph` 는 정확히 적용:
```rust
y += line_height + line_spacing_px;
```

각주 첫 paragraph만 `layout_footnote_paragraph_with_number`(번호 표시), 두 번째 이후는 `layout_composed_paragraph` 사용. 이 비대칭이 multi-paragraph 각주에서만 시각적 차이로 드러났다.

## 3. 수정

`src/renderer/layout/picture_footnote.rs:702`:

```rust
parent.children.push(line_node);
let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
y += line_height + line_spacing_px;
```

## 4. 검증

### 4-1. 핵심 회귀 케이스
| transition | Before | After | HWP vpos delta |
|-----------|--------|-------|---------------|
| 각주 1) p[0]→p[1] | 16.0px ❌ | **20.8px** ✓ | 20.8px |
| 각주 1) p[1]→p[2] | 20.8px | 20.8px | 20.8px |

### 4-2. 단위/통합 테스트
- `cargo test --release`: 1078 + 모든 통합 통과
- `cargo test --release --test svg_snapshot`: 6 통과

### 4-3. footnote-01.hwp 회귀 점검
- 페이지 1~3: SVG 미세 차이 (-209/-110/-176 bytes), 시각 거의 동일
- 페이지 4~6: identical

## 5. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/picture_footnote.rs` | line 702: `y += line_height` → `y += line_height + line_spacing_px` |

## 6. 영향 범위

| 케이스 | 영향 |
|--------|------|
| **multi-paragraph 각주** (예: 각주 1) "경상수입 / 조세수입 / 세외수입") | paragraph 간 line_spacing 정확화 (수정 의도) |
| **single-paragraph 각주** (예: 각주 2~5) | trailing line_spacing이 추가되어 다음 각주와의 gap이 line_spacing 만큼 증가. `layout_composed_paragraph`와 동일 동작 — 코드 일관성. |
| 빈 paragraph (`composed.lines.is_empty()`) | 별도 fallback 분기, 영향 없음 |

## 7. 잔여 / 후속 작업

단일 paragraph 각주 사이 gap +9.6px 증가가 한컴 출력과 정확히 일치하는지는 PDF/한컴 비교 필요(메모리 가이드 [PDF 비교 결과는 절대 기준이 아님](feedback_pdf_not_authoritative.md): 한컴 2010/2020 환경 비교 함께 점검). 본 수정은 **코드 일관성**(layout_composed_paragraph와 동일 동작)을 우선했고, 시각 회귀(골든 SVG + footnote-01.hwp)는 발생하지 않음.

## 8. 요약

- 각주 1) 줄 간격 좁음 증상 해결 ✓
- 회귀 없음 (단위 + 통합 + 골든 SVG 6건 모두 통과) ✓
- `layout_composed_paragraph` 와 코드 일관성 확보 ✓

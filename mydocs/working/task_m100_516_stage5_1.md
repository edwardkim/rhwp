# Task #516 Stage 5.1 완료 보고서 — PageLayerTree wrap 모드 분류 (Rust)

## 변경 요약

| 파일 | 변경 |
|------|------|
| `src/model/shape.rs` | `TextWrap` 에 `serde::Serialize` derive 추가 |
| `src/renderer/render_tree.rs` | `ImageNode` 에 `text_wrap: Option<TextWrap>` 필드 추가 |
| `src/paint/json.rs` | `PaintOp::Image` 직렬화에 `wrap` 필드 조건부 추가 + `text_wrap_str` 헬퍼 |
| `src/renderer/layout/paragraph_layout.rs` | 3 site 의 ImageNode 생성에 `text_wrap: Some(pic.common.text_wrap)` 전파 |
| `src/renderer/layout/picture_footnote.rs` | 2 site 동일 전파 |
| `src/renderer/layout/shape_layout.rs` | 1 site (그룹 내 그림) 전파. 1 site (셀 배경 ImageFill) 는 wrap 분리 대상 아님 → None 유지 |
| `src/renderer/layout/table_cell_content.rs` | 셀 내부 그림 (본문 layer) → `text_wrap: None` 명시 |
| `src/renderer/layout/table_layout.rs` | 셀 배경 ImageFill (본문 layer) → 변경 없음 (None default) |
| `tests/issue_516.rs` | wrap 필드 검증 테스트 1건 추가 |

소스 변경: ~+15 / 0 (대부분 1줄 추가)

## 변경 상세

### 1. ImageNode 의 text_wrap 필드

```rust
pub struct ImageNode {
    // ... 기존 필드
    /// 텍스트 흐름 wrap 모드 (Task #516, 다층 레이어 분리용).
    /// `None` 또는 `Some(Square/TopAndBottom/Tight/Through)` 는 본문 layer 에 포함되고,
    /// `Some(BehindText)` / `Some(InFrontOfText)` 는 overlay layer 로 분리 후보.
    pub text_wrap: Option<TextWrap>,
}
```

기본값 `None` — 기존 동작 보존 (본문 layer 에 포함).

### 2. PageLayerTree JSON `wrap` 필드 (조건부)

`PaintOp::Image` 직렬화에서 `text_wrap` 이 `Some(_)` 일 때만 `wrap` 필드 emit:

```json
{
  "type": "image",
  "effect": "grayScale",
  "brightness": -50,
  "contrast": 70,
  "watermark": { "preset": "custom" },
  "wrap": "behindText",
  "transform": ...
}
```

가능한 wrap 값: `square / tight / through / topAndBottom / behindText / inFrontOfText`

### 3. wrap 정보 전파 정책

| Site | wrap 전파 |
|------|----------|
| paragraph_layout 의 인라인 Picture (3 site) | ✅ Some(pic.common.text_wrap) |
| picture_footnote (2 site, 머리말/꼬리말 + 본문 그림) | ✅ Some(picture.common.text_wrap) |
| shape_layout 의 그룹 내 Picture (1 site) | ✅ Some(pic.common.text_wrap) |
| shape_layout 의 ImageFill (도형 배경, 1 site) | None (본문 layer 의 일부, 분리 대상 아님) |
| table_cell_content 의 셀 내부 그림 | None (셀 본문 layer) |
| table_layout 의 셀 배경 ImageFill | None (셀 본문 layer) |

### 4. schemaVersion 정책

- additive change → `PAGE_LAYER_TREE_SCHEMA_VERSION` 유지
- 기존 필드 의미 변경 없음
- 본 task #516 의 PR #510 정합 정책 동일

## 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ Finished |
| `cargo build --release` | ✅ Finished |
| **`cargo test --lib`** | ✅ **1110 passed** (회귀 0) |
| **`cargo test --test issue_516`** | ✅ **6 passed** (wrap 검증 1건 추가) |

### 신규 회귀 테스트

`issue_516_layer_tree_json_includes_wrap_for_behind_text`:

```rust
assert!(
    json.contains("\"wrap\":\"behindText\""),
    "복학원서.hwp 의 BehindText 그림에 wrap 필드가 직렬화되어야 함"
);
```

→ ✅ 통과 (복학원서.hwp 의 학교 로고 + 엠블렘 모두 BehindText 로 직렬화 확인)

## 위험 점검

| 위험 | 결과 |
|------|------|
| 8 site 중 wrap 전파 누락 | 🟢 0 (셀 내부 / 셀 배경은 의도적 None — 본문 layer) |
| schemaVersion 불일치 | 🟢 additive, 유지 |
| 기존 fixture 회귀 | ✅ lib 1110 + issue_416/501/514/516 회귀 0 |

## 다음 단계

Stage 5.1 완료 보고서 승인 후 **Stage 5.2** 진행:
- rhwp-studio TypeScript 의 PageLayerTree JSON consumer 작성
- wrap 모드별 layer 분리 (`splitByWrap(layerTree)` → `{ flowOps, behindImages, frontImages }`)
- 본문 (flowOps) 은 기존 Canvas 경로
- behindImages / frontImages 는 `<img>` overlay 로 DOM 추가

## 산출물

- 본 보고서 (`mydocs/working/task_m100_516_stage5_1.md`)
- 변경된 8 파일 (위 §변경 요약 참조)
- `tests/issue_516.rs` (6 tests)

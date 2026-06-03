# Task #1006 Stage 1 — 모델/파서 contract 분리

이슈: [#1006](https://github.com/edwardkim/rhwp/issues/1006)
계획서: [`task_m100_1006.md`](../plans/task_m100_1006.md), [`task_m100_1006_impl.md`](../plans/task_m100_1006_impl.md)

## 1. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/model/page.rs` | `PageBorderFill::basis` 필드 + `PageBorderBasis` enum 추가 |
| `src/parser/hwp3/mod.rs` | hwp3 → `BodyBased` 명시 |
| `src/parser/body_text.rs` | hwp5 → `PaperBased` 명시 |
| `src/parser/hwpx/section.rs` | hwpx → `PaperBased` 명시 |

## 2. 구현

### 2-1. `PageBorderFill::basis`

```rust
#[derive(Debug, Clone, Default)]
pub struct PageBorderFill {
    // ... 기존 필드
    /// [Task #1006] 쪽 테두리 기준 (포맷별 분리).
    /// HWP3 → BodyBased, HWP5/HWPX → PaperBased.
    pub basis: PageBorderBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageBorderBasis {
    #[default]
    BodyBased,
    PaperBased,
}
```

### 2-2. 파서 주입

- HWP3 (`src/parser/hwp3/mod.rs:2828`):
  ```rust
  // [Task #1006] HWP3: paper-based (작업지시자 Hancom Office close-up 시각 판정).
  // #987 의 body-based 재판정은 close-up 비교 부재로 인한 오판단이었음.
  basis: crate::model::page::PageBorderBasis::PaperBased,
  ```

- HWP5 (`src/parser/body_text.rs::parse_page_border_fill`):
  ```rust
  pbf.basis = crate::model::page::PageBorderBasis::PaperBased;
  ```

- HWPX (`src/parser/hwpx/section.rs::parse_page_border_fill_empty`):
  ```rust
  page_border_fill.basis = crate::model::page::PageBorderBasis::PaperBased;
  ```

## 3. 검증

- `cargo build --release`: 0 errors ✓ (struct field 추가만으로 기존 callsite 영향 없음)

## 4. Stage 2 진입

렌더러의 `attr bit 0` 분기를 `basis` 필드 직접 사용으로 변경.

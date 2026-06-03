# 구현 계획서: 수식 폰트 적용

- **타스크**: [#141](https://github.com/edwardkim/rhwp/issues/141)
- **마일스톤**: M100
- **브랜치**: `local/task141`
- **작성일**: 2026-04-14
- **수행계획서**: `mydocs/plans/task_m100_141.md`

## 단계 구성 (3단계)

---

### 1단계: SVG/Canvas 렌더러에 font-family 적용

**수정 파일**: `src/renderer/equation/svg_render.rs`, `src/renderer/equation/canvas_render.rs`

#### svg_render.rs 변경

font-family 상수를 정의하고, 모든 `<text>` 생성에 적용:

```rust
/// 수식 전용 font-family
const EQ_FONT_FAMILY: &str = r#""Latin Modern Math", "STIX Two Math", "Cambria Math", "Pretendard", serif"#;
```

수정 대상 `<text>` 생성 (render_box 함수 내):
- `LayoutKind::Text` (라인 42) — 이탤릭 변수
- `LayoutKind::Number` (라인 53) — 숫자
- `LayoutKind::Symbol` (라인 63) — 기호 (text-anchor="middle")
- `LayoutKind::MathSymbol` (라인 73) — 유니코드 수학 기호
- `LayoutKind::Function` (라인 83) — 함수명
- `LayoutKind::BigOp` (라인 155) — 큰 연산자 기호
- `LayoutKind::Limit` — 극한 텍스트

각 `<text>` 요소에 `font-family="{EQ_FONT_FAMILY}"` 속성 추가.

#### canvas_render.rs 변경

`set_font()` 함수(라인 219-223) 수정:

현재:
```rust
ctx.set_font(&format!("{}{}{:.1}px serif", style, weight, size));
```

변경 후:
```rust
ctx.set_font(&format!(
    "{}{}{:.1}px \"Latin Modern Math\", \"STIX Two Math\", \"Cambria Math\", \"Pretendard\", serif",
    style, weight, size
));
```

---

### 2단계: Latin Modern Math woff2 번들링

- Latin Modern Math OTF를 GUST 공식 사이트에서 다운로드
- woff2 변환 (또는 기존 woff2 배포본 사용)
- `web/fonts/` 디렉토리에 `latinmodern-math.woff2` 추가
- rhwp-studio의 CSS/JS에 `@font-face` 선언 추가

---

### 3단계: 테스트 및 검증

- `cargo test` 전체 통과
- exam_math.hwp SVG 출력 → font-family 속성 포함 확인
- rhwp-studio 브라우저에서 Canvas 렌더링 폰트 확인

---

## 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| 1단계 | `cargo test` 통과, SVG 출력에 font-family 포함 |
| 2단계 | woff2 파일 정상 로딩, @font-face 적용 |
| 3단계 | 브라우저에서 수식 렌더링 시 Latin Modern Math 적용 확인 |

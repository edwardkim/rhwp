# Stage 2 완료보고서 — Task #1562

> HWPX 폼 컨트롤 caption `&&` 표시 정합 — helper 및 renderer 적용

- **이슈**: [#1562](https://github.com/edwardkim/rhwp/issues/1562)
- **브랜치**: `local/task1562`
- **작성일**: 2026-06-26

---

## 1. 수행 내용

폼 컨트롤 caption 전용 표시 helper를 추가하고, 사용자에게 caption을 그리는 렌더러
경로에 적용했다.

변경 파일:

- `src/renderer/mod.rs`
- `src/renderer/form_caption.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`
- `src/renderer/skia/renderer.rs`

핵심 동작:

- 저장값 `R&&D`는 그대로 둔다.
- 표시 문자열 생성 시에만 `&&`를 `&`로 접는다.
- 단일 `&`는 보존한다.
- PushButton caption이 비어 있을 때 fallback으로 쓰는 `form.name`은 caption이 아니므로
  변환하지 않는다.

## 2. 구현 상세

`src/renderer/form_caption.rs`에 다음 helper를 추가했다.

```rust
pub(crate) fn display_form_caption(caption: &str) -> Cow<'_, str>
```

동작:

- `caption`에 `&&`가 없으면 borrowed 반환으로 allocation을 피한다.
- `&&`가 있으면 char 단위 좌→우 순회로 paired `&&`만 `&`로 접는다.
- single `&`는 추정 처리하지 않고 그대로 둔다.

적용 경로:

- SVG: `escape_xml(display_form_caption(...).as_ref())`
- Web Canvas: `ctx.fill_text(display_form_caption(...).as_ref(), ...)`
- Skia: `measure_str()`와 `draw_str()` 모두 display string 기준

## 3. 검증 결과

실행:

```text
cargo test --lib renderer::form_caption
cargo test --test issue_1562_hwpx_form_caption_display
cargo fmt
```

결과:

- `renderer::form_caption` 단위 테스트 3개 통과
- #1562 targeted SVG 표시 테스트 1개 통과
- `cargo fmt` 완료

## 4. 다음 단계

Stage 3에서 SVG golden을 갱신하고, #1534 저장 안정성 회귀 테스트와 snapshot 테스트를
함께 확인한다.

예상 변경:

- `tests/golden_svg/form-002/page-0.svg`

필수 검증:

- `UPDATE_GOLDEN=1 cargo test --test svg_snapshot form_002`
- `cargo test --test svg_snapshot form_002`
- `cargo test --test issue_1534_hwpx_form_caption_escape`
- `cargo test --test issue_1562_hwpx_form_caption_display`
- `cargo fmt --check`

## 5. 승인 요청

Stage 2 구현은 targeted green 상태다. Stage 3 golden 갱신 및 회귀 검증으로 진행한다.

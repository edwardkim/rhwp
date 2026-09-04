# 구현계획서 — Task #1562

> HWPX 폼 컨트롤 caption `&&`가 한컴과 다르게 `&&`로 표시됨 — 해결 구현

- **이슈**: [#1562](https://github.com/edwardkim/rhwp/issues/1562)
- **브랜치**: `local/task1562` (base: `local/devel`)
- **수행계획서**: [`task_m100_1562.md`](task_m100_1562.md)
- **작성일**: 2026-06-26

---

## 1. 채택 방향 — A안, renderer 공통 display helper

수행계획서의 A/B/C 중 **A안**을 채택한다.

저장 모델과 표시 문자열을 분리한다.

- `FormObject.caption` / `FormObjectNode.caption`은 저장값 그대로 둔다.
- HWPX parser/serializer는 수정하지 않는다.
- 폼 컨트롤 caption을 사용자에게 그리는 renderer 직전에서만 display helper를 호출한다.
- helper는 `&&`를 literal `&` 표시 escape로 해석해 `&` 한 글자로 접는다.

이번 구현은 관측된 한컴 표시 결과와 #1562 완료 조건에 맞춰 **`&&` collapse에 한정**한다.
단일 `&`를 mnemonic prefix로 제거하거나 밑줄 표시하는 동작은 이번 PR 범위에서 제외한다.

## 2. Helper 설계

신규 모듈:

```text
src/renderer/form_caption.rs
```

`src/renderer/mod.rs`에 다음 모듈을 추가한다.

```rust
pub(crate) mod form_caption;
```

API:

```rust
pub(crate) fn display_form_caption(caption: &str) -> std::borrow::Cow<'_, str>
```

동작:

| 입력 | 출력 | 비고 |
|------|------|------|
| `R&&D` | `R&D` | #1562 핵심 |
| `IP R&&D연계` | `IP R&D연계` | form-002 |
| `R&&D 자율성트랙(일반)` | `R&D 자율성트랙(일반)` | form-002 |
| `R&D` | `R&D` | 단일 `&`는 보존 |
| `&&&&` | `&&` | 좌→우 paired collapse |
| `abc` | borrowed `abc` | `&&`가 없으면 allocation 없음 |

구현 기준:

- `caption.contains("&&") == false`이면 `Cow::Borrowed(caption)` 반환
- `&&`가 있으면 char 단위로 좌→우 순회하며 paired `&&`를 `&`로 접는다.
- single `&`는 그대로 둔다.

단위 테스트는 같은 모듈의 `#[cfg(test)]` 테스트로 둔다.

## 3. 적용 지점

### SVG renderer

파일: `src/renderer/svg.rs`

현재:

- PushButton: `escape_xml(&form.caption)`
- CheckBox: `escape_xml(&form.caption)`
- RadioButton: `escape_xml(&form.caption)`

변경:

- `display_form_caption(&form.caption)` 결과를 만든 뒤 `escape_xml(display.as_ref())`로 출력
- PushButton의 caption empty fallback인 `form.name`은 caption이 아니므로 변환하지 않는다.

### Web Canvas renderer

파일: `src/renderer/web_canvas.rs`

현재:

- `ctx.fill_text(&form.caption, ...)`

변경:

- PushButton/CheckBox/RadioButton caption 출력 전에 `display_form_caption(&form.caption)` 사용
- `fill_text(display.as_ref(), ...)` 호출

### Skia renderer

파일: `src/renderer/skia/renderer.rs`

현재:

- PushButton label 측정/출력: `form.caption`
- CheckBox/RadioButton 출력: `form.caption`

변경:

- caption이 있을 때만 `display_form_caption(&form.caption)` 사용
- PushButton fallback `form.name`은 변환하지 않는다.
- `measure_str()`와 `draw_str()` 모두 display string 기준으로 수행해 text width와 실제 출력이 일치하도록 한다.

## 4. 테스트 계획

### Stage 1 — red 테스트 추가

신규 테스트:

```text
tests/issue_1562_hwpx_form_caption_display.rs
```

테스트 내용:

1. `samples/hwpx/form-002.hwpx` page 0을 `HwpDocument::render_page_svg_native(0)`로 렌더링한다.
2. SVG 출력에 다음이 포함되는지 확인한다.
   - `IP R&amp;D연계`
   - `R&amp;D 자율성트랙(일반)`
   - `R&amp;D 자율성트랙(지정)`
3. SVG 출력에 다음이 없어야 한다.
   - `R&amp;&amp;D`

현재 코드에서는 `R&amp;&amp;D`가 남으므로 red가 되어야 한다.

산출물:

- `tests/issue_1562_hwpx_form_caption_display.rs`
- `mydocs/working/task_m100_1562_stage1.md`

커밋:

```text
test: add issue 1562 form caption display regression
```

### Stage 2 — helper + renderer 적용

변경 파일:

- `src/renderer/mod.rs`
- `src/renderer/form_caption.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`
- `src/renderer/skia/renderer.rs`

검증:

- `cargo test --test issue_1562_hwpx_form_caption_display`
- `cargo test --lib renderer::form_caption`

산출물:

- `mydocs/working/task_m100_1562_stage2.md`

커밋:

```text
fix(renderer): display escaped form caption ampersands
```

### Stage 3 — SVG golden 갱신 + 회귀 검증

예상 변경:

- `tests/golden_svg/form-002/page-0.svg`

절차:

1. `UPDATE_GOLDEN=1 cargo test --test svg_snapshot form_002`
2. `cargo test --test svg_snapshot form_002`
3. golden diff에서 `R&amp;&amp;D` → `R&amp;D` 텍스트 변경만 의도된 범위인지 확인

저장 안정성 회귀:

- `cargo test --test issue_1534_hwpx_form_caption_escape`

renderer targeted:

- `cargo test --test issue_1562_hwpx_form_caption_display`
- `cargo fmt --check`

필요 시:

- `cargo clippy --all-targets -- -D warnings`

산출물:

- `tests/golden_svg/form-002/page-0.svg`
- `mydocs/working/task_m100_1562_stage3.md`

커밋:

```text
test(svg): update form caption display golden
```

### Stage 4 — 최종 보고서

파일:

```text
mydocs/report/task_m100_1562_report.md
```

포함 내용:

- 원인: 저장값을 표시값으로 그대로 사용
- 수정: form caption display helper + renderer 적용
- 저장값 보존 확인: #1534 테스트 통과
- 표시 확인: #1562 테스트와 SVG golden diff
- 잔여 리스크: 단일 `&` mnemonic prefix 해석은 미적용
- #1534 parent issue close 판단: #1562 완료 후 함께 close 가능 여부

커밋:

```text
docs: report task 1562 completion
```

## 5. 변경 파일 목록

| 파일 | 변경 |
|------|------|
| `src/renderer/mod.rs` | `form_caption` 모듈 추가 |
| `src/renderer/form_caption.rs` | form caption display helper + 단위 테스트 |
| `src/renderer/svg.rs` | form caption 출력 시 helper 사용 |
| `src/renderer/web_canvas.rs` | form caption 출력 시 helper 사용 |
| `src/renderer/skia/renderer.rs` | form caption 측정/출력 시 helper 사용 |
| `tests/issue_1562_hwpx_form_caption_display.rs` | SVG 표시 회귀 테스트 |
| `tests/golden_svg/form-002/page-0.svg` | 의도된 표시 diff 반영 |
| `mydocs/working/task_m100_1562_stage{1..3}.md` | 단계별 완료보고서 |
| `mydocs/report/task_m100_1562_report.md` | 최종 보고서 |

## 6. Definition of Done

1. `FormObject.caption` 저장값은 `R&&D` 형태로 유지된다.
2. HWPX roundtrip XML은 `R&amp;&amp;D` 형태를 유지한다.
3. SVG 출력은 `R&amp;D`로 표시하고 `R&amp;&amp;D`를 남기지 않는다.
4. Web Canvas와 Skia도 동일 helper를 사용한다.
5. `cargo test --test issue_1562_hwpx_form_caption_display` 통과.
6. `cargo test --test issue_1534_hwpx_form_caption_escape` 통과.
7. `cargo test --test svg_snapshot form_002` 통과.
8. `cargo fmt --check` 통과.

## 7. 제외 범위

- HWPX parser/serializer의 `caption` 저장값 변환
- 모든 XML attribute의 전역 `&&` 치환
- 본문 텍스트의 `&&` 변환
- 표/그림/도형 `<hp:caption>` 변환
- 단일 `&` mnemonic prefix 제거 또는 access-key 밑줄 표시
- 새 한컴 샘플 생성/추가

## 8. 승인 게이트

본 구현계획서 승인 전까지 소스 수정은 하지 않는다.

승인 후 Stage 1부터 진행하며, 각 stage 완료 시 단계보고서 작성 후 다음 단계 진행
승인을 요청한다.

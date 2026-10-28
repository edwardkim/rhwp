# 최종 결과보고서 — Task #1562

> HWPX 폼 컨트롤 caption `&&`가 한컴과 다르게 `&&`로 표시됨 — 해결

- **이슈**: [#1562](https://github.com/edwardkim/rhwp/issues/1562)
- **Parent**: [#1534](https://github.com/edwardkim/rhwp/issues/1534)
- **마일스톤**: v1.0.0 (M100)
- **브랜치**: `local/task1562` (base: `local/devel`)
- **작성일**: 2026-06-26
- **상태**: 구현·검증 완료 (PR 생성·이슈 클로즈 승인 대기)

---

## 1. 문제

#1534 / PR #1536에서 HWPX 폼 컨트롤 caption의 XML escape 누적 손상은 해결했지만,
표시 계층에는 별도 문제가 남아 있었다.

`samples/hwpx/form-002.hwpx` 원본은 다음 저장값을 가진다.

| 계층 | 값 |
|------|----|
| XML 원문 | `caption="IP R&amp;&amp;D연계"` |
| 저장 모델 | `IP R&&D연계` |
| 한컴 뷰어 표시 | `IP R&D연계` |
| 기존 rhwp 표시 | `IP R&&D연계` |

즉 XML/저장 계층에서는 `&&`가 유지되어야 하지만, 폼 UI caption 표시에서는 `&&`가
literal `&` 한 글자로 표시되어야 한다.

## 2. 원인

rhwp 렌더러가 `FormObject.caption` 저장값을 표시 문자열로 그대로 사용했다.

| 영역 | 기존 동작 |
|------|-----------|
| SVG renderer | `escape_xml(&form.caption)` 직접 출력 |
| Web Canvas renderer | `ctx.fill_text(&form.caption, ...)` 직접 출력 |
| Skia renderer | `measure_str(&form.caption)` / `draw_str(&form.caption, ...)` 직접 사용 |

`caption` 저장값과 사용자 표시값이 분리되어 있지 않아, 한컴 폼 caption 관례로 보이는
`&&` 표시 escape가 적용되지 않았다.

## 3. 해결

폼 컨트롤 caption 전용 표시 helper를 추가하고, 사용자에게 caption을 그리는 renderer
경로에서만 적용했다.

신규 helper:

```rust
pub(crate) fn display_form_caption(caption: &str) -> Cow<'_, str>
```

정책:

- `&&` → `&`
- 단일 `&`는 보존
- `&&`가 없으면 borrowed 반환으로 allocation 없음
- 저장 모델과 HWPX serializer는 변경하지 않음

적용 범위:

- PushButton caption
- CheckBox caption
- RadioButton caption
- SVG / Web Canvas / Skia renderer

제외 범위:

- HWPX parser/serializer
- 본문 텍스트
- 표/그림/도형 `<hp:caption>`
- ComboBox/Edit `text`
- 단일 `&` mnemonic prefix 제거 또는 access-key 밑줄 표시

## 4. 검증

| 검사 | 결과 |
|------|------|
| `cargo test --lib renderer::form_caption` | 3/3 통과 |
| `cargo test --test issue_1562_hwpx_form_caption_display` | 1/1 통과 |
| `env UPDATE_GOLDEN=1 cargo test --test svg_snapshot form_002` | 통과 |
| `cargo test --test svg_snapshot form_002` | 통과 |
| `cargo test --test issue_1534_hwpx_form_caption_escape` | 4/4 통과 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

SVG golden 변경은 다음 3줄로 제한됐다.

- `IP R&amp;&amp;D연계` → `IP R&amp;D연계`
- `R&amp;&amp;D 자율성트랙(일반)` → `R&amp;D 자율성트랙(일반)`
- `R&amp;&amp;D 자율성트랙(지정)` → `R&amp;D 자율성트랙(지정)`

좌표, 폰트, 체크박스 도형, 기타 본문 텍스트 변경은 없었다.

## 5. 저장값 보존

#1534 저장 안정성 테스트가 통과했으므로 이번 변경은 XML/IR roundtrip을 변경하지 않는다.

| 항목 | 결과 |
|------|------|
| 저장 모델 | `R&&D` 유지 |
| HWPX XML | `R&amp;&amp;D` 유지 |
| SVG 표시 | `R&amp;D`로 출력 |

따라서 #1534의 XML escape 누적 방지 수정과 충돌하지 않는다.

## 6. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/mod.rs` | `form_caption` 모듈 추가 |
| `src/renderer/form_caption.rs` | form caption display helper + 단위 테스트 |
| `src/renderer/svg.rs` | form caption 출력 시 helper 사용 |
| `src/renderer/web_canvas.rs` | form caption 출력 시 helper 사용 |
| `src/renderer/skia/renderer.rs` | form caption 측정/출력 시 helper 사용 |
| `tests/issue_1562_hwpx_form_caption_display.rs` | SVG 표시 회귀 테스트 |
| `tests/golden_svg/form-002/page-0.svg` | 의도된 표시 diff 반영 |
| `mydocs/orders/20260626.md` | 오늘 할일 등록 |
| `mydocs/plans/task_m100_1562.md` | 수행계획서 |
| `mydocs/plans/task_m100_1562_impl.md` | 구현계획서 |
| `mydocs/working/task_m100_1562_stage{1..3}.md` | 단계별 보고서 |
| 본 보고서 | 최종 결과 |

## 7. 커밋 이력

- `4e99c722` — `test: add issue 1562 form caption display regression`
- `8155f730` — `fix(renderer): display escaped form caption ampersands`
- `1396b5d9` — `test(svg): update form caption display golden`
- Stage 4 — 본 최종 보고서

## 8. 잔여 리스크

- 단일 `&`를 mnemonic prefix로 해석해 제거하거나 access-key 밑줄을 그리는 동작은 구현하지 않았다.
  한컴 샘플과 표시 근거가 추가로 확보되면 별도 이슈로 확장하는 편이 안전하다.
- `&&` 표시 규칙은 한컴 공개 스펙에 명시된 규칙이 아니라, 한컴 뷰어 표시 결과와
  폼 UI caption 관례를 근거로 적용했다.

## 9. 이슈 close 판단

#1562가 merge되면 이 sub-issue의 완료 조건은 충족된다.

#1534 parent issue는 #1536 merge로 XML escape 누적 손상이 해결되었고, #1562로 남긴
표시 호환성 문제까지 본 작업에서 해결되므로, #1562 PR merge 후 작업지시자 승인 하에
#1562와 #1534를 함께 close할 수 있다.

## 10. 결론

폼 컨트롤 caption 저장값과 표시값을 분리해 `R&&D` 저장값을 보존하면서 화면/SVG/Canvas/Skia
표시는 `R&D`로 맞췄다. targeted 테스트, SVG golden, #1534 저장 안정성 회귀가 모두 통과해
PR 생성 준비가 완료됐다.

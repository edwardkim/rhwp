# 구현 계획서 — Task M100-823

## 1. 구현 방향

`SkiaLayerRenderer`가 보유한 `FontMgr`와 같은 생명주기에 시스템 font family
캐시를 둔다. 이후 모든 시스템 family lookup은 이 캐시를 통과한 뒤에만
`match_family_style`을 호출한다.

예상 구조:

```rust
pub struct SkiaLayerRenderer {
    font_mgr: FontMgr,
    custom_typefaces: HashMap<String, Typeface>,
    system_families: HashSet<String>,
}
```

`FontMgr::family_names()` 결과의 대소문자/별칭 처리 정책은 먼저 단순 exact match로
시작한다. 현재 fallback 후보들은 정해진 family 문자열이고, 이슈의 직접 해법도
exact membership skip이다.

## 2. 세부 단계

### Stage 1: 공통 helper 추가

`src/renderer/skia/renderer.rs`:

- `SkiaLayerRenderer::new()`에서 `font_mgr.family_names()`를 수집한다.
- `has_system_family(&self, family: &str) -> bool` 또는
  `match_system_family_style(&self, family, style)` helper를 둔다.
- custom typeface lookup은 기존 `custom_typefaces`를 계속 우선한다.

### Stage 2: 본문 text replay fallback chain 정정

`src/renderer/skia/text_replay.rs`:

- `SkiaTextReplay`에 `system_families: &HashSet<String>` 또는 helper 접근을 전달한다.
- 본문 fallback chain에서 custom typeface 수집은 그대로 둔다.
- `self.font_mgr.match_family_style(family, font_style)` 호출 전
  `system_families.contains(family)`를 확인한다.
- `make_mark_font("DejaVu Sans")` 단발 호출도 캐시에 있을 때만 호출하고,
  없으면 `legacy_make_typeface(None, FontStyle::normal())`로 넘어간다.

### Stage 3: form control CJK fallback 정정

`src/renderer/skia/renderer.rs::make_form_font`:

- custom typeface hit는 기존처럼 허용한다.
- 시스템 family는 캐시에 있을 때만 `match_family_style`을 호출한다.

### Stage 4: 수식 Skia 렌더 fallback 정정

`src/renderer/skia/equation_conv.rs`:

- `render_equation` / 내부 `draw_text` 호출 경로에 `system_families` 참조를 전달한다.
- `EQ_FONT_FAMILY` split fallback chain에서 캐시에 있는 family만
  `match_family_style`에 넘긴다.
- 마지막 `legacy_make_typeface(None, font_style)` fallback은 유지한다.

### Stage 5: 회귀 가드

가능한 테스트 후보:

- helper 단위 테스트: missing family가 skip되는지 확인.
- Skia renderer 단위 테스트: `system_families`에 없는 family 후보가 있어도
  fallback path가 정상적으로 font를 만든다는 점 확인.
- 직접 `match_family_style` 호출 여부를 mock하기 어렵다면, helper를 순수 함수로
  분리해 membership filtering을 결정적으로 검증한다.

### Stage 6: 검증과 보고

- `cargo fmt --check`
- `cargo test --release --lib`
- 신규 테스트
- 필요 시 WASM 빌드
- 단계 보고서 `mydocs/working/task_m100_823_stage1.md`
- 최종 보고서 `mydocs/report/task_m100_823_report.md`

## 3. 리스크와 완화

| 리스크 | 완화 |
|--------|------|
| family name exact match로 일부 별칭 miss | 기존 fallback chain의 후속 family와 legacy fallback 유지 |
| custom font path가 시스템 캐시에 없어서 skip될 위험 | custom typeface lookup은 시스템 캐시와 무관하게 먼저 처리 |
| equation_conv free function에 인자 전파 필요 | `render_equation` 호출부는 `renderer.rs` 내부 2곳으로 제한됨 |
| headless macOS hang 재현 어려움 | missing family skip 구조 테스트 + maintainer/CI smoke로 보완 |

## 4. 승인 후 산출물

- 안전화 코드 패치
- 회귀 테스트
- Stage 보고서
- 최종 보고서
- 오늘할일 갱신

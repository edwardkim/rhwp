# task_m100_4379 Stage 1 — SVG editor_only 판정을 RenderProfile 로 통일

- **이슈**: [#4379](https://github.com/edwardkim/rhwp/issues/4379)
- **PR**: [#4394](https://github.com/edwardkim/rhwp/pull/4394)
- **브랜치**: `fix/issue-4379-svg-render-path-consolidation`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 전체 검증 + Skia 3종 + wasm-pack 통과, PR 게시
- **기록일**: 2026-08-09 KST

## 1. 발견 경위

#4326(PR #4374)의 시각 증적을 검증하다 나왔다. `rhwp export-svg` 를 옵션 없이 돌려 82페이지 바이트
동일을 얻고 "시각 증적"이라 적었는데, **옵션 없는 `export-svg` 가 paint 계층을 거치지 않는 legacy
경로**임을 뒤늦게 확인했다. 표현이 실제보다 넓었다.

그 확인 과정에서 SVG 출력 경로가 두 벌이고 같은 판정이 각자 다르게 구현돼 있음이 드러났다.

## 2. 결함

| | legacy | layer |
|---|---|---|
| 파이프라인 | `build_page_tree` → `SvgRenderer::render_tree` | `build_page_layer_tree_with_profile` → `SvgLayerRenderer` |
| paint 계층 | 미경유 | 경유 |
| `editor_only` 판정 | `svg.rs:271` `!self.show_editor_only_nodes`(bool, 기본 `true`) | `paint/builder.rs:75` `!profile.shows_editor_visuals()` |

`svg.rs` 주석이 이중성을 자인하고 있었다 — *"LayerBuilder 는 이미 `editor_only` 를 프로필로
걸러내지만 SVG 렌더러는 렌더 트리를 직접 순회해 그 계약 밖에 있었다."*

**둘 다 프로덕션에서 살아 있다**: studio 화면 SVG(`main.ts:1468`) → legacy, PDF 내보내기
(`file.ts:461`) → layer, CLI `export-svg` 기본 → legacy.

경로 선택이 `RHWP_RENDER_PATH` 환경변수인데, `wasm32-unknown-unknown` 에는 프로세스 환경이 없어
`std::env::var` 가 항상 `Err` 다 — **브라우저에서는 이 함수로 layer 경로에 도달할 수 없다.**

## 3. 두 경로가 같아야 하는 범위 — 먼저 정의했다

전부 같아야 하는 것이 아니다.

- **같아야 함**(같은 `RenderProfile` 에서) — `editor_only` 게이트 판정, 콘텐츠 노드
  (TextRun/Image/Table/Shape/Equation) 렌더 결과
- **달라도 됨**(의도적 직교 축) — `show_paragraph_marks`/`show_control_codes`(인스턴스 플래그,
  프로필과 무관), `debug_overlay`(legacy 전용), `--font-style`/`--embed-fonts`(legacy 전용, CLI 가
  이미 `--profile` 과 상호 배타로 강제)

## 4. 구현

`SvgRenderer.show_editor_only_nodes: bool` → `profile: RenderProfile`. 게이트가
`paint/builder.rs:75` 와 **문자 그대로 같은 술어**를 부른다. 기본값 `Screen` 이
`shows_editor_visuals() == true` 로 종전 bool 기본값과 동치라 기존 호출부 동작은 불변이다.

env 게이트를 `#[cfg(not(target_arch = "wasm32"))]` 로 감쌌다 — 장식이 아니라 **사실 표시**다.
주석에 그 이유를 남겼다. env var 자체는 유지했다(제거하면 `main.rs`/`wasm_api.rs`/기존 A/B 디버그
워크플로까지 건드려야 해 범위 밖).

`cli_commands.md` 의 `export-svg --profile` 설명을 정정했다 — "생략 시 인쇄 등가 억제"는 부정확하다.
missing-picture 만 억제되고 다른 `editor_only` 장식은 기본 표시된다.

## 5. 재분기 방지 장치

`svg_layer.rs::editor_only_gate_matches_across_legacy_and_layer_paths_for_every_profile` 신설.
`RenderProfile` 4종 각각에서 일반 콘텐츠 노드와 `editor_only` 노드를 함께 놓고 두 경로 출력이 바이트
단위로 같은지, 표시 여부가 `shows_editor_visuals()` 와 일치하는지 고정한다.

## 6. 범위 밖

`PlaceholderKind::MissingPicture` 억제는 `svg.rs`/`skia/renderer.rs`/`web_canvas.rs` 에 각각
재구현돼 있다. `editor_only` 로 통합하려면 노드 생성 시점(`picture_footnote.rs`)에
`.with_editor_only()` 를 걸어야 하는데, 그러면 **편집 캔버스가 항상 보여야 하는 것**과 **프로필로
갈려야 하는 것**의 경계를 재검증해야 한다.

경로 선택을 호출부 명시 인자로 완전히 대체하는 것도 범위 밖이다.

## 7. 검증 (완료)

- `cargo test --profile release-test --tests` 전체 통과(497 바이너리, 5500 테스트).
- Native Skia 3종 통과, `wasm-pack build --target web` 성공.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 통과. wasm32 타깃 check/clippy
  도 0 error, 신규 경고 0.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.

# Stage 4 보고서 — Task M100-1187: paint layer TextBox ClipRect 적용

## 목표

SVG 외 paint/replay 경로에서도 글상자 내부 콘텐츠를 `TextBox` 영역으로 clip 하도록 `PageLayerTree` 경로를 보강한다.

## 변경

### src/paint/layer_tree.rs

- `ClipKind::TextBox` 를 추가했다.

### src/paint/builder.rs

- `RenderNodeType::TextBox` 를 `LayerNodeKind::ClipRect { clip_kind: ClipKind::TextBox }` 로 내린다.
- ClipRect 의 child 는 `GroupKind::TextBox` group 으로 유지한다.
- builder 단위 테스트 `builds_textbox_clip_layer` 를 추가했다.

### src/paint/json.rs / src/paint/schema.rs

- `ClipKind::TextBox` 를 JSON `"clipKind":"textBox"` 로 직렬화한다.
- `schemaMinorVersion` 을 `14 -> 15` 로 올렸다.
- JSON 직렬화 테스트 `serializes_textbox_clip_kind` 를 추가했다.

### renderer / studio 소비 경로

- `src/renderer/svg_layer.rs`: layer-to-SVG 변환에서 `ClipKind::TextBox` 를 `RenderNodeType::TextBox` 로 복원한다.
- `src/renderer/web_canvas.rs`: web canvas layer replay 에서 `TextBox` clip 을 정확한 clip rect 로 적용한다.
- `src/renderer/canvaskit_policy.rs`: CanvasKit replay 정책 진단 문자열에 `textBox` 를 추가했다.
- `rhwp-studio/src/core/types.ts`: `LayerClipNode.clipKind` union 에 `'textBox'` 를 추가했다.

### tests/issue_1187_textbox_clip.rs

- 기존 SVG clipPath 검증에 더해, `BookReview.hwp` 1쪽 PageLayerTree JSON 에 `"clipKind":"textBox"` 가 최소 3개 존재하는지 확인하는 paint layer 회귀 테스트를 추가했다.

## 확인

```bash
cargo fmt --check
cargo test --test issue_1187_textbox_clip
cargo build --bin rhwp
cargo test --lib paint::builder::tests
cargo test --lib paint::json::tests::serializes_textbox_clip_kind
cargo test --lib paint::schema::tests::layer_tree_schema_constants_match_schema
cargo test --test issue_1052_footnote_in_textbox
cargo test --test issue_919_textbox_hit_test
```

- `cargo fmt --check`: 통과
- `issue_1187_textbox_clip`: 2 passed
- `cargo build --bin rhwp`: 통과
- `paint::builder::tests`: 7 passed
- `paint::json::tests::serializes_textbox_clip_kind`: 1 passed
- `paint::schema::tests::layer_tree_schema_constants_match_schema`: 1 passed
- `issue_1052_footnote_in_textbox`: 4 passed
- `issue_919_textbox_hit_test`: 5 passed

추가 확인:

```bash
npm run build
```

- 실패: `tsc: command not found`
- 원인: `/private/tmp/rhwp-task1187/rhwp-studio/node_modules` 및 전역 `tsc` 가 없는 로컬 환경 상태다.
- 네트워크 의존성 설치는 수행하지 않았다.

## 다음 (Stage 5)

최종 통합 확인, 산출 SVG/Layer JSON 근거 정리, 최종 보고서를 작성한다.

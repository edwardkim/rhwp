# PR #982 검토 — PageLayerTree textRun display text contract

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #982 |
| 제목 | Task #948: PageLayerTree textRun display text contract |
| 작성자 | postmelee (Taegyu Lee) — 기존 컨트리뷰터 |
| base ← head | `devel` ← `task-948-pagelayertree-display-text` |
| 연결 이슈 | Refs #948 (관련 #937/#947), assignee 본인 지정 완료 |
| mergeable | **CONFLICTING / DIRTY** ⚠️ (cherry-pick 으로 해소) |
| CI | Build & Test ✅ / Analyze ✅ / Canvas diff ✅ / CodeQL ✅ |
| 커밋 | 1 (babb6816) |

## 2. 배경 (이슈 #948)

#937/#947 에서 복학원서 서명란 한컴 PUA(`U+F012B`→`(인)`,
`U+F081C` TAC filler→숨김)가 SVG/WebCanvas/HTML/Canvas/Skia
렌더러는 core display text 규칙을 쓰도록 정정됨. 그러나
renderer-independent paint contract 인 **PageLayerTree JSON 은
여전히 `run.text`(source)만 내보내** consumer 가 core PUA 규칙을
중복 구현하거나 깨진 glyph 를 출력해야 했음.

## 3. 변경 내용 (Rust 코드 3 + 테스트 1 + 문서 7)

| 파일 | 변경 |
|------|------|
| `src/renderer/composer.rs` | `expand_pua_display_text()` 공용 helper 추출. 기존 `expand_pua_render_text()` 는 이를 호출 (동작 보존). U+F081C 제거, U+F012B→"(인)", old-hangul/bullet 매핑, 미지 PUA 원문 유지 |
| `src/paint/json.rs` | `TextRun` JSON 에 `displayText`/`displayPositions` additive 추가. `displayText != text` 일 때만 출력. `knownFeatures` 에 `text.displayText`, `usedFeatures` 는 실제 field 존재 tree 에만, `requiredFeatures` 는 비움 |
| `src/paint/schema.rs` | `schemaMinorVersion` 11 → 12 (additive revision) |
| `tests/issue_948.rs` (신규) | 복학원서 page 0 PageLayerTree JSON 검증 + json.rs synthetic unit |
| 문서 7 (plans/working/report/orders) | Task #948 산출물 |

## 4. 검토 의견

### 4.1 강점

- **순수 additive contract**: 기존 `text`/`positions`/`clusters`
  의미·값 불변. 신규 `displayText`/`displayPositions` 는 optional.
  `requiredFeatures: []` 로 기존 consumer fallback
  (`drawText = displayText ?? text`) 보존.
- `expand_pua_display_text()` helper 추출 + 기존 함수가 위임 —
  PUA 규칙 단일 출처화, 렌더러/JSON 경로 일관성 확보 (이슈 핵심).
- `displayText == text` 인 일반 run 은 신규 필드 생략 — JSON 크기
  및 하위 호환 배려.
- `usedFeatures` 는 실제 display field 있는 tree 에만 추가 —
  feature 보고 정확.
- 미지 PUA 는 추측 없이 원문 유지 — 안전.
- schema minor 증가가 additive revision 규약에 부합.

### 4.2 검토 포인트

- **CONFLICTING/DIRTY**: base devel 과 충돌. #971/#976/#979/#983-985
  와 동일하게 **cherry-pick (`babb6816` → 최신 local/devel)** 으로
  해소 후 검증.
- json.rs 는 renderer-independent paint contract 핵심. 회귀 가드
  (`paint::schema`, `page_layer_tree_export`, `paint::json`,
  `issue_937`, `issue_948`) 통과가 수용 전제.
- 본질이 시각 판정이 아닌 **JSON contract / 텍스트 매핑 정합**.
  시각 부담 낮음 — 단위/계약 테스트가 핵심 게이트.

## 5. 검증 계획

- [ ] cherry-pick (`babb6816` → 최신 devel, 충돌 해소)
- [ ] `cargo test --test issue_948` / `--test issue_937`
- [ ] `cargo test paint::json` / `paint::schema` /
      `page_layer_tree_export`
- [ ] 전체 `cargo test` + `cargo clippy -- -D warnings` +
      `cargo fmt --all -- --check`
- [ ] WASM 빌드 (composer/json core 변경 — WASM 영향 확인)

## 6. 판단 (잠정)

순수 additive 설계 + PUA 규칙 단일 출처화 + 기존 호환 보존이
타당. 충돌은 cherry-pick 으로 해소. 시각 본질 아님 —
계약/단위 테스트 통과가 판정 핵심. 검증 통과 시 수용 권고.

검증 결과에 따라 `pr_982_report.md` 작성.

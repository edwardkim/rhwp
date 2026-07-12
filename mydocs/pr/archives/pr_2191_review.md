# PR #2191 검토 - CanvasKit 첨자 텍스트 shaping 안전 경로

- PR: https://github.com/edwardkim/rhwp/pull/2191
- 작성자: `seo-rii` (외부 contributor)
- reviewer: `jangster77` 지정
- base/head: `devel` <- `seo-rii:render-p33-followup`
- 관련 추적: Follow-up #2188, Refs #536 (closing keyword 없음)
- 최종 코드 head: `3120f8c18e4d427f297205dc5a99774bdb6aa369`
- merge: 2026-07-11 squash merge `223f34f7e1a2eec18d901d07f853ab11c42bb389`

## 결론

**merge 완료.** 원 PR의 printable ASCII direct glyph replay 제한, PUA display projection,
glyph ID `0` 거부, 예외 정리 보강은 유효했다. 다만 CanvasKit `drawText`는 simple-text API라
결합문자·비-ASCII 첨자 텍스트를 shaping 없이 그리게 되는 경로가 남아 있었다. 메인터너 보완
커밋에서 해당 경로를 `ParagraphBuilder` shaping으로 분리한 뒤 최신 CI를 통과해 merge했다.

## 변경 범위

### contributor 원 변경 (`5634bf13d`)

- `src/renderer/canvaskit_policy.rs`: 위첨자/아래첨자의 direct replay를 projection 후 printable
  ASCII로 제한하고, shaping이 필요한 텍스트를 `scriptTextRequiresShaping`으로 정책에 노출한다.
- `rhwp-studio/src/core/types.ts`: `LayerTextRunOp`에 `displayText`/`displayPositions`를 추가한다.
- `rhwp-studio/src/view/canvaskit-renderer.ts`: visual projection 사용, glyph ID `0` 거부,
  Canvas state와 `Font`/`Paint`의 예외 정리를 보강한다.
- `rhwp-studio/e2e/renderer-contract.test.mjs`: Vite SSR에서 renderer를 실행하는 계약 회귀를 추가한다.

### 메인터너 보완 (`3120f8c18`)

- 기본 Noto Sans KR bytes로 `FontMgr`도 만들고 renderer dispose 시 typeface와 함께 해제한다.
- 비-ASCII superscript/subscript는 `drawText` 대신 CanvasKit `ParagraphBuilder`/`drawParagraph`
  shaping 경로로 보낸다. printable ASCII와 유효한 producer advance는 기존 direct glyph replay를 유지한다.
- 실제 `U+F012B -> (인)` display projection, shaping resource 부재 시 `drawText` 미사용,
  paragraph/builder 예외 정리를 회귀 테스트로 고정한다.

## 렌더 영향 및 시각 검증

CanvasKit renderer 출력 경로 변경이므로 visual 검증 대상으로 분류했다. 이 PR은 HWP/HWPX 문서 조판이나
페이지 수를 바꾸는 변경이 아니라 CanvasKit text API 선택을 보정한다. 따라서 MCP PDF 변환이나 문서
visual sweep 대신 아래 동등 검증을 사용했다.

- Vite SSR 계약 테스트가 실제 `CanvasKitLayerRenderer`를 실행해 direct glyph replay와 paragraph
  shaping 분기를 확인했다.
- CanvasKit 실번들에서 `ParagraphBuilder.RequiresClientICU() = false`, Noto Sans KR `FontMgr`
  생성, 결합문자 `U+0065 U+0301` paragraph layout을 확인했다.
- 최종 PR head의 GitHub Actions `Render Diff` 중 Canvas visual diff가 성공했다.

별도 HWP/HWPX 원본, MCP 기준 PDF, review PNG asset은 이 API-수준 검증의 입력이 아니므로 추가하지
않았다. 이 판단은 문서 fidelity나 PDF 페이지 수 정합을 주장하는 근거가 아니다.

## 검증

로컬 검토와 최종 CI에서 다음을 확인했다.

- `CARGO_INCREMENTAL=0 cargo test renderer::canvaskit_policy::tests --lib`: 19 passed
- `npm test`: 185 passed (원 contributor head 검토 시점)
- `npm run e2e:renderer-contract`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과 (원 contributor head 검토 시점)
- `npm run build`: 통과 (메인터너 TypeScript 보완 후 재확인)
- CanvasKit 실번들 paragraph probe: 통과
- `git diff --check`: 통과
- GitHub Actions: CI, CodeQL, Render Diff 모두 성공. CI의 default-feature tests는 format, build,
  default-feature tests, WASM target, clippy까지 통과했고 Canvas visual diff도 성공했다.

메인터너 보완은 `rhwp-studio/**` TypeScript와 계약 테스트만 변경한다. Rust/WASM 소스는 추가로
변경하지 않아 전체 cargo test/clippy는 이번 보완 커밋 후에는 재실행하지 않았다.

## 리스크 및 잔여

- paragraph shaping은 기본 Noto Sans KR 단일 `FontMgr`를 사용한다. 문서별 font family 매핑과 fallback
  체인은 #536의 후속 font fidelity 범위다.
- `scriptTextRequiresShaping` 정책 정보는 Rust replay plan에 계속 보존된다. Studio renderer는 이제
  해당 text를 실질적으로 shape할 수 있을 때 직접 처리하고, `FontMgr`가 없을 때만 unsupported diagnostic을 남긴다.
- `ParagraphBuilder` 경로는 producer의 개별 advance를 그대로 재현하지 않는다. 개별 advance 보존이 검증된
  printable ASCII만 direct glyph replay로 남긴 이유다.

## 후속 처리

- #536은 font fidelity 추적 이슈로 open 유지한다. #2191에는 closing keyword가 없어 별도 이슈 close는 없다.
- 메인터너 보완 사유와 검증은 [PR 코멘트](https://github.com/edwardkim/rhwp/pull/2191#issuecomment-4942793427)에
  기록했다.
- 이 문서, 구현 계획서, 오늘할일은 코드 PR merge 후 옵션 2 docs-only fast-pass PR로 보존한다.

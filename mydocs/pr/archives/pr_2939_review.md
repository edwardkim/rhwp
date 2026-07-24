# PR #2939 리뷰 — feat(render): replay positioned CanvasKit text visuals

## 1. PR 메타

| 항목 | 값 |
|---|---|
| PR | [#2939](https://github.com/edwardkim/rhwp/pull/2939) |
| 작성자 | `seo-rii` (외부 contributor, `author_association=CONTRIBUTOR`) |
| base | `devel` |
| head branch | `render-p39` (`seo-rii/rhwp` fork, `maintainer_can_modify=true`) |
| 규모 | +2,815 / −169, 19 files, 4 commits |
| 라벨 | `rendering` |
| 연관 이슈 | `Refs #536` (멀티 렌더러 트래킹 이슈 — P39 단계, close 대상 아님) |
| merge commit | `1ddfc7da12a63d8ffc9df42e6952a30db25f5e32` |
| 최종 head | `aa017fee` (update branch로 BEHIND 해소 후) |
| reviewer | `postmelee` (approval 완료) |
| 처리 경로 | §9 collaborator-mediated 외부 PR (reviewer 권한 `write`) |

## 2. 관련 이슈 요약

이슈 #536은 렌더러 백엔드 계약을 phase 단위로 추적하는 트래킹 이슈다. 이 PR은 **P39**로, P38(문서 단위
auto 판정 + fail-closed 경계) 위에서 다음을 CanvasKit direct replay 범위에 추가한다.

- 위치가 확정된 수평 특수 시각 연산: `CharOverlap`(글자 겹침), `TabLeader`(탭 리더),
  `TextDecoration`(밑줄/취소선/강조점), 공백·탭·문단 끝·줄바꿈 부호(`TextControlMark`).
- run당 문서 글꼴 → 기본 face → old-Hangul subset 글리프별 fallback span.
- 한컴 박스 숫자 PUA 결정적 fallback.

P39가 마지막 phase가 아니므로 이슈는 open 유지가 맞고, PR도 `Refs #536`으로 올바르게 참조한다.

## 3. 변경 범위 분석

### 핵심 기능 (Rust producer/policy)
- `src/paint/schema.rs`: layer schema minor `18 → 19`.
- `src/paint/mod.rs`: `MAX_POSITIONED_CONTROL_MARKS_PER_RUN = 4096` 상한 도입.
- `src/paint/builder.rs`: 글자 겹침 run은 일반 decoration/tab-leader paint를 우회(legacy 미러 방지);
  control-mark 판정을 공백·탭 포함으로 확장하고 field_marker 트리거 제거(빈 op 방지).
- `src/paint/json.rs` (+436/−19): bounded 위치 배열, `*Complete` 플래그, `text.*.bounded` /
  `text.controlMarkOp.positioned` used-feature, script 보정 baseline/fontSize, PUA 확장 위치,
  탭 리더 endpoint clamp 직렬화.
- `src/renderer/canvaskit_policy.rs` (+380/−34): 4개 특수 시각 연산을 overlay→direct로 승격,
  vertical/rotated/malformed/over-limit/invalid는 fail-closed. work-unit 계산에 반복 시각 항목 수
  반영. old-Hangul PUA projection에 shaping 폰트 required로 추가.

### 핵심 기능 (Studio/CanvasKit 런타임)
- `rhwp-studio/src/view/canvaskit-renderer.ts` (+857/−23): `renderCharOverlap`/`renderTextControlMark`/
  `renderTabLeader`/`renderTextDecoration` 구현, 글리프별 fallback span(≤4096), 박스 숫자 PUA,
  old-Hangul shaping 폰트/기호 fallback 폰트(D2Coding) 로딩, 삼중 검증(complete/geometry/limit).
- `rhwp-studio/src/main.ts`: `showParagraphMarks`를 auto blocker에서 제거(`showControlCodes`만 유지).
- `rhwp-studio/src/core/types.ts`: layer op 타입 optional→required 정밀화.
- `rhwp-studio/src/view/canvaskit/diagnostics.ts`: 승격된 op를 expected-unsupported 목록에서 제거.

### 메타/검증 변경
- `scripts/renderer_baseline_manifest.json`: readiness-gate 샘플 2건 추가
  (`paragraph-text-marks` = lseg-05-tab.hwp, `pua-special-glyphs` = pua-test.hwp — 둘 다 저장소에 존재).
- `rhwp-studio/e2e/renderer-baseline.mjs`: sample별 view option 적용/리셋 하니스.
- `rhwp-studio/e2e/canvaskit-font-coverage.test.mjs`: D2Coding 기호 커버리지 검증.
- README, `docs/canvaskit-parity-implementation.md`, `docs/text-ir-v2.md` 갱신.

### 범위 외
공개 기본 renderer Canvas2D 유지, `renderer=auto` opt-in, `showControlCodes` 구조 조판부호 계속 Canvas2D
blocker, 세로쓰기/회전/장평/효과 특수 텍스트 미개방, native Skia/direct PDF/SVG 기본 동작 불변.

## 4. 렌더 영향 · visual sweep 판정

renderer/paint 출력 경로가 바뀌므로 시각 검증 대상(§2.6). 성격은 CanvasKit ↔ Canvas2D 픽셀 근접이며,
판정 기준은 두 backend ink-mask 근접 — CI render-diff readiness gate가 실제 샘플로 강제한다.

- PR 첨부: `paragraph-text-marks` `0.00645915` < 0.02, `pua-special-glyphs` `0.00373927` < 0.01.
- CI `Canvas visual diff` + render-diff readiness gate(하드 게이트, lseg-05-tab / pua-test 포함) pass.
- 원본 HWP(`samples/lseg-05-tab.hwp`, `samples/pua-test.hwp`)가 저장소에 존재해 장기 재현 가능.
- 독립 재현 공백 없음. 본 리뷰어의 로컬 브라우저 ink-mask 재실행은 하지 않았고 CI headless readiness gate에 근거.

## 5. 로컬 사전 검증 (upstream/devel 위 merge 시뮬레이션)

충돌 0건 (자동 병합 성공).

| 검증 | 결과 |
|---|---|
| `git diff --check` / `cargo fmt --check` | OK |
| `cargo clippy --all-targets -- -D warnings` | 0 warning |
| `cargo test --profile release-test --lib paint::` | 89 passed |
| `cargo test --profile release-test --lib canvaskit` | 58 passed |
| `cargo test --test svg_snapshot` | 8 passed (결정성 확인) |
| `cargo test --profile release-test --tests` | 실패 0 |
| `npx tsc --noEmit` (rhwp-studio) | exit 0 |
| `npm test` (rhwp-studio) | 539 passed |
| `npm run e2e:renderer-contract` | passed |

최종 head `aa017fee` 기준 GitHub Actions 21건 pass, `WASM Build` 조건부 skip.

## 6. 설계 평가 · 리스크

**강점**
- producer(json.rs) / preflight(canvaskit_policy.rs) / runtime(canvaskit-renderer.ts) 3계층에 동일한
  4096 상한과 completeness 규칙으로, 잘림·불완전·비정상 payload가 direct replay로 수렴하지 않고 일관되게
  fail-closed. 미해결 글리프는 `textRun:glyphMapping` unexpected diagnostic으로 남음.
- 글자 겹침 run의 legacy decoration/leader 우회 + `legacyVisuals:"mirror"`로 이중 paint 구조적 차단
  (Canvas2D `render_text_run` 동작과 정합, 테스트로 잠김).
- fallback span 상한(≤4096)으로 교차 글리프 입력의 무한 font probe/draw call 방지.

**리스크 / 참고 (blocker 아님)**
- `paint_op_work_units`에 글자 수 기반 비용이 추가되어, 텍스트가 매우 많은 문서는 preflight work 상한
  (50,000)에 더 일찍 도달해 Canvas2D로 수렴할 수 있음 → 의도된 bounding, 안전한 기본값으로의 보수적 수렴.
- 탭 리더/decoration clamp·metrics를 CanvasKit은 script 보정값+PUA 확장 위치, Canvas2D는 raw로 계산 →
  superscript+탭리더 같은 드문 조합에서 미세 좌표차 가능. 대표 샘플 ink-mask 게이트가 커버.
- explicit `renderer=canvaskit` + `showControlCodes`는 구조 조판부호 미표시(문서화된 한계). auto에서는
  blocker로 Canvas2D 고정 → 사용자-visible 회귀 없음.
- Copilot 자동 리뷰는 quota 초과로 미수행(무시 가능).

## 7. 처리 결과

- approval review + `--merge` 로 merge 완료. merge commit `1ddfc7da12a63d8ffc9df42e6952a30db25f5e32`.
- 이슈 #536: open 유지(P39는 트래킹의 한 단계), P39 완료 + 남은 follow-up 코멘트 기록.
- @seo-rii 감사 코멘트 게시.
- 남은 follow-up(모두 #536 트래킹 범위): charOverlap/tabLeader/decoration의 실제 HWP 기반 focused
  visual fixture, 세로쓰기·회전·장평·효과 특수 텍스트, `showControlCodes` 구조 조판부호.

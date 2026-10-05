---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7504 리뷰 — 수정: #7503 커닝한 글자를 커닝 전 폭으로 그리고, 셀 캐럿과 입력 비용을 바로잡는다

## 최종 판정

**머지 보류 — 누적 후보 검증 진행 중.** 원 head의 CI와 이번 누적 head의 실행 결과를 구분한다. 필수 코드·회귀·시각 검증 결과를 확인한 뒤 판정을 갱신한다.

## 접수 정보

- 원 PR: [#7504](https://github.com/edwardkim/rhwp/pull/7504), semanticist21, devel 대상, non-draft.
- 원 head: `7dc340284bd84e2ee475da3b577146005b989500`; 접수 시점 `MERGEABLE` / `CLEAN`. 원 head의 상태이며 누적 후보 판정이 아니다.
- 누적 branch: `review/semanticist21-20261005`; 고정 base `cdba77b609c399fdef26a6c9e637716aa32c2177`; 누적 code candidate `1d809afe7b965c9ea6137012d59d139d63b038d0`.
- Reviewer: jangster77 지정. 기본 maintainer_general; intake_and_review, local_validation, multi_pr_update_branch, 렌더 영향 시 visual_fixture_evidence를 적용.
- 관련 이슈: #7503.
- 사용자 지시: non-draft 19건을 번호 순으로 누적 체리픽; 충돌은 메인터너 보정; 원 PR별 리뷰 기록을 개별 작성.

## 적용 이력

| 원 commit SHA | 상태 | 로컬 적용 SHA | 메인터너 보정 |
| --- | --- | --- | --- |
| `38ce705edd33c656faf7f5831a19ed8da3864370` | applied | `993f4bc14af375c8abd59d902ace954c775d39d8` | — |
| `8345ad79ab4334198ae195b417ef53c09bd09e54` | applied | `00c42120cc779e9f2855346ec5c3593cdccb763f` | — |
| `71fd3348ffb2125176cbf9ac8b7e38243f6cc285` | applied | `bcc5efc1142f65d9d712cf31654d803561b2e4d2` | — |
| `e9f38a16b24c59151c2f721a02172f9329b86675` | applied | `279607f753e44d9c0cc7fae12540d1a279f02558` | — |
| `7dc340284bd84e2ee475da3b577146005b989500` | applied | `d66a26cdcb09a58b9b341cd8f0b5b8166519749b` | — |

원 저자와 `cherry-pick -x` 출처를 보존했다. 이미 patch-id가 같은 원 commit은 중복 적용하지 않았다. 원 contributor branch는 수정하지 않았다.

## 변경·소비 경로 검토

- `src/document_core/commands/text_editing.rs`
- `src/renderer/kerning.rs`
- `src/renderer/mod.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`

ExactFontSourceRegistry의 불변 Arc와 등록 hash → kerning source session → layout 위치. SVG textLength와 Canvas scaleX는 glyph_fit_positions의 커닝 전 advance를 소비하고 글자 원점은 커닝 후 위치를 사용한다. 등록 글꼴 셀의 입력 직후 캐럿은 exact query로 폴백한다.

등록 TTF의 AV/To 독립 메트릭과 실제 FontFace 등록 출력이 필요하다. 등록하지 않은 CLI 렌더로 이 경로를 대체할 수 없다. 합성 입력의 계약 결과를 한컴 출력과의 일치 증거로 바꾸지 않는다.

## 검증 입력·결과

- `tests/cases/issue_7503_kerning_glyph_caret_cost.rs` (5개 테스트): 누적 head 실행 5 PASS / 0 FAIL

- 로컬 Cargo는 공유 `target/pr-review`에서 순차 실행한다. 같은 원 head의 CI를 19건 누적 후보의 전체 검증으로 재사용하지 않는다.
- 필수 fmt·Native/WASM/workspace-all-targets Clippy·workspace build·manifest/base 정책·source unit tier 검사: PASS. 전체 Rust·Native Skia·fresh WASM 및 직접 시각 검증은 진행 중.
- focused: `output/pr-review/semanticist21-20261005/run-records/focused-command.json`의 20 case 필터, 전체 74건 중 73 PASS / 1 FAIL. PR별 결과는 위 case 항목에서 구분한다. 실행 증거: `output/pr-review/semanticist21-20261005/logs/focused.log`.
- 실제 HWP/HWPX/PDF 입력과 commit의 해시, 독립 기준, source/build provenance: 입력 사용 시 기록한다.
- source 교정 또는 검사 실패가 생기면 이 PR의 보정과 재실행을 별도로 기록한다. Golden/baseline/래칫을 완화하지 않는다.

## 원 head CI 참고값

- Lint (fmt, clippy, WASM check): SUCCESS
- Build & Test: SUCCESS
- CI Impact Policy: SUCCESS

## 조판·시각 판정

적용 여부와 필요한 직접 증거를 확인 중이다. 원 PR 제공 before/after·수치를 누적 head의 Visual Sweep 통과로 간주하지 않는다. 자료 부족과 실제 회귀를 구분하여 미검증/미충족으로 판정한다.

## 남은 범위·후속 처리

원 PR 전체 해결 여부와 이슈 종료 표현은 직접 검증한 범위로 제한한다. 이 기록은 로컬 누적 검토이며 원격 approve/comment/merge를 의미하지 않는다. 통합 결과는 같은 누적 branch에 두고 원 PR별 판정이 확정된 뒤 게시 범위를 결정한다.

## fresh WASM 실제 등록 글꼴/Canvas 경로

fresh web package `rhwp_bg.wasm` SHA-256 `5c66e27f13dc1699a18aabcc1397c530e1bec05f2567b5a0414e9dabe714dd06`를 Chromium에서 실행했다. 새 문서 26pt `AVTo`, 저장소 TTF를 실제 FontFace로 올리고 charShapeId의 영문 slot 1에 registerExactFontSource를 호출했다. 2x Canvas fillText의 A/V/T/o 가로 배율은 각각 등록 전후 동일하다. V 원점 차이는 `-5.546661px / 2 = -2.77333px`, o는 누적 `-8.319977px / 2 = -4.15999px`로 TTF의 AV -80·To -40 / 1000 em, 26pt 기대값과 일치한다. 글리프 폭을 압축하지 않고 원점만 커닝하는 계약을 실제 renderer 호출에서 확인했다. 실행 로그 `output/pr-review/semanticist21-20261005/logs/kerning-browser.log`·[좌표/transform 원문](../assets/semanticist21-20261005/kerning-results.json). 동일 글꼴 조건의 한컴 PDF는 확보하지 않았으며 한컴 fidelity 판정과 구분한다.

## 최종 공통 회귀 결과 (폼 source cf2336295)

Rust source `cf2336295540ea8ce3e94eb6517cb406fca8d28f`, 정책 base `cdba77b609c399fdef26a6c9e637716aa32c2177`에서 fmt·Clippy Native/WASM/workspace-all-targets·workspace build·manifest/unit tier 정책 PASS. 전체 nextest10,437건 중10,436 PASS/1 FAIL/50 SKIP이며 실패는 #7491의 편집 뒤 표 우변 assertion1건이다. 이 실패는 고정 base에서도 관측했다. Native Skia lib·missing picture2개·direct PDF4개·ComboBox4개·암호4개는 모두 PASS다. 명령/exit/시간은 검증 정본 (`output/pr-review/semanticist21-20261005/run-records/appearance-final-validation.json`), 요약과 원 로그 SHA는 실행 요약 (`output/pr-review/semanticist21-20261005/run-records/appearance-final-validation-summary.txt`)에 보존했다.

#7491은 사용자가 지정한 실패 입력에서 MCP 재산출 PDF·90% 시각 gate와 독립 기대값을 추가 검증 중이며, #7521의 loose inline 길이 제한 우회도 보류 사유로 남는다. 전체 회귀 통과 또는 통합 merge를 선언하지 않는다. 이후 Rust source/test 변경에는 이 결과를 그대로 승계하지 않고 해당 검증을 다시 수행한다.

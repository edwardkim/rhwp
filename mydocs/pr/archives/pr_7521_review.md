---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7521 리뷰 — 수정: #7516 HTML 붙여넣기가 <p> 밖 인라인 서식을 한 문단으로 살린다

## 최종 판정

**머지 보류 — 긴 HTML 입력 제한의 신규 회귀 확인.** 원 head의 CI와 이번 누적 head의 실행 결과를 구분한다. 필수 코드·회귀·시각 검증 결과를 확인한 뒤 판정을 갱신한다.

## 접수 정보

- 원 PR: [#7521](https://github.com/edwardkim/rhwp/pull/7521), semanticist21, devel 대상, non-draft.
- 원 head: `6f0df1ae38d6830128916c4e5f773da23a6be804`; 접수 시점 `MERGEABLE` / `CLEAN`. 원 head의 상태이며 누적 후보 판정이 아니다.
- 누적 branch: `review/semanticist21-20261005`; 고정 base `cdba77b609c399fdef26a6c9e637716aa32c2177`; 누적 code candidate `1d809afe7b965c9ea6137012d59d139d63b038d0`.
- Reviewer: jangster77 지정. 기본 maintainer_general; intake_and_review, local_validation, multi_pr_update_branch, 렌더 영향 시 visual_fixture_evidence를 적용.
- 관련 이슈: #7516.
- 사용자 지시: non-draft 19건을 번호 순으로 누적 체리픽; 충돌은 메인터너 보정; 원 PR별 리뷰 기록을 개별 작성.

## 적용 이력

| 원 commit SHA | 상태 | 로컬 적용 SHA | 메인터너 보정 |
| --- | --- | --- | --- |
| `37f754d514fa944d428fac5a1754eb07ed49d0f5` | applied | `381c4b1b5cda885f665b864db342570b028df27e` | — |
| `6f0df1ae38d6830128916c4e5f773da23a6be804` | applied | `a0b0989ef5a026b8381acbe6e6921b73eceaf18b` | — |

원 저자와 `cherry-pick -x` 출처를 보존했다. 이미 patch-id가 같은 원 commit은 중복 적용하지 않았다. 원 contributor branch는 수정하지 않았다.

## 변경·소비 경로 검토

- `src/document_core/commands/html_import.rs`

loose inline을 flush_inline_run에 모아 parse_inline_content의 style stack으로 읽는다. 안쪽 weight가 우선하며 br 뒤에는 열린 서식을 다시 연다. span 내부의 그림도 실제 parse 경로로 소비한다.

새 loose b/i 경로가 기존 4000자 강제 절단을 우회하는지 경계 검증이 필요하다. 기존 source 테스트 6개에는 긴 입력 경계가 없다. 합성 입력의 계약 결과를 한컴 출력과의 일치 증거로 바꾸지 않는다.

## 검증 입력·결과

- `tests/cases/issue_7516_html_paste_loose_inline_format.rs` (6개 테스트): 누적 head 실행 6 PASS / 0 FAIL

- 로컬 Cargo는 공유 `target/pr-review`에서 순차 실행한다. 같은 원 head의 CI를 19건 누적 후보의 전체 검증으로 재사용하지 않는다.
- 필수 fmt·Native/WASM/workspace-all-targets Clippy·workspace build·manifest/base 정책·source unit tier 검사: PASS. 전체 Rust·Native Skia·fresh WASM 및 직접 시각 검증은 진행 중.
- focused: `focused-command.json`의 20 case 필터, 전체 74건 중 73 PASS / 1 FAIL. PR별 결과는 위 case 항목에서 구분한다. 실행 증거: `output/pr-review/semanticist21-20261005/logs/focused.log`.
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

## 신규 회귀: 긴 loose inline 입력의 제한 우회

public API `pasteHtml(0,0,0,...)`로 ASCII `x` 8,001자를 실행했다. 최신 base의 Native 결과는 plain과 `<b>...</b>` 모두 문단 길이 `[4000,4000,1]`; 누적 fresh WASM은 plain `[4000,4000,1]`, bold `[8001]`이다. 새 `flush_inline_run`의 태그 포함 분기가 `FLUSH_LINE_CHAR_CAP=4000`을 적용하지 않고 parse_inline_content 한 문단을 발행한다. 기존 source 주석이 길이 제한을 명시하고 있으며 이 PR의 6개 회귀에는 해당 경계가 없다. 문단 수·길이의 신규 계약 회귀를 검출한 것이며 실제 overlap/전체 브라우저 정지까지 실행했다고 주장하지 않는다.

[base 실행](../assets/semanticist21-20261005/base-probes.txt), [fresh WASM 관측](../assets/semanticist21-20261005/browser-observations.json), [관측값](../assets/semanticist21-20261005/browser-observations.json). Native base와 WASM 누적의 runtime 차이는 이후 같은 Native 경로에서도 확인하여 구분한다. source 회귀 추가와 스타일·그림·명시적 줄바꿈을 보존하는 제한 처리가 필요하다.

## 최종 공통 회귀 결과 (폼 source cf2336295)

Rust source `cf2336295540ea8ce3e94eb6517cb406fca8d28f`, 정책 base `cdba77b609c399fdef26a6c9e637716aa32c2177`에서 fmt·Clippy Native/WASM/workspace-all-targets·workspace build·manifest/unit tier 정책 PASS. 전체 nextest10,437건 중10,436 PASS/1 FAIL/50 SKIP이며 실패는 #7491의 편집 뒤 표 우변 assertion1건이다. 이 실패는 고정 base에서도 관측했다. Native Skia lib·missing picture2개·direct PDF4개·ComboBox4개·암호4개는 모두 PASS다. 명령/exit/시간은 [검증 정본](../assets/semanticist21-20261005/appearance-final-validation.json), 요약과 원 로그 SHA는 [실행 요약](../assets/semanticist21-20261005/appearance-final-validation-summary.txt)에 보존했다.

#7491은 사용자가 지정한 실패 입력에서 MCP 재산출 PDF·90% 시각 gate와 독립 기대값을 추가 검증 중이며, #7521의 loose inline 길이 제한 우회도 보류 사유로 남는다. 전체 회귀 통과 또는 통합 merge를 선언하지 않는다. 이후 Rust source/test 변경에는 이 결과를 그대로 승계하지 않고 해당 검증을 다시 수행한다.

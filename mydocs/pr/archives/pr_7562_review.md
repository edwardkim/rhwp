---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7562 리뷰 — fix(hwpx): 구역 첫 문단의 저장 전후 줄 나눔을 보존한다 (#7526)

## 최종 판정

**머지 보류 — 누적 후보 검증 진행 중.** 원 head의 CI와 이번 누적 head의 실행 결과를 구분한다. 필수 코드·회귀·시각 검증 결과를 확인한 뒤 판정을 갱신한다.

## 접수 정보

- 원 PR: [#7562](https://github.com/edwardkim/rhwp/pull/7562), semanticist21, devel 대상, non-draft.
- 원 head: `00fa31e4cc9438a2eaa583c0cded731723e468e3`; 접수 시점 `MERGEABLE` / `CLEAN`. 원 head의 상태이며 누적 후보 판정이 아니다.
- 누적 branch: `review/semanticist21-20261005`; 고정 base `cdba77b609c399fdef26a6c9e637716aa32c2177`; 누적 code candidate `1d809afe7b965c9ea6137012d59d139d63b038d0`.
- Reviewer: jangster77 지정. 기본 maintainer_general; intake_and_review, local_validation, multi_pr_update_branch, 렌더 영향 시 visual_fixture_evidence를 적용.
- 관련 이슈: #7526.
- 사용자 지시: non-draft 19건을 번호 순으로 누적 체리픽; 충돌은 메인터너 보정; 원 PR별 리뷰 기록을 개별 작성.

## 적용 이력

| 원 commit SHA | 상태 | 로컬 적용 SHA | 메인터너 보정 |
| --- | --- | --- | --- |
| `94622dcb53ce6e6e0c88ae82fff461055d2b3c90` | applied | `0bca1742aa0c0cb97d538e5d872273d223c10c93` | — |
| `8d3b51ed378692f47b37d79dd94dc58a1adaf758` | applied | `4ce4cb02f625ae5eb6d4bb379fecca9d60f39d8a` | — |
| `00fa31e4cc9438a2eaa583c0cded731723e468e3` | applied | `2093e2529545675d1535b3e09cbe0b6d9c6de929` | — |

원 저자와 `cherry-pick -x` 출처를 보존했다. 이미 patch-id가 같은 원 commit은 중복 적용하지 않았다. 원 contributor branch는 수정하지 않았다.

## 변경·소비 경로 검토

- `src/model/paragraph.rs`
- `src/parser/hwpx/section.rs`
- `src/serializer/hwpx/context.rs`
- `src/serializer/hwpx/section.rs`

rhwp origin marker와 secPr/colPr의 같은 run을 함께 확인해 hwpx_axis_shift를 정한다. SerializeContext는 origin marker 문서를 문단별 축으로 구분하고 stored shift가 있으면 재차 내리지 않는다. 편집 reflow는 HWP5 축으로 내려서 저장한다.

순수 Hancom HWPX는 기존 계약을 유지한다. HWP5/HWP3 origin의 첫 문단, stored 왕복과 편집 뒤 왕복을 구분하여 검사한다. 합성 입력의 계약 결과를 한컴 출력과의 일치 증거로 바꾸지 않는다.

## 검증 입력·결과

- `tests/cases/issue_7526_hwpx_first_para_textpos_axis.rs` (4개 테스트): 누적 head 실행 4 PASS / 0 FAIL

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

## 최종 공통 회귀 결과 (폼 source cf2336295)

Rust source `cf2336295540ea8ce3e94eb6517cb406fca8d28f`, 정책 base `cdba77b609c399fdef26a6c9e637716aa32c2177`에서 fmt·Clippy Native/WASM/workspace-all-targets·workspace build·manifest/unit tier 정책 PASS. 전체 nextest10,437건 중10,436 PASS/1 FAIL/50 SKIP이며 실패는 #7491의 편집 뒤 표 우변 assertion1건이다. 이 실패는 고정 base에서도 관측했다. Native Skia lib·missing picture2개·direct PDF4개·ComboBox4개·암호4개는 모두 PASS다. 명령/exit/시간은 검증 정본 (`output/pr-review/semanticist21-20261005/run-records/appearance-final-validation.json`), 요약과 원 로그 SHA는 실행 요약 (`output/pr-review/semanticist21-20261005/run-records/appearance-final-validation-summary.txt`)에 보존했다.

#7491은 사용자가 지정한 실패 입력에서 MCP 재산출 PDF·90% 시각 gate와 독립 기대값을 추가 검증 중이며, #7521의 loose inline 길이 제한 우회도 보류 사유로 남는다. 전체 회귀 통과 또는 통합 merge를 선언하지 않는다. 이후 Rust source/test 변경에는 이 결과를 그대로 승계하지 않고 해당 검증을 다시 수행한다.

## upstream/devel 위 rebase 적용 위치 — 2026-10-05

기준 `c167dc6abbebf69546575e2d16d06223791bab82`. 아래는 현재 이력의 실제 적용 위치이며 위의 이전 검증 SHA는 당시 이력으로 보존한다.

| 원 commit SHA | rebase 전 로컬 SHA | 현재 적용 SHA | 상태 |
| --- | --- | --- | --- |
| `94622dcb53ce6e6e0c88ae82fff461055d2b3c90` | `0bca1742aa0c0cb97d538e5d872273d223c10c93` | `082bdb354c72e5ba3c032a1061a687045b45f40f` | rebased |
| `8d3b51ed378692f47b37d79dd94dc58a1adaf758` | `4ce4cb02f625ae5eb6d4bb379fecca9d60f39d8a` | `0e45bbacdfa180853855d74d7c5ce72aade5fc58` | rebased |
| `00fa31e4cc9438a2eaa583c0cded731723e468e3` | `2093e2529545675d1535b3e09cbe0b6d9c6de929` | `90730a209638301ac3c061772a4371db1cf88de7` | rebased |

원 저자와 cherry-pick 출처를 유지했다. #7491의 원4개는 #7599를 통해 이미 base에 포함되어 중복 적용하지 않았다. 메인터너 보정과 개별 리뷰 기록은 재배치했다. 최종 후보의 시각·전체 회귀 및 CI는 별도 확인한다.

## rebase 후 MCP 직접 출력 검토 — 2026-10-05

공개 API의 원 회귀 입력을 각각 저장해 Windows MCP engine2020의 Print(method0, one-up, 한컴11.0.0.9136)로 대조했다. 첫 문단 긴 한글112자 HWPX는 MCP job `60a3190e-9586-4812-9761-30e1d4d88cdc`에서 정상 출력됐으나 Native 최저81.98786%로 `re_review_required`다. 대표 review PNG를 직접 열어 첫 줄의 글자 분배와 줄 소속 차이를 확인했다. 같은 단어 나누기 HWP 대조군은 job `0e81e5dd-c5cd-4331-afd6-270a3179a6da`, Native100%다. HWPX 단어 나누기 입력은 job `88205a33-fcb9-4617-b29b-fe1b54b4f2c9`에서 손상/변경 경고로 무인 열기가 거절됐다. 기존4개 왕복 검사가 통과해도 독립 한컴 출력과의 일치를 입증하지 못한다. 메인터너 보정과 재실행 전 보류를 유지한다.

- 입력·PDF·TSV·PNG는 ignored `output/pr-review/semanticist21-20261005/{integration-visual-inputs,integration-mcp-pdf,diagnostics-native-scores,diagnostics-native-review}`에 실패 진단으로 보존한다. 수용용 fixture/golden으로 추가하지 않았다.
- CLI 편집 출력은 확장자를 `.hwpx`로 지정해도 원 형식 HWP5를 유지한다. 그 잘못된 진단 파일은 기준 자료에서 제외했고 공개 API `export_hwpx_native`의 실제 ZIP 입력으로 재실행했다.

## 메인터너 보정 계획: 독립 Print의 문단 축 계약

한컴2020이 동일 HWP를 HWPX로 저장한 독립 대조본은 첫 run의 `secPr`·`ctrl/colPr`를 포함한 문단 축에서 줄 시작 `0,59,102`를 쓴다. rhwp 산출은 같은 XML 슬롯을 갖고 `0,43,86`을 썼고, 자체 읽기에서 16을 보상해 내부 검사만 통과했다. 같은 잘못된 HWPX의2024 진단 Print도 첫 줄27자로 갈려 제품 버전 차이가 원인이 아님을 확인했다.

보정 범위는 HWP5 계보 산출물의 실제 슬롯 축이다. 첫 run 또는 별도 secPr run으로 옮겨 쓴 정의를 삭제 슬롯으로 세지 않는다. 템플릿이 원본에 없는 정의를 추가할 때는 뒤 줄 시작도 그 슬롯만큼 이동한다. HWPX를 읽으며 올린 source 축은 `Paragraph.line_seg_text_start_of`로 같은 문단 축에 정규화해 저장한다. 새 생산자 계약은 기존 HWP5 origin 엔트리에 `2:paragraph-utf16`으로 명시하고, 읽기에서 이전 `1`의 보상은 호환 경로로 남긴다. 순수 외부 HWPX와 직접 HWP3의 기존 생산자 계약을 이 값으로 추정하지 않는다. 이 메타데이터는 rhwp의 읽기 계약이며 한컴 Print는 실제 XML/textpos로 독립 검증한다.

소비 경로: `hwpx_document_for_export`의 사본 마커 → `SerializeContext.line_segs_on_paragraph_axis` → `render_control_slot_tracked`의 실제 슬롯 보존/`render_paragraph_parts`의 줄 시작 매핑 → section XML → parser의 생산자 계약과 `hwpx_axis_shift` → `line_seg_text_start_of` → 줄 구성·측정·배치. 독립 Print와 Native/fresh WASM을 먼저 확인한 뒤 새 회귀를 추가한다. 기존4개 검사, 이전 origin 산출물 재저장, template의 정의 추가, 직접 HWP3 대조 경계를 검증하며 미실행 경로는 별도로 남긴다.

## 보정 1차 검증 — 2026-10-06

기존4개 nextest PASS. fmt·Native/WASM/workspace-all-targets Clippy·workspace build·base 고정 manifest·source unit tier의 전체 lint 묶음 PASS(`logs/axis-fix-lint-*`). 새 public API 저장본5개는 모두 한컴2020 Print(method0/one-up) 성공이며 Native 전쪽 최저는 긴 문단·어절 나누기·재편집·이전 `1` 산출 재저장100%, 직접 HWP3 대조93.40356%다. 긴 문단 대표 review/overlay를 직접 판독해 줄 분배·위치가 맞음을 확인했다. fresh WASM 재빌드·대조와 최종 전체 회귀는 진행 중이다. 원 로그/TSV/실행 JSON은 ignored output에 보존하고 새 fixture/golden이나 시각 회귀는 이 단계에서 추가하지 않았다.

## 전쪽 선행 시각 검증과 정식 회귀 — 2026-10-06

production source `693b63b26`의 Native/fresh WASM 24개 입력·28쪽을 같은 Print PDF로 재출력했다. 누락 쪽 없이 두 경로 모두 최저93.40356%다. 해당 PR의 상세 입력은 아래에 고정한다. raw TSV·실행 JSON은 ignored `output/pr-review/semanticist21-20261005/final-693-{native,wasm}-scores/`에 보존했다. 이 수치는 2px 이웃 관용 내용 실루엣이며 엄격 픽셀 동일률과 구분한다. 최종 전체 회귀·lint·CI 및 개별 직접 판독은 완료하지 않았다.

| 입력 | 입력 SHA-256 | Print PDF | PDF SHA-256 | MCP job |
| --- | --- | --- | --- | --- |
| `mydocs/pr/assets/semanticist21-20261005/pr7562/pr7562-axis-fixed-long.hwpx` | `1dd7b2ad77fefa5a6e9bc4dce6b363aad851f13aa20db0857279dc792390b1bc` | `pdf/semanticist21-20261005/pr7562/mcp/pr7562-axis-fixed-long-hwpx-2020.pdf` | `ccf2502a65d975ec6b21e7fcb6cc1e6b73144ec8ce3c0a67fcf17f3fae58e19f` | `b1e4fa01-2394-48e5-8e59-46029e9c7c39` |
| `mydocs/pr/assets/semanticist21-20261005/pr7562/pr7562-axis-fixed-word.hwpx` | `50df6ed04de992ae00fa8278119cc4f6bc5eba48629c7f8e629eaaf891621115` | `pdf/semanticist21-20261005/pr7562/mcp/pr7562-axis-fixed-word-hwpx-2020.pdf` | `20ee6d6f8b5f376c3568201d46cbdc9837ae776e98ae606fad04b5e24b24cf0e` | `b4b28fab-87b2-46fe-9a5a-6e59159c98d1` |
| `mydocs/pr/assets/semanticist21-20261005/pr7562/pr7562-axis-fixed-edited.hwpx` | `1dd7b2ad77fefa5a6e9bc4dce6b363aad851f13aa20db0857279dc792390b1bc` | `pdf/semanticist21-20261005/pr7562/mcp/pr7562-axis-fixed-edited-hwpx-2020.pdf` | `bc03790bc442a0f8f4d6ca02829c981cb80dc35e9090ada42f792ddceb8085bc` | `4d46285d-596d-4e05-80b7-8e964b1db1d3` |
| `mydocs/pr/assets/semanticist21-20261005/pr7562/pr7562-axis-fixed-legacy-reexport.hwpx` | `1dd7b2ad77fefa5a6e9bc4dce6b363aad851f13aa20db0857279dc792390b1bc` | `pdf/semanticist21-20261005/pr7562/mcp/pr7562-axis-fixed-legacy-reexport-hwpx-2020.pdf` | `898847aee527f3c81eccf4621994e6cc59f1c109d8715436476c818edaa6705b` | `af4857c7-25b0-4ee3-8684-cc7a9115d024` |
| `mydocs/pr/assets/semanticist21-20261005/pr7562/pr7562-axis-control-hwp3.hwpx` | `3c00427b9028f57e4d8e6b13e6cf5e80cc7294d42888060ffb8d7d39ae53751d` | `pdf/semanticist21-20261005/pr7562/mcp/pr7562-axis-control-hwp3-hwpx-2020.pdf` | `7946bcefdef13d191b35c049350a0c4bf9a0b5f28288b5088559c5d3e9625b95` | `e6463863-be91-4227-af32-c269c5a6fd1c` |

- 긴 문단·어절 나누기·재편집·이전 계보 재저장100%, 직접 HWP3 대조93.40356%. 자체 왕복 기존4개와 실제 공개 ZIP textpos의 독립 축 검사1개, **nextest5 PASS**. 신규 검사 기대값 `0,59,102`는 동일 원문의 독립 한컴 저장·Print에서 정했다. 수정 전 생산자를 실제 실행한 FAIL 확인은 이어서 수행한다.

- 수정 전 실제 producer `d38c86d15`의 라이브러리를 별도 clean worktree에서 빌드해 같은 정식5개 검사를 연결했다. 자체 왕복4개는 PASS하고 신규 실제 XML 축 검사만 FAIL(`0,43,86` ≠ 독립 기준 `0,59,102`)했다. 보정 후5개 모두 PASS다. 빌드 실패가 아닌 의도한 직렬화 결함을 검출했고 baseline·기대값을 완화하지 않았다. 원 로그는 `logs/axis-before-producer-build.log`, `logs/axis-formal-before-result.log`, `logs/axis-new-publish-test-focused.log`.

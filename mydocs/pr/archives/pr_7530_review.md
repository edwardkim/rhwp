---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-06
---

# PR #7530 리뷰 — 수정: #7528 머리말 필드 마커를 저장 사본에서 자동 번호·파일 경로 필드로 바꾼다

## 최종 판정

**통합 후보 범위의 로컬 검토 충족 — 최신 원격 CI 대기.** 실제 production HWP/HWPX2쪽 저장·Print를 충족했고 전체 쪽수/파일 경로의 한컴 UI 갱신은 미검증으로 #7528을 유지한다. 원 head 자체의 approve/merge와 구분하며, 누적 source8569f49ce의 최신 원격 CI가 끝난 뒤 통합한다.

## 접수 정보

- 원 PR: [#7530](https://github.com/edwardkim/rhwp/pull/7530), semanticist21, devel 대상, non-draft.
- 원 head: `c3c0c070fa905c6aecbe7cffdc0c0dee18ab132f`; 접수 시점 `MERGEABLE` / `CLEAN`. 원 head의 상태이며 누적 후보 판정이 아니다.
- 누적 branch: `review/semanticist21-20261005`; 고정 base `cdba77b609c399fdef26a6c9e637716aa32c2177`; 누적 code candidate `1d809afe7b965c9ea6137012d59d139d63b038d0`.
- Reviewer: jangster77 지정. 기본 maintainer_general; intake_and_review, local_validation, multi_pr_update_branch, 렌더 영향 시 visual_fixture_evidence를 적용.
- 관련 이슈: #7528.
- 사용자 지시: non-draft 19건을 번호 순으로 누적 체리픽; 충돌은 메인터너 보정; 원 PR별 리뷰 기록을 개별 작성.

## 적용 이력

| 원 commit SHA | 상태 | 로컬 적용 SHA | 메인터너 보정 |
| --- | --- | --- | --- |
| `eac507104fef759c9c8292a3189906e4ba674c80` | applied | `4e255c46bebd5bfd691bbb974b8d12ef97c86ae2` | — |
| `dd775bd8703e7b43fc8c042296480ee5942c5a62` | applied | `bc4ef71be312c3fa6064dec4f4445a4754051207` | — |
| `c3c0c070fa905c6aecbe7cffdc0c0dee18ab132f` | applied | `cfd3c77f37fd5fe038179900c13714b97873916a` | — |

원 저자와 `cherry-pick -x` 출처를 보존했다. 이미 patch-id가 같은 원 commit은 중복 적용하지 않았다. 원 contributor branch는 수정하지 않았다.

## 변경·소비 경로 검토

- `src/document_core/commands/document.rs`
- `src/document_core/commands/header_footer_ops.rs`
- `src/model/paragraph.rs`
- `src/serializer/body_text.rs`

HWP snapshot 및 HWPX 저장 사본만 lower_header_footer_field_markers를 호출한다. 자동번호 8-unit 슬롯, 파일 이름 필드 begin/end와 UTF-16 위치 메타를 동시에 갱신한다. serialize_para_text는 FIELD_END를 다음 자동번호 placeholder보다 먼저 발행한다.

인접 필드·탭·한글 파일 이름·HWP/HWPX 왕복 및 live document 불변을 검사한다. 전체 쪽수와 파일 경로 필드의 한컴 UI 갱신은 별도 확인이 필요하다. 합성 입력의 계약 결과를 한컴 출력과의 일치 증거로 바꾸지 않는다.

## 검증 입력·결과

- `tests/cases/issue_7528_hf_field_controls.rs` (5개 테스트): 누적 head 실행 5 PASS / 0 FAIL

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
| `eac507104fef759c9c8292a3189906e4ba674c80` | `4e255c46bebd5bfd691bbb974b8d12ef97c86ae2` | `5607397ea0ac68fe7fce809018d180bbf5abba5f` | rebased |
| `dd775bd8703e7b43fc8c042296480ee5942c5a62` | `bc4ef71be312c3fa6064dec4f4445a4754051207` | `087f705079087870bb48df58bf4daedf97721f94` | rebased |
| `c3c0c070fa905c6aecbe7cffdc0c0dee18ab132f` | `cfd3c77f37fd5fe038179900c13714b97873916a` | `b7fca20aa48be36c8c3f5ef83e37262d99a3afec` | rebased |

원 저자와 cherry-pick 출처를 유지했다. #7491의 원4개는 #7599를 통해 이미 base에 포함되어 중복 적용하지 않았다. 메인터너 보정과 개별 리뷰 기록은 재배치했다. 최종 후보의 시각·전체 회귀 및 CI는 별도 확인한다.

## 실제 저장 API의 2쪽 필드와 독립 Print — 2026-10-05

`HwpDocument.export_hwp()`의 adapter snapshot → `lower_header_footer_field_markers` → HWP serializer, `HwpDocument.export_hwpx()`의 저장 사본 lowering → HWPX serializer를 각각 실행했다. 본문은 「첫 페이지」/「두 번째 페이지」, 가운데 머리말은 페이지 번호 필드다. Native/fresh WASM 모두 HWP와 HWPX의 전체2쪽에서 100%이며 빠진 쪽이 없다. 대표 전쪽 review/overlay를 직접 판독하고 PDF의 실제 1·2 페이지 번호와 본문 소속을 확인했다.

초기 진단 생성기의 `DocumentCore.export_hwp_native()`는 production adapter 경로를 우회해 필드가 빠졌다. 그 입력의 100% 결과는 필드 보존 증거에서 제외하고 위 실제 저장 경로로 재생성했다. 서로 다른 형식의 같은 파일명 PDF는 별도 경로에 받아 덮어쓰기를 피했다.

| 형식 | 입력 SHA-256 | MCP Print job | PDF SHA-256 | Native / fresh WASM 최저 |
| --- | --- | --- | --- | --- |
| HWP | `7177e63d01faa889980e6e0765871f9c608e1eced6ccc1b9f650255a2e540d61` | `cf91703b-9a93-4a66-b8a2-6dd63b8488b8` | `e6697af84ccd877f795153311c1ebe96a415183cebc99632d4c728caf76c4664` | 100% / 100% |
| HWPX | `90291528ed86cbd58c4a0c7d7d97a515959db0f9724bf39c237eb9c0a65f3592` | `ae441e21-48e9-4678-a64f-c4cf75ee1956` | `3f1ac3de35ec6a03821b9cd7ec01cd2723b8ea3e0c30532527512b8c3f4004c0` | 100% / 100% |

두 입력 모두 버전 bucket2020, 한컴11.0.0.9136, PrintMethod0/one-up 출력이다. 입력과 대표 PNG는 `mydocs/pr/assets/semanticist21-20261005/pr7530/`, 독립 PDF는 `pdf/semanticist21-20261005/pr7530/mcp/`에 고정했다. 생산 코드 source `85f3d021ab67328e4c8f5e77670125a2c3ab0fe8`; fresh WASM SHA-256 `51141da77d73e54dc6bfef4b16a1049f22905cd315441e9c743f53e57114f43b`. TSV와 실행 로그는 ignored `output/pr-review/semanticist21-20261005/hf-production-{native,wasm}-{scores,review}` 및 `logs/hf-*`에 보존한다. 합성 입력의 파일 경로 필드 갱신 등 원 PR의 나머지 경계는 기존5개 검사와 별도로 구분하고 최종 후보 전체 검증은 진행 중이다.

## 2026-10-06 최종 후보의 Print·Native/fresh WASM 재검증

정책 base `c167dc6abbebf69546575e2d16d06223791bab82`, production source `2b1f21ef1ab35a13ebcae11f562a3ebf3a998e4d`, 회귀 source `9af7586586587fa0aa617a9e57fd6acd0d4e3ba6`. 두 head 사이에는 #7527의 Native 전용 회귀와 리뷰/PNG만 추가됐고 production source는 동일하다. 최종 fresh WASM SHA-256 `24565cae976b3c6929c858f13c52785a4651a26dc61c0fd566f5a8f801d631f7`, JS `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`; root pkg/Studio public 해시를 대조했다.

| 검증 입력 | 출력 경로 | 전체 쪽별 실루엣(%) | Gate |
| --- | --- | --- | --- |
| `pr7530-production-save-two-pages-hwp` | native | p1 100.00000, p2 100.00000 | passed / 누락0 |
| `pr7530-production-save-two-pages-hwpx` | native | p1 100.00000, p2 100.00000 | passed / 누락0 |
| `pr7530-production-save-two-pages-hwp` | wasm | p1 100.00000, p2 100.00000 | passed / 누락0 |
| `pr7530-production-save-two-pages-hwpx` | wasm | p1 100.00000, p2 100.00000 | passed / 누락0 |

- [pr7530-production-save-two-pages.hwp](../../../mydocs/pr/assets/semanticist21-20261005/pr7530/pr7530-production-save-two-pages.hwp), SHA-256 `7177e63d01faa889980e6e0765871f9c608e1eced6ccc1b9f650255a2e540d61` → [독립 Print PDF](../../../pdf/semanticist21-20261005/pr7530/mcp/production-save-two-pages-hwp-2020.pdf), SHA-256 `e6697af84ccd877f795153311c1ebe96a415183cebc99632d4c728caf76c4664`.
- [pr7530-production-save-two-pages.hwpx](../../../mydocs/pr/assets/semanticist21-20261005/pr7530/pr7530-production-save-two-pages.hwpx), SHA-256 `90291528ed86cbd58c4a0c7d7d97a515959db0f9724bf39c237eb9c0a65f3592` → [독립 Print PDF](../../../pdf/semanticist21-20261005/pr7530/mcp/production-save-two-pages-hwpx-2020.pdf), SHA-256 `3f1ac3de35ec6a03821b9cd7ec01cd2723b8ea3e0c30532527512b8c3f4004c0`.

각 명령·TSV·manifest·runtime 원시는 ignored `output/pr-review/semanticist21-20261005`에 보존했다. 렌더는 같은 입력/Print 전체 페이지와 `--embed-fonts=full`을 사용했고 WASM은 `--wasm-pkg pkg`를 추가했다(#7504는 실제 등록 API replay adapter). 최종 전체 Rust 회귀와 GitHub CI는 별도 진행 중이다.

![fresh WASM 직접 비교](../../../mydocs/pr/assets/semanticist21-20261005/pr7530/hwp-wasm-review-all-pages.png)
![같은 쪽 standalone overlay](../../../mydocs/pr/assets/semanticist21-20261005/pr7530/hwp-wasm-overlay-all-pages.png)

최종 페이지별 TSV: `output/pr-review/semanticist21-20261005/final-tsv/native/<key>/silhouette.tsv` 및 `wasm/<key>/silhouette.tsv`. 최신 full Sweep PNG 쌍에서 canonical `--silhouette-only --png-pair`로 산출하고 PNG SHA를 manifest에 고정했다. 해당 입력 전체 쪽수도 독립 PDF·원문 exporter에서 별도로 대조했으며 90% 미만/누락0이다.

## Merge 후 contributor PR comment 계획

원 기여에 감사한 뒤 실제 통합 PR 링크·merge SHA·정확한 최종 head CI와 이 PR의 회귀 실행 결과를 한국어 존댓말로 게시한다. 원 head는 merge 직전에 다시 확인하고 동일할 때만 통합으로 대체된 원 PR을 닫는다. 원 contributor fork branch는 삭제하지 않는다.

이 PR의 원시 주소/저장/조회 계약과 누적 보정 범위를 구분한다. 합성 계약 검사를 한컴 조판 일치로 표현하지 않는다.

- 실제 비교 `pr7530-production-save-two-pages-hwp`의 p1 100.00000%, p2 100.00000%를 페이지별 실루엣 보조값으로 적는다. 같은 입력 Native/fresh WASM 전체 쪽 TSV·누락0·직접 구조 판정을 함께 설명한다.
- merge SHA에서 존재를 확인한 `mydocs/pr/assets/semanticist21-20261005/pr7530/hwp-wasm-review-all-pages.png` / `mydocs/pr/assets/semanticist21-20261005/pr7530/hwp-wasm-overlay-all-pages.png`를 `raw.githubusercontent.com/edwardkim/rhwp/<merge-SHA>/...`의 실제 Markdown 이미지로 표시한다. 임시 output 링크로 대신하지 않는다.
- 이슈는 확인된 해결 범위만 다루고, 남은 조판·입력 축은 `Refs`와 원 이슈 링크로 유지한다. 게시 뒤 API로 실제 줄바꿈·한글·이미지 URL을 다시 확인한다.


## 최종 production 전쪽 재검증 — 2026-10-06

Production·검증 source `8569f49ce051ee343d58866a4f1e20a642d9e7fa`, 정책 base `c167dc6abbebf69546575e2d16d06223791bab82`를 검증했다. fresh WASM SHA-256 `410f8f3540f2856fcd7200a115f191a87aed4b2e726267bc54d960b35948735a`, JS `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`; root `pkg`와 Studio public의 실제 바이트가 같다. Native binary SHA-256 `d4ff621918e81070809efe96555f9e484905f12c20d77f9a78f81ce4095fbe46`. 아래 결과는 원점 리셋 보정까지 포함한 최신 source의 재출력이다.

Native/fresh WASM 각각30항목·37쪽(합계74쪽 대응) 최신 재출력, 최저91.96451%, 90% 미만/누락/측정 불가/글꼴 예외0이다. canonical TSV: ignored `output/pr-review/semanticist21-20261005/ladder-reset-tsv/<native|wasm>/<key>/silhouette.tsv`. 입력·Print 출처와 직접 판독은 위 개별 증거를 따르며, [공통 렌더/TSV 명령·재출력 검증](../assets/semanticist21-20261005/README.md#문단-원점-리셋-보정의-최종-nativefresh-wasm-검증)에 연결한다. 전체 Rust 및 원격 CI 완료 여부는 다음 최종 판정에서 별도로 기록한다.

| 이 PR의 검증 입력 | 경로 | 독립 Print 전체 쪽 실루엣(%) | 판정 |
| --- | --- | --- | --- |
| `pr7530-production-save-two-pages-hwp` | native | p1 100.00000 / p2 100.00000 | passed / 누락0 |
| `pr7530-production-save-two-pages-hwpx` | native | p1 100.00000 / p2 100.00000 | passed / 누락0 |
| `pr7530-production-save-two-pages-hwp` | wasm | p1 100.00000 / p2 100.00000 | passed / 누락0 |
| `pr7530-production-save-two-pages-hwpx` | wasm | p1 100.00000 / p2 100.00000 | passed / 누락0 |

![최신 fresh WASM 직접 비교](../../../mydocs/pr/assets/semanticist21-20261005/pr7530/hwp-wasm-review-all-pages.png)
![같은 출력 standalone overlay](../../../mydocs/pr/assets/semanticist21-20261005/pr7530/hwp-wasm-overlay-all-pages.png)

- 최신 source의 실제 브라우저3종 PASS(화면/질의/폼), source registry·fmt·Native/WASM/workspace-all-targets Clippy·workspace build·base manifest/unit-tier 정책 PASS. Native 선행33건 및 Cargo 집중32건은 각각 모두 PASS이며 최종 전체 nextest/Native Skia 결과는 다음 판정에 기록한다. 원시 로그·중간 JSON·TSV는 ignored `output/pr-review/semanticist21-20261005`에 보존한다.


## 최종 로컬 게이트 — production8569f49ce

정책 base `c167dc6abbebf69546575e2d16d06223791bab82`, 검증 source `8569f49ce051ee343d58866a4f1e20a642d9e7fa`. fmt·Native/WASM32/workspace-all-targets Clippy `-D warnings`·workspace build·manifest 및 source unit tier `--check --base-ref <base>` 모두PASS다. 파생 suite를 준비한 동일 review checkout의 `target/pr-review`에서 Cargo를 순차 실행했다.

- 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:10,491 PASS/0 FAIL/50 SKIP,706.118초, exit0.
- Native 선행33/ Cargo 집중32 모두PASS; 겹침16 partition 전체를 포함한다. 기존 fixture/baseline/래칫을 완화하지 않았다.
- Native Skia lib·missing picture·direct PDF·ComboBox·password 모두PASS. optional backend 검사를 원 head CI 또는 SVG 점수로 대신하지 않았다.
- fresh WASM/Studio 동기화·실제 화면/질의/폼3종·최신 Native/fresh WASM74쪽 대응/TSV 모두PASS(최저91.96451%, 미달/누락/측정 불가0, font exception0).

실제 명령·exit·시간은 ignored `output/pr-review/semanticist21-20261005/oct06-ladder-reset-validation-progress.json`, 원 출력은 `logs/oct06-ladder-reset-*.log`에만 보존한다. 각 단계의 마지막 summary는 아래 공통 증거 README에 기록한다. source가 바뀌면 이 실행 결과를 그대로 승계하지 않는다. 원 PR의 별도 CI와 누적 후보의 최신 원격 CI를 구분하며 통합 PR의 최종 head CI를 확인한 뒤 merge한다.

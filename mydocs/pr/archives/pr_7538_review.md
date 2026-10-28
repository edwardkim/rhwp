---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7538 기여자 변경 검토

## 최종 판정

**수용 가능 — 최신 trailing head 확인 후 통합 병합**. 원 기여와 메인터너 보정의 로컬 검증 및 [통합 PR #7576](https://github.com/edwardkim/rhwp/pull/7576) code candidate `04c4a1c0ccdf3159d6e934c06540c2dc2afcf28e`의 새 Full CI·CodeQL·Render Diff·Adapter·Proptest·trusted policy가 모두 성공했습니다. 원 #7538은 source conflict 때문에 직접 merge하지 않고 같은 변경과 보정을 이 통합 PR로 반영합니다. 문서만 담은 trailing head의 exact CI/fast-pass·보호 조건을 다시 확인한 뒤 병합하며 현재 기록은 병합 전 사전 판단입니다. 대용량 문서 전체 피델리티 잔여는 [#7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5984016636)에서 계속 추적합니다. #7575의 CI로 이 PR을 대체 검증하지 않았습니다.

## 접수·범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/7538
- 기여자: planet6897. 제목: 수정(조판): 단 맨 위 문단 기준 자리차지 표 테두리를 바깥 위 여백 아래에 그린다 (#4068)
- 원 head: `e4abc99f2de1da071964bb3a86c22e4b3d0a4d64`. base: devel.
- 현재 검토 branch: `review/planet6897-7538-20261005`, base `1e488ac370ec715a5b97c765542e16fc9bf65714`. 로컬 검증 head `cc59be7146142614f9a1f2ca74c2b078668a6f79`. 이전 #7575 batch에는 미적용이었습니다.
- 원 PR reviewer: jangster77 지정
- 경로: collaborator_external_pr 9.1.1; intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, post_merge.
- 작성 시점 CI/mergeability는 참고값이며 수용·push 전에 최신 source head를 다시 확인합니다.

## 원 기여와 체리픽 출처

| 원 commit | 통합 commit/처리 | 비고 |
| --- | --- | --- |
| `5e4a82579ab93ff15e38bfadf25d25470988e4e4` | `2743e2788` | `-x` 적용·기존 #7062 관계 회귀 보존 |
| `e4abc99f2de1da071964bb3a86c22e4b3d0a4d64` | `8d9fc8bd0` | 원 기여의 시각 증적 적용 |

## 조판 원칙과 검증 현황

| 항목 | 현재 근거·해제 조건 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 저장 LineSeg의 단 시작·되감김, 원본 outMargin·이전 단 vOff를 소비합니다. HFT 결측 U+2024는 독립 PDF Type3 전진과 원본 선언·face 폭 표에 근거합니다. 문서 ID·고정 페이지 좌표 분기는 없습니다. | 확인 |
| 측정·배치 일관성 | 표의 paint inset은 실제 테두리에 반영되고 뒤 문단 흐름에서는 반복 소비되지 않습니다. 기호는 `resolved_to_text_style → char_width_decision → 줄 구성/최종 TextRun` 공통 전진을 소비합니다. | 확인 |
| 분할·이어받기 계약 | 구역 첫 단·동일 단 둘째 표·다음 쪽 carry의 원점/오프셋을3개 회귀와 직접 시각 비교로 확인했습니다. 내용 컷·rowspan·각주 예약 계산은 이번 변경 범위 밖입니다. | 적용 범위 확인 |
| 줄 소속과 점유 높이 | 저장 앵커와 뒤 문단·둘째 표 흐름 및 #7062 관계 회귀를 보존했습니다. 표 행이 본문 경계 안에 있고 후속 글자의 전진이 원본 HFT 계약을 지키는지 확인했습니다. | 확인 |
| 사례와 증거의 독립성 | 기존 한컴 PDF16개 입력의 Git blob/hash, 정확한 devel A/B, Native/실제 Chrome fresh WASM을 대조했습니다. 원 source PNG만으로 승인하지 않았습니다. | 확인 |
| 기준값 변경 | 원본 HU·표 흐름·본문/행 높이 관계로 회귀를 수정했습니다. API golden은 HFT26개 전각 분류의 두 카운트와 aggregateHash만 바꾸고 TrueType89개와 categories/legacy hash는 유지했습니다. | 확인 |
| 주장과 검증 범위 | 로컬10324/0, Skia4109+6/0, CI Full4 archive workers10131/0 및 해당 CI 내부/Native 게이트 통과입니다. 시각 수용17쪽씩 최저95.07329%; 비교236쪽씩의12개 대용량 미달은 전체 수용에서 분리했습니다. | 범위 제한 수용 |

## 검증 입력 커밋 확인

[입력16개 출처](../assets/planet6897_7538_20261005/final/input-provenance.json)에 실제 HWP/PDF 경로·Git blob·SHA256·크기를 기록했습니다. 로컬 검증 head `cc59be714`와 모두 일치합니다. renderer 소스는 Native/WASM 빌드 head `c6ece0a6e`부터 동일하고 최종 source equivalence·SVG hash를 증적으로 보존했습니다. 새 기준 PDF를 산출하지 않았으며 기존 #7445 문서와 회귀를 삭제하지 않았습니다.

## 시각 검증 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 Native/fresh WASM의 전체 대상 페이지 TSV를 먼저 산출합니다. 영향 페이지·미달 페이지·대표 경계는 review/overlay PNG로 직접 확인하며 쪽수·각주·문단 소유·누락/중복을 별도 판단합니다. 새 회귀는 실제 검증 범위 최저 90% 이상만 수용합니다.

## 다음 단계

원 PR 단위로 실패 원인과 증적을 먼저 분석하고, 필요한 보정은 코드 수정·결과 보고·커밋을 완료한 뒤 다음 보정으로 진행합니다. 통합 code candidate의 최종 검증 뒤 수용 판정·contributor 후속 comment 계획을 확정합니다. 아직 원 PR 또는 통합 PR을 병합한 것으로 표시하지 않습니다.

## 최신 접수 재확인 — 2026-10-05

원 head `e4abc99f2de1da071964bb3a86c22e4b3d0a4d64`는 접수 시점과 동일합니다. CI Impact Policy가 PENDING이므로 미적용으로 유지했습니다. 통합 branch의10,318 PASS는 이 원 head를 승인하는 근거가 아닙니다. 정확한 원 head와 CI 재실행이 조건을 충족하면 다음 접수에서 별도로 검토합니다. [원18개 CI 조회 기록](../assets/planet6897_20261004/final_validation.json).

## 2026-10-05 보류 해소 착수

- 검토 branch `review/planet6897-7538-20261005`, base `1e488ac370ec715a5b97c765542e16fc9bf65714`에서 원 head `e4abc99f2de1da071964bb3a86c22e4b3d0a4d64`의 두 commit을 `-x` 체리픽했습니다. 통합 commit은 `2743e2788`·`8d9fc8bd0`입니다.
- 충돌은 `issue_7062_tac_object_host_line_height`에서 발생했습니다. #7575의 기존 관계 기반 검사(제목·책임자의 셀 소속·내용 순서)를 유지해 해결했으며 원 PR의 절대141.37px·269.58px 고정값을 복원하지 않았습니다.
- 최초 focused 컴파일에서 새 devel 호출부 `paragraph_layout.rs`의 bool 인자가 원 PR의 f64 paint inset 계약과 충돌했습니다(E0308). 삽입 없는 호출의 인자를0.0으로 바꿨습니다. 저장 앵커 판정 helper의 오래된 주석도 실제 규칙(이월 시 이전 단 vOff 제거·새 단 outMargin 적용)에 맞췄습니다. 이는 문서별 위치 보정이나 공유 흐름 조건 확대가 아닙니다.
- 원 CI는 모두 green이나 Controller37012662838이 `missing-workflow:CI|CodeQL|Render Diff`로 pending을 발행했습니다. 현재 동일 branch/repository/head의 실행5개가 API에 조회되고 trusted selector가 CI37010657364를 선택합니다. Controller만 attempt2로 재실행했습니다. 회귀·시각 검증과 정책 최종 결과는 아직 확인 중입니다.

### 정책 보류 해소 결과

Controller [37012662838](https://github.com/edwardkim/rhwp/actions/runs/37012662838)의 attempt2가 success이며 정확한 원 head의 CI Impact Policy가 SUCCESS로 갱신됐습니다. 현재 API·trusted selector에서 같은 repository/branch/SHA의 CI를 정상 선택했습니다. 당시 수집 누락의 내부 원인은 재현되지 않았으므로 정책 구현 결함으로 단정하지 않습니다. 정책 코드를 바꾸거나 성공 status를 수동 발행하지 않았습니다. [현재 판정 증적](../assets/planet6897_7538_20261005/policy_resolution.json).

### 보정1 결과 — 새 devel 호출 계약 정합

`8d9fc8bd0` 이후 호출 인자·주석 보정에서 기존 #7538 focused3개가 **3 PASS/0 FAIL**입니다. 새 devel의 기존 관계 기반 #7062 검사는 충돌에서 보존했습니다. 전체·fresh WASM·시각 gate는 다음 단계이며 현재 최종 수용 판정은 보류입니다.

### 보정2 결과 — 원본 여백·흐름 관계 기반 회귀

- 신규 기여 검사3개의283/141HU와560HU를 원본 표 outMargin·vertical offset에서 직접 읽도록 바꿨습니다. 동일 쪽 둘째 표의 고정11.84px 기대값은 제거했습니다. 첫 표 paint 여백을0으로 바꾼 대조군에서도 뒤 표의 저장 흐름이 움직이지 않으며 두 표가 겹치지 않는 계약을 검사합니다. 허용 오차는 수치 환산 관용이며 절대 페이지 좌표를 정답으로 고정하지 않습니다.
- 수정한 기존 함수3개 **3 PASS/0 FAIL**, 기존 #7062 관계 기반 함수3개 **3 PASS/0 FAIL**입니다. 검사 함수를 추가하지 않았습니다.
- fresh Native/fresh WASM에서1쪽·2쪽·10쪽 문서의 전체13쪽은 두 경로 각각 최저99.95151%·98.62979%·95.17961%, 미달0입니다. Native 대표 PNG에서 표 위 여백·셀 안 그림·뒤 그림·둘째 표·머리 표의 제목/책임자 소속을 직접 확인했습니다.
- 정책연구215쪽의 Native24쪽은99.95810%이며 그림23·24와 설명·뒤 문단 소속이 정상입니다. 전체 비교의7쪽 미달은 devel과의 A/B로 새 회귀 여부를 확인 중입니다. 이 단계에서 문서 전체 수용을 주장하지 않습니다.

### 시각 비교와 최종 검증 착수

Native/fresh WASM 전체228쪽씩 TSV 산출을 완료했습니다. 단독1쪽·신청서2쪽·보도자료10쪽은 모두90% 이상이며, 정책연구의 영향24쪽은99.95810%입니다. 두 출력의 대표4쪽 PNG를 직접 확인했으며 내용 소속·표 위 여백·뒤 개체의 흐름과 그림 설명이 정상입니다. 정책연구의44·158·168·173·184·208·211쪽은 두 출력에서 모두90% 미만입니다. 정확한 base 빌드 A/B로 변경 출처를 확인하며 이 문서 전체의 수용은 주장하지 않습니다. 기존 회귀를 삭제하거나 기준값을 완화하지 않았습니다.

검사 설명에 남아 있던 고정11.84px 기대값도 실제 관계 기반 검사 내용에 맞춰 고쳤고,24쪽 TSV 값을 원본99.95810%로 정정했습니다. 필수 fmt·Native/WASM/all-target Clippy·workspace build·manifest는 모두 통과했습니다. 정확한 base에서 신규 검사의 수정 전 실패를 확인한 뒤 현 branch의 전체 nextest·Native Skia 검증을 순차 진행합니다.

### 보정3 사전 분석 — 작은 정상 대조군의 기호 전진폭

추가 경계 검증에서2쪽 `float-stack-defer.hwp`의2쪽이 Native/WASM85.47994%(base88.57882%)입니다. 한컴2022 정본 두 PDF가 같은 결과이며 단순 세로 이동이나 폰트 교체만으로는90%에 도달하지 않았습니다. 원본은 HFT `한양중고딕`, 문제 글자의 charPr7은 장평95%·자간-10%입니다. 독립 PDF의 Type3 `/T2` U+2024 글리프 전진은1000/1000이며 첫 `․`→`인`은13.75px입니다. 현재 공통 측정은 원본 HFT와 폭이 다른 대체 메트릭/반각 경로로6.84px를 계산합니다. 진단 SVG에서 오른쪽 칸의 이 전진폭만 복원하면93.02841%입니다(진단 산출물이며 제품 통과 증거가 아닙니다).

원본 한글 HFT 슬롯 여부를 ASCII 반각 profile과 별개로 전달하고, 대체되지 않은 원본 HFT 선언과 해당 face의 폭 표가 확인되고 U+2024 폭만 결측인 갈래에서 전각 전진을 복원합니다. 기존 메트릭 hit는 바꾸지 않습니다. 명시적으로 선택한 TrueType 프로그램에는 적용하지 않습니다. 문서명·특정 좌표·페이지 번호로 분기하지 않습니다. `resolved_to_text_style → 공통 글자 폭 결정 → 줄 구성/최종 TextRun`이 같은 전진을 소비하며 실제 PDF·fresh Native/WASM과 기존 회귀로 최종 판정합니다.

- 보정3 검증: Native Visual Sweep 전체 2쪽 완료, 1쪽 98.08627%·2쪽 95.07329%, 미달 0쪽입니다. 두 review PNG에서 표 행·문단 시작·후속 글자 배치와 누락 여부를 직접 확인했습니다. 기존 `issue_5906_float_stack_declared_tail` 1건을 원본 HFT 글자 전진·본문 경계·원본 마지막 행 높이 관계로 수정했고, Native Skia focused nextest 1 PASS입니다. 최종 전체 및 fresh WASM 검증은 이어서 수행합니다.

### 보정4 사전 분석 — 폭 분류 golden 동기화

첫 전체 nextest는 10,324건 중10,323 PASS/1 FAIL/50 skip,652.786초입니다. 실패는 픽셀/좌표 검사가 아닌 read-only 폭 분류 API의 golden입니다. HFT 결측 한점 리더26개가 반각에서 전각으로 바뀌어 `heuristicFullwidth`524→550, `heuristicHalfwidth`118→92와 aggregateHash만 달라졌습니다. 동일 API로 공개 입력3개를 다시 조회했으며 format parity HWP/HWPX는 모든 golden 필드가 동일합니다. 문제 입력의 문자 총량·coverage categories·legacyProjectionHash도 그대로입니다. 원래 분류 계약을 유지하고 해당 두 카운트와 aggregateHash만 동기화하며 focused 후 전체를 다시 실행합니다.

- 보정4 결과:19쪽 전체 trace가 complete이며 원본 HFT26개만 전각, TrueType89개는 반각을 유지합니다. classification/parity·privacy·분류 총량·불변성·선형 보행을 포함한 기존 API 회귀8개가 **8 PASS/0 FAIL**입니다. golden은3필드만 변경했고 생산 소스·PDF·렌더링 기준은 추가로 바꾸지 않았습니다. 최종 전체 nextest와 Native Skia를 이어서 확인합니다.

### 로컬 최종 검증 완료 — 통합 CI 대기

[최종 검증 정본](../assets/planet6897_7538_20261005/final_validation.json)에 전체 nextest10,324 PASS/0 FAIL/50 skip(509.897초, threads8), Skia4,109 PASS/13 ignore 및 focused2+4 PASS, 필수 lint/build/manifest/fresh WASM을 기록했습니다. Native/실제 Chrome fresh WASM 각각236쪽 TSV를 보존했습니다. 단독1쪽·신청서2쪽·보도자료10쪽·float-stack2쪽 전체와 영향24쪽·rowbreak13쪽의 수용 범위17쪽씩은 최저95.07329%, 미달0입니다. Native/WASM 대표 PNG와 새 float-stack2쪽 PNG를 직접 확인했습니다. 기준 HWP/PDF16개는 로컬 검증 head의 Git blob과 일치합니다.

정책연구의 미달7쪽은 정확한 devel과 SVG가 동일하며 해당 문서에서는24쪽만 바뀝니다. hwpspec의18·35·44·46쪽은 base보다 개선됐지만 여전히 미달이고, CBTA58쪽은 base78.71686%→73.17570%입니다. 이 대용량 문서들의 전체 피델리티 수용은 보류하며 #7445에 현재 TSV와 정확한 차이를 연결합니다. 해당 미달을 정상 회귀의 기준으로 고정하거나 기존 회귀를 제거하지 않았습니다. 통합 code candidate의 정확한 CI와 최신 head 조건을 확인한 뒤에만 최종 수용·오늘할일·후속 comment를 trailing 문서에 기록합니다.

## 통합 code candidate CI 완료 — 2026-10-05

- PR [#7576](https://github.com/edwardkim/rhwp/pull/7576), exact head `04c4a1c0ccdf3159d6e934c06540c2dc2afcf28e`, base `1e488ac370ec715a5b97c765542e16fc9bf65714`.
- [CI37231838199](https://github.com/edwardkim/rhwp/actions/runs/37231838199) preflight는 `fast_pass=false reason=no-green-build-candidate`로 새 Full 검증을 선택했습니다. 실제 A3870/B1936/C1709/D2616, 총10131 PASS이며4 worker가 모두 attempt1에서 성공했고 Build & Test·lint·Native Skia가 통과했습니다. 로컬 전체10324 PASS와 CI의 configured archive count를 혼동하지 않습니다.
- [CodeQL37231838202](https://github.com/edwardkim/rhwp/actions/runs/37231838202), [Render Diff37231837993](https://github.com/edwardkim/rhwp/actions/runs/37231837993), [Adapter37231838244](https://github.com/edwardkim/rhwp/actions/runs/37231838244), [Proptest37231838309](https://github.com/edwardkim/rhwp/actions/runs/37231838309), trusted controller37231838032 및 exact `CI Impact Policy`가 성공했습니다. CodeQL Analyze(rust)는 성공이고 publisher neutral/범위별 conditional skip은 해당 workflow·정책 결과로 보존했습니다.
- CI witness: tested merge `863caa2ec1290308d3feea335f5e5015b6f517d7`, tree `284ce01fc9d9e01bb197e3bf567621217baf7d55`, 부모는 정확한 base와 candidate입니다. marker·archive artifact IDs·각 run/head/result·worker counts는 [CI 정본](../assets/planet6897_7538_20261005/final/ci-code-candidate.json)에 있습니다. trusted job의 성공만으로 reuse를 주장하지 않았습니다.
- 원 head는 `e4abc99f2de1da071964bb3a86c22e4b3d0a4d64`로 동일합니다. 오늘할일을 보존해 추가 기록하며 이번 trailing에는 이 review·오늘할일·CI 증적만 포함합니다. production/test/fixture/workflow 변경은 없습니다. 최신 trailing head의 same-PR fast-pass 또는 Full provenance와 보호 조건 확인은 병합 직전에 별도 수행합니다.

## Merge 후 contributor PR comment 계획

원 #7538의 최신 head와 기존 comment를 확인하여 새 기여나 중복 게시를 막습니다. 실제 #7576 merge SHA와 CI run·Visual Sweep 정본·merge SHA 고정 raw PNG를 넣어 한국어 존댓말로 다음을 구분해 설명하고 통합 대체 병합으로 close합니다.

1. 기여자께서 해결하신 단 맨 위 표 outMargin·이전 단 offset 제거·후속 흐름 보존과 원 두 commit의 `-x` 출처.
2. 최신 devel 충돌에서 #7062의 개선된 회귀를 보존한 이유와 `bool`→`f64` 호출 계약 보정.
3. 고정11.84px/HU 회귀를 원본 여백·오프셋·표 소속·흐름 관계로 바꾼 이유, 정확한 devel3 FAIL→현재3 PASS.
4. 2쪽 정상 대조군에서 드러난 HFT U+2024 결측 폭 문제·독립 Type3 전각 근거·TrueType/hit 보존·기존5906 관계 회귀와 전2쪽98.08627/95.07329%.
5. 공개 API golden의 HFT26개 분류 변경과 TrueType89개 불변·3필드 갱신·기존8개 API 회귀 통과, 최종10324 PASS와 실제 code candidate CI.
6. 정책연구7쪽·hwpspec4쪽·CBTA58 대용량 전체 피델리티의 #7445 분리와 전체 수용을 주장하지 않는 범위. #4068/#7445의 미완료 범위·원본·PDF·기여자 fork는 보존합니다.

게시에는 `--body-file`을 사용하고 API로 본문·close를 확인합니다. 정상 merge/CI·source 후속처리·devel fast-forward·소유 정리까지 완료한 뒤 종료합니다.

---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7534 기여자 변경 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 글리프/줄 상자 분리·저장 줄 소유·공백 잉크의 개별 수용 범위와 최신 base 통합 후보의 최종 로컬 검증을 완료했습니다. 원 PR을 직접 병합하지 않고 기록한 체리픽 통합 후보를 대상으로 합니다. #7445 문서 전체의 피델리티 승인은 아닙니다. code candidate CI는 완료됐으며 trailing head의 CI·보호 요건 통과 후 병합합니다.

## 접수·범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/7534
- 기여자: planet6897. 제목: fix: relSz 는 글리프에만·줄 상자는 선언 크기로, 글뒤 그림 뒤 TAC host 다음 문단의 저장 top 을 지킨다 (#7398 #7431)
- 원 head: `032c778f47e8742f00a6cd91ab436e6abedb4191`. base: devel.
- 검토 branch: `review/planet6897-20261004`. 통합 base: `732047aa253e92772fd7625e4a5d7f6a9897fd73`. 적용 단계 head: `caa9d67d3f5171dae027798a8248218343ef5696`.
- 원 PR reviewer: jangster77 지정
- 경로: collaborator_external_pr 9.1.1; intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, post_merge.
- 작성 시점 CI/mergeability는 참고값이며 수용·push 전에 최신 source head를 다시 확인합니다.

## 원 기여와 체리픽 출처

| 원 commit | 통합 commit/처리 | 비고 |
| --- | --- | --- |
| `685c82b12f58976c16bf4b6f9326cae6b0401a37` | `86b06ded3fd1cced7cac308080a2b3c3dcf0836a` | applied |
| `07dca362e40e34d0cb17734a027ccd3742a472d7` | `eee69daa9f76747ee839bc50d9e6c9426ea3b7ec` | applied |
| `bcc058a312a6a78bdf56adb2b4b5e7627b4d48e1` | `af8caf5e8525579f661d9f058a57abe1e697623c` | applied |
| `6a1aeab5b7e241bce9aecc53889d8432fc52f8fc` | `caad632172f2570ebc5972cc39a057f7661ee3b9` | applied |
| `032c778f47e8742f00a6cd91ab436e6abedb4191` | `889b6636b803c83047e46b7ace87ebda529119fa` | applied |

## 조판 원칙과 검증 현황

| 항목 | 현재 근거와 범위 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 공백의 측정 폭과 잉크 점유 분리·저장 줄 소유 | 개별 범위 확인 |
| 측정·배치 일관성 | 아래 저장 사양·내용 소유 및 직접 결과의 범위로 확인 | 개별 범위 확인 |
| 분할·이어받기 계약 | 아래 개별 구조 검사·시각 결과에 한정; 전체 corpus 수용 주장 없음 | 개별 범위 확인 |
| 줄 소속과 점유 높이 | 아래 저장 줄·개체 소유 및 정상 대조 검증 범위에 한정 | 개별 범위 확인 |
| 사례와 증거의 독립성 | 아래 통합 직접 실행·독립 한컴 PDF 또는 사양 및 입력 hash 참조 | 확인 |
| 기준값 변경 | 아래 기존 회귀 보정·유지 이유 참조; 출력 좌표를 새 정답으로 고정하지 않음 | 확인 |
| 주장과 검증 범위 | 개별 범위와 최종 로컬 검증 통과; 최신 head 통합 CI 완료 전 병합 불가 | 통합 CI 대기 |

## 검증 입력 커밋 확인

아래 개별 단계의 실제 입력·독립 PDF·실행 head와 증적 JSON/manifest에 내용 hash를 기록했습니다. 기존 #7445 이관 자료는 해당 단계에 적은 범위만 유지하며 전체 피델리티 승인을 주장하지 않습니다. 최종 전체 검증 결과는 별도 완료 후 기록합니다.

## 시각 검증 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 Native/fresh WASM의 전체 대상 페이지 TSV를 먼저 산출합니다. 영향 페이지·미달 페이지·대표 경계는 review/overlay PNG로 직접 확인하며 쪽수·각주·문단 소유·누락/중복을 별도 판단합니다. 새 회귀는 실제 검증 범위 최저 90% 이상만 수용합니다.

## 다음 단계

원 PR 단위로 실패 원인과 증적을 먼저 분석하고, 필요한 보정은 코드 수정·결과 보고·커밋을 완료한 뒤 다음 보정으로 진행합니다. 통합 code candidate의 최종 검증 뒤 수용 판정·contributor 후속 comment 계획을 확정합니다. 아직 원 PR 또는 통합 PR을 병합한 것으로 표시하지 않습니다.

## 단계7 통합 직접 검증과 남은 보류

- 생산 code candidate `c8c5f535cd3ebf331cadc2714c68af9fdc663a4d`의 Native/fresh WASM으로 원문 `samples/exam_eng.hwp`와 독립 한컴 2022 `pdf/exam_eng-2022.pdf`를 전체 8쪽 대조했습니다. 저장 product도 한컴 2022입니다. 두 출력 모두 최저 93.01297%(2쪽), 미달·누락 0, 쪽수 8쪽 일치입니다. [실행 결과](../assets/planet6897_20261004/7534-stage7/results.json)·Native/WASM TSV 및 manifest를 보존했습니다.
- [2쪽](../assets/planet6897_20261004/7534-stage7/native_review_p002.png)·[7쪽](../assets/planet6897_20261004/7534-stage7/native_review_p007.png)을 직접 열었습니다. 원 기여의 줄 진행·답지 시작 복구는 확인됐지만 2쪽 Woman/Man/Sophia의 NBSP 공백에 사각형 잉크가 나타납니다. 90% 통과만으로 이 출력 결함을 승인하지 않습니다.
- 기존 `test_521`은 SVG에서 595..600px 영역을 찾고 24px 간격을 고정했습니다. 8쪽 시각 근거를 확인한 뒤 동일 기존 시험을 표 host pi104·다음 답안 pi105의 소유 관계 및 원문 저장 vpos에서 선언 표 높이를 뺀 간격으로 변경했습니다. 새 테스트는 추가하지 않았습니다. 변경된 기존 unit 1/1 PASS, 새 기여 회귀 5/5 PASS, unit tier 검사 PASS(개수 증가 0)입니다.
- 다음 보정은 NBSP 잉크입니다. 문서에는 NBSP가 저장되어 있고 현재 HY신명조 글꼴 cmap은 U+00A0을 `uni0080`(윤곽선 2개)으로 매핑합니다. 공백이 실제 글리프처럼 그려지는 경로를 대조해 저장 폭·밑줄을 보존하며 잉크만 그리지 않도록 검토합니다. 메인터너 보정과 재출력 전 **개별 시각 보류**입니다.

## 단계8 공백 잉크 보정 후보

NBSP를 포함해 Unicode 공백만으로 구성된 클러스터는 SVG의 본문·그림자, Canvas 기본 글리프, Skia 텍스트 pass에서 잉크를 그리지 않도록 맞췄습니다. 이미 계산된 글자 전진폭·뒤 글자 위치·밑줄과 배경 장식은 같은 경로에 남습니다. 입력·폰트 파일·메트릭·비교 지표를 바꾸지 않았습니다.

기존 SVG unit 및 변경한 `test_521` **51/51 PASS**입니다. [소스·폰트 hash와 결과](../assets/planet6897_20261004/7534-stage8/results.json)를 기록했습니다. Native/fresh WASM 재출력과 native-skia 기능 검증 전에는 보정 후보로 보류합니다.

## 단계8 보정 후 판독 완료 — 개별 보류 해소 / 통합 검증 대기

- 생산 code candidate `4cfe96e52`에서 기존 SVG/관계 unit 51/51 PASS, native-skia 기능을 켠 관련 unit 100/100 PASS입니다. Native 기능 빌드와 새 WASM 빌드 모두 성공했습니다. 이후 실행 head `4de143ddb`까지 생산·테스트 변경이 없음을 git diff로 확인했습니다. [소스·build·로그 hash와 현재 결과](../assets/planet6897_20261004/7534-stage8/results.json)를 보존했습니다.
- 전체 8쪽 Native/fresh WASM TSV가 페이지별로 같으며 최저 **95.29650%(4쪽)**, 2쪽 **96.11420%**, 7쪽 95.84947%입니다. 미달·누락 0, 원문/PDF/출력 모두 8쪽입니다. 새 회귀를 추가하지 않고 기존 `test_521`의 저장 관계 검사만 고쳤습니다.
- [보정 2쪽 review](../assets/planet6897_20261004/7534-stage8/native_review_p002.png)와 [7쪽 review](../assets/planet6897_20261004/7534-stage8/native_review_p007.png)를 직접 열었습니다. 듣기 답안 NBSP의 사각형 잉크가 사라지고 밑줄·내용·뒤 문단 위치는 유지됩니다. 오른쪽 편지 및 답안 시작, 양 단 줄 소속이 유지됩니다. 2쪽 엄격 픽셀 18.70%와 관용 실루엣 96.11%를 구분합니다.
- [Skia 실제 2쪽 PNG](../assets/planet6897_20261004/7534-stage8/skia_page002.png)와 fresh WASM 2쪽 raster도 직접 열어 같은 NBSP 잉크/밑줄 관계를 확인했습니다. Skia PNG는 2쪽 진단이며 전체 8쪽 Skia 실루엣 검증으로 주장하지 않습니다. 도구 라벨과 본문은 판독 가능합니다.
- 앞서 수용한 #7547 대상 17쪽, 정상 반례 두 벌 각 3쪽, #7543 대상 1쪽까지 **32쪽 전체를 Native/fresh WASM으로 재검증**했습니다. 각각 최저 95.23424% / 97.99734% / 98.00366% / 99.41288%, 미달·누락 0으로 유지됩니다. 원 기여의 줄 높이·TAC host 수정과 메인터너의 공백 잉크/좌표 회귀 보정으로 **개별 발견 보류는 해소**했습니다. 최종 통합 검증 전 머지 보류를 유지합니다.
- merge 후 contributor 설명에는 두 원 기여와 독립 저장 관계 검사, 공백 글리프 보정의 필요·범위·반례, 정확한 CI/merge SHA 및 위 PNG를 함께 기록합니다. 아직 게시·close하지 않았습니다.

## 리베이스 전 통합 로컬 검증 — 2026-10-05

전체 회귀 head `aa10d4093f60c4e0c12aa186f2464861acbf89ea`, 최종 lint/build·fresh WASM·Skia head `42ef60c96`(테스트 모듈 순서만 정리), base `731de9e1b4bb946d76f35108ed7e186ebe4ebecb`입니다. 전체 nextest는 **10,318 PASS/0 FAIL/50 skip**,563.885초, threads=8로 완료했습니다. Format·Native/WASM32 Clippy·workspace build·workspace all-target Clippy·manifest/unit tier 정책을 통과했습니다. fresh WASM은 Mac 로컬 `--no-opt` 대체 빌드이며 Docker 최적화 검증으로 보고하지 않습니다. Skia lib·그림 placeholder·직접 PDF 출력 범위도 실제 실행했습니다. Studio 전체1,815 PASS/0 FAIL/2 skip와 TypeScript/Vite build도 완료했습니다.

[정확한 head·명령·결과·원 source CI·개별 시각 범위](../assets/planet6897_20261004/final_validation_before_7574.json)를 참고합니다. 시각 자료는 각 단계의 생산 code head와 입력/PDF hash로 고정되어 있으며, 이후 변경은 테스트·문서 범위입니다. 최종 fresh WASM hash와 기존 단계의 hash, 생산 본문 동일성 증적을 함께 기록합니다. 대용량 미달 문서는 #7445의 기존·추가 이관 기록으로 추적하고 그 문서 전체의 정확성을 승인하지 않습니다. 위의 “통합 검증 미완료/대기”는 해당 단계 작성 시점의 기록이며 최종 로컬 판정은 이 절을 따릅니다. 통합 CI는 아직 미실행입니다.

## Merge 후 contributor PR comment 계획

기여자의 글리프/줄 상자 분리·저장 줄 소유·공백 잉크 결과와 이 문서에 적은 추가 보정의 원인·범위를 한국어 존댓말로 구분해 설명합니다. 실제 통합 merge SHA·CI URL·Visual Sweep 정본과 이관 잔여 범위를 안내하고, 통합 대체 병합으로 원 PR을 close합니다. UTF-8 `--body-file`로 게시 후 API로 본문·close 상태를 재확인하며 중복 댓글은 게시하지 않습니다. [대표 PNG](../assets/planet6897_20261004/7534-stage8/native_review_p002.png)는 실제 merge SHA에 고정한 raw URL로 표시합니다.

### 최종 출력 provenance 재확인

최종 Native CLI(`native-skia`)와 실제 Chrome/fresh WASM으로 수용 범위20개 target·104쪽씩을 다시 내보냈습니다. **두 경로 최저90.80846%,90% 미만0쪽**입니다. SVG가 byte-identical인 쪽의 raster만 재사용하고, 달라진 SVG·글꼴 정책 블록은 새 raster로 비교했습니다. [전쪽 수용 범위 TSV·입력/PDF hash·현재 SVG hash와 재사용/재출력 구분](../assets/planet6897_20261004/final-visual/results.json). #7445 이관 문서의 전체 수용 주장은 아닙니다. 자산/속성 기여는 원래의 제한된 검토 범위를 유지하며 Studio crop은 동일한 최종 WASM의 실제 Canvas2D/CanvasKit 증적을 유지합니다.

## 최신 base 동기화 — #7574

base `732047aa253e92772fd7625e4a5d7f6a9897fd73` 위로69개 commit을 충돌 없이 리베이스했습니다. 후보 `2fae901137a3d20662ce203fc4f5fb1ea7cd68ec`입니다. 이전 문서에 기록한 실행 head와 결과는 리베이스 전 실제 실행 기록이며 최신 후보의 통과로 재사용하지 않습니다. [이전 검증 보존](../assets/planet6897_20261004/final_validation_before_7574.json), [현재 재검증 상태](../assets/planet6897_20261004/final_validation.json). 원 contributor head/history는 변경하지 않았습니다.

## #7574 리베이스 후 전체 회귀 완료

- 최신 base `732047aa253e92772fd7625e4a5d7f6a9897fd73`, 실행 head `9e5f0c3fda484f756c8aca819c87d0578b428141`에서 전체 nextest **10,321 PASS / 0 FAIL / 50 skip**, 495.993초(threads=8, slow8)를 확인했습니다. upstream의 기존3개 검사가 포함되었으며 이번 보정으로 새 검사를 추가하지 않았습니다.
- 전체 회귀 완료 시점에는 나머지 검사가 진행 중이었습니다. 아래 최종 검증 절에 실제 완료 결과를 기록합니다.

## #7574 리베이스 후 최종 검증

- 최신 base `732047aa253e92772fd7625e4a5d7f6a9897fd73`, 전체 회귀 head `9e5f0c3fda484f756c8aca819c87d0578b428141`에서 **10,321 PASS / 0 FAIL / 50 skip**, 495.993초(threads=8)입니다. 필수 Format·Native/WASM32/전체 target Clippy·workspace build·manifest·unit tier·fresh WASM·Skia 검사를 순차 실행하여 통과했습니다. Skia lib **4,109 PASS / 0 FAIL / 13 skip**, placeholder2·직접 PDF4 PASS입니다.
- 현재 후보의 Native/fresh WASM 수용 범위 **104쪽씩, 최저 90.80846% / 90.80846%, 90% 미만 0쪽**입니다. 실제 Chrome에서 현재 pkg를 로드했으며 모든 대상 SVG를 재출력했습니다. byte-identical SVG의 기존 raster만 재사용하고 달라진 쪽은 다시 raster했습니다. #7445 이관 문서의 전체 수용을 주장하지 않습니다.
- Studio 그림 자르기도 현재 fresh WASM의 실제 Canvas2D/CanvasKit에서 3쪽씩 재캡처했습니다. backend fallback 없이 각각 최저 **98.90923% / 95.73803%**이며 기관 로고·사진·대조군을 직접 확인했습니다. Studio 소스와 테스트는 리베이스 전 전체1,815 PASS/0 FAIL/2 skip 및 TypeScript/Vite/PWA build 이후 변경이 없습니다.
- [최신 검증 head·명령·결과](../assets/planet6897_20261004/final_validation.json), [전체 수용 범위 TSV·source/PDF/SVG hash](../assets/planet6897_20261004/final-visual-rebased/results.json), [Studio 재캡처 증적](../assets/planet6897_20261004/7529-final-rebased/results.json). 이전 base의 실제 실행 기록은 보존했고 현재 판정은 이 절을 따릅니다. 최신 통합 PR CI·보호 요건·merge는 아직 완료 전입니다.

최신 [대표 review PNG](../assets/planet6897_20261004/final-visual-rebased/7534/native/review_002.png)와 [overlay PNG](../assets/planet6897_20261004/final-visual-rebased/7534/native/overlay_002.png)는 merge 후 실제 merge SHA에 고정한 raw URL로 contributor comment에 표시합니다.

## 통합 PR #7575 코드 후보 CI 완료 — 2026-10-05

- [PR #7575](https://github.com/edwardkim/rhwp/pull/7575)의 code candidate `86a6fe85087e4371ac929df2036461ace7a0e986`, base `732047aa253e92772fd7625e4a5d7f6a9897fd73`에서 Full 검증이 완료됐습니다. preflight는 `no-green-build-candidate`로 Full을 선택했습니다. trusted job 성공을 재사용으로 해석하지 않았습니다.
- [CI 37221230992](https://github.com/edwardkim/rhwp/actions/runs/37221230992) · [CodeQL 37221231003](https://github.com/edwardkim/rhwp/actions/runs/37221231003) · [Render Diff 37221230707](https://github.com/edwardkim/rhwp/actions/runs/37221230707) · [Adapter 37221230904](https://github.com/edwardkim/rhwp/actions/runs/37221230904) · [Proptest 37221230929](https://github.com/edwardkim/rhwp/actions/runs/37221230929)는 같은 repository·source branch·PR event·candidate SHA에서 모두 success입니다. CI Impact Policy를 포함한 최신34개 check는 success 또는 정당한 skipped/neutral로 완료됐습니다.
- 실제 CI tested merge `b0c5f9c3758fb35eb37121f15fb8566062ca2d0c`, tree `568fcbc3e837d7f61abda92043ebaf34d341868c`의 parent는 위 base/code candidate와 일치합니다. [run·job·artifact·head 증적](../assets/planet6897_20261004/ci_candidate_7575.json).
- 개별 수용 범위·잔여 판정과 로컬10,321 PASS/0 FAIL/50 skip를 유지합니다. 이 기록은 source/test/fixture/workflow를 변경하지 않는 single-parent trailing 문서 commit입니다. 새 head의 required checks·실제 fast-pass provenance·mergeability·보호 요건을 확인한 뒤 정상 경로로 병합합니다. 아직 merge 완료를 주장하지 않습니다. 병합 후 실제 merge SHA와 보정 이유를 원 contributor PR에 설명하고 통합 대체 병합으로 close합니다.

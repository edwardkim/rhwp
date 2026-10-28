---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7542 기여자 변경 검토

## 최종 판정

**승인** — lineWrap 파일 속성 세 경로 왕복 보존의 개별 수용 범위와 최신 base 통합 후보의 최종 로컬 검증을 완료했습니다. 원 PR을 직접 병합하지 않고 기록한 체리픽 통합 후보를 대상으로 합니다. #7445 문서 전체의 피델리티 승인은 아닙니다. code candidate CI는 완료됐으며 trailing head의 CI·보호 요건 통과 후 병합합니다.

## 접수·범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/7542
- 기여자: planet6897. 제목: fix(hwpx): 문단 "한 줄로 입력"(lineWrap) 을 HWP5 attr2 bits 0-1 로 왕복한다 (#6875)
- 원 head: `9db08a6698b9d124d74836122859874d1730ed4b`. base: devel.
- 검토 branch: `review/planet6897-20261004`. 통합 base: `732047aa253e92772fd7625e4a5d7f6a9897fd73`. 적용 단계 head: `caa9d67d3f5171dae027798a8248218343ef5696`.
- 원 PR reviewer: jangster77 지정
- 경로: collaborator_external_pr 9.1.1; intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, post_merge.
- 작성 시점 CI/mergeability는 참고값이며 수용·push 전에 최신 source head를 다시 확인합니다.

## 원 기여와 체리픽 출처

| 원 commit | 통합 commit/처리 | 비고 |
| --- | --- | --- |
| `9db08a6698b9d124d74836122859874d1730ed4b` | `a75e7650b07492cf5bb2370431173531721e0455` | applied |

## 구현 검토와 독립 근거

HWP5 명세의 ParaShape `attr2` bit 0~1(한 줄로 입력)과 한컴이 저장한 HWP/HWPX 쌍둥이를 대조했습니다. 파서는 해당 두 비트만 덮어쓰며 나머지 비트를 보존하고, 직렬화기는 동일 비트를 BREAK/SQUEEZE/KEEP으로 내보냅니다. 미정 값은 BREAK로 처리합니다. 새 회귀는 좌표가 아닌 문단 모양 id와 파일 저장 후 속성 보존을 검사합니다.

- [HWP5 명세](../../tech/한글문서파일형식_5.0_revision1.3.md)의 한 줄로 입력 필드.
- 쌍둥이의 문단 모양 358개 중 SQUEEZE id 117·120·121·156의 동일성 및 세 저장 경로: **4/4 PASS**.
- renderer는 문단 ParaShape의 해당 비트를 직접 소비하지 않습니다. 이 PR의 판정 범위는 파일 속성 보존입니다. 한컴에서 다시 조판한 쪽수나 시각 정확도는 별도로 입증한 것으로 표시하지 않습니다.

| 항목 | 근거와 범위 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 사양 비트와 한컴 저장 쌍둥이, enum 양방향 대응 | 확인 |
| 측정·배치 일관성 | 문단 lineWrap의 측정/배치 소비 변경 없음 | 적용 안 됨 |
| 분할·이어받기 계약 | 저장 속성 보존만 변경; 분할 경로 변경 없음 | 적용 안 됨 |
| 줄 소속과 점유 높이 | 현재 renderer의 줄 소속을 변경하지 않음 | 적용 안 됨 |
| 사례와 증거의 독립성 | 한컴 원본 두 벌의 전제 및 재파싱 검사 통과 | 확인 |
| 기준값 변경 | 출력 좌표 기준값 변경 없음 | 확인 |
| 주장과 검증 범위 | 네 계약 및 최종 로컬 검증 통과; 최신 head 통합 CI 대기 | 통합 CI 대기 |

## 검증 입력과 실제 결과

두 입력 `samples/hwpx/mel-001.hwpx`, `samples/hwpx/hancom-hwp/mel-001.hwp`는 이미 커밋되어 있습니다. 검증 head·입력 SHA-256·실행 명령·전체 focused 실패 목록은 [단계1 검증 증거](../assets/planet6897_20261004/stage1_focused_validation.json)에 기록했습니다. 로그는 ignored `output/pr-review/planet6897-20261004/focused-nextest.log`에만 보관합니다.

관련 묶음 결과는 65개 중 63 PASS / 2 FAIL이며, 실패 두 건은 #7547의 표 분할 계약입니다. #7542 자체 네 검사는 모두 통과했습니다. 새 회귀 또는 메인터너 코드 보정은 추가하지 않았습니다.

## 다음 단계와 contributor 후속 계획

통합 전체 검증 완료 후 최종 수용 판정을 갱신합니다. 병합 후에는 기여자의 파일 속성 왕복 보정이 한컴 독립 입력 네 계약을 통과했음을 한국어 존댓말로 원 PR에 설명하고 정확한 통합 merge SHA를 안내합니다. 현재 comment·close·통합 merge는 수행하지 않았습니다.

## 리베이스 전 통합 로컬 검증 — 2026-10-05

전체 회귀 head `aa10d4093f60c4e0c12aa186f2464861acbf89ea`, 최종 lint/build·fresh WASM·Skia head `42ef60c96`(테스트 모듈 순서만 정리), base `731de9e1b4bb946d76f35108ed7e186ebe4ebecb`입니다. 전체 nextest는 **10,318 PASS/0 FAIL/50 skip**,563.885초, threads=8로 완료했습니다. Format·Native/WASM32 Clippy·workspace build·workspace all-target Clippy·manifest/unit tier 정책을 통과했습니다. fresh WASM은 Mac 로컬 `--no-opt` 대체 빌드이며 Docker 최적화 검증으로 보고하지 않습니다. Skia lib·그림 placeholder·직접 PDF 출력 범위도 실제 실행했습니다. Studio 전체1,815 PASS/0 FAIL/2 skip와 TypeScript/Vite build도 완료했습니다.

[정확한 head·명령·결과·원 source CI·개별 시각 범위](../assets/planet6897_20261004/final_validation_before_7574.json)를 참고합니다. 시각 자료는 각 단계의 생산 code head와 입력/PDF hash로 고정되어 있으며, 이후 변경은 테스트·문서 범위입니다. 최종 fresh WASM hash와 기존 단계의 hash, 생산 본문 동일성 증적을 함께 기록합니다. 대용량 미달 문서는 #7445의 기존·추가 이관 기록으로 추적하고 그 문서 전체의 정확성을 승인하지 않습니다. 위의 “통합 검증 미완료/대기”는 해당 단계 작성 시점의 기록이며 최종 로컬 판정은 이 절을 따릅니다. 통합 CI는 아직 미실행입니다.

## Merge 후 contributor PR comment 계획

기여자의 lineWrap 파일 속성 세 경로 왕복 보존 결과와 이 문서에 적은 추가 보정의 원인·범위를 한국어 존댓말로 구분해 설명합니다. 실제 통합 merge SHA·CI URL·Visual Sweep 정본과 이관 잔여 범위를 안내하고, 통합 대체 병합으로 원 PR을 close합니다. UTF-8 `--body-file`로 게시 후 API로 본문·close 상태를 재확인하며 중복 댓글은 게시하지 않습니다. 배치 변경 없는 자산/파일 속성 기여로 시각 렌더 승인 주장은 하지 않습니다.

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

## 통합 PR #7575 코드 후보 CI 완료 — 2026-10-05

- [PR #7575](https://github.com/edwardkim/rhwp/pull/7575)의 code candidate `86a6fe85087e4371ac929df2036461ace7a0e986`, base `732047aa253e92772fd7625e4a5d7f6a9897fd73`에서 Full 검증이 완료됐습니다. preflight는 `no-green-build-candidate`로 Full을 선택했습니다. trusted job 성공을 재사용으로 해석하지 않았습니다.
- [CI 37221230992](https://github.com/edwardkim/rhwp/actions/runs/37221230992) · [CodeQL 37221231003](https://github.com/edwardkim/rhwp/actions/runs/37221231003) · [Render Diff 37221230707](https://github.com/edwardkim/rhwp/actions/runs/37221230707) · [Adapter 37221230904](https://github.com/edwardkim/rhwp/actions/runs/37221230904) · [Proptest 37221230929](https://github.com/edwardkim/rhwp/actions/runs/37221230929)는 같은 repository·source branch·PR event·candidate SHA에서 모두 success입니다. CI Impact Policy를 포함한 최신34개 check는 success 또는 정당한 skipped/neutral로 완료됐습니다.
- 실제 CI tested merge `b0c5f9c3758fb35eb37121f15fb8566062ca2d0c`, tree `568fcbc3e837d7f61abda92043ebaf34d341868c`의 parent는 위 base/code candidate와 일치합니다. [run·job·artifact·head 증적](../assets/planet6897_20261004/ci_candidate_7575.json).
- 개별 수용 범위·잔여 판정과 로컬10,321 PASS/0 FAIL/50 skip를 유지합니다. 이 기록은 source/test/fixture/workflow를 변경하지 않는 single-parent trailing 문서 commit입니다. 새 head의 required checks·실제 fast-pass provenance·mergeability·보호 요건을 확인한 뒤 정상 경로로 병합합니다. 아직 merge 완료를 주장하지 않습니다. 병합 후 실제 merge SHA와 보정 이유를 원 contributor PR에 설명하고 통합 대체 병합으로 close합니다.

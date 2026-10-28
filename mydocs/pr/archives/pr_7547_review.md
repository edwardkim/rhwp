---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7547 기여자 변경 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 이어받는 표 조각 패딩·컷·소유 경계의 개별 수용 범위와 최신 base 통합 후보의 최종 로컬 검증을 완료했습니다. 원 PR을 직접 병합하지 않고 기록한 체리픽 통합 후보를 대상으로 합니다. #7445 문서 전체의 피델리티 승인은 아닙니다. code candidate CI는 완료됐으며 trailing head의 CI·보호 요건 통과 후 병합합니다.

## 접수·범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/7547
- 기여자: planet6897. 제목: 수정(조판): 나눔 표 일반 행을 쪽 경계에서 자르고, HWP5 문단 기준 표의 이어받은 조각에 바깥 위 여백을 연다 (#5585)
- 원 head: `ad09a2599907d782ac452b3abff91760698e0dab`. base: devel.
- 검토 branch: `review/planet6897-20261004`. 통합 base: `732047aa253e92772fd7625e4a5d7f6a9897fd73`. 적용 단계 head: `caa9d67d3f5171dae027798a8248218343ef5696`.
- 원 PR reviewer: jangster77 지정
- 경로: collaborator_external_pr 9.1.1; intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, post_merge.
- 작성 시점 CI/mergeability는 참고값이며 수용·push 전에 최신 source head를 다시 확인합니다.

## 원 기여와 체리픽 출처

| 원 commit | 통합 commit/처리 | 비고 |
| --- | --- | --- |
| `0bcbe595176f8afef0225bec7bd1ac9751fae332` | `165c11cbe1cd7613bde422ba7beeff0e0e31a329` | applied |
| `ad09a2599907d782ac452b3abff91760698e0dab` | `1a977bc3d29d3388c662a690dc84f9eb1df42a05` | applied |

## 조판 원칙과 검증 현황

| 항목 | 현재 근거와 범위 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 이어받는 표 조각의 빈 패딩·내용 컷 소유 | 개별 범위 확인 |
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

## 단계1 실제 통합 검증과 보정 분석 (2026-10-04)

전체 focused 65개 중 #7547의 새 회귀 3개는 쪽수 1개 PASS / 내용 컷·종료 여백 2개 FAIL입니다. 전체 17쪽 Native/fresh WASM TSV는 15쪽 41.14622%, 16쪽 32.75887%만 미달이고 나머지 15쪽은 95.37132% 이상입니다. 원문·PDF·통합 출력의 쪽수는 17쪽입니다. 새 회귀의 기대값을 낮추거나 지우지 않았습니다.

- [전체 fresh WASM TSV](../assets/planet6897_20261004/7547-stage1/wasm_silhouette.tsv), [WASM export·빌드 provenance](../assets/planet6897_20261004/7547-stage1/wasm_export_manifest.json). Native와 동일한 두 미달을 재현했습니다. Mac 로컬 `--no-opt` 빌드이며 Docker 최적화 통과가 아닙니다.
- [전체 Native TSV](../assets/planet6897_20261004/7547-stage1/native_silhouette.tsv), [실행 환경](../assets/planet6897_20261004/7547-stage1/native_run_manifest.json), [전체 export 쪽수](../assets/planet6897_20261004/7547-stage1/native_export.json).
- [15쪽 review PNG](../assets/planet6897_20261004/7547-stage1/native_review_p015.png), [16쪽 review PNG](../assets/planet6897_20261004/7547-stage1/native_review_p016.png)를 직접 판독했습니다. 15쪽의 첫 빈 밴드가 없고 `【구글】` 행이 먼저 나타납니다. 16쪽의 표와 다음 문단도 위로 당겨집니다. 통합 출력의 문제로 보류합니다.
- 원문 pi=169 행 3 양쪽 셀 선언 높이는 각각 14721HU입니다. 첫 조각 높이 덮어쓰기(`source_complete_frame_last_row`)를 의심한 첫 시도는 두 실패를 해결하지 못해 코드 변경을 철회했습니다. 단계2는 35개 중 33 PASS / 동일 2 FAIL입니다.
- 추가 `RHWP_TABLE_DRIFT=1` 진단에서 측정·컷 행 높이 10개는 모두 일치합니다. 첫 조각 예산은 481.6px인데 저장 프레임 초과 허용 24.3px 때문에 495.8px의 전체 행을 받아 endCut 없이 끝납니다. 실제 paint는 첫 프레임에 맞추어 행 3을 178.9px로 잘라 남은 선언 밴드를 버립니다. 이 초과 수용이 신규 일반 행 컷보다 먼저 실행되는 것이 직접 원인입니다.
- 보정 계획: 기존 `ordinary_band_row_shape`로 판별하는 비병합·글줄만 포함한 선언 일반 행은 저장 프레임 초과 허용으로 통째 수용하지 않고 기존 내용 컷 판정에 맡깁니다. 실제 예산에 들어가는 전체 행, 중첩 개체·rowspan·선언보다 큰 내용은 기존 경로를 유지합니다. 파일명·쪽 번호 분기는 추가하지 않습니다.
- 보정 후 #5585와 기존 #6123·#2439·#7379·#7234 경계 회귀를 실행하고 전체 17쪽 Native/fresh WASM TSV 및 대표 PNG를 재산출합니다. 아직 보정 후 결과와 수용 여부는 미확정입니다.

## 단계3·4 결과와 정상 반례 보류

- 초과 수용보다 일반 밴드 컷을 우선한 단계3: 기본 관련 35/35 PASS. Native 17쪽 중 16쪽은 98.90594%로 복구됐지만 15쪽은 69.32327%로 여전히 미달입니다. 대표 PNG를 직접 확인했고 앞의 blank band와 뒤 모든 행이 기준보다 아래로 밀립니다. 검사 통과를 시각 완료로 표시하지 않습니다.
- 추가 정상 반례 #1133(HWP/HWPX 각각 3쪽): 보정 전 fresh WASM 전쪽 최저 97.99746%/98.00378%, 3쪽 100%입니다. 단계3 Native는 두 벌 모두 3쪽 39.49135%로 악화됐고 기존 내용 조각 계약도 깨집니다. 이 회귀는 정상 기준이 있어 삭제하지 않습니다.
- 단계4에서 전체 내용의 높이·정렬 후 끝이 예산에 들어가는 조건도 확인했으나 39개 중 37 PASS / #1133 2 FAIL입니다. source 파일 hash와 측정 범위는 [단계4 결과](../assets/planet6897_20261004/7547-stage4/results.json)에 기록했습니다. 아직 **메인터너 보정 미완료 / 머지 보류**입니다.
- 다음 분석: 위 정렬의 빈 꼬리 밴드와 가운데 정렬의 닫힌 원본 프레임은 같은 높이 축소로 취급할 수 없습니다. 정상 반례의 원본 정렬·물리 프레임 소유를 보존하고 #6929의 빈 밴드도 실제 첫 조각과 동일한 높이로 이어받게 합니다. 코드 보정 전후의 Native/fresh WASM을 다시 비교합니다.

## 단계5 정상 반례 보존·저장 빈 밴드 보정

- 가운데·아래 정렬은 상자 높이가 바뀌면 내용 원점도 바뀌므로 닫힌 저장 프레임의 전체 행 수용을 보존합니다. 초과 수용보다 먼저 일반 밴드를 나누는 경로는 위 정렬 행에만 적용합니다.
- 일반 행의 전체 내용이 끝난 첫 조각도 독립 저장 프레임 높이를 사용합니다. 첫 행부터 마지막 행 전까지의 저장 높이를 빼서 마지막 행 높이를 정하고, 동일 선언 공간의 나머지를 다음 조각으로 넘깁니다. 용지 상한에 따른 paint 절삭으로 원본 선언 공간을 줄이지 않습니다. 문서명·쪽 번호·새 절대 좌표 분기는 없습니다.
- 관련 nextest **39/39 PASS, 0 FAIL**입니다. #5585 신규 3개와 #1133 HWP/HWPX 정상 반례, #6123·#2439·#7379·#7234·#7288 경계를 포함합니다. [실행 결과·소스 hash](../assets/planet6897_20261004/7547-stage5/results.json)에 기록했습니다.
- Native/fresh WASM 17쪽 및 정상 반례 각 3쪽 비교는 아직 대기입니다. 시각 검증과 통합 최종 검증 전에는 수용 완료로 판단하지 않습니다.

## 단계5 직접 시각 검증 완료 — 개별 보정 해소 / 통합 검증 대기

code candidate `c8c5f535cd3ebf331cadc2714c68af9fdc663a4d`에서 Native와 새 WASM을 직접 실행했습니다. WASM은 Mac 로컬 대체 빌드입니다. 판정 범위는 원 PR의 #5585 문서 전체 17쪽과 정상 반례 #1133 HWP/HWPX 각 3쪽입니다.

| 대상 | Native 최저 | fresh WASM 최저 | 미달/누락 | 쪽수 |
| --- | --- | --- | --- | --- |
| #6929 / #5585 | 95.23424% | 95.23424% | 0 / 0 | 원문·PDF·출력 17쪽 일치 |
| #1133 HWP | 97.99734% | 97.99734% | 0 / 0 | 원문·PDF·출력 3쪽 일치 |
| #1133 HWPX | 98.00366% | 98.00366% | 0 / 0 | 원문·PDF·출력 3쪽 일치 |

- 전체 TSV·실행/실루엣 manifest 및 build/source hash는 [단계5 결과](../assets/planet6897_20261004/7547-stage5/results.json)에 연결했습니다. CLI 원문·PDF의 각 hash와 fresh WASM package hash를 기록했습니다. 최초 Native 명령의 HWPX 경로 오류도 숨기지 않고 별도 올바른 경로 실행 결과로 구분했습니다.
- [14쪽](../assets/planet6897_20261004/7547-stage5/native_review_p014.png)·[15쪽](../assets/planet6897_20261004/7547-stage5/native_review_p015.png)·[16쪽](../assets/planet6897_20261004/7547-stage5/native_review_p016.png) review PNG를 직접 열었습니다. 15쪽 첫 빈 밴드 뒤에 행 4가 시작하고 행 8로 끝나며, 16쪽은 `【구글】` 행으로 시작합니다. 14쪽 마지막 내용은 그대로 남습니다. 내용 누락·중복이 없으며 아래 문단·표 종료 관계도 보존합니다. 15쪽 99.85633%, 16쪽 97.84616%입니다. 엄격 내용 픽셀 일치율은 각각 35.92%/22.57%이며 글꼴 잉크 차이가 포함되므로 2px 관용 값과 혼동하지 않습니다.
- 정상 반례 HWP/HWPX 각각 2·3쪽 PNG도 열어 가운데 정렬 행의 닫힌 프레임과 3쪽 마지막 세 행 소유가 보존됨을 확인했습니다. 3쪽은 두 벌 모두 100%입니다. fresh WASM 15쪽 원문 raster도 직접 확인했습니다. 도구 라벨·내용·범례는 판독 가능합니다.
- 원 PR의 일반 행 컷·이어받는 바깥 여백 기여를 유지하고, 기존 저장 프레임 초과 수용과의 충돌 및 빈 밴드 높이 축을 메인터너 보정했습니다. 기존 39개 경계 검사와 시각 근거로 **이 PR의 발견된 보류 사유는 해소**했습니다. 다른 PR과 최종 전체 검증 전에는 통합 머지를 승인한 것으로 쓰지 않습니다.
- merge 후 contributor 설명에는 원 기여, 초과 수용 충돌 보정 이유, 정상 가운데 정렬 반례 보존, 정확한 CI/merge SHA와 위 14~16쪽 증적을 함께 기록합니다. 아직 게시·close하지 않았습니다.

## 공백 잉크 보정 뒤 재확인

후속 생산 보정 `4cfe96e52`의 Native/fresh WASM에서도 이 문서와 정상 반례의 전체 TSV가 90% 이상으로 유지됩니다. 현재 32쪽 통합 시각 실행은 [단계8 결과](../assets/planet6897_20261004/7534-stage8/results.json)에 입력/build 출처와 함께 기록했습니다. 기존 개별 범위 충족 판정은 유지하며 최종 전체 검사·다른 PR 검토는 대기입니다.

## 리베이스 전 통합 로컬 검증 — 2026-10-05

전체 회귀 head `aa10d4093f60c4e0c12aa186f2464861acbf89ea`, 최종 lint/build·fresh WASM·Skia head `42ef60c96`(테스트 모듈 순서만 정리), base `731de9e1b4bb946d76f35108ed7e186ebe4ebecb`입니다. 전체 nextest는 **10,318 PASS/0 FAIL/50 skip**,563.885초, threads=8로 완료했습니다. Format·Native/WASM32 Clippy·workspace build·workspace all-target Clippy·manifest/unit tier 정책을 통과했습니다. fresh WASM은 Mac 로컬 `--no-opt` 대체 빌드이며 Docker 최적화 검증으로 보고하지 않습니다. Skia lib·그림 placeholder·직접 PDF 출력 범위도 실제 실행했습니다. Studio 전체1,815 PASS/0 FAIL/2 skip와 TypeScript/Vite build도 완료했습니다.

[정확한 head·명령·결과·원 source CI·개별 시각 범위](../assets/planet6897_20261004/final_validation_before_7574.json)를 참고합니다. 시각 자료는 각 단계의 생산 code head와 입력/PDF hash로 고정되어 있으며, 이후 변경은 테스트·문서 범위입니다. 최종 fresh WASM hash와 기존 단계의 hash, 생산 본문 동일성 증적을 함께 기록합니다. 대용량 미달 문서는 #7445의 기존·추가 이관 기록으로 추적하고 그 문서 전체의 정확성을 승인하지 않습니다. 위의 “통합 검증 미완료/대기”는 해당 단계 작성 시점의 기록이며 최종 로컬 판정은 이 절을 따릅니다. 통합 CI는 아직 미실행입니다.

## Merge 후 contributor PR comment 계획

기여자의 이어받는 표 조각 패딩·컷·소유 경계 결과와 이 문서에 적은 추가 보정의 원인·범위를 한국어 존댓말로 구분해 설명합니다. 실제 통합 merge SHA·CI URL·Visual Sweep 정본과 이관 잔여 범위를 안내하고, 통합 대체 병합으로 원 PR을 close합니다. UTF-8 `--body-file`로 게시 후 API로 본문·close 상태를 재확인하며 중복 댓글은 게시하지 않습니다. [대표 PNG](../assets/planet6897_20261004/7547-stage5/native_review_p015.png)는 실제 merge SHA에 고정한 raw URL로 표시합니다.

### 최종 출력 provenance 재확인

최종 Native CLI(`native-skia`)와 실제 Chrome/fresh WASM으로 수용 범위20개 target·104쪽씩을 다시 내보냈습니다. **두 경로 최저90.80846%,90% 미만0쪽**입니다. SVG가 byte-identical인 쪽의 raster만 재사용하고, 달라진 SVG·글꼴 정책 블록은 새 raster로 비교했습니다. [전쪽 수용 범위 TSV·입력/PDF hash·현재 SVG hash와 재사용/재출력 구분](../assets/planet6897_20261004/final-visual/results.json). #7445 이관 문서의 전체 수용 주장은 아닙니다. 자산/속성 기여는 원래의 제한된 검토 범위를 유지하며 Studio crop은 동일한 최종 WASM의 실제 Canvas2D/CanvasKit 증적을 유지합니다.

최종 출력에서 이전 SVG와 달라진1·4쪽을 Native/WASM 모두 새 raster로 확인했습니다. [Native1쪽](../assets/planet6897_20261004/final-visual/7547a/native/review_001.png)·[Native4쪽](../assets/planet6897_20261004/final-visual/7547a/native/review_004.png)·[WASM4쪽](../assets/planet6897_20261004/final-visual/7547a/wasm/review_004.png)을 직접 판독했습니다. 본문·표·그림·checkbox·후속 문단의 소속을 유지하며 수용 범위17쪽 최저95.23424%입니다.

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

최신 [대표 review PNG](../assets/planet6897_20261004/final-visual-rebased/7547a/native/review_004.png)와 [overlay PNG](../assets/planet6897_20261004/final-visual-rebased/7547a/native/overlay_004.png)는 merge 후 실제 merge SHA에 고정한 raw URL로 contributor comment에 표시합니다.

## 통합 PR #7575 코드 후보 CI 완료 — 2026-10-05

- [PR #7575](https://github.com/edwardkim/rhwp/pull/7575)의 code candidate `86a6fe85087e4371ac929df2036461ace7a0e986`, base `732047aa253e92772fd7625e4a5d7f6a9897fd73`에서 Full 검증이 완료됐습니다. preflight는 `no-green-build-candidate`로 Full을 선택했습니다. trusted job 성공을 재사용으로 해석하지 않았습니다.
- [CI 37221230992](https://github.com/edwardkim/rhwp/actions/runs/37221230992) · [CodeQL 37221231003](https://github.com/edwardkim/rhwp/actions/runs/37221231003) · [Render Diff 37221230707](https://github.com/edwardkim/rhwp/actions/runs/37221230707) · [Adapter 37221230904](https://github.com/edwardkim/rhwp/actions/runs/37221230904) · [Proptest 37221230929](https://github.com/edwardkim/rhwp/actions/runs/37221230929)는 같은 repository·source branch·PR event·candidate SHA에서 모두 success입니다. CI Impact Policy를 포함한 최신34개 check는 success 또는 정당한 skipped/neutral로 완료됐습니다.
- 실제 CI tested merge `b0c5f9c3758fb35eb37121f15fb8566062ca2d0c`, tree `568fcbc3e837d7f61abda92043ebaf34d341868c`의 parent는 위 base/code candidate와 일치합니다. [run·job·artifact·head 증적](../assets/planet6897_20261004/ci_candidate_7575.json).
- 개별 수용 범위·잔여 판정과 로컬10,321 PASS/0 FAIL/50 skip를 유지합니다. 이 기록은 source/test/fixture/workflow를 변경하지 않는 single-parent trailing 문서 commit입니다. 새 head의 required checks·실제 fast-pass provenance·mergeability·보호 요건을 확인한 뒤 정상 경로로 병합합니다. 아직 merge 완료를 주장하지 않습니다. 병합 후 실제 merge SHA와 보정 이유를 원 contributor PR에 설명하고 통합 대체 병합으로 close합니다.

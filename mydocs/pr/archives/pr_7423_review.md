---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7423 검토 — 수정(조판): 쪽을 끝내는 조각의 마지막 행 상자를 배치 뒤에 접는다 (#6976, #7063)

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 통합 코드·직접 시각 검증은 아직 실행 전입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7423, 작성자 planet6897, base `devel`, 정확한 head `07ec370f2c035d81007efffd415e3b740e752e55`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `3cb7daba69fc734879e672434f1547c4a7d24100` | candidate | 수정(조판): 자리차지 RowBreak 조각의 바깥 위 여백을 원본 HWPX 계보에도 연다 (#7063) |
| `040fa9b2bf9a343c6117a670235be1f518e06963` | candidate | 수정(조판): 쪽 중간 조각의 상자·내용·정렬 높이를 가른다 (#7063 레인②) |
| `e3a4456bd263884d73fe59f759421b1ce222baf6` | candidate | 수정(조판): 쪽을 끝내는 조각의 마지막 행 상자를 배치 뒤에 접는다 (#6976) |
| `07ec370f2c035d81007efffd415e3b740e752e55` | merge excluded | Merge branch 'devel' into work/7063-lane1 |

## 변경 범위와 검토 계획

- 원 PR 변경 823줄 추가·36줄 삭제·32파일입니다.
- 생산 경로: `src/document_core/queries/cursor_rect.rs`, `src/renderer/layout/table_partial.rs`, `src/wasm_api/tests.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

체리픽·충돌 보정·최종 검증은 아직 미실행입니다. 분석·코드·결과 보고·commit 순서로 단계별 갱신합니다.


## 단계1 선행 commit 충돌 분석과 보정

- 원 선행3cb7daba는 issue4771의 과거 절대좌표 기대에 여백283HU 이동량을 더합니다. 최신 devel은 이미 문단/그림 소유·셀 관계 검사로 교정되어 이 좌표 지역변수가 없습니다. 최신 의미 회귀를 유지하고 사용하지 않는 픽셀 이동량을 다시 도입하지 않습니다.
- text_overlap baseline의 최신 PrEP 보정 주석과 source의 rowbreak-problem-pages 유보 주석이 충돌했습니다. #7505에서 제외한 실제 대형 차단 행을 독립 전쪽 검증 없이 다시 넣거나 상한을 완화하지 않고 현재 baseline을 유지합니다. 생산 경로의 HWPX 원본 여백 개방은 적용하고, paint·예산의 서로 다른 계보 조건과 분할 반례는 통합 검토에서 확인합니다. 적용 단계의 검증은 아직 미실행이며 최종 판정은 보류입니다.


## 단계2 이어받기 충돌 분석과 보정

- source040fa9b2는 #7505에서 실제 대형 미달 입력으로 보류한 issue6923의 절대좌표 함수를 다시 추가합니다. 최신 유지 검사·보류 범위를 보존하며 독립 전쪽 검증 없이 이 함수를 되살리지 않습니다. 같은 source의 off_canvas hwpx_sample2 행 재추가도 현재 baseline으로 유지합니다.
- 생산 코드의 상자 높이·내용 예산·정렬 높이 분리는 우선 누적 적용합니다. source의 `d7063_pin=true` 상수로 조건을 무조건 여는 부분은 일반성 검토 대상으로 남깁니다. 컷·행/내용 소유·예산/paint의 소비 일치와 정상 반례 검증 전에는 수용하지 않습니다.


## 단계3 셀 배치·테두리 충돌 분석과 보정

- sourcee3a4456b의 마지막 행 paint 접기 인자가 최신 devel의 첫 저장 프레임 정렬 인자와 같은 위치에 추가되어 충돌했습니다. 셀 함수와 호출에 세 인자를 함께 전달해 각 상태를 유지합니다.
- source 접기 후처리와 최신 어울림 표 이어받기 열린 윗변 처리가 같은 함수 말미에 있어 충돌했습니다. 접기 후처리와 열린 윗변 처리, 이어지는 종료 프레임 확장을 모두 보존하며 어느 쪽 코드도 통째 교체하지 않습니다. 실제 접기·마지막 행 소유·셀 내용 포함·후속 문단과 예산 예약 관계는 집중 검증에서 확인합니다. 아직 수용/전수 통과가 아닙니다.

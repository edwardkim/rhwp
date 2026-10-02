---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7411 검토 — fix(layout): 원점이 접힌 프레임과 저장 행의 좌표계를 맞춘다 (#7408)

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 체리픽은 완료했고 통합 검증 중입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7411, 작성자 planet6897, base `devel`, 정확한 head `e116877a532eb02370fd7af35c982f407315d3b3`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `74b6e664e9ff26b7fd4f94bc4ad2c41a32463003` | candidate | fix(layout): 원점이 접힌 프레임과 저장 행의 좌표계를 맞춘다 (#7408) |
| `e116877a532eb02370fd7af35c982f407315d3b3` | merge excluded | Merge branch 'devel' into fix/7408-stored-lineseg-rewrap |

## 변경 범위와 검토 계획

- 원 PR 변경 320줄 추가·9줄 삭제·8파일입니다.
- 생산 경로: `src/renderer/layout_frame.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

고유 source commit 63개 체리픽을 완료했습니다. 출처와 보정은 [적용 원장](../assets/planet6897_green_20261002/applied_commits.json)에 기록했습니다. 현재 후보 `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`의 Native Clippy는 exit0이며 전체 nextest는 실행 중입니다. 최종 회귀·시각 검증은 미완료입니다.


## 단계1 충돌 분석과 보정

- 최신 devel의 KoPub 양쪽 정렬 공백 필드와 원 PR의 원점 권위 필드가 같은 struct/생성자 위치에 추가되어 충돌했습니다. 두 필드와 각 초기값을 함께 유지합니다. 원점 보류 시 첫 슬롯 이동량을 사용하되 폭·슬롯 간격의 정확한 검증은 유지합니다. 새 주석은 한국어로 정리합니다.
- 원 PR baseline 패치는 #7505에서 실제 대형 차단으로 제외한 issue6764·issue6776 등의 행을 다시 추가하려 했습니다. 통합 후 독립 전쪽 검증 전에는 이전 제외를 되돌리거나 수치 제한을 완화하지 않고 최신 devel baseline 계약을 보존합니다. 적용은 승인 판단이 아니며 집중·시각·통합 전수 검증은 아직 미실행입니다.

### 누적 후보 검증 시작

- 검증 코드 head: `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`. Native Clippy exit0(34.38초). 전체 nextest release-test/threads8/no-fail-fast 실행 중이며 통합 시각 검증은 아직 미완료입니다. 원 PR의 green CI와 구분합니다.

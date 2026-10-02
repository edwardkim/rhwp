---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7471 검토 — 수정(각주): 번호 문단도 문단 모양의 내어쓰기·정렬을 받는다 (#7469)

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 체리픽은 완료했고 통합 검증 중입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7471, 작성자 planet6897, base `devel`, 정확한 head `492aef1eefd06240a6a49d52f6b10c4a15acc657`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `d481852b4968fdc2f1eee9dd0e6f2d05784c9e74` | candidate | 수정(각주): 번호 문단도 문단 모양의 내어쓰기·정렬을 받는다 (#7469) |
| `492aef1eefd06240a6a49d52f6b10c4a15acc657` | candidate | 증적(#7469): 각주 번호 문단 Visual Sweep 대표 쪽 review·overlay |

## 변경 범위와 검토 계획

- 원 PR 변경 295줄 추가·14줄 삭제·18파일입니다.
- 생산 경로: `src/renderer/layout/picture_footnote.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

고유 source commit 63개 체리픽을 완료했습니다. 출처와 보정은 [적용 원장](../assets/planet6897_green_20261002/applied_commits.json)에 기록했습니다. 현재 후보 `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`의 Native Clippy는 exit0이며 전체 nextest는 실행 중입니다. 최종 회귀·시각 검증은 미완료입니다.


## 단계1 각주 분할·번호 경로 충돌 분석과 보정

- 최신 devel은 각주 측정과 paint가 FootnoteParagraphPlacement를 공유하고 저장 빈 줄·원래 autoNum 서식을 보존합니다. source의 번호 일반 문단 조합을 이 공유 route 위에 연결하고 spacing이 전달되는 in_frame 호출을 유지합니다. 첫 fragment 번호 한 번과 후속 tail 무번호 계약을 보존합니다.
- 새 일반 문단 경로에서도 저장 autoNum의 원래 접두/접미를 사용합니다. 여러 선두 번호 슬롯은 기존 번호 경로로 보내 원본 슬롯 서식을 잃지 않도록 합니다. 번호의 가시 표현을 읽는 기존 helper만 display_or_text로 교정합니다.
- source에 남아 있던 #7505 이전의 대형 테스트 함수·픽셀 hash/count·겹침 상한218→251을 독립 검증 없이 되살리지 않습니다. 최신 의미 검사와 #7445 보류 표식을 유지하고 baseline 상한을 완화하지 않습니다. 통합 출력의 번호·문단 모양·각주 수량·원본 쪽수와 기존 경계 검증 전에는 승인하지 않습니다.

### 누적 후보 검증 시작

- 검증 코드 head: `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`. Native Clippy exit0(34.38초). 전체 nextest release-test/threads8/no-fail-fast 실행 중이며 통합 시각 검증은 아직 미완료입니다. 원 PR의 green CI와 구분합니다.

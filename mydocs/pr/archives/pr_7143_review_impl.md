---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7143 메인터너 보정 계획

[검토 결과](pr_7143_review.md)는 머지 보류다. 현재 작업은 local review이며 원격 source 수정·새 PR·merge는 하지 않았다.

1. 시작점은 `upstream/devel` `38af2aae3` + 원 PR `34e1186f4`를 체리픽한 `codex/pr7143-review-20260914` / `4a9546f8c`다. 원 contributor commit의 출처를 보존한다.
2. 보정 시 새 head/base를 먼저 확인한다. direct source push 경로를 선택하려면 `planet6897/rhwp`의 일반 push 권한과 이 PR의 maintainer 수정 권한, 정확한 branch의 push 가능성을 구분해 확인한다. 불가하면 현재 누적 branch에서 원본 저장소 대상 통합 PR 경로를 사용한다.
3. 분할 조각이 실제 소유하는 start/end unit과 높이·예산 결정을 공통 결과로 만든다. 일반 splittable 행도 renderer의 증가분을 예약하고, 증가분이 fit하지 않으면 실제 컷/이월을 생성한다. `start_cut` 누락과 기존 native two-row owner 경로의 차이도 확인한다. 문서별 예외나 임의 slack으로 덮지 않는다.
4. 별도 code/test commit으로 새 보정을 기록한다. 기존 4건, 메모리 축소 p3 body overflow, 원래 h만 fit하는 경계, 시작 컷 및 다수 rowspan 누적 경계를 검증한다. 합성 입력은 정상 한컴 PDF 사례와 분리한다.
5. 새 code head의 필수 lint·변경 범위 CI와 직접 visual sweep을 완료한다. target 문구뿐 아니라 본문/다음 행 위치, #6981의 p82·152 잔여 범위를 확인한다. 정상 기준으로 baseline을 완화하지 않는다.
6. CI가 녹색인 뒤 최종 검토·오늘할일·증적을 동일 처리 branch의 trailing commit으로 정렬한다. 기존 HWP/PDF는 같은 경로로 재사용하고 누락 파일만 보존한다. source head에 직접 넣는 경우 최신 base의 오늘할일을 통째 복사하지 않고 merge simulation·링크·기존 기록 보존을 확인한다.
7. 작업지시자의 push/PR/merge 승인 범위에 따라 진행한다. 최종 exact head CI와 MERGEABLE/CLEAN 확인 후 merge하고, post_merge 순서로 duration 결과·devel sync·관련 issue/PR comment·본 작업 소유 산출물 정리를 완료한다. contributor fork branch와 공유 target은 보존한다.

현재 보정 SHA는 없으며 3번 이후는 미실행 계획이다. rollback은 새 보정 commit에 한정하고 contributor 원 변경 또는 다른 작업의 파일을 되돌리지 않는다.

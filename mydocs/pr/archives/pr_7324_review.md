---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-22
---

# PR #7324 검토 기록 — HWP5 표 `pageBreak`·`repeatHeader` 저장 보존

- 원 PR: [#7324](https://github.com/edwardkim/rhwp/pull/7324)
- 관련 이슈: [#7323](https://github.com/edwardkim/rhwp/issues/7323) (`Closes`; 통합 PR merge 뒤에만 종료 확인)
- 작성자: `zlzlzlmo` (기존 기여자; 이전 PR #6773 확인)
- 원 code head: `ff5d6e855048992886ea06fea1e991b6588c30b1`
- 통합 검토 branch: `review/open-prs-20260921`
- 적용 commit: `bd974e19b42e40e1207ed7de05fc33f3acd2f6c8` (`cherry-pick -x`, 충돌 없음)
- 작성 시점 원 PR 상태: `MERGEABLE / CLEAN`, 원 head CI 성공

## 변경과 검토

`raw_table_record_attr` 전체를 원본대로 다시 쓰던 HWP5 serializer를 바꿨다. 원본 attr의
기타 bit는 보존하되 page break bit 0–1과 repeat-header bit 2만 현재 IR 값으로 반영한다.
비표준 page-break raw 값 3은 IR이 변경되지 않은 저장 왕복에서 보존한다.

`setTableProperties`로 명시적으로 바뀐 값이 HWP5 저장·재파싱 후에도 남는지와, raw attr의
비대상 bit가 보존되는지를 같은 실물 fixture로 검사한다. serializer 구조 보존 변경이며
rhwp renderer/layout/paint 출력 경로는 바꾸지 않아 Visual Sweep은 비대상이다.

## 검증 입력과 결과

- 실제 검증 입력: `samples/2010-01-06.hwp` (저장소 기존 commit, SHA-256
  `d2562d9219fc1d491dd6b9f6d787314153246efb79e18c42c63830ac22194958`).
- integration head `bd974e19b`에서 `cargo nextest run --cargo-profile release-test -j 8
  --test regression_suite_027 -E 'test(issue_7323_table_page_break_save)'` — **3 passed**.
- 각 test는 `pageBreak=0/1/2`, `repeatHeader` 토글, 무편집 raw attr 보존을 HWP5
  serialize → parse로 직접 확인한다. HWPX/PDF를 실제 검증 입력으로 쓰지 않았다.

## 공통 조판 원칙과 입력 보존

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | raw attr를 기반으로 수정 축만 덮고 비표준 3의 무편집 보존을 분리 |
| 측정·배치, 분할·이어받기, 줄 소속 | 비해당 | renderer/typeset 경로를 변경하지 않음 |
| 사례와 증거의 독립성 | 충족 | 실제 HWP fixture의 serialize·reparse 결과를 API 결과와 분리해 검사 |
| 기준값 변경 | 비해당 | baseline/golden 변경 없음 |
| 검증 입력 commit 포함 | 충족 | 위 HWP fixture는 추적된 동일 파일이며 실제 명령도 저장소 경로를 사용 |

## 최종 판정

**승인.** 통합 PR merge 뒤 closing keyword가 가리키는 #7323의 자동 종료 여부를 확인하고, 원 PR은
통합 merge SHA를 붙인 contributor comment 뒤에 처리한다. 통합 PR 생성·merge 전에는 최신
integration head의 CI와 작업지시자 승인을 다시 확인한다.

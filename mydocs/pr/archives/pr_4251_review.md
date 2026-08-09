---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4251 검토 - IME 조합 replace 거부 뒤 캐럿 재정박

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4251](https://github.com/edwardkim/rhwp/pull/4251) / @humdrum00001010 |
| contributor 원 head | `ec61a349fadb15ab68c4eccccf3cd013a2a458ec` |
| base / 규모 | `devel`, 8개 파일, +260/-4 |
| 관련 이슈 | [#4245](https://github.com/edwardkim/rhwp/issues/4245), [#4150](https://github.com/edwardkim/rhwp/issues/4150) |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

deferred replace가 오래된 조합 범위를 거부할 때 예외를 입력 handler 밖으로 전파하지 않는다. 현재
캐럿에 조합 상태를 재정박하고 삽입을 재시도하며, 재시도도 실패하면 해당 업데이트만 버린다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- 통합 후보에서 `composition-replace-reanchor.test.ts`를 포함한 Studio focused 27건과 전체
  `npm test` 813건이 통과했다.
- #4261의 실제 HWP/HWPX 셀 Enter Chromium E2E도 두 형식 모두 115쪽, Enter flush 0, split 1로
  통과해 인접 입력·pagination 경로가 함께 유지됨을 확인했다.

**통합 수용 권고.** 통합 PR 본문의 closing keyword에는 #4245와 #4150을 모두 포함한다.

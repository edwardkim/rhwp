# chrisryugj HWP3 원격 확정 — 2026-11-09 KST

| 대상 | 검증·보존 head | 실제 결과 |
| --- | --- | --- |
| 통합 #8851 | `9fa72c60365f6b1f3f168223e7ac56d2fa4b6402` | merge `50b6098853afdb45625e5d505dc3b2070998fc46`, 2026-11-09 20:03:51 KST |
| 원 #8789 | `3c2c5fdf1bea13912b2eefaf4008ea974a2b4fd9` | MERGED, 원 PR 기록의 merge `158f45c9795d893362441908e7f0a57ca743fc03`도 최종 devel에 포함 |
| 원 #8791 | `383f66acca8c966da9a198fd046c3da0ed94c3b6` | MERGED, 원 PR 기록의 merge `a90852db333d145ede87c15c9e5064029d20ab20` 뒤 셀 보정까지 최종 devel에 포함 |

원 두 PR은 간접 수용으로 자동 완료돼 별도 close를 실행하지 않았다. GitHub 자동 종결 연결은 없지만, 실제 관련 미완료 이슈는 아래와 같이 별도로 확인했다. 원 기여 SHA와 fork branch는 유지한다.

[최종 검토·CI·self-review·중립 안내의 실제 의미](../../archives/pr_8851_review.md)를 따른다. 제품 CI 1회 실행에서 A–E 총 12,819 PASS, 집중 8건 실행을 확인했다. 로컬 전체는 추가 workspace·CLI binary 단위 검사까지 13,013 PASS다. CodeQL 실제 Rust 분석은 결과·오류 0건이며 PR ref의 열린 보안 경고도 0건이다. 다른 세 언어는 경로 정책에 따른 skip이며 구성 누락 neutral 안내를 실패·실제 분석 성공과 구분했다.

[실행 시간 메타데이터 갱신](https://github.com/edwardkim/rhwp/actions/runs/42605953144)은 성공했다. 이번 PR CI의 B/C/D/E attempt 1 실측으로 `ready=true`·`published=true`이며 `ci-metrics/nextest-target-durations`에 commit `c1c416d805795ae4ba910c37c5843aa007e1953a`를 게시했다. source merge는 `50b6098853afdb45625e5d505dc3b2070998fc46`이다. 병합 후 제품 검증 CI를 다시 실행하지 않았다.

후속 문서는 maintainer option M의 승인된 mydocs 운영 기록으로 반영한다. 이 기록 push·devel 동기화 뒤 원 PR별 감사 댓글 2건과 이미지/API·브라우저 확인을 수행하고, 전체 output·공유 debug와 검토 전용 branch/worktree를 정리한다. 마지막 실제 결과는 `target/pr-review/cleanup-pr8851-20261109.json`에서 확인한다. 임시 로그나 중간 산출물은 백업·압축·이관하지 않는다.

## 관련 이슈 종료 여부 추가 점검

2026-11-09 KST 작업지시자의 “관련된 이슈도 close했는지 확인 후 처리” 지시에 따라 자동 종결 연결과 실제 원본·해결 범위를 분리해 다시 조사했다. `closingIssuesReferences`는 원 PR 2건과 통합 PR 모두 0건이며, 추가 댓글 게시 전 timeline 조회의 cross-reference도 세 PR 사이의 참조뿐이었다. 이것은 관련 기존 이슈가 없다는 뜻이 아니다.

| 관련 이슈 | 확인 상태 | 이번 처리 |
| --- | --- | --- |
| [#5860](https://github.com/edwardkim/rhwp/issues/5860)·[#6380](https://github.com/edwardkim/rhwp/issues/6380) | 이미 CLOSED/completed | 과거 문자 매핑 해결 건이다. 이번에 새로 닫았다고 기록하거나 중복 close하지 않는다. |
| [#7642](https://github.com/edwardkim/rhwp/issues/7642)·[#1692](https://github.com/edwardkim/rhwp/issues/1692) | 이미 CLOSED/completed | 기존 조판·글색상 해결 이력이며 상태를 유지한다. |
| [#7807](https://github.com/edwardkim/rhwp/issues/7807) | OPEN 유지 | 같은 SO-SUEOP 원본의 전쪽 시각 미달이 남는다. 문자 복원·남은 표시 차이·완료 조건 미충족을 [후속 댓글](https://github.com/edwardkim/rhwp/issues/7807#issuecomment-6738899526)로 게시했다. |
| 부모 [#7445](https://github.com/edwardkim/rhwp/issues/7445) | OPEN 유지 | #7807을 포함한 미완료 하위 범위가 있어 전체 종료하지 않는다. |
| [#8624](https://github.com/edwardkim/rhwp/issues/8624)·[#6671](https://github.com/edwardkim/rhwp/issues/6671) | OPEN 유지 | 각각 HWP5 변환본과 HWP5 2022 저장본의 다른 증상이다. 이번 HWP3 원본의 전쪽 결과를 해결 증거로 대용하지 않는다. |

#7807의 HWP3 원본 SHA와 독립 Print SHA는 이번 검증 원본과 같다. 후보 전 46쪽의 Native/fresh WASM 최저는 61.69802%·61.97235%여서 이슈의 전쪽 90% 조건을 충족하지 못한다. HWPX/2024 Print 전쪽 검증과 기존 저장·미주 후속 범위도 이번 문자 source 대조로 완료 처리하지 않는다. **PR 수용 예외는 #7807 완료 조건의 예외가 아니다.**

새로 닫을 수 있는 연결 이슈는 확인되지 않아 추가 close는 0건이다. 해결 사실은 댓글로 연결하고 미해결 이슈는 유지했다. 과거 원 해결 PR·현재 통합 source·검증 증거는 기존 최종 검토와 원장을 재사용하며 새 제품 빌드나 CI 재실행을 하지 않는다.

검색에 함께 나타난 #6976의 총괄 pagination 계약, #6611의 순수 HWPX 흐름, #7632의 편집 중 행 삭제, #7803의 다른 원본 및 #8232의 전수 검사 정비도 이번 HWP3 문자 해석·입력 격자 보정의 완료로 대체하지 않는다. 검색 결과에 등장했다는 이유로 종결하지 않았다.

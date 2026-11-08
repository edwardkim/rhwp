---
kind: plan
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-17
---

# donggyun112 열린 PR 3개 누적 검토 계획

## 승인 범위와 시작점

사용자 요청: 열린 PR을 체리픽 통합한 뒤 개별 검토하고 필요하면 메인터너 보정한다.
리뷰어 선행 지정 요청에 따라 세 PR 모두 `jangster77` 요청을 게시하고 API로 확인했다.
기본 작업공간이 clean하므로 별도 worktree 없이 `review/donggyun112-20261017`에서 진행한다.
기준 devel은 `f9d2ab17ab438c6bed97d91c3c2a98a5be27e573`이다.

| 원 PR | 원 head | 검토 범위 |
| --- | --- | --- |
| [#7978](https://github.com/edwardkim/rhwp/pull/7978) | `5558cbcfb8e0d2cb30422e12aa719288371164a5` | [#7977](https://github.com/edwardkim/rhwp/issues/7977): HF/FN 문자 위치의 Unicode scalar 계약 |
| [#7985](https://github.com/edwardkim/rhwp/pull/7985) | `5dd0c86ac1a86c977e9d80a7fe263ac60358f720` | [#7983](https://github.com/edwardkim/rhwp/issues/7983): 단일 구역 코어 계약 밖의 선택 변경 사전 거절 |
| [#7988](https://github.com/edwardkim/rhwp/pull/7988) | `a7c884a79efed47ad59d0fbf78227cc5dbe46624` | [#7984](https://github.com/edwardkim/rhwp/issues/7984): 대소문자 무시 검색·치환의 원문 문자 범위 |

원 기여자는 기존 기여자로, 첫 기여자 절차를 새로 적용하지 않는다.
원 author와 `cherry-pick -x` 연결을 보존하며 fork source를 수정하지 않는다.
원 PR CI와 통합 후보 로컬 검증을 구분한다. PR 생성·push·merge는 실제 승인 범위에 따라 처리한다.

## 독립 계약과 소비 경로

- #7977: 코어 문단 편집 위치는 Unicode scalar 단위다. `charCount` 결과가 일반 입력의
  HF/FN 커서와 입력 command의 초기/execute/redo 문맥으로 전달되는지 확인한다.
  후속 글자·문단 끝·중간 삽입, undo/redo 반복, 한글·이모지·보조 평면 한자를 대조한다.
- #7983: 코어 `deleteRange`가 받지 않는 끝 구역을 누락한 채 다른 내용을 지우면 안 된다.
  `canDeleteSelectionInFormMode`의 판정부터 실제 키·일반 입력·IME·cut/paste와
  `DeleteSelectionCommand` 생성까지 추적한다. 거절 때 모델·선택·히스토리 보존,
  같은 구역 편집과 양식 Tab 탐색의 정상 대조군을 확인한다.
- 모델·저장 조판 정보·치수·paint의 변경 여부를 실제 diff와 소비 경로에서 판정한다.
  편집기 위치/사전 거절만 바꾸는 범위는 출력 피델리티 개선으로 주장하지 않는다.
  조판 영향 보정이 필요해지면 Native/fresh WASM·독립 Print 시각 검증 경로를 추가한다.

## 실행 순서와 완료 조건

1. 위 순서로 원 기능 커밋을 누적 적용하고 충돌·추가 변경을 기록한다.
2. 원본 단위 검사의 수정 전 FAIL/후 PASS를 실제 구현의 격리된 메모리 변이로 확인한다.
3. 변경 범위의 타입 검사·전체 Studio 단위·production build·E2E manifest 검사를 수행한다.
4. 공용 `target/pr-review`에서 fresh WASM을 준비하고 실제 Chrome의 해당 E2E를 수행한다.
   Cargo 빌드를 겹치지 않으며 배포 파일을 fresh source 증거로 대용하지 않는다.
5. 발견한 결함은 원 기여와 별도 메인터너 커밋으로 보정하고 영향 검사를 재수행한다.
6. 원 PR별 archive review·적용 기록에 실제 source SHA·명령·결과·한계를 기록한다.
   통합 PR을 만들 때는 최신 base를 확인하고 실제 code CI 이후 같은 PR의 문서 trailing으로
   개별 검토와 오늘할일을 완료한다.

실행 로그와 원본 API 응답은 ignored `output/pr-review/donggyun112-20261017/`에 기록한다.
필수 증거가 남으면 완료·승인으로 표시하지 않는다.

## 초기 2개 PR의 로컬 검토 완료 이력

통합 code 후보 `a7c9e30ffebc843d5f5763f349f79ad3ea93cfb1`에서 로컬 필수 검증과 실제 브라우저 검사를 완료했다.
[PR #7978 검토](../pr/archives/pr_7978_review.md)와 [PR #7985 검토](../pr/archives/pr_7985_review.md)에
원 기여·보정·증거·남은 원격 단계를 구분했다. 원격 게시 승인과 통합 head CI는 아직 남아 있다.

## #7988 추가 통합과 재검증

사용자가 PR 생성을 승인한 뒤, 게시 전에 #7988을 추가하여 다시 검증하도록 요청했다.
원격 브랜치·통합 PR은 아직 생성하지 않았다. #7988도 reviewer `jangster77`을 먼저 요청하고
API로 확인했다. 원 기능·검사 6개 커밋을 앞선 후보 위에 누적 적용한다.

독립 계약은 검색·치환의 위치와 길이가 원문의 Unicode scalar 범위를 표현해야 한다는 것이다.
`find_in_text → SearchHit/GrepMatch → replace_one/all/nth → 실제 본문·셀·글상자·수식 변경`을
추적한다. 소문자 확장 양방향에서 후속 X 보존, search/grep의 원문 길이와 대소문자 구분
대조군을 확인한다. 일부 확장 문자만 매치하거나 매치가 겹치는 기존 정책은 별도로 구분한다.

이번에는 Rust source·정식 회귀가 추가되므로 앞선 Studio 전용 검증 범위를 확대한다.
정식 4개 회귀의 수정 전 FAIL/후 PASS, Rust lint 묶음·workspace build·전체 release-test,
fresh WASM과 실제 Chrome의 검색·모두 바꾸기 및 앞선 두 PR E2E를 재검증한다.
Cargo는 공용 `target/pr-review`에서 jobs 8로 순차 실행한다. 기존 성공을 새 Rust 후보의
통과로 바꾸지 않고 source SHA·명령·종료 코드·로그를 새 원장으로 구분한다.

## 최종 3개 PR 로컬 검증

사용자 요청으로 최신 `7d01ef610f378157b79d36ef23e8ed4a3c8ab051`에 rebase했고 검증 head는 `2dee7730306dcc7f57615c87c25cb09af3c90f63`다.
[추가 PR 검토](../pr/archives/pr_7988_review.md)에 재실행한 전체 회귀·fresh WASM·실제 브라우저의
최종 결과를 기록했다. 최초 base와 producer는 이력으로 보존한다. 사용자 원격 게시 승인은 받았고
통합 PR의 실제 code CI·문서 trailing·최종 CI는 별도 원격 단계다.

## 통합 PR code CI 완료

통합 [PR #7997](https://github.com/edwardkim/rhwp/pull/7997)의 `fd3d97b0b9effa2d0d4a1f65f47e3475ba4ec5c8`에서 실제 CI·CodeQL·Render Diff·
Adapter·Proptest와 정책 검사가 완료됐다. 사용자가 병합·후속 처리를 승인했으며 원 PR별 검토와
2026-10-17 오늘할일을 같은 PR의 trailing 기록으로 반영한다. 최신 trailing CI·정상 merge와
후속 이슈/원 PR 처리·소유 branch·root output 정리는 남은 조건이다.

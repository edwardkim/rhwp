---
kind: plan
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-18
---

# donggyun112 후속 PR 2개 통합 검토

## 승인·기준·경로

사용자는 열린 PR의 체리픽 통합·검토, 완료 후 PR·CI·정상 병합·후속 처리를 승인했다.
기존 #7997 통합 이후 새로 열린 #8000·#8002가 대상이며 원 contributor fork는 보존한다.
메타데이터 접수 후, 로컬 fetch·체리픽 전에 두 원 PR에 jangster77 reviewer를 지정하고 API로 확인했다.

- 기본 작업공간 `/home/tsjang/rhwp`, clean한 devel에서 작업 시작.
- 기준 `upstream/devel`: `4296fa045df2743a0eaa1b0cc71b2a3c1dd90bb6`.
- 작업 branch `review/donggyun112-20261018`, 별도 worktree 없음.
- collaborator 매개 외부 PR 통합 경로. 원 PR별 archive review/impl는 통합 code CI 성공 뒤 같은 PR의 trailing으로 반영.
- 공용 `/home/tsjang/rhwp/target/pr-review`를 순차 사용, 기존 사용자 선호 jobs 8·테스트 threads 12와 현재 부하를 확인해 실행.

| 원 PR | source head | 해결 범위 |
| --- | --- | --- |
| [#8000](https://github.com/edwardkim/rhwp/pull/8000) | `01d775800d0473205899dd24f3adcd39b57c9707` | [#7999](https://github.com/edwardkim/rhwp/issues/7999), 겹치지 않는 원문 검색 범위와 치환 suffix 보존 |
| [#8002](https://github.com/edwardkim/rhwp/pull/8002) | `5769c4e7bed9206403cc4f6c052d0ceaf6bb96ae` | [#8001](https://github.com/edwardkim/rhwp/issues/8001), CRLF/단독 CR 정규화·커서·응답·저장 보존 |

## 독립 계약·반례·소비 경로

- 검색은 왼쪽부터 선택한 겹치지 않는 원문 문자 범위를 search/grep/n번째/일괄 치환이 동일하게 소비한다.
  `검토......완료` → `검토……완료` 2건, `AAAA`/`AA`의 0·2, Unicode 확장 원문 범위와 각 컨테이너의 독립성을 확인한다.
- 입력 정규화는 CRLF/CR을 기존 LF로 바꾼 뒤 길이·커서·이벤트·문단 변형·CLI 응답이 같은 문자열을 소비해야 한다.
  suffix의 한글·emoji, 일반·중첩 셀, 범위/첫째/전체/번째 치환과 raw HWP·adapter HWP·HWPX 재열기로 확인한다.
  newline 거부 API는 기존 거부 계약을 유지하고 이미 잘려 저장된 파일 복구로 확대하지 않는다.
- #8002의 기존 표 회귀 3개는 CR을 자동 줄바꿈용 무시 문자로 사용한 합성 입력이었다.
  단순 통과 목적의 기대값 완화인지, LF 정규화와 분리한 원래 pagination 계약의 유효한 입력 복원인지
  실제 builder·독립 경계·수정 전후 실행으로 판정한다. 행 하나의 수용과 두 행 전체의 예산 초과,
  NONE/CELL/분할의 행 소유·내용 보존을 확인하며 기존 assertion 삭제/허용치 완화를 수용하지 않는다.
- 실제 조판 소비 경로와 저장 LineSeg·폭·메트릭 변경 여부를 코드로 추적한다. 같은 LF 입력의 동일 IR을
  전달하는 입력 정규화와 검색 범위 계약을 조판 계산 변경과 구분하고, 시각 영향이 있으면 편집 저장본의
  독립 한컴 Print·Native/fresh WASM 전쪽 TSV·대표 review/overlay를 추가한다. PDF 부족을 비해당으로 바꾸지 않는다.

## 순서와 검증

1. 원 author·-x SHA를 보존해 #8000의 1개와 #8002의 5개 commit을 순차 적용한다.
2. 변경 코드·기존 코멘트·이슈·호출 경로를 정밀 검토하고 focused와 수정 전 실패/정상 대조군을 먼저 확인한다.
3. 필요한 보정은 별도 code/test commit으로 기록하고 동일 입력의 원문 보존·커서·Undo/Redo·dirty 및 저장 경로를 검증한다.
4. 모든 Rust lint 묶음·workspace·manifest base 검사, focused·전체 nextest 및 실제 CLI/stdio MCP·fresh WASM/브라우저 계약을 실행한다.
   이미 통과한 소스와 바이트 동일한 검증은 지문을 남겨 재사용하되 새 code head에 이전 결과를 자동 적용하지 않는다.
5. PR 직전 실제 devel head 변경을 확인한다. 변경 시 필요한 rebase·영향 검증 후 통합 PR을 생성한다.
6. 정확한 code head CI 성공 뒤 원 PR별 검토/적용·오늘할일을 문서 trailing으로 반영하고 merge-tree·링크·기록 보존을 검사한다.
7. trailing CI·정상 merge 후 duration·devel sync·이슈/원 PR 개별 증거 댓글·통합 완료 close·소유 branch·root output 전체 정리를 완료한다.

증적·실행 로그는 ignored `output/pr-review/donggyun112-20261018/`에 보존하고, 종료 시 필요한 자료를 영구 증거 경로로 해시 확인해 옮긴다.

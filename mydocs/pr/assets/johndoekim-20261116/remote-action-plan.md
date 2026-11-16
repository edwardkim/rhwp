# johndoekim 2건 승인된 원격 처리안

2026-11-16 작업지시자가 아래 원격 전 범위를 승인했고, 새 #9106 반영 뒤 Native 10쪽 한 행의 기존 차이 수용도 별도로 승인했다. 최종 실제 제품·검증 source는 `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`다. 전체 회귀 13,632개·Native 4,073개·모든 lint·목표 Native/fresh WASM 100.00%·편람·공개 공통 검사를 완료했다. 보호 3,542쪽은 승인된 1행 외 하락 0건이다. 문서 기록만 뒤에 추가하며 원 기여 SHA·fork branch를 보존한다.

## 범위와 순서

1. 본인 계정의 열린 PR (Draft 포함)과 reviewer 지정 여부를 재조회한다. 현재는 0건이다.
   최신 `upstream/devel`, 두 원 PR head를 다시 확인하고 후보의 `git merge-tree --write-tree`를 통과한다.
2. `review/integration-9076-johndoekim-fields`를 upstream에 push하고 devel 대상 Open 수용 PR을 만든다.
   제품 코드·원 기여 SHA를 보존한다. fork branch에는 push하거나 삭제하지 않는다.
3. 새 PR의 정확한 head에서 필수 CI·CodeQL·Render Diff·Adapter·Proptest·정책이 모두 성공했는지 확인한다.
   실패하면 실제 로그로 원인을 조사한다. workflow 변경·보안 경고 분류·취소·재실행의 별도 승인 경계는 유지한다.
4. 성공한 code head의 self-review·오늘할일을 현행 절차에 따라 문서-only trailing commit으로 기록한다.
   최신 base와 문서 병합·링크 검증 후 push하고 실제 fast-pass와 최종 required check를 확인한다.
5. merge 직전 head·CI·mergeability·새 base 병합 시뮬레이션을 다시 확인하고 정확한 head를 지정해
   **merge commit**으로 통합한다. squash/rebase merge로 원 기여 SHA를 바꾸지 않는다.
6. merge SHA 확인 → archive review·오늘할일의 확정값을 mydocs 운영 기록으로 devel에 반영 →
   기본 devel fast-forward 순서로 처리한다. 기록 범위를 넘는 코드·test·baseline을 이 push에 섞지 않는다.
7. #9076·#9082 완료 상태를 재조회하고 OPEN이면 준비한 완료 댓글을 게시한 뒤 close한다.
   #9058은 OPEN을 유지하며 관련 없는 부모·이슈 상태는 변경하지 않는다.
8. 원 #9081·#9084의 간접 MERGED 여부를 확인한다. 자동 반영되지 않으면 통합 수용 근거·SHA를 명시해
   완료 댓글 후 close한다. 두 원 PR의 감사 댓글은 각각의 실제 주장·검증을 사용한다.
   #9081에는 각 입력·backend의 review/overlay를 실제 merge SHA의 raw 이미지로 넣고 API·이미지 해시를 확인한다.
9. 원 fork branch는 보존한다. 이번 전용 upstream branch·clean 로컬 branch/worktree를 정리한다.
   실제 사용 종료를 확인하고 저장소 루트 `output/` 전체와 공용 `target/pr-review/debug`를 정리한다.
   소스·정식 PDF·검증 증적·저장소 밖 설치한 사설 검증 글꼴은 보존한다.

## 메인터너 보완

#9081의 회귀 주석만 `e958314dc9078fdb55e7cde04227a1a22054a72d`에서 보완했다.
실제 검사는 관계형과 전체 1쪽 고정형을 함께 사용하므로 두 유형을 표기하고 Git에서 확인한 직전 변경
SHA/시각과 실제 새 갱신 시각을 기록했다. 제품·검사 동작·기준값은 변경하지 않았다.

## 댓글 문안

최종 수치와 실제 merge SHA는 실행 단계에서 대입한다. 미확정 성공이나 issue close를 미리 게시하지 않는다.

- #9081: 글상자 안 미기입 누름틀의 두 표 줄 배치 개선, 실제 1쪽 raw Native/WASM 각 100.00%,
  두 입력·두 backend의 review/overlay·전쪽 TSV, 최소 글꼴·특별 보호·최종 CI, 원 기여 보존과 감사.
- #9084: 한 글자·네 글자·안내문 3입력의 두 차례 실제 HWP 편집 저장·HWPX 교차 저장,
  값과 FIELD_END/표 순서 보존, 신규·인접 필드 슬롯/이동 회귀, #9058 미해결 범위 구분과 감사.
- #9076·#9082: 해당 이슈의 개선 동작·실제 merge SHA·검토 증적을 각각 적어 완료 처리.

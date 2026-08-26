# PR #4755 구현 검토 - LayoutFrame 기반 LineSeg 재조판

## 구현 순서와 소유 경계

1. `LayoutFrame::carve()`가 현재 물리 행의 exclusion을 가로 구간으로 만들고, 내용 계산은 구간별로
   이어서 채운다.
2. 후보 행 높이가 달라지면 cursor와 Frame checkpoint를 복원한 뒤 다시 carve한다. 확정 전에는
   `commit_carved_row()`가 호출되지 않으므로 부분 LineSeg가 공개되지 않는다.
3. 확정된 물리 행은 동일한 수직 metrics를 공유하는 LineSeg 묶음으로만 투영한다. 표 셀은 자기 content
   폭으로 Frame을 만들고, 그림 band는 영향을 받는 문단 전체를 shadow 상태로 계산한다.
4. picture frame 본문 편집은 전체 shadow 결과가 성공할 때 한 번에 게시한다. 실패하면 기존 공개 상태를
   유지한다.

## 검토 이력과 rollback 경계

외부 contributor의 기능 commit 열 개(`eafd8b335`부터 `d534f738`까지)는 수정하지 않았다. 그 뒤의
`399e8065`는 구현계획서만 추가한 문서 commit이며, 이 검토 기록과 오늘할일도 별도 review-only commit으로
추가한다. source를 rebase·amend·force-push하지 않는다.

rollback이 필요하면 병합된 PR의 squash commit을 되돌린다. trailing 문서 기록은 런타임 동작을 바꾸지
않으므로 동작 복구의 별도 rollback 대상이 아니다.

## 검증 및 merge 조건

`layout_frame` 9건, 실제 325쪽 picture band geometry 1건, `frame_reflow` 10건,
`paragraph_frame_owner_width` 2건의 focused test가 모두 통과했다. code candidate `d534f738`에서 같은
PR identity의 CI·CodeQL·Render Diff도 성공했다.

최신 reviewer 문서 head는 review-only fast-pass A 경로여야 한다. 최신 aggregate가 success이고
mergeable 상태와 작업지시자 승인이 유지될 때만 squash merge한다. `Cancel stale PR runs`의 독립적인
workflow API 404가 재현되면 required Build & Test aggregate와 분리해 기록하되, required check가
실패·pending이면 merge하지 않는다.

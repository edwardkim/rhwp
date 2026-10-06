# #6706 저장 줄의 표·그림 소유 — 로컬 수정 완료

Issue: [#6706](https://github.com/edwardkim/rhwp/issues/6706).
기준 `769582fc856f162e57604b318d41414d7b026345`, 브랜치
`codex/issue-6706-stored-inline-rows`.

같은 가시 문자 위치로 투영된 표·그림을 원본 UTF-16 저장 줄에 각각 배정하고,
측정과 배치에서 같은 결과를 사용했다. 중복 왼쪽 여백을 제거해 표 제목 x=107.1px,
그림 좌상단 (108.1,340.2)px로 한컴 PDF와 약 0.4px 이내에 배치된다.

전체 회귀 **9,885/9,885 통과**, 집중 102개, 세 Clippy, Native Skia 3종, fresh WASM을
검증했다. OVR5 142쪽은 수정 전후 render tree가 동일하며, 대상 64쪽 중 18쪽만 바뀐다.
18쪽의 fresh WASM PNG는 최종 native와 byte-identical이다.

[분석·실제 호출 경로·검증 결과·시각 증적](../working/task_m100_6706_stage1.md)에
명령, source/fixture hash와 미수행 범위를 기록했다. 기존 그림 색상과 글꼴·장식 차이는
문서 전체의 완전 일치로 판정하지 않는다.

#6706의 로컬 수정과 검증만 완료했다. 원격 push·PR·CI·merge·이슈 종료는 아직이며,
다른 열린 이슈는 이 보고서의 완료 범위가 아니다.

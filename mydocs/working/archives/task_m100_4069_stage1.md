# Task #4069 Stage 1 중간 보고 — 중첩 RowBreak 재귀 cursor

- Issue: [#4069](https://github.com/edwardkim/rhwp/issues/4069)
- 기준: `upstream/devel` `d76d4e98b`
- 작업 브랜치: `local/task4069-redesign`
- 중간 커밋: `7c9ce05e6` (`fix(renderer): #4069 중첩 RowBreak cursor를 재귀 투영`)
- 기준 문서: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 한컴 정답지: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf` (17쪽,
  SHA-256 `9b0390f856bb9ad43337679babf6677209b7c7ab678b6616fcc6d6d5551ff1c4`)

## 단계 결과

수정 전 24쪽이던 42065 문서를 17쪽으로 수렴시켰다. 2쪽의 큰 조문 비교 행은 페이지
하단까지 분할되고 3쪽은 제2호부터 재개한다. 제1호 중복과 마지막 조항 누락은 없다.

이 시점의 자동 쪽수·cursor 계약은 통과했지만 최종 완료는 아니었다. 작업지시자의 시각
검토에서 10쪽 셀 안의 같은 문단이 저장 프레임 경계를 넘어 상단으로 되감기며 겹치는 현상이
추가로 확인됐다. 따라서 `7c9ce05e6`은 2·3쪽 해결을 보존하는 중간 커밋이고, 10쪽 이후는
[Stage 2/3 최종 보고](task_m100_4069_stage2.md)에서 다룬다.

메인 작업 트리의 Claude Code WIP와 stash는 변경하지 않았다. 최신 `upstream/devel`에서 만든
별도 worktree에서 원인을 다시 추적하고 구현했다.

## 원인과 구현

rhwp CLI의 render tree·SVG·쪽수 출력을 한컴 PDF와 대조했다. 바깥 셀의 `CellUnit` 원장은
중첩 RowBreak 표의 큰 행을 하나의 원자 높이로만 기록했다. 페이지네이션이 선택한 바깥 컷을
렌더러가 자식 표의 행·셀 컷으로 복원할 수 없어, 첫 조각은 하단을 비우고 후속 조각은 자식 표를
scalar clip으로 다시 계산했다.

1. 중첩 표 조각에 자식 `row/start_cut/end_cut` cursor를 기록하는 `NestedTableCut`을 추가했다.
2. 빈 host 문단의 auto-height RowBreak 행은 콘텐츠가 가장 긴 셀의 canonical `CellUnit` 경계를
   공통 높이 축으로 삼고 모든 셀의 누적 cursor를 각 조각에 투영한다.
3. 여러 저장 페이지 프레임을 가진 1×1 중첩 표는 자식 셀의 canonical unit과 hard break를
   재사용한다. 고정 높이 행·rowspan·단일 경계 문서는 기존 경로를 유지한다.
4. 부분 렌더러는 페이지마다 자식 split을 다시 추정하지 않고 페이지네이션이 기록한 자식 cursor를
   `layout_partial_table`에 전달한다.
5. #2007 회귀를 17쪽 정확 일치로 강화하고, 2쪽 제1호/3쪽 제2호 재개와 제1호 비반복,
   3쪽 마지막 조항 존재를 render tree 텍스트로 고정했다.

## Stage 1 검증과 판정

- `cargo test --test issue_2007_nested_cell_pagination`: 당시 2 passed
- 관련 중첩 표 focused 회귀와 전체 `release-test --tests`: 통과
- Native Skia 3종, fmt, diff, clippy, doc test, WASM build: 통과
- 17쪽 visual sweep: 쪽수와 2·3쪽 재개는 정합
- 수행자 시각 검토: 10쪽 문단 내부 저장 프레임 겹침을 추가 발견하여 **중간 단계**로 판정

Stage 1 시각 근거는 `output/4069/redesign-17/`에 보존한다. 최종 근거는
`output/4069/stage3-final-validated/`이며, 로컬 바로가기는 `output/4069/README.md`에 있다.

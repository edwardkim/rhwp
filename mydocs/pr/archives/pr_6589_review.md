# PR #6589 검토 - HWPX 주석 줄 `vpos=0` 보존

- 원 PR head: `153cfdad191c40d23bd2216a38b9a24a225a6714`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 기능 통합 후보: `review/planet6897-open-batch-20260902`의 `208a18b8d7cd86568a3b1c15e026f202454631a9` 이전 18개 `-x` 체리픽 집합
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

양수 `vpos`가 되감기는 HWPX note-line에만 0 값을 보존하고, 그 외 입력의 기존 정규화는 유지한다. 페이지 기하를 추정으로 바꾸지 않고 파서 계약을 좁게 수정한다.

## 검증

- `issue_6495_hwpx_note_vpos_reset_preserved` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.
- 이 변경은 note-line 메타데이터 보존 계약이며, 원 PR에는 독립 시각 산출물이 없다. 시각 결과를 새로 주장하지 않고 파서 회귀 시험으로 한정한다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 통합 PR에서 이 `-x` 적용분을 수용한다.

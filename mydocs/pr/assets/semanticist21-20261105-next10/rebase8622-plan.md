# 검증 중 반영된 PR #8622의 기능 보존 계획

2026-11-05 KST, 원격 재확인에서 devel에 [#8622](https://github.com/edwardkim/rhwp/pull/8622)가 추가된 것을 확인했다.

- 직전 base: `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`.
- 새 base: `6e1de406209ba9a58032f6ca6de45b18aa687d1c`.
- 통합 직전 후보: `99130c09abd397f365318aafcbe41e8cc3645096`; 제품 source `6337fe4d10`.
- `git merge-tree --write-tree --name-only HEAD upstream/devel`은 충돌 없이 tree `48479891e45855dd1b7979d96bb736381d14260a`를 만들었다. 자동 병합을 기능 보존의 증거로 사용하지 않는다.
- 새 base에는 planet6897의 원 PR 39개와 메인터너 보정이 들어왔다. 표의 시작·분할·rowspan·저장 줄 프레임·세로쓰기·개요 번호·각주 예약과 PDF 가져오기/CLI 경로가 달라졌다. 현재 10건의 테두리·문자 획·본문 원점·각주·셀 편집·문단 머리 경로와 연결되므로 실제 동작을 확인한다.

현재 검증을 완료·보존한 뒤 로컬 커밋하고 새 base를 통합한다. 새 base의 독립 worktree를 만들어 source와 산출물을 분리하며 공용 Cargo target은 순차로 사용한다. 새 base와 후보의 실제 Native·최적화 fresh WASM을 준비해, 현재 10건의 입력과 영향받은 새 기능을 같은 조건에서 대조한다.

새 기능의 입력은 [원 PR 39개 적용표](../planet6897_20261103/integration-commits.md)와 [기존 재현 가이드](../planet6897_20261103/visual-reproduction.md)를 출발점으로 삼는다. 가이드의 80개 입력/Print/compat 행은 실제로 37개 PR에 대응하며, 나머지 비시각 API/CLI 2개는 개별 리뷰·실제 호출로 따로 확인한다. 같은 원본의 서로 다른 compat·Print는 임의로 합치지 않는다. 과거 source의 전쪽 TSV를 최신 재출력으로 바꾸어 부르지 않는다.

최신 source의 필수 회귀·lint·Native 공식 묶음·fresh WASM·Studio 및 383쪽 편람과 새 devel 보존 증거를 확보하기 전에는 최종 수용·PR 제출 완료로 표시하지 않는다. 현재 승인 범위는 로컬 보정·필수 검증이며 원격 push·PR 생성·merge는 수행하지 않는다.


## 실제 보존 검증 결과

통합 source `d53115fabc92bd392044d9b906c599baddb4368d`에서 새 base와 후보의 Native/fresh WASM을 분리해 새로 출력했다. [현재 10건 46상태·124쪽](validation/post8622/summary.json)과 [새 devel의 80행·406쪽](validation/inherited8622/summary.json)은 전쪽 gate를 통과했다. [직접 판독](validation/post8622/direct-reading.md)과 canonical Native 편람 383쪽 gate도 완료했다. 마지막 계산 API의 비유한 결과 보정은 영향 경로만 별도 재실행하며 producer를 구분한다. 최종 필수 회귀 결과는 validation.md에 기록한다.

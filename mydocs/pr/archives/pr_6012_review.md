# PR #6012 검토 기록

- 대상: <https://github.com/edwardkim/rhwp/pull/6012>
- 작성일: 2026-08-25
- 원 PR head: `69238e29888b`
- 통합 검토 branch: `review/open-ci-green-20260825`
- 최신 기준선: `upstream/devel@898e75930a6c`
- 통합 code candidate: `1748b5cf33cb`

## 변경 요약

조각 회계의 유닛 합을 저장 사다리 구간 span에 맞춰 중첩 표 하단 clip 절단을 줄인다.

## 코멘트 검토

초기 maintainer 코멘트는 `issue_3637_nested_table_starts_inside_parent_cell` 실패와 overflow 14줄 증가,
`devel` 충돌 때문에 보류했다. contributor 후속 코멘트에서 최신 `devel` 병합, #5885 인접 블록과의 충돌
해소, baseline 미변경 원인 정정이 설명됐다. 최신 head CI는 완료됐고 실패가 없다.

## 통합 적용

`src/renderer/layout/table_layout.rs`의 인접 후처리 block 충돌은 contributor의 최종 head 의도와 맞춰
#5885 block과 #5782 block을 모두 순차 보존했다. 이후 메인터너 보정 commit `1748b5cf33cb`에서 rustfmt
공백만 정리했다.

## 로컬 검증

- 전체 Rust nextest 통합 검증 `8350 passed, 43 skipped`.
- `cargo fmt --all -- --check`, `git diff --check` 통과.
- manifest prepare/check와 unit-tier check 통과.
- WASM locked build 통과.

## 판정

수용 가능. 통합 후보에 포함한다.

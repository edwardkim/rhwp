# PR #6200 검토 기록

- PR: [#6200](https://github.com/edwardkim/rhwp/pull/6200)
- 관련 이슈: [#6197](https://github.com/edwardkim/rhwp/issues/6197)
- merge commit: `abeb29ca2fdbad2983d5562af0da75b57663449f`
- 기준: `upstream/devel` `40ea938664b575f5660219cc92f51ce10aa4af9b`

## 변경 범위

`adapter-diff`와 `proptest-roundtrip` preflight가 기준선 병합 뒤에 이어진 문서 전용 tail을 안전하게 해석하도록 보완했다. 검증된 source 후보의 green run, merge-tree 또는 문서 경로만의 충돌 해소, 실행 표면 변경의 fail-closed 처리를 함께 유지한다.

workflow 자체를 변경한 PR은 자신의 실행을 fast-pass로 재사용하지 않는다. 따라서 #6200에서는 adapter/proptest full worker가 의도적으로 실행됐고, 둘 다 성공했다. 이 기록 PR은 코드 변경 없는 옵션 B 문서 PR로 분리해 일반 문서 tail의 skip/fast-pass 경로를 후속 CI에서 확인한다.

## 검증

| 항목 | 결과 |
| --- | --- |
| workflow 계약 테스트 | Python unittest 38건 통과 |
| Rust format | `cargo fmt --all -- --check` 통과 |
| diff 공백 검사 | `git diff --check` 통과 |
| GitHub CI | Build & Test, CodeQL, Lint, Native Skia, Adapter inter-diff, Proptest roundtrip 성공 |

## 후속 상태

#6200은 2026-08-27에 merge됐고, closing keyword에 따라 #6197은 자동 close됐다. 이 별도 기록 PR은 source/test/workflow 변경 없이 검토 기록과 오늘할일만 보존한다.

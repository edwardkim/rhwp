# PR #1435 리뷰 기록

## PR 정보

- PR: https://github.com/edwardkim/rhwp/pull/1435
- 제목: `task 1352: 표 셀 TAC 그림과 텍스트 세로 정렬 보정`
- 연결 이슈: https://github.com/edwardkim/rhwp/issues/1352
- 작성자: `jangster77`
- base: `edwardkim/rhwp:devel`
- head: `edwardkim/rhwp:task_m100_1352`
- 상태: Open, Draft 아님
- mergeable: `MERGEABLE`
- 작성 시점: 2026-06-18 12:17 KST
- 변경 규모: 19 files, +2282 / -7

## 변경 범위

- 표 셀 안에서 TAC picture와 실제 텍스트가 같은 줄에 있을 때 그림이 아래로 밀리는 회귀를 보정했다.
- `tac_picture_label_extra_px`가 텍스트 없는 12쪽 picture 보정에는 유지되지만, visible text가 있는 셀 줄에는 적용되지 않도록 분기했다.
- `hy-001.hwpx` 첫 셀의 TAC picture/text 정렬을 고정하는 회귀 테스트를 추가했다.
- 수정 전 `upstream/devel`과 수정 후 렌더링을 비교한 SVG/PNG 시각 검증 자료를 추가했다.
- 완료 보고서에 회귀 유입 커밋과 원인 분석을 기록했다.

## 로컬 검증

통과 확인:

```text
git diff --check upstream/devel..HEAD
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
wasm-pack build --target web --out-dir pkg
```

문서 추가 커밋은 문서 전용 변경이므로 `git diff --check`와 변경 범위 확인으로 검증한다.

## 시각 검증 자료

PR diff에 포함된 주요 증적:

- `mydocs/report/assets/task_m100_1352_visual_compare/compare/before_after_header_cell_compare.png`
- `mydocs/report/assets/task_m100_1352_visual_compare/compare/before_after_page1_compare.png`

판정:

- 수정 전에는 첫 셀의 TAC picture가 텍스트 baseline보다 아래로 내려가 한컴 PDF와 어긋났다.
- 수정 후에는 첫 셀에서 TAC picture와 실제 텍스트가 같은 줄에 정렬된다.
- 회귀 유입 지점은 `321aee69 task 1139: 12쪽 글자처럼 취급 그림 흐름 보정`의 label extra 보정 적용 범위로 분석됐다.

## 리뷰 결론

로컬 필수 검증과 시각 비교 기준은 충족했다. 이 리뷰 문서와 오늘할일 커밋을 PR head에 포함해 GitHub Actions를 다시 확인한 뒤, required check가 통과하면 merge 가능으로 판단한다.

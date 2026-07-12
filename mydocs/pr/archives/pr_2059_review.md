# PR #2059 리뷰 — 표 속성 raw_ctrl_data FLAGS 저장 보존

- PR: #2059 `Task #2055: 표 속성 attr 비트 변경을 raw_ctrl_data FLAGS 에도 반영`
- URL: https://github.com/edwardkim/rhwp/pull/2059
- 기준 브랜치: `devel`
- head branch: `fix/table-attr-save`
- 작성자: @lpaiu-cs
- 관련 이슈: #2055
- 문서 작성 시점 참고값: 원 PR head `78921bccf414adc7a7a83e8174758dc0c3b419c5`, merge state `BEHIND`
- 처리 경로: 여러 PR 체리픽 누적 검토. 통합 PR #2062 에 기능 커밋을 포함하고, 본 review 문서를 같은 PR head 에 포함한다.
- 최종 merge 조건: 통합 PR #2062 최신 head 기준 GitHub Actions 통과 + 작업지시자 승인.

## 결론

merge 후보로 본다. HWP5 파싱 표는 `raw_ctrl_data` 를 보존하고 직렬화기가 이를 우선 기록하므로, `table.attr` 와
`table.common` 만 바꾸면 저장본에서 표 배치 속성이 옛 값으로 남는다. 이 PR 은 같은 함수의 offset raw 패치와
동일한 규칙으로 FLAGS(0..4) 에 attr 를 다시 써서 저장 왕복 유실을 막는다.

원 PR 은 `BEHIND` 상태라 #2057, #2058 과 함께 `upstream/devel` 기준 체리픽 통합 PR #2062 로 처리한다.

## 변경 범위

- `src/document_core/commands/table_ops.rs`

## 체리픽 기록

| 항목 | 값 |
|------|----|
| 원 커밋 | `78921bccf414adc7a7a83e8174758dc0c3b419c5` |
| 통합 PR 커밋 | `902031fbb200b6e1bb4934405562066f52c7a3f6` |
| 체리픽 순서 | 3 / 3 |
| 충돌 | 없음 |
| 선행 PR 의존 | 없음. #2057/#2058 과 같은 통합 PR 에 포함 |

## 검증

| 검증 | 결과 |
|------|------|
| 원 PR GitHub Actions | CodeQL/CI 계열 통과. WASM Build 는 원 PR CI 에서 skip |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib` | 통과, 2150 passed / 7 ignored |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | 통과 |
| `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| 실제 앱 검증 | `samples/calc-cell.hwp` 로드, 표 속성 변경, `exportHwp()`, 재로드 후 속성 보존 확인 |

실제 앱 검증 관측값:

```text
after reload: treatAsChar=true, textWrap=Square, vertRelTo=Para, allowOverlap=true
```

증적:

- `mydocs/pr/assets/pr_2059_calc_cell_roundtrip.png`

## visual sweep 판단

이 PR 은 저장 왕복에서 표 배치 속성이 유실되는지 확인하는 serializer/roundtrip 성격이다. 렌더링 개선 자체를
주장하는 PR 이 아니므로 visual sweep 차이는 blocker 기준이 아니다. 대신 HWP export 후 재로드 속성 보존을
브라우저/WASM 경로에서 직접 확인했다.

## merge 후 처리

- #2062 merge 후 #2055 auto-close 여부를 확인한다.
- 원 PR #2059 에 통합 PR #2062 로 처리됐음을 남기고 close 한다.

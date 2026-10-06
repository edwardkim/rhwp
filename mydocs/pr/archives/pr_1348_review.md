# PR #1348 검토 - 단 구분선 부분 페이지 길이 콘텐츠 높이 복원 (#1347)

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1348 |
| 작성자 | `planet6897` |
| 상태 | open / draft 아님 |
| base | `devel` |
| head | `planet6897:fix/column-separator-content-height-1347` |
| 관련 이슈 | #1347 |
| 변경 규모 | 5 files, +169 / -18 |
| mergeable | `MERGEABLE`, `CLEAN` |

## 2. 변경 요약

문제:

- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx` 23쪽 가운데 단 구분선이 본문 전체 높이로 그려진다.
- 한컴 2022 PDF 기준으로는 해당 페이지가 부분 페이지라 단 구분선이 콘텐츠 하단까지만 내려가야 한다.
- #1333에서 콘텐츠 높이 기준으로 맞췄던 동작이 이후 `4e7f191f`에서 일반 2단/연속 페이지에 대해 full body height를 강제하면서 다시 회귀했다.

수정:

- `src/renderer/layout.rs`에서 `prev_zone_sep_full_body` 분기를 제거한다.
- zone 단 구분선 emit 시 두 경로 모두 `prev_zone_y_end`를 넘겨 콘텐츠 높이 기준으로 통일한다.
- `emit_zone_column_separators()` 내부의 body 하단 cap은 유지한다.

의도:

- 꽉 찬 페이지는 콘텐츠 높이가 body 하단까지 내려가므로 기존처럼 전체 높이로 보인다.
- 부분 페이지는 콘텐츠 하단까지만 구분선을 그린다.

## 3. GitHub 상태

GitHub Actions:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL Analyze (javascript-typescript) | pass |
| CodeQL Analyze (python) | pass |
| CodeQL Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

관련 이슈:

- #1347은 open 상태.
- PR 본문에 `closes #1347` 포함.
- 통합 후 자동 close 여부를 확인하고, 필요 시 수동 close가 필요하다.

## 4. 로컬 검토 방식

검토 기준 브랜치:

```text
local/devel @ 4574299f
```

PR head fetch:

```text
local/pr1348-upstream @ fcaf4998
```

통합 시뮬레이션:

```text
local/pr1348-merge-test @ 1129f6c8
```

적용 방식:

- `origin/devel` 기준 검증 브랜치 생성
- PR 단일 커밋 `fcaf4998` cherry-pick
- 충돌 없음

변경 파일:

- `src/renderer/layout.rs`
- `mydocs/plans/archives/task_m100_1347.md`
- `mydocs/plans/archives/task_m100_1347_impl.md`
- `mydocs/report/task_m100_1347_report.md`
- `mydocs/working/task_m100_1347_stage2.md`

## 5. 로컬 검증

실행 완료:

```bash
cargo fmt --all -- --check
cargo test --lib
cargo clippy --lib -- -D warnings
cargo run --bin rhwp -- export-svg 'samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx' -o output/poc/pr1348-column-separator -p 15 --debug-overlay
cargo run --bin rhwp -- export-svg 'samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx' -o output/poc/pr1348-column-separator -p 22 --debug-overlay
```

결과:

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib` | 1622 passed, 0 failed, 6 ignored |
| `cargo clippy --lib -- -D warnings` | 통과 |
| SVG export p16 | 통과 |
| SVG export p23 | 통과 |

생성 파일:

- `output/poc/pr1348-column-separator/3-09월_교육_통합_2024-구분선아래20구분선위20_016.svg`
- `output/poc/pr1348-column-separator/3-09월_교육_통합_2024-구분선아래20구분선위20_023.svg`

23쪽 SVG에서 가운데 단 구분선은 `y=90.706...`에서 `y=600.133...`까지 출력되어, body 전체 하단이 아니라 콘텐츠 하단 기준으로 짧아진 것을 확인했다.

## 6. 검토 의견

차단 이슈는 발견하지 못했다.

수정 방향은 타당하다.

- PR은 #1347 증상과 직접 대응한다.
- 변경은 `layout.rs`의 단 구분선 높이 결정 분기에만 한정된다.
- body 하단 cap은 유지되므로 콘텐츠 높이가 body를 초과하는 경우에도 과도한 선이 출력되지 않는다.
- 꽉 찬 페이지는 콘텐츠 높이가 body 높이와 사실상 같으므로 full-height 동작을 별도 플래그로 유지할 필요가 줄어든다.

주의점:

- 이번 PR은 메인테이너가 이전에 넣은 `4e7f191f`의 일반 페이지 full-height 강제 분기를 되돌리는 성격이 있다.
- 따라서 수용 전에는 생성된 p16/p23 SVG를 기준으로 메인테이너 시각 판정이 필요하다.
- contributor 작업 문서 중 report/working 문서가 활성 폴더에 남는다. 수용 후 프로젝트 정리 단계에서 archive 이동 여부를 판단해야 한다.

## 7. 권장 처리

권장: **수용 가능**.

권장 절차:

1. 작업지시자 승인.
2. 생성 SVG 2건 메인테이너 시각 판정.
3. 필요 시 WASM 빌드 후 rhwp-studio 확인.
4. PR 커밋을 `local/devel`에 반영.
5. contributor 작업 문서 정리 여부 결정.
6. 처리 보고서 작성.
7. 승인 시 `devel` push, PR #1348 close, issue #1347 close.

## 8. 승인 요청

위 검토 결과 기준으로 PR #1348 수용 절차를 진행해도 되는지 승인 요청한다.

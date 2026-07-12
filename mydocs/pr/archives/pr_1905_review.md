# PR #1905 리뷰

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1905 |
| 제목 | Task #1892: 대법원 서식(HWP3) HWP5 라운드트립 렌더 대변위 수정 — 4중 결함 체인 |
| 작성자 | planet6897 |
| base/head | `devel` ← `planet6897:pr/devel-1892` |
| 검토 기준 head | `391550d9e1e14db240c92c65c982a1e06f57897b` |
| 최종 merge head | `f4dee14aa854e514232ea480161c52963305020a` |
| merge commit | `23aea574acca7d1f20cfeb27e4ab4d06df8ad317` |
| 규모 | 12 files, +466/-29 |
| merge 전 최종 상태 | `MERGEABLE` / `CLEAN` |
| 관련 이슈 | Closes #1892 |

## 변경 범위

- HWP3 도형/그룹 파서의 빈 `$con` 컨테이너 판별 보강.
- HWP3 `build_common_obj_attr` 에 개체 크기 기준 bit 15-17/18-19 기록 추가.
- HWP5 shape component 직렬화의 rendering matrix fallback 에서 `offset_x/y` 를 translation 으로 승격하지 않도록 변경.
- HWP body text 파서에서 직렬화기 null tab marker `[0,0,0,0,0,0,0x0009]` 를 `tab_extended` IR 로 싣지 않도록 정규화.
- #1892 샘플 2건과 render-diff/IR pin 테스트 추가, #1244 탭 회귀 테스트 계약 정제.
- 계획서/구현계획서/최종보고서 추가.

## PR 내용 검토

PR 설명의 4개 결함 축과 실제 변경은 대응한다.

- 빈 묶음 컨테이너: `src/parser/tags.rs` 의 `SHAPE_CONTAINER_ID` 추가와 `src/parser/control/shape.rs` 의 최상위/중첩 컨테이너 판별이 `$con` 을 직접 본다.
- rendering matrix fallback: `src/serializer/control.rs` 의 `write_shape_component_base` fallback 경로가 translation 을 identity 로 기록한다. `raw_rendering`, 회전, explicit rendering matrix 경로는 유지된다.
- HWP3 크기 기준: `src/parser/hwp3/mod.rs` 의 `build_common_obj_attr` 가 `parse_common_obj_attr` 와 같은 width/height criterion 매핑을 기록한다.
- null tab marker: `src/parser/body_text.rs` 가 `[0,...,0,0x0009]` 를 탭 확장 실데이터가 아닌 "데이터 없음" marker 로 보고 `tab_extended` 에 넣지 않는다. HWPX 실 tab 확장 보존 테스트는 그대로 별도 검증된다.

## 렌더 영향 및 시각 검증 판단

렌더링 결과가 달라지는 라운드트립 PR 이므로 시각 검증 또는 동등 검증 대상이다. 이 PR 은 한컴 기준 PDF와 직접 맞추는 수정이 아니라, HWP3 파스 → HWP5 export adapter → 재파스의 render geometry 자기정합을 복구하는 PR 이다.

따라서 기준 PDF visual sweep 대신 PR 에 포함된 `roundtrip_geom(..., Via::Hwp)` 테스트와 CLI `render-diff --via hwp` 로 대표 샘플 2건을 확인했다.

## 로컬 검증

검증 시작 전 PR review 규칙에 따라 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다.

| 명령 | 결과 |
|---|---|
| `git diff --check upstream/devel...HEAD` | pass |
| `cargo test --test issue_1892` (`CARGO_INCREMENTAL=0`) | pass, 4 passed, 40.25s |
| `cargo test --test issue_1244_tab_extended_fallback` (`CARGO_INCREMENTAL=0`) | pass, 3 passed, 0.98s |
| `cargo run --bin rhwp -- render-diff samples/issue1892_hwp3_drawing_group_roundtrip.hwp --via hwp` (`CARGO_INCREMENTAL=0`) | pass, A=2/B=2, max 0.00px |
| `cargo run --bin rhwp -- render-diff samples/issue1892_hwp3_tab_roundtrip.hwp --via hwp` (`CARGO_INCREMENTAL=0`) | pass, A=1/B=1, max 0.00px |
| `cargo test --profile release-test --tests` (`CARGO_INCREMENTAL=0`) | pass, 283.25s, `svg_snapshot` 포함 |
| `cargo fmt --check` | pass, 2.72s |
| `cargo clippy --all-targets -- -D warnings` (`CARGO_INCREMENTAL=0`) | pass, 30.92s |

## GitHub CI

코드 검증 기준 head `391550d9e1e14db240c92c65c982a1e06f57897b` 기준 required checks 가 모두 성공했다.

- CI preflight: success
- Build default-feature tests: success
- Native Skia tests: success
- Build & Test: success
- Render Diff preflight / Canvas visual diff: success
- CodeQL preflight / Analyze javascript-typescript / Analyze python / Analyze rust / CodeQL: success
- WASM Build: skipped

이후 maintainer 문서-only 보정 커밋 `f4dee14aa854e514232ea480161c52963305020a` 에서는 preflight 성공,
heavy job skip, `Build & Test` 성공을 확인했다. 이 커밋은 보고서 placeholder 치환만 포함한다.

## 확인 사항

보고서 `mydocs/report/task_m100_1892_report.md` 에 남아 있던 `cargo test 전 스위트: <!-- CARGO_TEST_RESULT -->` placeholder 는 maintainer 권한으로 보정했다.

- 보정 커밋: `f4dee14aa854e514232ea480161c52963305020a`
- 보정 내용: GitHub Actions 최신 head 기준 `Build default-feature tests` / `Build & Test` 성공과 maintainer 로컬 `cargo test --profile release-test --tests` 통과로 치환.

## 결론

코드와 테스트 기준으로 merge 후보로 판단했고, 문서 placeholder 도 maintainer 보정 완료했다. PR #1905 는
`23aea574acca7d1f20cfeb27e4ab4d06df8ad317` 로 merge 완료. #1892 는 GitHub auto-close 로 `CLOSED` 상태가 됐다.

후속 리뷰 문서는 옵션 2 경로에 따라 별도 문서-only PR 로 반영한다.

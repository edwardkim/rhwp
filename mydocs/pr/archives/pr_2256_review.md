# PR #2256 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | [#2256](https://github.com/edwardkim/rhwp/pull/2256) |
| 작성자 | `jangster77` (collaborator integration) |
| base | `devel` |
| head 참고값 | `1df49f19a` 최초 PR 생성 기준. 문서/asset 후속 커밋 push 뒤 최신 head와 CI를 재확인한다. |
| 통합 원본 PR | [#2232](https://github.com/edwardkim/rhwp/pull/2232) → [#2242](https://github.com/edwardkim/rhwp/pull/2242) → [#2245](https://github.com/edwardkim/rhwp/pull/2245) → [#2247](https://github.com/edwardkim/rhwp/pull/2247) → [#2251](https://github.com/edwardkim/rhwp/pull/2251) |
| 관련 이슈 | [#2195](https://github.com/edwardkim/rhwp/issues/2195), [#2240](https://github.com/edwardkim/rhwp/issues/2240), [#2238](https://github.com/edwardkim/rhwp/issues/2238), [#2239](https://github.com/edwardkim/rhwp/issues/2239), [#2243](https://github.com/edwardkim/rhwp/issues/2243), [#2236](https://github.com/edwardkim/rhwp/issues/2236), [#2097](https://github.com/edwardkim/rhwp/issues/2097) |

## 변경 범위

- NO_LS 재계산, 표 anchor/padding/fragment, vpos/lazy 회계, rowspan band cut, 쪽 하단 압축 수용을 하나의 검증된 스택으로 통합한다.
- contributor 원본 코드는 체리픽으로 유지했다. 최신 `devel`의 HML/XML import 의미와 겹친 `document.rs` 한 곳만 셀 순수 빈 문단 조건을 일반 규칙으로 병합했다.
- 원 PR별 review 문서 5개, HWP 2020 MCP 기준 PDF 7개, 오늘할일과 representative visual asset 5개를 포함한다.

## 로컬 검증

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: exit 0.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 경고 없이 성공.
- `wasm-pack build --target web --out-dir pkg`: 성공.
- focused page pin: 76076=82, 86712=65, 21761835=6, 1741000=2, 21298295=2, 36395325/36382819/36386907/156631374=5/3/5/1.
- HWP 2020 MCP PDF 7개는 모두 `status: success`, `run_status: 0`, `validation: ok`였다. 각 SHA-256은 원 PR review 문서에 기록했다.

## visual sweep

- [36395325 정상 p1](../assets/pr_2256_36395325_gyeoljae_consulting_p1_review.png): 수정 후 rhwp/PDF 5쪽 정합. 기준 이전 rhwp는 7쪽이었다.
- [36395325 잔여 p4](../assets/pr_2256_36395325_gyeoljae_consulting_p4_review.png): 같은 페이지 내용을 유지하지만 `line_order_overlap`, `column_line_band_drift` 후보가 남는다. font/layout fidelity 축으로 보존하며 #2243의 sliver 페이지 수 수정 blocker로 보지 않는다.
- [21761835 p1](../assets/pr_2256_21761835_jeonjik_exemption_table_p1_review.png): rhwp/PDF 6쪽 정합, 자동 후보 0건.
- [1741000 p1](../assets/pr_2256_1741000_project_application_p1_review.png), [21298295 p1](../assets/pr_2256_21298295_byeolpyo5_disaster_p1_review.png): 각각 rhwp/PDF 2쪽 정합, 자동 후보 0건.
- macOS 로컬 글꼴 차이로 visual accuracy proxy가 낮은 표본은 절대 pixel fidelity pass/fail이 아니라 페이지 수·구조 검증의 보조 지표로 해석했다.

## 이슈 판단

- close 대상: [#2195](https://github.com/edwardkim/rhwp/issues/2195), [#2240](https://github.com/edwardkim/rhwp/issues/2240), [#2238](https://github.com/edwardkim/rhwp/issues/2238), [#2239](https://github.com/edwardkim/rhwp/issues/2239), [#2243](https://github.com/edwardkim/rhwp/issues/2243), [#2236](https://github.com/edwardkim/rhwp/issues/2236).
- [#2097](https://github.com/edwardkim/rhwp/issues/2097)은 압축 수용 축만 해소했다. 남은 sliver pagination 백로그를 위해 open 유지한다.
- [#2070](https://github.com/edwardkim/rhwp/issues/2070)은 초대형 CellBreak 표의 별도 잔여 축이므로 open 유지한다.

## 최종 권고

원본 stacked patch와 통합 코드의 source/sample diff는 동일하며, 로컬 전체 회귀·Clippy·WASM·MCP 기준 PDF·visual sweep을 확인했다. 이 후속 문서/asset commit의 최신 GitHub Actions가 통과하면 수용 및 merge한다.

# PR #2245 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | [#2245](https://github.com/edwardkim/rhwp/pull/2245) |
| 작성자 | `planet6897` |
| 연관 이슈 | [#2243](https://github.com/edwardkim/rhwp/issues/2243), [#2148](https://github.com/edwardkim/rhwp/issues/2148), [#1898](https://github.com/edwardkim/rhwp/issues/1898) |
| stacked base | [#2242](https://github.com/edwardkim/rhwp/pull/2242) |
| 원격 head 참고값 | [`ecaa116d`](https://github.com/edwardkim/rhwp/commit/ecaa116deccea2802ee8c369f2da2044ea240317), 2026-07-13 조회 |
| reviewer | `jangster77` review request 등록 |
| 규모 참고값 | 25 files, +1,883 / -414 |

## 변경 범위

- HWPX 기계 생성 결재문서의 표-경로 vpos anchor와 저장 anchor 사다리를 조정해 sliver 쪽을 줄인다.
- lazy vpos 역산 이중 계상 가드를 추가하고 진단 환경 변수를 보강한다.
- HWP5 native 경로는 기존 핀을 유지하도록 source gate를 둔다. 원격 PR diff에 `README.md`는 없었다.

## 사전 검증

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2243`가 4 tests passed / 0 failed였다.
- HWP 2020 MCP CLI로 아래 기준 PDF를 새로 생성했고, 모두 `status: success`, `run_status: 0`, `validation: ok`였다.
  - `pdf/task2243/36395325_gyeoljae_consulting-2020.pdf`: 5쪽, SHA-256 `a786294ca3b903052afa23586892157d0d765c3d49823a077a90e9b81ca9d187`
  - `pdf/task2243/36382819_gyeoljae_pm_traffic-2020.pdf`: 3쪽, SHA-256 `72c039fa80de571cf4f71b5990f082d8f91269625b659c3e6e1fab2edc4d1e79`
  - `pdf/task2243/36386907_gyeoljae_sewoon-2020.pdf`: 5쪽, SHA-256 `f005e79be14c8dcae5e8416bdde1dd3e3a2836725c7e0fddc7e47b2015c795b9`
  - `pdf/task2243/156631374_taxi_press-2020.pdf`: 1쪽, SHA-256 `59c4b40db7cffc11ed574bc4bca8843dbb46bc53a9cc3a2e24c1bf68be9224c3`
- `wasm-pack build --target web --out-dir pkg`가 성공했다.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`가 경고 없이 성공했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 전체 회귀가 exit 0으로 통과했다.

## visual sweep

- 통합 브랜치 결과는 위 PDF와 각각 5/3/5/1쪽으로 정합했다.
- 기준 이전 `upstream/devel`에서 `36395325_gyeoljae_consulting.hwpx`는 7쪽이었고, 통합 결과는 5쪽으로 줄어 기준 PDF와 맞았다.
- sweep 자동 후보는 `36395325` 4쪽의 `line_order_overlap`, `column_line_band_drift` 2건만 남았다. 나머지 6개 문서에는 후보가 없었고 render tree tail overflow도 없었다.
- `36395325` 4쪽의 현재 결과는 기준 PDF와 같은 `성과관리 체계` 페이지이나, macOS 로컬 font/layout 차이로 line geometry는 완전 동일하지 않다. 이 항목은 페이지 수 fix의 blocker는 아니지만 font fidelity 잔여로 보존한다.
- 임시 산출물은 `target/visual-sweep-planet6897/task2243-36395325/` 아래에 있다. 대표 증적은 [정상 p1 review PNG](../assets/pr_2256_36395325_gyeoljae_consulting_p1_review.png)와 [잔여 후보 p4 review PNG](../assets/pr_2256_36395325_gyeoljae_consulting_p4_review.png)로 영구 보존했다.

## 리스크와 권고

- HWPX source gate는 [#2247](https://github.com/edwardkim/rhwp/pull/2247)과 [#2251](https://github.com/edwardkim/rhwp/pull/2251)의 페이지 핀까지 포함한 전체 회귀에서 통과했다.
- 현재 쪽수 정합과 focused/full regression, Clippy 및 원격 CI는 수용 근거가 충분하다. p4의 font/layout 잔여는 페이지 수 fix를 막지 않는 별도 fidelity 축으로 남긴다.
- 최종 권고: 수용. 현재 원격 head의 소스·샘플 patch는 통합 검증 코드와 동일하며, p4 대표 visual asset을 증적으로 보존했다.

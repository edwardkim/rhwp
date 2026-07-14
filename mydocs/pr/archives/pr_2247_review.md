# PR #2247 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | [#2247](https://github.com/edwardkim/rhwp/pull/2247) |
| 작성자 | `planet6897` |
| 연관 이슈 | [#2236](https://github.com/edwardkim/rhwp/issues/2236), [#2246](https://github.com/edwardkim/rhwp/issues/2246) |
| stacked base | [#2245](https://github.com/edwardkim/rhwp/pull/2245) |
| 원격 head 참고값 | [`50af58df`](https://github.com/edwardkim/rhwp/commit/50af58df05f88df68e82ff48f71a24b2daf4f7ed), 2026-07-13 조회 |
| reviewer | `jangster77` review request 등록 |
| 규모 참고값 | 25 files, +1,948 / -414 |

## 변경 범위

- RowBreak 표의 rowspan 걸침 행에서 콘텐츠 소진 cut을 밴드 cut으로 수용해, 표 행 전체를 다음 쪽으로 넘기던 흐름을 보정한다.
- 발동 범위는 RowBreak 표, rowspan, `MIN_TOP_KEEP` 조건으로 제한한다.
- 저장 anchor 안전 마진과 lazy guard를 세분화해 상위 PR의 1쪽 문서 회귀를 막는다. 원격 PR diff에 `README.md`는 없었다.

## 사전 검증

- 통합 브랜치 focused release test에 `issue_2097_squeeze`가 포함됐고 1 test passed / 0 failed였다. 이 테스트는 `21761835_jeonjik_exemption_table.hwp`의 6쪽 핀을 확인한다.
- HWP 2020 MCP CLI로 `pdf/task2146/21761835_jeonjik_exemption_table-2020.pdf`를 생성했고 6쪽, SHA-256 `9b63df86d926cfc66e18045fb41e4200581803b7ec2fd191120fbedde02e5e08`, `status: success`, `run_status: 0`, `validation: ok`였다.
- `wasm-pack build --target web --out-dir pkg`가 성공했다.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`가 경고 없이 성공했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 전체 회귀가 exit 0으로 통과했다.

## visual sweep

- 통합 결과는 MCP 기준 PDF와 6쪽으로 정합했고 자동 후보는 0건이었다.
- visual accuracy proxy 평균은 8.26927%로 낮다. 한컴 PDF와 macOS 로컬 폰트의 차이, 마스킹/텍스트 rendering 차이를 포함한 지표여서 페이지 수 기준 fix의 단독 pass/fail 근거로 사용하지 않았다. 대표 [p1 review PNG](../assets/pr_2256_21761835_jeonjik_exemption_table_p1_review.png)를 증적으로 보존했다.
- layout overflow diagnostic은 1건(약 4.8px)이 남았지만 render tree tail overflow는 없었다. 향후 table-row fine fidelity 축으로 추적할 비차단 잔여다.

## 리스크와 권고

- 행 분할 규칙은 pagination 핵심 경로이며, 전체 회귀와 Clippy가 모두 통과했다.
- 제안한 6쪽 정합은 재현됐고, 1쪽 `156631374` 표본은 [#2245](https://github.com/edwardkim/rhwp/pull/2245)의 핀으로 함께 보호된다.
- 최종 권고: [#2232](https://github.com/edwardkim/rhwp/pull/2232)부터의 스택 순서로 수용. 현재 원격 head의 소스·샘플 patch는 통합 검증 코드와 동일하고 원격 CI도 성공했다.

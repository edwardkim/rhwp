# PR #2251 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | [#2251](https://github.com/edwardkim/rhwp/pull/2251) |
| 작성자 | `planet6897` |
| 연관 이슈 | [#2097](https://github.com/edwardkim/rhwp/issues/2097), [#1073](https://github.com/edwardkim/rhwp/issues/1073), [#1748](https://github.com/edwardkim/rhwp/issues/1748), [#2237](https://github.com/edwardkim/rhwp/issues/2237) |
| stacked base | [#2247](https://github.com/edwardkim/rhwp/pull/2247) |
| 원격 head 참고값 | [`4cbbe70c`](https://github.com/edwardkim/rhwp/commit/4cbbe70c4134eafdfac2f1c9982674b1c6467587), 2026-07-13 조회 |
| reviewer | `jangster77` review request 등록 |
| 규모 참고값 | 34 files, +2,280 / -414 |

## 변경 범위

- 표 마지막 행/블록이 쪽 하단을 소폭 초과할 때, 4중 gate로 압축 수용한다.
- 행/블록 cut retry 실패 뒤에도 같은 gate의 압축 cut 수용을 적용한다.
- `samples/task2097/`에 1741000, 21298295 재현 파일을 추가하고 `tests/issue_2097_squeeze.rs`에 페이지 수 핀을 둔다.
- [#2245](https://github.com/edwardkim/rhwp/pull/2245), [#2247](https://github.com/edwardkim/rhwp/pull/2247) 표본도 이 PR의 회귀 gate로 포함한다. 원격 PR diff에 `README.md`는 없었다.

## 사전 검증

- 통합 브랜치에서 `issue_2097_squeeze` 1 test passed / 0 failed 및 `issue_2243` 4 tests passed / 0 failed였다.
- HWP 2020 MCP CLI로 아래 기준 PDF를 생성했고 모두 `status: success`, `run_status: 0`, `validation: ok`였다.
  - `pdf/task2097/1741000_project_application-2020.pdf`: 2쪽, SHA-256 `e728ebac08b5d6101b46d9c40978b066d17abba0e56a65a44ae969fbac0a3a3c`
  - `pdf/task2097/21298295_byeolpyo5_disaster-2020.pdf`: 2쪽, SHA-256 `53dbae22029198dc1107bef33ad187cb7cbcde16c871deb4f8b1f87482df07b3`
- 통합 rhwp 결과는 각 2쪽으로 기준 PDF와 정합했다.
- `wasm-pack build --target web --out-dir pkg`가 성공했다.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`가 경고 없이 성공했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 전체 회귀가 exit 0으로 통과했다.

## visual sweep

- 두 task2097 표본 모두 자동 후보 0건이었다.
- `1741000` proxy 평균은 9.33597%, `21298295`는 9.12393%였다. macOS의 로컬 글꼴/렌더 차이에 민감한 지표이므로 쪽수 fix의 단독 pass/fail 지표로 삼지 않았다.
- 각각 layout overflow diagnostic 2건, 3건이 있었으나 render tree tail overflow는 없었다.
- 임시 sweep 산출물은 `target/visual-sweep-planet6897/task2097-*` 아래에 있다. MCP PDF와 대표 [1741000 p1 review PNG](../assets/pr_2256_1741000_project_application_p1_review.png), [21298295 p1 review PNG](../assets/pr_2256_21298295_byeolpyo5_disaster_p1_review.png)를 통합 PR 증적으로 보존했다.

## 리스크와 권고

- 13px/100px/12px gate는 실제 조판 경로에 영향을 준다. positive page-pin과 기존 비수용 경계 regression을 포함한 전체 회귀는 통과했으며, gate별 독립 negative test 보강은 후속 품질 개선 후보로 남긴다.
- [#2097](https://github.com/edwardkim/rhwp/issues/2097)은 본 압축 축을 해소해도 다른 sliver 계열이 남아 있으므로 자동 close 대상이 아니다.
- 최종 권고: stacked top PR로 수용. 현재 원격 head의 소스·샘플 patch는 통합 검증 코드와 동일하고 원격 CI도 성공했다. 대표 visual asset을 검증 증적으로 보존했다.

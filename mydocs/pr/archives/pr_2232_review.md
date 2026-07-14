# PR #2232 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | [#2232](https://github.com/edwardkim/rhwp/pull/2232) |
| 작성자 | `planet6897` |
| 연관 이슈 | [#2195](https://github.com/edwardkim/rhwp/issues/2195), [#2070](https://github.com/edwardkim/rhwp/issues/2070) |
| base | `devel` |
| 원격 head 참고값 | [`31e2d705`](https://github.com/edwardkim/rhwp/commit/31e2d7056dc584cac03761d7aa586238e00130c5), 2026-07-13 조회 |
| reviewer | `jangster77` review request 등록 |
| 규모 참고값 | 22 files, +1,487 / -405 |

## 변경 범위

- NO_LS 생성계의 셀 순수 빈 문단, 표 anchor/padding/margin, 중첩 표 fragment 처리와 글자 폭 측정을 조정한다.
- [#2195](https://github.com/edwardkim/rhwp/issues/2195)의 76076=82쪽, 86712=65쪽, sijang=307쪽 핀을 함께 유지하는 것이 목적이다.
- `tools/task2195/`의 오라클 보조 도구와 stage/report 기록도 포함한다. 원격 PR diff에 `README.md`는 없었다.

## 통합 검토와 충돌 처리

- 통합 브랜치에는 최초 검토 시점의 [`c7fba900`](https://github.com/edwardkim/rhwp/commit/c7fba90093bbf7917900bae5061bbc860bda4e39) 계열을 체리픽했다.
- `src/document_core/commands/document.rs`의 충돌은 최신 `devel`의 HML/XML import 의미를 보존하면서, HWP5 문서에서만 셀 순수 빈 문단 합성 여부를 계산하도록 일반 규칙으로 합쳤다. 문서명·페이지 수 기반 분기는 추가하지 않았다.
- 현재 원격 head는 위 통합 시점보다 새 SHA이므로, merge 전에는 최신 head와 통합 브랜치의 diff를 다시 비교해야 한다.

## 사전 검증

- 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1891 --test issue_1939 --test issue_1921_59043_pagination_pin --test issue_1100_exam_social_hwpx_header --test issue_2097_squeeze --test issue_2243`를 실행했고, 10 tests passed / 0 failed였다.
- 이 PR 직접 관련 `issue_1891`은 3 tests passed로 76076/86712 HWP5-origin HWPX 쪽수 핀을 포함한다.
- `wasm-pack build --target web --out-dir pkg`가 성공했다.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`가 경고 없이 성공했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 전체 회귀가 exit 0으로 통과했다.
- 작성 시점 참고로 원격 CI의 Rust/Build/Render Diff/CodeQL은 성공이었다. 최종 merge 조건은 최신 head 기준 CI 재통과다.
- `git range-diff upstream/devel...upstream/pr-2251 upstream/devel...399df58fe`와 source diff를 대조한 결과, 현재 stacked 원격 head와 통합 코드의 소스·샘플 patch는 동일했다. 차이는 이미 `devel`에 반영된 [PR #2255](https://github.com/edwardkim/rhwp/pull/2255) 관련 기록과 통합 stage 기록뿐이다.

## 렌더 검토

- 렌더/조판 동작을 바꾸므로 visual sweep 대상이다.
- 통합 검증의 86712/76076 focused gate는 통과했지만, 이 문서에는 PR #2242 이후 갱신된 86712 fixture도 함께 들어간다. 따라서 아래 stacked PR을 순서대로 검토한 뒤에만 최종 수용한다.

## 리스크와 권고

- 영향 범위가 넓은 typeset/reflow 변경이라 하위 stacked PR의 쪽수 핀을 함께 확인했다. 전체 회귀와 Clippy도 통과했다.
- 단독 merge가 아니라 [#2242](https://github.com/edwardkim/rhwp/pull/2242) → [#2245](https://github.com/edwardkim/rhwp/pull/2245) → [#2247](https://github.com/edwardkim/rhwp/pull/2247) → [#2251](https://github.com/edwardkim/rhwp/pull/2251) 순서의 통합 수용 대상으로 처리한다.
- 최종 권고: 수용. [#2242](https://github.com/edwardkim/rhwp/pull/2242)의 멈춘 Native Skia GitHub job은 merge 전에 force-cancel/re-run으로 운영 상태만 정리한다.

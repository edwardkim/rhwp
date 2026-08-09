---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4265 검토 - humdrum00001010 표 편집·렌더 성능 통합

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 통합 PR / 작성자 | [#4265](https://github.com/edwardkim/rhwp/pull/4265) / @jangster77 |
| 가시성 검토 브랜치 | `review/humdrum00001010-20260808` |
| 기준 `devel` | `e919655a78d5928cdf7236152fce04d6aa6f6377` |
| 통합 code head | `da2d9ae751b42a1eb04690f5c19bba5752d1c1d3` |
| 원 PR | #4246, #4247, #4248, #4249, #4250, #4251, #4258, #4259, #4260, #4261, #4262 |
| 원 작성자 | @humdrum00001010 |
| 통합 규모 | 63개 파일, +5,326/-248 |

통합은 표 셀 조판·캐럿 질의의 비용을 줄이고, 셀 분할 뒤 line segment 정합, 저장 캐럿, IME 조합,
셀 블록 서식과 거대 셀 Enter 상호작용을 함께 보정한다. 개별 변경 범위와 원 source head는
[#4246](pr_4246_review.md)부터 [#4262](pr_4262_review.md)까지의 개별 review에 분리해 기록했다.

## 메인터너 보정

#4248의 KTX nested-table 형상에서 fast path가 뒤 문단 border clip을 생략해 `cellBounds`가 legacy보다
좁아졌다. `da2d9ae75`는 대상 셀에만 필요한 후속 문단 compose를 허용하고, 기존 `window_paras` 제한은
유지한다. 따라서 KTX 결과 정합과 giant-cell latency 계약을 동시에 유지한다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| 원 PR source 상태 | 11개 원 head의 `Build & Test` 통과를 재확인했다. #4248/#4249 단독 PR의 현재 충돌은 stack 의존성으로 통합에서 해소한다. |
| 공백 검사 | `git diff --check upstream/devel...HEAD` 통과 |
| Rust 전체 | `CARGO_TARGET_DIR=target/review-humdrum00001010-20260808 CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 종료 코드 0 |
| #4248 보정 | KTX parity `3/371`, giant-cell warm latency fast `6.185ms`, legacy `92.136ms/call`, 전체 parity 3건 통과 |
| focused Rust | #4167, #4138, #4179, #4180 관련 focused test 통과 |
| Studio | focused 27건, `npm test` 813 passed, production build 통과 |
| WASM | `wasm-pack build --target web --out-dir pkg` 통과 |
| 실제 브라우저 | HWP/HWPX 셀 Enter E2E 각각 115쪽, Enter flush 0, split 1, ArrowDown barrier flush 1 |
| GitHub Full CI | [CI 31256100624](https://github.com/edwardkim/rhwp/actions/runs/31256100624), [CodeQL 31256100547](https://github.com/edwardkim/rhwp/actions/runs/31256100547), [Render Diff 31256100556](https://github.com/edwardkim/rhwp/actions/runs/31256100556)가 `557bb8526`에서 성공. Native Skia, slow shard, regular shard 1/3·2/3·3/3, Build & Test를 포함한다. |

Native Skia 3종은 로컬에서는 작업지시자 지시로 중단했으므로 로컬 통과로 기록하지 않았다. 대신
`557bb8526`의 GitHub Full CI Native Skia job이 성공해 원격 필수 게이트를 충족했다.

## 최종 권고

**메인터너 보정 포함 통합 수용 권고.** `557bb8526`의 Full CI·CodeQL·Render Diff는 성공했다.
이 결과 기록 commit의 최신 docs-only gate, `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`, 그리고
작업지시자 merge 승인을 확인한다. merge 뒤 원 PR 11개는 통합 PR 링크와 기여 credit을 남긴 뒤 close하고,
closing keyword 대상 이슈의 실제 종료 상태를 확인한다.

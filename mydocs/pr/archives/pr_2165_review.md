# PR #2165 통합 재검토 - HWPX 저장 vpos 리셋 보존

- PR: https://github.com/edwardkim/rhwp/pull/2165
- 관련 이슈: #2158
- 작성자: `planet6897`
- reviewer: `jangster77` 지정 완료
- base/head: `devel` <- `fix/2158-hwpx-vpos-reset-preserve` (원 PR), 현재는 통합 브랜치에서 재검토
- 원 PR 참고 head: `61d03890ca9d3333e326c8e009421ac4d24c0f83`
- 통합 기준: `8d695deef` + PR #2165 cherry-pick `9dd908727`
- 문서 작성 시점 참고 상태: 원 PR `BEHIND`, `MERGEABLE`; 통합 PR의 최신 CI를 최종 확인 대상으로 한다
- 작성일: 2026-07-10

## 결론

**통합 브랜치에서 merge 후보.** 원 PR의 behind 상태는 최신 `devel` 위 통합 branch로
대체됐다. sample16 HWPX/HWP5 64쪽 pin과 인접 회귀가 모두 통과했다.

핵심 변경은 HWPX의 `lineSegArray` 재계산이 원본 문단의 양수 쪽-상대 vpos 리셋을
누적 좌표로 덮어쓰지 않도록 보존한다. PR 직전 부모 `595f1a486`과 변경 head를 각각
재현했을 때 `hwp3-sample16-hwp5.hwpx`가 실제로 63쪽에서 64쪽으로 회복했고, 보존된
HWP 2022 PDF도 64쪽이다.

변경은 최신 `upstream/devel` (`8d695deef`)을 기반으로 #2163·#2168과 함께 cherry-pick했고
충돌이 없었다. 최종 판단은 개별 원 PR head가 아니라 통합 PR head의 CI로 한다.

## 변경 범위

- `src/document_core/commands/document.rs`
  - 원본 lineseg 문단에서 직전 저장 vpos가 60000HU를 넘고 첫 vpos가 0보다 크며
    5000HU 미만으로 급감하면, `running_vpos`를 저장값으로 되돌린다.
  - 기존 #1920의 `vpos == 0` 고정 틀 host 예외는 유지한다.
- `tests/issue_2158_hwpx_vpos_reset_preserve.rs`
  - sample16 HWPX/HWP5 64쪽과 온새미로 HWPX 47쪽을 pin한다.
- 계획/최종 보고서 추가.

## 검증

- 통합 브랜치 `git diff --check`: 통과.
- 최신 `upstream/devel` 위 cherry-pick: 충돌 없음.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2158_hwpx_vpos_reset_preserve --test issue_1749_saved_bounds_cumulative --test issue_1749_saved_bounds_page_break --test issue_2093_saved_single_line_spacing_after --test issue_2093_1192000_real_doc_pin`: 11 passed.
- #2156, #1891, #1842, #1623, #2146, `svg_snapshot`을 포함한 통합 focused suite: 41 passed.
- 부모 `595f1a486` 재현:
  - `hwp3-sample16-hwp5.hwpx`: 63쪽.
  - `[2027] 온새미로 1 본교재.hwpx`: 47쪽.
- PR head 재현:
  - `hwp3-sample16-hwp5.hwpx`: 64쪽.
  - `[2027] 온새미로 1 본교재.hwpx`: 47쪽.
- 기준 PDF:
  - `pdf/hwp3-sample16-hwp5-2022.pdf`: 64쪽.
  - MCP HWP2020 변환본: [hwp3-sample16-hwp5-2020.pdf](../assets/pr_2165/hwp3-sample16-hwp5-2020.pdf),
    64쪽, SHA-256 `dc5f9946897ab87d2711c3af0ae286dfa25a8f5e32b407bde5f2eae9b53585ae`.
- visual sweep, `hwp3-sample16-hwp5.hwpx`와 새 HWP2020 MCP PDF의 1-3, 62-64쪽:
  - 자동 구조 후보: 4/6 (3, 62, 63, 64쪽).
  - page 1/2/3/62/63/64 visual accuracy proxy: 25.08513 / 13.05225 / 26.10618 / 10.67714 / 6.47698 / 7.45184.
  - 임시 검토 이미지: `output/integration-2163-2165-2168-visual/integration-2165-sample16/review/review_003.png`.
  - compare: `output/integration-2163-2165-2168-visual/integration-2165-sample16/compare/compare_003.png`.
  - overlay: `output/integration-2163-2165-2168-visual/integration-2165-sample16/overlay/overlay_003.png`.
  - 보존 review asset: [page 1](../assets/pr_2165/pr_2165_sample16_review_001.png),
    [page 3](../assets/pr_2165/pr_2165_sample16_review_003.png),
    [page 62](../assets/pr_2165/pr_2165_sample16_review_062.png),
    [page 63](../assets/pr_2165/pr_2165_sample16_review_063.png),
    [page 64](../assets/pr_2165/pr_2165_sample16_review_064.png).
- 깨끗한 `target` 전체 사전 검증: `cargo build --release`, `cargo test --release --lib`
  (2191 passed), `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --doc`, Studio `npx tsc --noEmit`/`npm test` (183 passed), WASM build 모두 통과.

자동 후보는 page 62~64의 기존 HWP2022 PDF 대조에서도 같은 내용 분포와 낮은 ink match가
관찰된 기존 font/layout fidelity 축이다. #2158은 저장 vpos 리셋으로 인한 **63→64쪽 페이지 수
회복**을 다루며, 이 페이지별 픽셀 차이는 이번 serializer 보정이 새로 만든 회귀가 아니다.

## 온새미로 참고

온새미로 47쪽 assertion은 새 분기의 직접 회귀 증명은 아니다. PR 직전 부모도 이미
47쪽이므로, 이 assertion은 현 상태 고정용이다. sample16 assertion이 63쪽 부모를
실제로 실패시키므로 핵심 회귀 가드는 유효하다.

HWP2020 MCP Print 방식으로 HWP와 HWPX를 각각 PDF로 변환하면 두 출력은 모두 46쪽이며
텍스트 추출 hash도 동일했다. 변환 증적은 다음과 같이 보존한다.

- [onsaemiro-hwp-2020.pdf](../assets/pr_2165/onsaemiro-hwp-2020.pdf): 46쪽,
  SHA-256 `be7c037f5cf76f5fa7d164d710862eb994ea2042152d65c74e3cf3c19463d05c`.
- [onsaemiro-hwpx-2020.pdf](../assets/pr_2165/onsaemiro-hwpx-2020.pdf): 46쪽,
  SHA-256 `9da6c0762f6b9a53bada26e86eaebedde679c5cc5758ade4dd64f6eb40020085`.

이는 HWP 편집기 논리 PageCount 47과 다른 물리 인쇄 결과다.
PR 전후 온새미로 SVG와 render tree는 바이트 단위로 같았고, 44-46쪽 PDF sweep 차이도
이번 변경이 만든 회귀가 아니다. 이 축은 #2158의 merge 판단 근거로 사용하지 않는다.

## 잔여 및 최종 조건

1. sample16의 페이지별 font/layout fidelity는 별도 축으로 남긴다.
2. 통합 PR의 최신 CI·CodeQL을 재확인한 뒤 승인한다.

## 옵션 1 기록

이 문서, 기준 PDF, 대표 review PNG와 오늘할일 `mydocs/orders/20260710.md`를 통합 PR에 함께 포함한다.

## Merge 결과

- 통합 PR [#2170](https://github.com/edwardkim/rhwp/pull/2170)은 2026-07-10에 merge commit
  `c95d8fd743ae4cfcbcbb0e26444ebef4e42b84ba`로 `devel`에 반영됐다.
- 최신 head CI, CodeQL, Render Diff는 모두 성공했다. 갱신 전 head `19abd763b`의 CI/CodeQL/Render Diff는
  force-cancel 후 `completed/cancelled`를 확인했다.
- #2158은 PR body의 closing keyword로 auto-close된 것을 확인했다.

# PR #6530 검토 - legacy HanYang fallback chain

- 검토일: 2026-08-31
- 작성자: `planet6897`
- base: `devel` (`upstream/devel@887b4ce15`로 rebase)
- 원 PR head: `e374be85c554c8b4b502f1f253aaa7c3d7b425f0`
- 통합 보정 commit: `b0109a1f6`
- 상태: 메인 터너 보정을 포함한 통합본 수용. 원 PR 직접 병합 금지.

## 범위와 충돌 보정

- 원 PR은 GitHub에서 `CONFLICTING`/`DIRTY`였으므로, `src/renderer/mod.rs`와 golden SVG 충돌을 maintainer가 통합 브랜치에서 보정했다.
- 현재 `devel`의 Windows alias 순서를 보존하면서 `한양중고딕`, `HY중고딕`, `한양견고딕`, `한양견명조`에 `HCR Dotum` 또는 `HCR Batang`, `함초롬` fallback을 누적했다.
- 기존 `한양신명조` 체인은 의도적으로 변경하지 않았다.
- golden SVG는 현재 `devel`을 기준으로 동일 fallback 순서를 반영했다.

## 검토 결과

- `issue_6171_corner_bracket_fallback_order`가 `release-test`에서 종료 코드 `0`으로 통과했다.
- #6526의 p3 직접 visual sweep에서 해당 한양 계열 글꼴이 포함된 입력을 대조했다. fallback 보정 후 split line 및 도형 흐름의 위험 신호는 `0`건이었다.
- 시각 증적: [공유 p3 review 패널](assets/pr_6526_issue6524_p3_review.png)
- Rust format, native/WASM/workspace/all-target Clippy, workspace build와 전체 `nextest`가 통합본에서 통과했다.

## 병합 조건

- #6530 원 PR은 독립 merge 대상으로 쓰지 않는다.
- `b0109a1f6`을 포함한 통합 브랜치를 PR로 게시할 경우에만 원 PR 내용을 승계한다.
- 게시 직전에 원 head, 통합 head, CI를 모두 재확인한다.

## Merge 후 contributor PR comment 계획

- 대상: [#6530](https://github.com/edwardkim/rhwp/pull/6530)와 관련 issue #6171.
- 선행 조건: 통합 PR의 merge SHA가 `upstream/devel`에 포함되고 공유 p3 review asset이 실제 merge commit에 존재할 것.
- 내용: 원 PR은 충돌 상태여서 직접 merge하지 않았고, maintainer 보정 `b0109a1f6`으로 통합 PR에 승계했음을 명확히 적는다. fallback order regression, 전체 nextest, 공유 p3 visual sweep의 flagged `0/1`, asset direct link를 함께 남긴다.
- issue가 OPEN이면 merge 반영과 보정 경위를 comment로 남긴 뒤 close 여부를 확인한다.

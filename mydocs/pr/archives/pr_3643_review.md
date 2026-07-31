---
kind: pr-review
status: active
---

# PR #3643 review — 셀 안 쪽 밖 배치 진단

| 항목 | 값 |
| --- | --- |
| 작성자 / base | planet6897 / `devel` |
| head 참고값 | `f83d9aaacfa6ccd65726b60478c78124326fc058` |
| 관련 이슈 | #3637 |
| 권고 | 통합 PR로 반영 |

셀 context에서는 기존 본문 overflow 진단이 닿지 않던 관측 사각지대를 `LAYOUT_OVERFLOW_CELL`로
분리한다. 페이지 하단과 줄 윗변을 기준으로 잡아 정상 descender bleed를 경고하지 않는 범위가
명확하다. 동작 변경이 아닌 진단 추가이며 전체 release-test·native-Skia·WASM 검증에 포함됐다.

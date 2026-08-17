---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4977 검토 - 눈금자 핀의 쪽 여백·문단 들여쓰기 편집

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4977](https://github.com/edwardkim/rhwp/pull/4977) |
| 작성자 / source | @humdrum00001010 / `rhwp-studio/ruler-margin-pin-drag` |
| 원 source head | `b14db80da980220b2edecc2093ddba2c86a85e9b` |
| 기준 devel | `8d4fb781c2f253f4a9993938f51e6bf415d8488e` |
| 가시성 검토 branch | `review/nondraft-prs-20260817` |
| local 적용 commit | `e8ab0b959`..`671dcbdc9`, `6e970e5ca` |
| 원 PR 상태 참고값 | `OPEN` / `MERGEABLE`; 실패·대기 check 없음 |

눈금자 △ 핀은 쪽 여백, ▽ 핀은 문단 들여쓰기를 편집하고 InputHandler commit 경로를 통해 undo/재배치를
일관되게 적용한다. 여백·단 설정 재접힘, 제본/맞쪽 원시 여백, 편집 용지 대화상자의 본문 보존도 함께
수정한다. 최신 suite 규약에 맞춘 #4956 fixture 등록은 메인터너 보정 `a2907c573`으로 반영했다.

## 검증과 판단

| 범위 | 결과 |
| --- | --- |
| 전용 fixture | `issue_4956_page_margin_rewrap` 6 passed |
| Studio | TypeScript, 954 unit tests, production build 통과 |
| 실제 browser | 페이지 설정 방향 아이콘 E2E 통과: A4, 세로 28x36, 가로 40x28 |
| renderer | nextest 6,529 passed; Native Skia 58, fixture 2/4 passed |
| WASM | Docker 부재로 `wasm-pack --no-opt` 진단 build 통과 |
| 원 source CI | Build & Test, Render Diff, CodeQL 성공 |

시각 경로와 재배치 회귀를 모두 확인했다. Docker 표준 최적화 build만 이 호스트에 실행 환경이 없어
대체 경로로 기록한다. **통합 수용 권고.**

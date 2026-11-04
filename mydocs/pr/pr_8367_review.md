# PR #8367 검토 — fix(layout): 머리말·꼬리말의 번호·개요 문단에 번호를 그리고 본문과 한 계수기로 센다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8367, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `0f73bc67fcffb6126c745d86685df893ffb5267b`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:27:08.268860+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 번호 생산과 실제 배치

`assign_numbering_markers`가 구역·본문·표 칸에 이어 머리말/꼬리말의 문단을 조판부호 순서로 방문하고 같은 계수기에서 번호/개요 문자열을 생산한다. 줄 구성은 이 마커 폭을 사용하고 배치도 저장된 마커를 소비한다. 페이지를 반복 그릴 때 계수기를 새로 증가시키는 방식으로 처리하지 않는다. 글머리표의 기존 계약은 변경하지 않는다.

독립 Mac Print의 머리말 번호 1·2와 본문 3·4를 실제 1쪽 Native/fresh WASM에서 확인했다. 전쪽 raw는 94.92/94.81%다. 원본에 없는 다페이지 Print 검증을 수행했다고 기록하지 않는다. 기존 본문 글머리표의 차이를 이 PR이 새로 만든 것으로 분류하거나 해결 성과로 포함하지 않는다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

머리말·꼬리말의 1·2와 본문 3·4 번호의 연속·소속을 확인했다. 기존 글머리표 차이는 그대로 남는다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `header_footer_numbering` 1쪽 | native | 94.91525% | 99.41065% | 28.17113% | [review](assets/semanticist21-20261105/comment-evidence/pr8367-header_footer_numbering/native/review/review_001.png) / [overlay](assets/semanticist21-20261105/comment-evidence/pr8367-header_footer_numbering/native/overlay/overlay_001.png) |
| `header_footer_numbering` 1쪽 | wasm | 94.80661% | 99.55858% | 46.20746% | [review](assets/semanticist21-20261105/comment-evidence/pr8367-header_footer_numbering/wasm/review/review_001.png) / [overlay](assets/semanticist21-20261105/comment-evidence/pr8367-header_footer_numbering/wasm/overlay/overlay_001.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 해당 기능 검증 문서의 Native/fresh WASM 전쪽 raw 90% 미만은 0쪽이다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](assets/semanticist21-20261105/visual-manifest.json)를 연결한다.

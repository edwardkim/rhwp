# PR #8363 검토 — fix(renderer): 칸 대각선 모양 값의 꼭짓점 비트만 대각선으로 그린다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8363, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `9d6e420da55d4b9d9684c0e9c8f61cdf84ff29ff`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:27:02.842479+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 대각선 속성의 비트 계약

공통 border 해석에서 꼭짓점 대각선 bit `0b010`을 확인한다. 하위 필드 값 전체를 truthy로 읽어 중심선·꺾인선까지 꼭짓점 대각선으로 그리지 않는다. 다른 대각선 및 center-line 속성은 기존 해석을 유지한다. 공개 1쪽 문서의 각 shape code를 독립 Mac Print와 Native/fresh WASM에서 직접 대조했으며 양쪽 raw 100.00%다. 새 분기는 문서 ID·우연한 크기로 제한하지 않는다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

CELL_3D 대각선 세 가지의 방향·끝점과 셀 경계를 독립 Print와 확대 비교했다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `cell_diagonal_shapes` 1쪽 | native | 100.00000% | 95.76510% | 16.38582% | [review](assets/semanticist21-20261105/comment-evidence/pr8363-cell_diagonal_shapes/native/review/review_001.png) / [overlay](assets/semanticist21-20261105/comment-evidence/pr8363-cell_diagonal_shapes/native/overlay/overlay_001.png) |
| `cell_diagonal_shapes` 1쪽 | wasm | 100.00000% | 96.03291% | 20.87816% | [review](assets/semanticist21-20261105/comment-evidence/pr8363-cell_diagonal_shapes/wasm/review/review_001.png) / [overlay](assets/semanticist21-20261105/comment-evidence/pr8363-cell_diagonal_shapes/wasm/overlay/overlay_001.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 해당 기능 검증 문서의 Native/fresh WASM 전쪽 raw 90% 미만은 0쪽이다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](assets/semanticist21-20261105/visual-manifest.json)를 연결한다.

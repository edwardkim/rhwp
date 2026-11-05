# PR #8349 검토 — fix(renderer): 아래 일점쇄선을 글자 크기와 원본 run 위상에 맞춤

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8349, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `ffc0550e8b9c6db8627f9a82cc7f0f3ff39a5415`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:55.219970+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: Build & Test: failure, build-test-archive-b / Build test archive (b): cancelled, Lint (fmt, clippy, WASM check): cancelled.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](../assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 실제 head와 공통 소비 지점

PR 본문의 다른 변경 축을 실제 head에 포함된 것으로 취급하지 않았다. `ffc0550e8b`가 제공하는 아래 일점쇄선(shape 3)을 통합한다. 공통 `text_decoration`이 글자 크기와 원본 run에서 이미 소비한 길이로 기하·위상을 생산하고 Native/SVG/Canvas2D가 이를 소비한다. JSON 계약의 `topUnderline` 필드를 아래 장식에도 재사용하며 Studio CanvasKit도 같은 기하를 소비한다. 임의의 `bottomUnderline` 필드를 추가하지 않는다.

Hancom 2024 Print가 있는 공개 대조군 20개를 각각 1쪽 전체로 비교했다. Native/fresh WASM 모두 90% 이상이며 목표 shape 3은 양쪽 100.00%다. 실제 SVG browser 출력도 100.00%다. 실제 최종 WASM layer tree의 장식 32개(아래 공통 기하 16개)를 production `CanvasKitLayerRenderer.renderTextDecoration`과 CanvasKit 0.42 software surface에서 실행했고 unsupported operation은 없었다. 이 검사는 CanvasKit 전체 글자 글꼴 일치를 주장하는 검사가 아니다.

기준 PDF·Native·Canvas2D·CanvasKit의 같은 배율 확대에서 run 경계의 장단선 위상을 확인했다. 색 장식 부분의 2px raw 보조값은 세 backend 모두 99.83003%였다. Studio 전체 2,722개 중 2,720개 통과·기존 skip 2개, 해당 CanvasKit 집중 21개 통과, TypeScript·production build 성공을 실제 실행 기록에 보존한다. 기존 형광펜/다른 PR을 이 head의 개선으로 기록하지 않는다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](../assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](../assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](../assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

shape 3의 파란 아래 일점쇄선이 원본 run 위상과 일치한다. 빨간 위 장식과 16개 run의 크기별 선·기호·간격을 확대 판독했다. 나머지 19개 정상 대조군도 전쪽 검증했다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `decoration-top-probe-shape-3` 1쪽 | native | 100.00000% | 98.70924% | 44.40093% | [review](../assets/semanticist21-20261105/comment-evidence/pr8349-decoration-top-probe-shape-3/native/review/review_001.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8349-decoration-top-probe-shape-3/native/overlay/overlay_001.png) |
| `decoration-top-probe-shape-3` 1쪽 | wasm | 100.00000% | 98.69027% | 44.23628% | [review](../assets/semanticist21-20261105/comment-evidence/pr8349-decoration-top-probe-shape-3/wasm/review/review_001.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8349-decoration-top-probe-shape-3/wasm/overlay/overlay_001.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 해당 기능 검증 문서의 Native/fresh WASM 전쪽 raw 90% 미만은 0쪽이다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](../assets/semanticist21-20261105/visual-manifest.json)를 연결한다.

## GitHub CI와 self-review

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)의 code head `da0074f41ee01d7fa79e6bd4ffd5cadf7c69665d`에서 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41937077195)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41937077068)·Adapter·Proptest와 최종 CI 정책 집계의 성공을 확인했다. self-review의 추가 차단 사항은 없으며 통합 후보는 승인이다. [실제 check·job 결과](../assets/semanticist21-20261105/github-ci-code-head.json), [self-review 근거](../assets/semanticist21-20261105/self-review.json).

이 기록은 녹색 code head 뒤의 문서 전용 single-parent trailing commit이다. 새 trailing head의 실제 fast-pass·필수 집계·최신 base 충돌 상태를 확인한 뒤 승인된 merge를 진행한다.

공통 보안 후속으로 기존 행 높이 진단 출력 두 그룹과 같은 채널의 span 진단만 제거했다. 보정 source `d75809517f`에서 전체 회귀 12,499개·Native CI 22개 명령 192개·Native 전체 라이브러리 4,040개를 통과했고, 실제 암호 Native CLI의 SVG 24쪽은 이전과 같다. 새 code head `da0074f41e`의 독립 GHAS CodeQL도 성공했다. 진단 외 조판 코드와 기존 시각 producer를 구분한 [실제 근거](../assets/semanticist21-20261105/ci-security-followup.md)를 후속 댓글과 함께 연결한다.

## Merge 확인

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)을 merge commit `e9d4a23be4523a97c29ce6f5e407aba6c4cfb26c`로 통합했다. 원 PR 10건 모두 간접 MERGED를 확인했으며 원 기여 commit·fork branch를 보존했다. [code CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[문서 head CI](https://github.com/edwardkim/rhwp/actions/runs/41941236661)와 실제 fast-pass·최종 정책 status·fresh-base merge-tree를 확인했고, [COMMENT self-review](https://github.com/edwardkim/rhwp/pull/8609#pullrequestreview-5778015173)를 최종 head `1557f70b03b92f0af88fb0e6f4ef280df4b8683a`에 게시했다. 이 원 PR도 GitHub `MERGED`다. [원 PR별 확정 상태](../assets/semanticist21-20261105/remote-completion.json). 위 `Merge 후 contributor PR comment 계획`의 해당 문서·페이지 review/overlay를 실제 merge SHA에 고정해 감사 댓글을 게시한다.

후속 처리 완료: [해당 원 PR 감사 댓글](https://github.com/edwardkim/rhwp/pull/8349#issuecomment-6646332657)을 게시하고 API 본문 및 GitHub 화면의 4개 review/overlay 표시를 확인했다. 통합 PR의 전용 branch/worktree·root output·검토 debug 종료 정리도 완료했다.

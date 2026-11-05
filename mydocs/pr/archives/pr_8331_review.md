# PR #8331 검토 — perf(web_canvas): 회색 JPEG 판정이 디코드한 픽셀을 첫 캔버스 그리기에 넘긴다

## 최종 판정

**메인터너 보정 후 수용 가능.** 기존 성능 문서의 동일 출력 차이에 한정한 예외를 작업지시자가 승인했다. 수치·미달 페이지는 그대로 기록하며 [정확한 범위](../assets/semanticist21-20261105/maintainer-approval.json)를 벗어나 적용하지 않는다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8331, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `5b36dbd078acd5add18592511682ce268dd66e82`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:46.696813+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: Canvas visual diff: cancelled.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](../assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 메인터너 보정과 실측

- 픽셀 인계의 다음 그림 예산 비교가 `u64` 곱셈으로 넘칠 수 있어 `u128`으로 계산한다. 기존 source unit 함수 안에 실제 `take_decoded` 경로의 거대 헤더 반례를 보강했으며 source unit 총량은 4099개로 유지했다.
- 실제 production의 handoff 함수 본체와 PNG header parser를 격리한 음성 대조를 실행했다. PNG 픽셀을 맡긴 뒤 거대 metadata를 읽는 순서에서 원 PR의 `fits_beside`는 픽셀을 남겨 FAIL(exit 101), 보정식은 버려 PASS(exit 0)였다. 별도 거대 pixel buffer를 할당하지 않았다. 함수 대역이나 Python 산술 결과로 제품 경로 실행을 대신하지 않았다. 원래 header 모듈의 자식 경로만 명시해 같은 코드를 컴파일했고 release처럼 overflow 검사를 끈 조건이다. source·명령·해시는 `assets/semanticist21-20261105/negative-handoff-*`에 보존했다.
- 동일한 최적화 WASM·Chrome에서 JPEG 11쪽 첫 그리기 중앙값이 1772.8→1118.6ms였다. 최초 출력 이후 WASM linear memory 최고 크기는 443,482,112→315,555,840바이트였다. 3회 교대 관측의 범위와 전체 출력 동일성은 [성능 검토](../assets/semanticist21-20261105/performance-control-review.md)에 구분해 기록했다.
- 원 head Render Diff는 checkout 36분 51초 후 45분 상한에 도달해 Canvas 검사를 시작하지 못했다. 제품 오류로 판정하거나 기여자 재작업을 요청하지 않으며 현재 메인터너 후보의 실제 검증 경로로 처리한다.
- 전17쪽 Native/fresh WASM PNG는 고정 devel과 각각 바이트 동일하다. 독립 Print의 기존 7·15쪽 차이에 대한 한정 예외를 작업지시자가 승인했다. 최종 runtime의 필수 전체 검증은 완료했다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](../assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](../assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](../assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

11쪽 JPEG가 같은 크기와 위치로 표시되며 첫 그리기 중앙값은 1772.8→1118.6ms였다. 7쪽 그림 상자와 본문 흐름의 기존 차이는 남고, 17쪽 전체 base/후보 PNG는 같다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `jpeg-sandbox` 11쪽 | native | 93.04570% | 91.92832% | 53.29072% | [review](../assets/semanticist21-20261105/comment-evidence/pr8331-jpeg-sandbox/native/review/review_011.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8331-jpeg-sandbox/native/overlay/overlay_011.png) |
| `jpeg-sandbox` 11쪽 | wasm | 92.67169% | 91.53500% | 51.27039% | [review](../assets/semanticist21-20261105/comment-evidence/pr8331-jpeg-sandbox/wasm/review/review_011.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8331-jpeg-sandbox/wasm/overlay/overlay_011.png) |
| `jpeg-sandbox` 7쪽 | native | 75.22330% | 84.82547% | 36.64886% | [review](../assets/semanticist21-20261105/comment-evidence/pr8331-jpeg-sandbox/native/review/review_007.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8331-jpeg-sandbox/native/overlay/overlay_007.png) |
| `jpeg-sandbox` 7쪽 | wasm | 75.47344% | 85.08915% | 36.39432% | [review](../assets/semanticist21-20261105/comment-evidence/pr8331-jpeg-sandbox/wasm/review/review_007.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8331-jpeg-sandbox/wasm/overlay/overlay_007.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 이 문서의 raw 90% 미만 쪽과 승인된 기존 차이 예외를 댓글에도 명시한다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](../assets/semanticist21-20261105/visual-manifest.json)를 연결한다.

## GitHub CI와 self-review

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)의 code head `da0074f41ee01d7fa79e6bd4ffd5cadf7c69665d`에서 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41937077195)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41937077068)·Adapter·Proptest와 최종 CI 정책 집계의 성공을 확인했다. self-review의 추가 차단 사항은 없으며 통합 후보는 승인이다. [실제 check·job 결과](../assets/semanticist21-20261105/github-ci-code-head.json), [self-review 근거](../assets/semanticist21-20261105/self-review.json).

이 기록은 녹색 code head 뒤의 문서 전용 single-parent trailing commit이다. 새 trailing head의 실제 fast-pass·필수 집계·최신 base 충돌 상태를 확인한 뒤 승인된 merge를 진행한다.

공통 보안 후속으로 기존 행 높이 진단 출력 두 그룹과 같은 채널의 span 진단만 제거했다. 보정 source `d75809517f`에서 전체 회귀 12,499개·Native CI 22개 명령 192개·Native 전체 라이브러리 4,040개를 통과했고, 실제 암호 Native CLI의 SVG 24쪽은 이전과 같다. 새 code head `da0074f41e`의 독립 GHAS CodeQL도 성공했다. 진단 외 조판 코드와 기존 시각 producer를 구분한 [실제 근거](../assets/semanticist21-20261105/ci-security-followup.md)를 후속 댓글과 함께 연결한다.

## Merge 확인

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)을 merge commit `e9d4a23be4523a97c29ce6f5e407aba6c4cfb26c`로 통합했다. 원 PR 10건 모두 간접 MERGED를 확인했으며 원 기여 commit·fork branch를 보존했다. [code CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[문서 head CI](https://github.com/edwardkim/rhwp/actions/runs/41941236661)와 실제 fast-pass·최종 정책 status·fresh-base merge-tree를 확인했고, [COMMENT self-review](https://github.com/edwardkim/rhwp/pull/8609#pullrequestreview-5778015173)를 최종 head `1557f70b03b92f0af88fb0e6f4ef280df4b8683a`에 게시했다. 이 원 PR도 GitHub `MERGED`다. [원 PR별 확정 상태](../assets/semanticist21-20261105/remote-completion.json). 위 `Merge 후 contributor PR comment 계획`의 해당 문서·페이지 review/overlay를 실제 merge SHA에 고정해 감사 댓글을 게시한다.

후속 처리 완료: [해당 원 PR 감사 댓글](https://github.com/edwardkim/rhwp/pull/8331#issuecomment-6646331115)을 게시하고 API 본문 및 GitHub 화면의 8개 review/overlay 표시를 확인했다. 통합 PR의 전용 branch/worktree·root output·검토 debug 종료 정리도 완료했다.

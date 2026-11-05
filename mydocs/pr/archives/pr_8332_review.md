# PR #8332 검토 — perf(equation): 같은 수식 값의 흐름 메트릭을 다시 계산하지 않는다

## 최종 판정

**메인터너 보정 후 수용 가능.** 기존 성능 문서의 동일 출력 차이에 한정한 예외를 작업지시자가 승인했다. 수치·미달 페이지는 그대로 기록하며 [정확한 범위](../assets/semanticist21-20261105/maintainer-approval.json)를 벗어나 적용하지 않는다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8332, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `c773e5060c0f186b4bf8281133004f5afe120fa4`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:49.714995+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](../assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 선행 해결과 중복 보정

원 PR의 핵심 메트릭 캐시는 이미 [#8348](https://github.com/edwardkim/rhwp/pull/8348)의
`15ff366244`에서 flow/paint 공통 `equation/cache.rs`로 구현됐고 `c28d998091`에 편집 뒤 새 스레드 대조가 있다.
현재 base는 값 키·source frame seal·16,384개 flow/4,096개 paint의 유한 캐시를 유지한다.
원 head를 그대로 누적하면 기존 flow 캐시 안에 다른 flow 캐시가 생긴다. 메인터너는 기존 공통 캐시·source frame
판정을 유지하고 중첩 캐시와 불필요한 pass 초기화를 제거했다. 기여의 키워드 탐색 할당 제거·값 조건 뒤 토큰화와
공개 편집 계약 검사는 보존한다. 원 캐시 주장의 과거 15초→2.7초 수치를 최신 base의 개선으로 재인용하지 않는다.
현재 base/최종 후보의 실제 성능과 출력 보존은 [최종 성능 검토](../assets/semanticist21-20261105/performance-control-review.md)에 기록했다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](../assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](../assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](../assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

22쪽 전체 base/후보 출력이 같다. 19쪽의 수식·본문 시작과 두 단 흐름 차이는 기존 현상이며 이번 성능 수정의 해결 성과로 주장하지 않는다. 기존 공통 수식 캐시를 보존했다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `equation-flow` 19쪽 | native | 42.22444% | 88.70259% | 14.77119% | [review](../assets/semanticist21-20261105/comment-evidence/pr8332-equation-flow/native/review/review_019.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8332-equation-flow/native/overlay/overlay_019.png) |
| `equation-flow` 19쪽 | wasm | 42.20281% | 88.63446% | 13.63772% | [review](../assets/semanticist21-20261105/comment-evidence/pr8332-equation-flow/wasm/review/review_019.png) / [overlay](../assets/semanticist21-20261105/comment-evidence/pr8332-equation-flow/wasm/overlay/overlay_019.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 이 문서의 raw 90% 미만 쪽과 승인된 기존 차이 예외를 댓글에도 명시한다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](../assets/semanticist21-20261105/visual-manifest.json)를 연결한다.

## GitHub CI와 self-review

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)의 code head `da0074f41ee01d7fa79e6bd4ffd5cadf7c69665d`에서 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41937077195)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41937077068)·Adapter·Proptest와 최종 CI 정책 집계의 성공을 확인했다. self-review의 추가 차단 사항은 없으며 통합 후보는 승인이다. [실제 check·job 결과](../assets/semanticist21-20261105/github-ci-code-head.json), [self-review 근거](../assets/semanticist21-20261105/self-review.json).

이 기록은 녹색 code head 뒤의 문서 전용 single-parent trailing commit이다. 새 trailing head의 실제 fast-pass·필수 집계·최신 base 충돌 상태를 확인한 뒤 승인된 merge를 진행한다.

공통 보안 후속으로 기존 행 높이 진단 출력 두 그룹과 같은 채널의 span 진단만 제거했다. 보정 source `d75809517f`에서 전체 회귀 12,499개·Native CI 22개 명령 192개·Native 전체 라이브러리 4,040개를 통과했고, 실제 암호 Native CLI의 SVG 24쪽은 이전과 같다. 새 code head `da0074f41e`의 독립 GHAS CodeQL도 성공했다. 진단 외 조판 코드와 기존 시각 producer를 구분한 [실제 근거](../assets/semanticist21-20261105/ci-security-followup.md)를 후속 댓글과 함께 연결한다.

## Merge 확인

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)을 merge commit `e9d4a23be4523a97c29ce6f5e407aba6c4cfb26c`로 통합했다. 원 PR 10건 모두 간접 MERGED를 확인했으며 원 기여 commit·fork branch를 보존했다. [code CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[문서 head CI](https://github.com/edwardkim/rhwp/actions/runs/41941236661)와 실제 fast-pass·최종 정책 status·fresh-base merge-tree를 확인했고, [COMMENT self-review](https://github.com/edwardkim/rhwp/pull/8609#pullrequestreview-5778015173)를 최종 head `1557f70b03b92f0af88fb0e6f4ef280df4b8683a`에 게시했다. 이 원 PR도 GitHub `MERGED`다. [원 PR별 확정 상태](../assets/semanticist21-20261105/remote-completion.json). 위 `Merge 후 contributor PR comment 계획`의 해당 문서·페이지 review/overlay를 실제 merge SHA에 고정해 감사 댓글을 게시한다.

후속 처리 완료: [해당 원 PR 감사 댓글](https://github.com/edwardkim/rhwp/pull/8332#issuecomment-6646331578)을 게시하고 API 본문 및 GitHub 화면의 4개 review/overlay 표시를 확인했다. 통합 PR의 전용 branch/worktree·root output·검토 debug 종료 정리도 완료했다.

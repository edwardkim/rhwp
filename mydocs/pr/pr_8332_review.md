# PR #8332 검토 — perf(equation): 같은 수식 값의 흐름 메트릭을 다시 계산하지 않는다

## 최종 판정

**메인터너 보정 후 수용 가능.** 기존 성능 문서의 동일 출력 차이에 한정한 예외를 작업지시자가 승인했다. 수치·미달 페이지는 그대로 기록하며 [정확한 범위](assets/semanticist21-20261105/maintainer-approval.json)를 벗어나 적용하지 않는다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8332, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `c773e5060c0f186b4bf8281133004f5afe120fa4`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:49.714995+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 선행 해결과 중복 보정

원 PR의 핵심 메트릭 캐시는 이미 [#8348](https://github.com/edwardkim/rhwp/pull/8348)의
`15ff366244`에서 flow/paint 공통 `equation/cache.rs`로 구현됐고 `c28d998091`에 편집 뒤 새 스레드 대조가 있다.
현재 base는 값 키·source frame seal·16,384개 flow/4,096개 paint의 유한 캐시를 유지한다.
원 head를 그대로 누적하면 기존 flow 캐시 안에 다른 flow 캐시가 생긴다. 메인터너는 기존 공통 캐시·source frame
판정을 유지하고 중첩 캐시와 불필요한 pass 초기화를 제거했다. 기여의 키워드 탐색 할당 제거·값 조건 뒤 토큰화와
공개 편집 계약 검사는 보존한다. 원 캐시 주장의 과거 15초→2.7초 수치를 최신 base의 개선으로 재인용하지 않는다.
현재 base/최종 후보의 실제 성능과 출력 보존은 [최종 성능 검토](assets/semanticist21-20261105/performance-control-review.md)에 기록했다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

22쪽 전체 base/후보 출력이 같다. 19쪽의 수식·본문 시작과 두 단 흐름 차이는 기존 현상이며 이번 성능 수정의 해결 성과로 주장하지 않는다. 기존 공통 수식 캐시를 보존했다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `equation-flow` 19쪽 | native | 42.22444% | 88.70259% | 14.77119% | [review](assets/semanticist21-20261105/comment-evidence/pr8332-equation-flow/native/review/review_019.png) / [overlay](assets/semanticist21-20261105/comment-evidence/pr8332-equation-flow/native/overlay/overlay_019.png) |
| `equation-flow` 19쪽 | wasm | 42.20281% | 88.63446% | 13.63772% | [review](assets/semanticist21-20261105/comment-evidence/pr8332-equation-flow/wasm/review/review_019.png) / [overlay](assets/semanticist21-20261105/comment-evidence/pr8332-equation-flow/wasm/overlay/overlay_019.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 이 문서의 raw 90% 미만 쪽과 승인된 기존 차이 예외를 댓글에도 명시한다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](assets/semanticist21-20261105/visual-manifest.json)를 연결한다.

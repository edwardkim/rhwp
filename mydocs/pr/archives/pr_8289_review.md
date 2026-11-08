# PR #8289 검토 기록 — 붙은 키워드 접두사 탐색 상한

## 최종 판정

**승인 — 토큰 의미를 유지하는 탐색 상한 변경을 검토하고 원 PR을 병합했다.**
원 기여자 [semanticist21](https://github.com/semanticist21)의 이력을 일반 merge로 보존했다.
추가 제품·테스트 보정 없이 원 기여를 수용했다.

## 대상과 확정 이력

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8289](https://github.com/edwardkim/rhwp/pull/8289) |
| 검토자 | [postmelee](https://github.com/postmelee) |
| 원 head | `d31221e9ac1f9c18cd7081bba695a21fa6ffe32b` |
| 고정 base | `96827b3accd98ad0b80f76260b42fba24eded504` |
| 로컬 합성 후보 | `58a6565eac95fee72a8a17ab011b5f6ebc462d1d` |
| 검증·실제 병합 tree | `34041268f701d27128f1f42a0c281ace56f0f383` |
| 실제 merge | [`140cb02b7fd6184e58209b29383ddaed18e49457`](https://github.com/edwardkim/rhwp/commit/140cb02b7fd6184e58209b29383ddaed18e49457) |
| 병합 시각 | 2026-11-04 10:19:31 UTC |
| 승인 리뷰 | [5766809166](https://github.com/edwardkim/rhwp/pull/8289#pullrequestreview-5766809166) |

## 원 기여와 검토 범위

긴 수식 식별자의 모든 접두사를 해시하던 탐색을 허용 키워드와 기호표의 최대 이름 길이로 제한했다.
최장 우선 순서와 기존 토큰 분리 결과가 유지되는지 검토했다. 조판 원칙 검토의 적용 범위는 수식
토큰 의미와 출력 계약 보존이며, 새로운 줄 배치·글꼴 메트릭·문서별 보정을 도입하지 않았다.

## 로컬 검증

macOS, Rust 1.93.1, 최소 글꼴 환경에서 위 후보를 검증했다.

- tokenizer 37개, 긴 식별자 회귀 1개, SVG 계약 4개: 집중 42 PASS.
- 같은 후보의 SVG 계약 반복: 4 PASS.
- suite manifest 도구 23 PASS, unit-tier 도구 12 PASS.
- 최신 base 기준 manifest·unit-tier 정책 및 diff 검사: PASS.
- 포함 소스가 base/후보와 동일한 진단 실행파일에서 입력 4,535개의 모든 토큰 속성이 일치했다.

40,000자 진단 처리시간은 base 약 1.56~1.57초, 후보 약 0.00021~0.00024초였다.
제품 전체 성능을 측정한 값이 아니다. 이 Mac에서는 base도 회귀의 2초 상한을 만족했으므로
수정 전 시간 assertion 실패를 재현했다고 주장하지 않는다. 개선 근거는 코드의 탐색 상한,
동일 입력의 토큰 동등성과 처리시간 비교로 구분한다.

후속 기록 작성 시 개별 worktree의 원래 실행 로그·producer를 다시 열 수 없었다. 보존된 통합
검토 요약의 완료 결과·소스 일치 기록·파일 해시 목록, 후보·병합 영수증과 공개 승인 기록을 대조했다.
이번 문서 작업에서 tokenizer·SVG·성능 검사를 새로 실행한 것으로 기록하지 않는다.

## GitHub 검증과 재사용

- [원 head Full CI 40505892791](https://github.com/edwardkim/rhwp/actions/runs/40505892791): 성공.
- [원 head Render Diff 40505892445](https://github.com/edwardkim/rhwp/actions/runs/40505892445): 성공.
- [CI Impact Policy 40512785522](https://github.com/edwardkim/rhwp/actions/runs/40512785522): 해당 head 상태 성공.
- [병합 후 duration 갱신 41813412846](https://github.com/edwardkim/rhwp/actions/runs/41813412846): 성공.

정책의 stale pending은 재실행 후 exact-head success로 해소됐다. 광범위 전체 회귀는 검토 당시
[정본의 재사용 조건](../../manual/pr_review_workflow.md#322-녹색-github-code-head의-중복-로컬-전체-회귀-생략)에
따라 원 head CI를 재사용했다. 제품·테스트 추가 보정이 없고 clean merge와 집중 출력 계약을
확인했다. 로컬 합성 후보에 새 GitHub CI가 실행됐다는 의미는 아니다.

실제 병합 tree와 검증 tree가 일치했다. 새 브라우저 WASM·한컴 변환은 실행하지 않았다.
토큰 동등성과 SVG 계약을 출력 보존 근거로 사용했으며 전쪽 시각 품질 점수를 주장하지 않는다.
함께 반영하는 기록은 [#8203](pr_8203_review.md), [#8285](pr_8285_review.md)다.

## Merge 후 contributor PR comment 계획

별도 감사·병합 결과 댓글은 작업지시자 확인 전 게시하지 않는다. 게시 시 긴 식별자 처리 개선의
원 기여를 먼저 인정하고 실제 merge, 집중 검사, 토큰 동등성과 성능 측정의 한계를 존댓말로 적는다.
후속 기록 PR에서는 이미 끝난 원 PR 승인·병합을 반복하지 않는다.

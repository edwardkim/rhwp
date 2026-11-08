# PR #8631 메인터너 검토 — HWP5 연결 BinData 레코드 순번

현재 판정: **수용 권고·검증 증적 보강 완료**. 로컬 필수 검증은 완료했다. 원격 처리 승인과 통합 후보의 새 head CI는 별도다.

- 원 PR: https://github.com/edwardkim/rhwp/pull/8631
- 기여자: `hyunmyungJaneLee`, fork `kidsnote/rhwp`.
- 관련 이슈: #8594.
- source head: `b76d7b9312815d6099b97e63923b0a2585fe03cd`.
- 기준 devel: `5716a226b562b7300193dd0b573ded9cd71ac890`. 처음 기준 `170b886f`에서 최신 #8659를 충돌 없이 반영했다.
- 오래된 순서의 누적 적용: #8515 → #8599 → #8608 → #8631 → #8656.
- 이 PR을 보존한 로컬 merge: `c2ae1cde51f7bd8e71b0d5638b670c20b20257da`. 충돌 없음. 별도 PR에 의존한다고 단정하지 않는다.
- reviewer `edwardkim` 지정을 API로 확인했다. 기존 #8585 수용 이력과 GitHub FIRST_TIME_CONTRIBUTOR 표시는 구분했다.
- 원 fork의 실제 push 권한이 없어 승인된 upstream 통합 경로를 사용했다. 원 기여 SHA와 fork branch는 보존했다.

## 주장·기존 해결 여부·소비 계약

Link 레코드 자리 때문에 뒤 내장 그림의 순번이 밀리지 않게 빈 자리를 보존한다. HWP/HWPX 실제 내장 파일을 쓸 때 그 자리만 제외한다. 저장·재열기에서 그림별 바이트·자르기를 검증한다.

최신 devel에서도 원 문제를 재현했다. 원 PR의 주장을 이미 해결된 중복으로 분류하지 않는다. 원본·독립 Print·보정 출력의 실제 비교와 사양/선행 PR 연결은 [공통 검토 근거](../assets/hyunmyungjanelee-20261106/README.md)를 따른다. 저장 줄 수용과 편집 후 재조판 계약을 혼동하지 않으며, HWP5와 HWPX 저장의 바이트/참조 검증은 해당 formal case로 확인한다.

## 시각·회귀 증거

[최종 로컬 검증](../assets/hyunmyungjanelee-20261106/final-validation.md), [전체 실행과 보안 재사용 집계](../assets/hyunmyungjanelee-20261106/required-validation-completion.json), [실제 소비 경로](../assets/hyunmyungjanelee-20261106/consumer-path-review.md)를 확인했다. 검증 src tree는 `c1ea19fbc94dd6aac77d4368755b6da5a9544e42`다. 원 사례 전체 7쪽의 최저는 Native 99.91617%·fresh optimized WASM 99.91621%이고, 경계·정상까지 각각 12쪽이 모두 90% 이상이다. 각 원 PR의 입력·쪽별 raw와 직접 판독용 review/overlay는 아래 계획에 연결한다.

집중 27개·전체 12,752개·Native 공식 3종 4,064개와 모든 lint가 통과했다. 원본 8개를 실제 전달한 보안 코퍼스 6개는 같은 전체 회귀의 통과 결과로 확인했다. 최종 편람 383쪽은 99.96465% 이상·기준 하락 0쪽이다. #8659의 Native 55쪽은 같은 조건의 base/후보 PNG가 모두 같으며 fresh WASM 55쪽도 전부 90% 이상이다. 시각 글꼴 보정과 처음 실패한 보조 드라이버는 성공 결과와 구분해 보존했다. 작성자의 자체 코퍼스 집계는 독립 검증으로 올리지 않았다.

## Merge 후 contributor PR comment 계획

[댓글 초안](../assets/hyunmyungjanelee-20261106/comment-pr-8631-draft.md)과 [7쪽·양쪽 출력 경로의 대응 manifest](../assets/hyunmyungjanelee-20261106/comment-visual-manifest.json)를 따른다. 첫 게시부터 이 원 PR의 실제 재현물·각 페이지에 맞는 Native/WASM review와 overlay를 포함한다. 다른 PR이나 추가 경계의 대표 이미지로 대체하지 않는다.

원격 승인·새 head CI·실제 self-review·merge·운영 기록 반영·devel sync 뒤에만 게시한다. `integration_pr`와 `merge_sha`를 실제 확정값으로 치환하고 고정 raw 응답의 MIME·SHA-256을 로컬 asset과 대조한다. 원 기여 성과를 먼저 인정하고 메인터너 보정과 남은 범위를 명확히 적는다. 게시 뒤 API 본문·이미지 표시를 확인한다. merge는 완료했다. 이 운영 기록 시점에는 댓글을 아직 게시하지 않았으며, 기록 push·devel 동기화 뒤 아래 계획대로 게시하고 API 본문과 실제 화면을 확인한다.

## 원격 범위

작업지시자가 기존 차이 한정 예외·통합 branch push·Open PR·새 head CI·self-review·merge·원 PR 완료·감사 댓글·운영 기록·devel 동기화·정리를 승인했다. [승인 기록](../assets/hyunmyungjanelee-20261106/remote-approval.json)에 범위를 고정했다. CI 및 원격 실행의 실제 완료는 [통합 완료 기록](../assets/hyunmyungjanelee-20261106/remote-completion.md)에 구분했다.

최종 제출·후속 범위는 [원격 처리안](../assets/hyunmyungjanelee-20261106/remote-action-plan.md)에 기록했다. 원 fork에는 push 권한이 없어 원 기여 SHA를 보존하는 upstream 통합 PR 경로로 병합했다.

## 최종 원격 검증·병합

- 검증 head: `71052fccc6fc5090fa94ca9dd013be0ff5984ff7`. 최신 base `5716a226b562b7300193dd0b573ded9cd71ac890`와 실제 merge-tree를 확인했다.
- 통합 merge: [`c80f355faa2bd810eda0724ec606027e7bbc019f`](https://github.com/edwardkim/rhwp/commit/c80f355faa2bd810eda0724ec606027e7bbc019f), 2026-11-06 14:39:03 KST. 원 기여 SHA 5개를 모두 조상으로 보존했다.
- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/42166096972)·CodeQL·Render Diff·Adapter·Proptest와 [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42169452466)가 성공했다. CI Archive A–E의 실제 결과는 12,558 passed·기존 45 skipped이며 로컬 12,752 passed와 구분한다.
- self-review의 새 차단 사항은 없다. 기존 #8599 뒤 본문 차이에 한정한 승인 예외를 적용하며 #8579는 OPEN으로 유지한다. 보안 봇 기존 3건은 변경 없는 선택적 진단 로그로 판독했고 분류를 바꾸지 않았다.
- 후속 기록은 maintainer 직접 반영 옵션 M을 사용한다. 제품·test·workflow·golden을 바꾸지 않고 이 archive와 공통 assets·오늘할일만 한 운영 기록 commit으로 반영한다. 이후 관련 이슈·각 원 PR 댓글·작업 공간 정리를 순서대로 완료한다.

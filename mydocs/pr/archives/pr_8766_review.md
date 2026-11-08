# PR #8766 self-review — Oracle의 내장 리소스 checkout 보정

## 판정

승인·병합 완료. code head `569020c1cc5f74c8bcd4de39c65b6baa5f58ddcf`의 필수 CI와
실제 Oracle 비교를 확인한 뒤, 2026-11-08 11:18:59 KST에
`08914c0ed00119db2f26b4fa7ea8eb0495b76cfd`로 devel에 병합했다.
[공개 self-review](https://github.com/edwardkim/rhwp/pull/8766#pullrequestreview-5816106632).

경로는 collaborator self-review와 CI 변경 후속 기록 옵션 2다. 작업지시자가 보정 PR·CI·
self-review·merge·옵션 2 기록·새 승격 검증을 승인했다. 이 기록은 원 PR에 trailing commit을
추가하지 않고 병합 후 별도 기록 PR로 제출한다.

## 검토 근거

0.9.0 승격의 [기존 Oracle 실행](https://github.com/edwardkim/rhwp/actions/runs/42434354283)은
workflow 결과가 success여도 실제 verdict가 `build-failed`, `promotionEligible=false`였다.
sparse checkout에서 renderer가 include하는 글꼴 메트릭·수식 outline·글꼴 규칙 3개가 빠졌다.
원 해결 #6689의 include 보강과 #7605의 vendored Cargo 입력 보강을 추적했다.

checkout에 `assets` 한 줄을 추가하고 실제 production source의 상대 include 경로를 해석해
checkout 포함 여부를 검사했다. 제품 Rust·feature·공개 API·조판·검사 명령·허용 기준·권한을
바꾸지 않았다. 새 시각 회귀와 독립 Print는 이 변경에 비해당이다. 새 suite 호출도 추가하지 않았다.

## 실제 검증

- Oracle·승격 계약 65건 통과. 새 입력 계약은 보정 전 실패 → 보정 후 성공이다.
- 실제 새 cone sparse checkout에서 세 입력 바이트를 원본과 대조했고 `cargo check --locked
  --profile release-test --bin rhwp`가 118초에 성공했다.
- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/42435768237) attempt 1 성공:
  Archive A–E 12,803건, 선택 Native 237건, Studio 2,777건 통과·기존 skip 2건.
  Lint·Frontend 후속 검증·Build & Test 집계도 성공했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42435768269)·
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42435768232)·
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42435768190)와 최종 정책 감사가 성공했다.
  수동 재실행·취소·보안 경고 분류 변경은 없었다.
- [수정된 Oracle](https://github.com/edwardkim/rhwp/actions/runs/42435782440)의 실제 artifact를
  확인했다. build/compare/pack은 모두 success, verdict는 `completed`,
  `promotionEligible=true`다. 859쌍 중 match 857·mismatch 0·error 2이며 unpaired 372가
  남는다. 이는 advisory 원장으로 보존하며 전 사례 무결함을 주장하지 않는다.
- merge 직전 head·base·모든 check를 다시 조회했다. base `78615c52a7c7d7a6b25fad57c3bf1041fb2dfcf2`와의
  merge tree `96f11da5c33f2ddcfe7500516fd3b571532b0dac`는 GitHub 합성 merge tree와 같다.
  정상 `--merge --match-head-commit`을 사용했으며 admin 우회는 없었다.

[CI·로그 요약·병합 근거](../assets/pr8766/ci-results.json),
[Oracle 실제 verdict](../assets/pr8766/oracle-verdict.json).

## 후속 범위

이 기록 PR의 실제 review-only preflight·required check·병합을 확인한다. 별도 승인된
Full Renderer Sweep의 public 표본 전달과 Legacy/Layer 본문 clip 보정·로컬 필수 검증은
진행 중이다. 그 보정까지 통합한 최종 devel SHA에서 13개 승격 workflow를 실행한다.
main 승격·태그·실제 게시를 이 PR의 완료에 포함하지 않았다. #8745는 OPEN을 유지한다.
릴리즈 빌드가 실제 사용 중인 output·target은 사용 종료 뒤 정리하고 다른 작업은 보존한다.

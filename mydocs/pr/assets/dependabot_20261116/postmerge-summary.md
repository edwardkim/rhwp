# Dependabot 통합 PR #9134 완료 기록

## 최종 판정

승인 — [PR #9134](https://github.com/edwardkim/rhwp/pull/9134)의 code head
`77393b9f96f1011d624d2c4d72a579e9d77f2a89`에서 Full CI와 필수 검증을 완료했습니다.
정상 merge `34fa7c621029e3efc287591b2d37e31e09de0366`의 devel 포함을 확인했습니다.
보호 규칙 우회는 사용하지 않았고 원 작성자의 체리픽 이력을 보존했습니다.

## 적용과 검증

- 원 봇 PR #9110–#9125의 목표 버전·원 SHA·실제 체리픽은 [원장](source-prs.json)을 따릅니다.
- 로컬 최소 글꼴 Rust 전체13,617PASS·0FAIL·45SKIP과 원격 CI 5개 archive13,438PASS·0FAIL·45SKIP을 구분합니다.
- Native/fresh WASM 각각18종1,771쪽의 공식 보호 표시값 하락·누락은0이며, 각 경로의 base/head 원본 PNG도 같습니다. 기준 상승이 없어 정본은 수정하지 않았습니다.
- Studio 전체2,805PASS·0FAIL·2SKIP, 실제 출력30캡처의 runtime hard violation0,
  Firefox 다운로드3건·뷰어1개·편집 내용 보존까지 확인했습니다.
- 수식의 기존 report-only parity4건은 같은 browser·TS·기준 WASM으로 재생한6개 PNG가 후보와
  모두 byte-identical임을 [추가 확인](equation-report-only-base-comparison.json)했습니다.
- 게시 TSV72개·PNG13개의 실제 다운로드 바이트가 제출 commit과 같음을 [검증](posted-evidence-validation.json)했고,
  사용자 지시에 따른 본문 변경 후에도 같은85개 URL이 유지됨을 [확인](posted-evidence-scope-update.json)했습니다.

[정확한 CI·병합 영수증](code-ci-and-merge-final.json), [통합 분석](analysis.md),
[최종 소비 검증](applicable-gates-validation.json), [검증 재사용 범위](post-repair-reuse-audit.json)를 따릅니다.

## 후속 기록 범위

CI action 변경이 포함되어 옵션2로 통합 검증 기록과 오늘할일만 별도 기록 PR에 반영합니다.
사용자가 봇의 개별 PR review를 생략하도록 지정했으므로 원16개별 archive 리뷰나 상세 리뷰 댓글은
추가하지 않습니다. [범위 변경 기록](bot-review-scope-update.json)을 따릅니다.
원 PR16개는 통합 반영 확인 후 짧은 병합 링크 댓글과 supersede close로 처리하는 대상입니다.
이 문서는 원 PR close·기록 PR 병합·종료 정리가 실행되기 전에 그 완료를 주장하지 않습니다.

병합 후에는 검증 workflow를 재실행하지 않으며 자동 duration 갱신 상태를 확인합니다.
사용자 지시에 따라 output과 다른 작업·사용자 변경은 보존하고 적용되는 소유 branch·검토 산출물만 정리합니다.

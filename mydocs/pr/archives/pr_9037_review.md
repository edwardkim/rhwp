# PR #9037 검토와 후속 처리

## 최종 판정

**승인·merge 완료.** seo-rii의 [#8952](https://github.com/edwardkim/rhwp/pull/8952)·[#8953](https://github.com/edwardkim/rhwp/pull/8953)·[#8954](https://github.com/edwardkim/rhwp/pull/8954)를 원 기여 SHA를 보존한 merge commit으로 통합했다. macOS는 작업지시자의 기여자 설명 수용 결정을 적용했고 실제 Apple SDK 검증으로 확대하지 않았다.

- 검증 head: `728fc70dbd4de0d1fc26a6aaec4a507eb0ddfe5f`.
- base: `41c889cfd8a869ad23199d6018f8b569879a575c`.
- merge: `2c7644c53171e21c27b8a532ce4c53f01c40ddff` (2026-11-13 09:29:20 KST).
- [게시 self-review](https://github.com/edwardkim/rhwp/pull/9037#issuecomment-6840051408).
- [그룹 검토](seo_rii_20261112_review.md)·[검증 원장](../assets/seo-rii-20261112/windows-correction-20261113/README.md)·[명령·hash·결정](../assets/seo-rii-20261112/windows-correction-20261113/validation-record.md).

## CI와 실제 검증

32개 success·정상 skip 5개와 최종 정책 감사 success를 확인했다. Native·Lint·Frontend·CodeQL 4개 언어·Adapter·Proptest·회귀 A–E가 성공했다. 회귀 실행은 아래와 같다.

| Archive | PASS | 정상 skip |
| --- | --- | --- |
| A | 3,796 | 12 |
| B | 2,460 | 8 |
| C | 2,417 | 2 |
| D | 2,276 | 2 |
| E | 2,287 | 21 |
| 합계 | 13,236 | 45 |

CI [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/43333224593)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43333224573)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43333224506)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43333224421)를 각각 확인했고 재실행·workflow/보호 설정 변경은 없었다. Frontend 실제 Studio runner는 2,666 PASS·0 FAIL이다. 로컬 Native 연결 Flutter 280개와 혼용하지 않는다.

실제 Windows junction·실행 파일 metadata·종료 messenger 오류를 메인터너가 보정했다. 최종 Windows 두 타일 4단계·자원 회수, 양 OS 이동 번들의 문서 열기·쪽 이동·문서/창 정상 종료를 다시 실행했다. Native/fresh WASM 각각 전 8쪽은 독립 Desktop Print와 쪽수가 같고 최저 99.75%·99.86%, 원 범위 7쪽은 모두 100.00%다. Windows 실제 DPR 1이며 consumer GPU fence·hardware acceleration·target reuse·driver memory·두 OS DPR 비교는 완료로 주장하지 않는다.

Rust 조판·저장 규칙은 바꾸지 않았다. 최신 base와 후보의 공통 생산 입력 및 실제 merge tree가 검증 결과와 일치한다. source4ab의 실제 Windows 소비자 재빌드와 source60의 Native/WASM 재사용을 구분했다. 기준 입력·PDF는 정식 저장소 경로를 재사용했고 glyph·음영·외곽선·간격을 직접 판독했다.

## 원 PR과 이슈

원 PR 세 건은 모두 GitHub API `merged=true`로 자동 완료됐다. 원 fork branch·Draft 6개는 보존한다. 부모 [#536](https://github.com/edwardkim/rhwp/issues/536)·[#8701](https://github.com/edwardkim/rhwp/issues/8701)은 전체 로드맵이 남아 OPEN을 유지한다.

## 후속 운영 경로

[duration refresh](https://github.com/edwardkim/rhwp/actions/runs/43337460272)는 success다. 병합 뒤 제품 검증 CI를 시작·재실행하지 않았다. archive review·오늘할일만 maintainer 운영 기록(옵션 M)으로 devel에 직접 반영한다. 원 소스·test·workflow·golden은 운영 기록에 섞지 않는다.

본 기록의 원격 반영·최종 devel sync 뒤 원 PR별 댓글은 각 [8952 계획](pr_8952_review.md#merge-후-contributor-pr-comment-계획)·[8953 계획](pr_8953_review.md#merge-후-contributor-pr-comment-계획)·[8954 계획](pr_8954_review.md#merge-후-contributor-pr-comment-계획)을 따른다. 원 기여·메인터너 보정·범위별 미실행을 구분한다. 최종 PNG 7개는 merge SHA의 실제 raw 바이트와 같음을 확인했다. 댓글 게시 뒤 API 본문과 이미지 로딩을 확인하고 정본의 전용 branch/worktree·output/debug 종료 정리를 수행한다. 정리 결과와 게시 permalink는 ignored `local/pr-review-cleanup/seo-rii-pr9037-20261113.json`에 확정한다.

## 추가 provenance 확인

GitHub가 실제 CI에서 검사한 merge `cccc87ed81543e01b62a30480f1434fd7436e90a`도 Git Data API의 tree가 `188efd7a0414605f12ba4bb55e677500f73dd6c9`다. 로컬 simulation·실제 CI merge·최종 제품 merge의 tree가 모두 일치한다. duration refresh는 `successful-pr-worker-measurements`로 자료를 수집했고 `published=true`, data commit `a0e97ef58ed2de7db46ebd45302b46dc4a0cc948`, source merge `2c7644c53171e21c27b8a532ce4c53f01c40ddff`를 기록했다.

# PR #8760 self-review — 0.9.0 안내와 crates.io 초회 배포 범위

## 최종 판정

승인 — 제출 head `2919f7cc59b5c468bb79694463fac40483f6ad99`의 실제 Full CI·CodeQL·Adapter·Proptest와 최종 정책 감사가 모두 성공했다. 제품·문서 검토에서 머지를 막는 문제를 발견하지 않았다. 이 기록 뒤의 문서-only trailing head는 required check·mergeability를 별도로 확인한 뒤 병합한다.

## 접수와 승인

- PR: https://github.com/edwardkim/rhwp/pull/8760
- 제출 head: `2919f7cc59b5c468bb79694463fac40483f6ad99`
- base: `0e6cfe91df7aabb886319346f78af568def25a64`; devel 대상 Open PR.
- 기본 경로: collaborator_self_merge. 보조: intake_and_review, local_validation, review_only_fast_pass, post_merge.
- 사용자 승인: 이 문서 후보의 push·Open PR·해당 head CI·self-review·merge·동기화·종료 정리.
- 원 코드 PR #8757의 옵션 2 후속 기록과 새 배포 문서 보완을 포함한다. 이번 PR은 CI 실행 정의를 변경하지 않는다.

## 검토 결과

- 사용자 결정: crates.io의 초회 배포를 0.9.0에 포함하도록 MD01을 갱신했다. 과거 결정 날짜·출처는 보존했고, 14개 패키지·Rust 1.93.1·수동 게시 책임 계약은 유지했다.
- 배포 상태: 모든 설치 예제는 게시 후의 사용법으로 표시하고 준비 중에는 같은 checkout을 쓰도록 안내했다. tag 자동화가 crates.io를 게시한다고 주장하지 않는다.
- 사용자 문서는 main 승격 전에 devel에 통합한다. main 이후로 미룬 사용자 안내는 없으며 실제 게시 결과는 실행 뒤 확정한다.
- Firefox AMO: 0.9.0용 노트를 새로 작성하고 기존 0.8.7 자료를 보존했다. 현재 manifest·build.mjs와 이전 실제 source ZIP 재빌드 근거를 대조했다. 최종 source로 다시 포장해야 함을 명시했다.
- Chrome/Firefox README: 현행 안내를 0.9.0으로 추가하고 이전 0.8.7 변경 목록을 변경 이력에 보존했다. 자동 문서 감지 범위와 공통 PDF 엔진 기능을 구분했다.
- 실제 manifest의 permissions·host_permissions·CSP·browser_specific_settings를 v0.8.7과 대조해 동일함을 확인했다. 새 브라우저 권한·최소 버전·수집 항목을 추가하지 않았다.
- 기여자: 네 국·영문 CHANGELOG/릴리즈 노트의 33명 명단·범위 SHA·4,852개 커밋 수가 원장과 일치한다. #8754·#8269 기능은 추가했고 관련 저자는 기존 명단에 있다. 최종 승격 범위 재감사는 게시 전 남은 단계로 표시했다.
- #8757 기록은 원 head·실제 CI·Native 1회 재실행·최종 merge tree·실제 duration 게시 결과와 연결했다. 이전 CI 수치를 새 후보의 CI 결과로 쓰지 않았다.
- 조판 원칙과 검증 입력: 이번 diff는 Markdown와 과거 CI JSON 증거뿐이다. 제품 source·tests·workflow·fixture·시각 baseline 변경이 없으므로 새 Native/WASM Visual Sweep과 입력 PDF 첨부는 비해당이다.

## 완료한 로컬 검증

- 배포 문서 19개와 별도 후속 기록 3개 링크 검사 통과.
- 기존 릴리즈 버전·명단·포장 계약 검사 6건 통과.
- 과거 제출 기록 미변경·manifest 권한 동일성·git diff --check 통과.
- 고정 base와 git merge-tree 종료 0. tree `0e711ec69eaf3921953a21b5bcfad90ca5ac50c5`.
- 제품 구현에 변경이 없어 기존 광범위 로컬 빌드·회귀를 반복하지 않았다.

## GitHub CI

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/42427026044) attempt 1 성공: Archive A–E 합계 **12,803건**, CI 선택 Native **237건**, Studio **2,777건 통과·2건 건너뜀**. Lint와 Frontend 후속 검사·Build & Test 집계도 성공했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42427026009), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42427026096), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42427026035) 성공. 최종 `CI Impact Policy`도 success다. check 31개 성공·5개 skipped이며 실패는 없다.
- 제출 head와 실제 합성 merge `213f5f2b8300e547af91b0357f00e666d2d3393a`의 base는 `0e6cfe91df7aabb886319346f78af568def25a64`다. 합성 tree는 로컬 병합 시뮬레이션과 같다.
- 기존 classifier가 하위 package README를 미분류 경로로 보아 Full CI를 선택했다. 검증 기준·분류 정책을 수정하지 않았다. Render Diff는 path filter상 비대상이며 실행 성공으로 집계하지 않았다.
- 실패·수동 재실행·취소·보안 경고 재분류는 없었다. 기존 #8757의 검사 수와 구분해 실제 job 로그·artifact로 수치를 산출했다. [CI 증거](../assets/pr8760/ci-results.json).

## 후속 처리

이 review·오늘할일을 CI 성공 뒤 문서-only trailing commit으로 기록하고 해당 head의 실제
check·mergeability를 확인한다. 일반 merge 후 devel 동기화·duration 자동 실행 결과 또는
실행 대상 아님을 확인한다. 이번 PR로 #8174·#8745를 닫지 않으며 실제 registry 게시를 수행하지 않는다.
다른 작업의 worktree를 보존하고 이번 임시 branch·output·사용 종료한 debug만 규정에 따라 정리한다.

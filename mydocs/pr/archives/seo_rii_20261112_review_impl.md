# seo-rii P117–P119 누적 검토 계획

## 목표와 승인 범위

Ready인 #8952·#8953·#8954를 검토하며 Draft #8962–#8967은 제외한다. 기본 경로는 maintainer 일반이며 intake·local validation·multi PR·시각 증적·후속 절차를 적용한다. 현재 승인은 접수·로컬 검토와 Windows 메인터너 보정·필수 재검증이며 공개 댓글·push·승인 리뷰·merge는 별도다.

작업지시자는 “macOS 쪽은 기여자의 주장을 수용하면서 진행합니다. 현재 메인테이너가 macOS 테스트 환경을 제공할 수 없기 때문입니다.”라고 결정했다. 이번 #8952 및 다른 두 PR의 macOS 관련 주장에 이 결정을 적용한다. 메인테이너의 Apple SDK·Metal/AppKit·ScreenCaptureKit 실기 검증은 미실행으로 구분하고 portable/SDK 계획 검사를 실제 Mac 실행 증거로 확대하지 않는다. 실행으로 검출한 코드 결함과 Linux·Windows 필수 검증은 별도로 판단한다. 같은 Mac 결정을 재확인하지 않는다.

## 고정 기준과 순서

- 기준 devel: `a85d3265dd10b316d1a0ae029ca3f67245b38b77`.
- 검토 branch: `review/integration-8701-seo-rii-reference-qualification`.
- worktree: `/home/edward/mygithub/rhwp-worktrees/review-integration-8701-seo-rii-reference-qualification`.
- #8952 원 head: `3077c620f3ec7c09cd93c2b79e7ea0825be12488`.
- #8953 원 head: `9cfe2754d92a720135445aed285ff91c0c75dc7e`.
- #8954 원 head: `1e14e4b8466b7ce46d9016faf1824bc05c9789de`.

각 원 PR에 reviewer edwardkim을 지정하고 API로 head와 배정을 확인했다. 개별 최신 devel 병합 simulation 3건은 exit 0이다. 원 SHA는 보존 refs와 merge ancestry로 유지하고 오래된 순서로 누적 검토한다. 충돌이 생기면 실제 파일·방침을 기록하고 승인된 보정 범위 안에서만 해결한다.

## 선행 처리와 CI

선행 #8869–#8874 → 수용 #8907 → merge `0f91d29ca81f00be3d5adb91857875b40ec5e355`와 Linux accepted snapshot receipt 보정, 당시 Mac 제외 결정을 읽었다. 현재 코드의 해당 공통 ownership/receipt·close·자원 한도를 보존한다. Mac의 이번 수용 결정은 앞선 예외와 구분한다.

원 PR 본문에 남은 Ubuntu mirror DNS·Poppler 준비 실패는 과거 attempt 1 기록이다. 현재 동일 head의 CI run 43053929805·43053940397·43054349242는 attempt 2 success이며 PR rollup도 모두 성공/정상 skip이다. Copilot COMMENTED 외 다른 제출 리뷰는 없다. 원격 재실행·취소·정책 변경을 하지 않았다. GitHub CI와 직접 실행하는 플랫폼·화면 검증을 구분한다.

## 한 단위의 로컬 검토·검증

1. 실제 diff와 선행 계약을 읽고 #8701·#536의 미완료 로드맵과 각 PR의 고유 범위를 연결한다. 기여자 테스트·증거의 source/hash와 실행·미실행·재사용을 구분한다.
2. #8952는 공통 C++ bounded 두 타일·shared lease·부분 실패 rollback·전체 receipt·cover/close/ack와 provenance 반례를 실행한다. Apple SDK·실기 미실행에는 위 사용자 결정을 적용한다.
3. #8953은 bundle manifest/entry/source/native hash·상대 경로·실패 거절 검사를 실행하고, 실제 Linux 참조앱을 빌드해 이동한 공백 경로에서 환경 변수 없이 문서 열기·표시·닫기·정상 종료를 확인한다. Windows 호스트의 도구·실행 경로도 조사하여 가능한 실제 SDK·소비 경로를 완료한다.
4. #8954는 실제 collector·stored evidence 정렬/전이 반례와 Dart/Native probe를 검증한다. Windows SDK·desktop·descriptor opening·viewport CPU 대조·종료/복구 및 실제 DPR 조건을 확인하며 주입·portable 검사를 실제 OS/GPU 검증으로 바꾸어 쓰지 않는다.
5. package/example analyze·관련 Python/C++/Flutter 테스트와 수정 범위의 필수 게이트를 완료한다. Rust/공통 조판 생산자 변경 여부를 코드로 판정하고 실제 producer/Native source·기본 출력 무회귀와 플랫폼 실제 화면을 연결한다. 새로운 source 보정이 필요하면 사유·소유자·필수 검증을 구체화한다.
6. 개별 원 PR review에 판정·해제 조건·검증과 댓글 계획을 남기고, 승인 가능한 실제 후보의 원격 처리 범위만 별도로 요청한다. #536·#8701 전체 로드맵은 OPEN을 유지한다.

## 2026-11-13 승인 범위 완료

Windows 보정·필수 재검증 승인을 적용했다. 최신 base `41c889cfd8`를 merge하고 원 SHA를 보존했다. CMake junction, 실제 실행 파일 stat 경계와 종료 시 messenger 접근을 공통 경로에서 수정했다. 최종 Windows runner·4단계 타일·이동 참조앱 정상 종료와 Linux 대조, 최신 Native/fresh WASM 및 새 devel 기능 보존을 [최종 원장](../assets/seo-rii-20261112/windows-correction-20261113/README.md)에 기록했다. 초기 실패를 현재 미완료로 남기지 않으며 원격 처리·해당 head CI는 별도 승인 단계다.

## 원격 후속 확정

수용 PR [#9037](https://github.com/edwardkim/rhwp/pull/9037)의 CI·self-review·merge와 원 PR 3건의 간접 MERGED를 확인했다. [최종 종료 기록](pr_9037_review.md)을 따른다. 초기 승인 대기·보류 문단은 당시의 조사 이력이다.

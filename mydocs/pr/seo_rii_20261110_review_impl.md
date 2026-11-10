# seo-rii P111–P116 누적 PR 검토 계획

## 범위와 승인

2026-11-10 작업지시자의 “seo-rii 기여자의 PR 처리를 시작합니다.”에 따른 접수·로컬 검토다.
원격 comment·push·통합 PR 생성·merge와 CI 취소·재실행은 별도 승인 범위로 구분한다.
확인된 결함의 제품 보정 또는 충돌 해결이 필요하면 실제 근거와 제한 보정안을 먼저 기록한다.

```text
base route: maintainer_general
modifiers: intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, rework_and_exceptions, post_merge
loaded documents: pr_review_workflow.md, pr_review/README.md 및 위 자식 문서
current base: 5f4e95c187f64116f48f710c3c048162786c1c14
```

- 작업 브랜치: `review/integration-8701-seo-rii-gpu-desktop`.
- 작업공간: `/home/edward/mygithub/rhwp-worktrees/review-integration-8701-seo-rii-gpu-desktop`.
- Draft 제외, 열린 대상 6건 모두 reviewer `edwardkim`을 원 PR별 지정한 뒤 API로 확인했다.
- 대형 누적 PR이므로 단계별 고유 변경·의존성과 실제 소비 경로를 분리한다.
- 이전 [P103–P110 수용 기록](archives/seo_rii_20261109_review.md)·#8809와 부모 #536/#8701을 선행 근거로 확인한다.
- 기준 devel에는 [#8889 보정](archives/pr_8889_review.md)과 #8893 후속 기록이 추가됐다. 과거 기여자 검증의 base `3170fddc48`와 구분하며 현재 renderer·자원·소유 계약을 보존한다.

## 고정 원 head와 순서

| PR | 단계 | 원 head | 개별 검토 |
| --- | --- | --- | --- |
| [#8869](https://github.com/edwardkim/rhwp/pull/8869) | P111 | `f7f35452d91886ee24d0f80202ddaaa2d21047af` | [검토](pr_8869_review.md) |
| [#8870](https://github.com/edwardkim/rhwp/pull/8870) | P112 | `20eb3d8d902f03586db6a946e44b7b9b1308f0c3` | [검토](pr_8870_review.md) |
| [#8871](https://github.com/edwardkim/rhwp/pull/8871) | P113 | `638469a18d49f5e5e1b943167134b293c3198752` | [검토](pr_8871_review.md) |
| [#8872](https://github.com/edwardkim/rhwp/pull/8872) | P114 | `8fbd30694ce2e265ec23b8fedeb4e8765f94d455` | [검토](pr_8872_review.md) |
| [#8873](https://github.com/edwardkim/rhwp/pull/8873) | P115 | `3e219520e229e7c4aa78c7f0ccc53a608ea6dffc` | [검토](pr_8873_review.md) |
| [#8874](https://github.com/edwardkim/rhwp/pull/8874) | P116 | `84c88a661a0d21c6d7b3ddf59f331dbcaffb1df8` | [검토](pr_8874_review.md) |

## 현재 관측과 로컬 통합

- 원 기능은 최신 devel의 조상이 아니며 개별 `git merge-tree --write-tree`는 6건 모두 exit 0이다.
- 원 PR의 최신 head를 보존한 채 오래된 순서로 로컬 merge한다. 겹치는 선행 이력은 Git 조상 관계로 확인하고 중복 적용하지 않는다.
- 예상 밖 충돌이 생기면 구현을 고치기 전에 충돌 파일·양쪽 계약과 보정안을 기록한다.
- 현재 변경은 `bindings/Flutter` 47개 경로에 한정된다. Rust parser·조판·Native ABI 생산자는 변경하지 않으며 Flutter 소비·소유·복구·배포 계약의 비해당 판정은 실제 최종 diff로 확인한다.
- #8869·#8872의 Adapter preflight 취소 이력은 성공으로 표시하지 않는다. 실제 run·job 로그와 최신 required check를 조사한다. 다른 네 원 head에는 조회한 실패 check가 없다.
- 원 PR 본문의 “후속 qualification 위임”은 기여자의 범위 설명이며, 이번 검토의 필수 검증에 대한 사용자 예외 승인으로 해석하지 않는다.

## 검증 계획

1. 선행 #8809 source와 누적 P111–P116의 고유 변경을 대조해 최신 devel에서 기능이 이미 해결됐는지 확인한다.
2. Flutter SDK·analyze·변경 Dart format·전체 unit/widget와 실제 Native 연동 경로를 실행한다. 대역·가상 시간·실제 Native·실제 compositor를 구분한다.
3. 공통 타일·lease·부분 실패·중복 ID·종료 계수, Windows backpressure와 production dispatcher, Python/Dart bundle의 경로·링크·상한·파일 변경·ABI 거절 및 실제 이동 경로를 실행한다.
4. Linux 실제 Flutter SDK·compositor와 Windows 호스트 SDK에서 변경 경로를 검증한다. macOS는 source·SDK 실행 증적의 동일성을 먼저 확인한다. 실제 SDK 미실행을 portable 검사의 통과로 대신하지 않는다.
5. GPU 다중 페이지·tile seam·pause/복구·DPR/resize·입력/문서 보존의 실제 소비 결과를 확인한다. 렌더링 출력은 독립 Print와 최신 devel/후보의 Native·fresh WASM 대조 및 직접 판독으로 검증한다.
6. P115는 readiness 호출·stale guard·bounded trace와 개인정보 범위를 실제 제품 경로에서 확인한다. 가상 시간 개선을 물리 GPU 성능으로 표시하지 않는다.
7. 원 PR별 판정·미해결 필수 증적·재사용의 exact source 범위를 기록한다. 수용 후보가 준비되면 원 기여 SHA를 보존하는 원격 처리안과 실제 검증 결과로 승인을 요청한다.

Cargo는 기본 작업공간의 `/home/edward/mygithub/rhwp/target/pr-review`만 사용하며 다른 작업과 겹쳐 실행하지 않는다. 원시 API·실행 로그·진단 JSON은 ignored `output/pr-review/seo-rii-20261110`에만 두고, 필요한 최종 사실은 Markdown과 대표 직접 판독 PNG로 남긴다.

## 2026-11-10 메인터너 수용 범위 결정

작업지시자가 “기존 Mac 유지·새 타일 후속 처리”를 선택했다. #8870의 Mac native 타일 구현
두 파일은 최신 devel의 기존 소스를 유지하고 새 Mac 타일 지원은 #8701 후속으로 남긴다.
기여자의 원 commit SHA는 로컬 통합 이력에 보존하며 Linux·Windows·공통 Dart 기능은 계속 검증한다.
README·P112 안내에서 Mac 타일 구현 완료 주장을 제거한다. 기존 Mac GPU 단일 target 경로와
example의 Mac tileGroups 비활성 조건은 유지한다. 이 결정은 원격 push·PR·merge 승인이 아니다.

초기 통합 `4f6bc33f8e3940947f3d942cdb3af26bb1d6f7cb`에서 Flutter 계약·위젯 269건과 실제
Native 연동 6건, bundle Python 11건·실제 Dart FFI fixture 3경로, Windows portable·실제 SDK
compile/link, Linux portable·실제 SDK/GLES·GTK 두 타일 초기/편집/resize 및 실제 Native 번들
이동 검사를 통과했다. 원시 결과는 ignored output에서 확인하며 정식 검토에 실제 source·명령·수치와
직접 판독한 대표 PNG만 보존한다. Native Rust input은 기준 devel과 같으며 이번 Mac 제외로
변경되지 않는다. 이후 결과를 새 source에서 실행했다고 표시하지 않고 실제 소비 source의 동일성으로 연결한다.

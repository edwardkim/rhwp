# seo-rii P111–P116 누적 PR 검토 계획

**현재 상태: 승인한 보정·검증·통합 PR 머지 완료, 후속 기록과 종료 처리 중.** 원 head 6개는 그대로 보존했고,
최신 base `411d4a91fd`를 통합한 제품 source `857594293f`에서 실제 Native·최적화 fresh WASM·
GTK 검증을 완료했다. 아래 진단 시점의 보류 기록은 수정 전 근거이며 현재 미해결 결함을 뜻하지 않는다.
개별 판정은 `메인터너 보정 후 수용 가능`이며, 이후 원격 처리 승인을 받아 [통합 PR #8907](https://github.com/edwardkim/rhwp/pull/8907)을 `0f91d29ca81f00be3d5adb91857875b40ec5e355`로 머지했다. 원 PR 6건도 모두 MERGED다.

## 범위와 승인

2026-11-10 작업지시자의 “seo-rii 기여자의 PR 처리를 시작합니다.”에 따른 접수·로컬 검토다.
접수 당시 원격 comment·push·통합 PR 생성·merge와 CI 취소·재실행은 별도 승인 범위로 구분했다. 이후 작업지시자는 통합 PR·CI·self-review·merge·후속 처리를 승인했다. CI 취소·재실행은 수행하지 않았다.
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
- 이전 [P103–P110 수용 기록](seo_rii_20261109_review.md)·#8809와 부모 #536/#8701을 선행 근거로 확인한다.
- 기준 devel에는 [#8889 보정](pr_8889_review.md)과 #8893 후속 기록이 추가됐다. 과거 기여자 검증의 base `3170fddc48`와 구분하며 현재 renderer·자원·소유 계약을 보존한다.

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

## 실제 다중 페이지 GPU 경로에서 발견한 인수 확인 누락

후보 `1bfa7b298272249ef533310ea4f07ade944e2e09`의 실제 GTK plugin·Dart controller와 fresh
Native `4f6bc33f8e3940947f3d942cdb3af26bb1d6f7cb` 빌드로 재현했다. 두 source의 Rust/Native 입력은
동일하며 Native 라이브러리 SHA-256은 `397223df7aa3ff9a2a622651d4c01210b3c02aa9f906897f66c30568e6868424`다.
추적을 추가한 진단 runner는 제품 코드를 변경하거나 channel을 대역으로 바꾸지 않았다.

- 입력: 기존 공개 6쪽 `tests/fixtures/gradient/radial-corner-print-contract.hwpx`, SHA-256
  `01884d8ea148e21fdb68194dd23c353c635903e4fc19d8b242e1efbe1d0aaf0d`.
- 실제 `NativePagedDocumentView` 600×800, zoom 0.4, DPR 1에서 첫 visible batch의 snapshot ID는
  `[2, 3, 4]`였다. Native `submit`은 `accepted: true`와 일치하는 full `acceptedStamp`를 반환했다.
- Linux `submit`과 이어진 실제 `status` 3회 모두 `acceptedSnapshots`가 없었다.
  Dart는 모든 ID를 확인할 수 없어 `NativeCallException(ambiguousOwnership)`로 실패했다.
  첫 실패 후 wire 추적을 추가한 독립 실행에서도 같은 실패를 재현했다.
- `linux/rhwp_texture_plugin.cc:506`의 공통 `status_value`는 stamp만 직렬화하며,
  `texture_state.h`의 `Status`에도 accepted ID 목록이 없다. 기존 Linux 구현은
  `3de0f2eacc066726401cf153b6d9ba8be72ff9f7`에서 시작한 경로다.
- #8869 원 기능 commit `f03568a52eff4ac74c90ccdaa079c8fbaa10b122`는 여러 페이지에서
  `acceptedSnapshots`를 요구하도록 Dart를 바꾸지만 Linux 응답을 바꾸지 않았다. 최신 devel에서도
  응답 목록은 없으며 이미 해결된 문제가 아니다. Windows는 submit/status 모두 목록이 있고,
  기존 Mac 경로는 status에서 목록을 제공하므로 같은 누락이라고 추정하지 않는다.
- P112 group submit/status는 별도 accepted ID 목록을 제공한다. 앞선 실제 두 타일 검사는
  한 페이지가 두 타일에 걸친 경우로, 이번 단일 target의 여러 페이지 검증을 대신하지 않는다.
- 275개 Flutter 검사 중 다중 페이지 계약은 fake plugin이 목록을 제공하는 경로다.
  기존 통과 결과는 유지하되 실제 Linux 다중 페이지 GPU 표시의 성공 증거로 쓰지 않는다.

### 제한 보정 제안과 해제 조건

메인터너 보정 승인을 받으면 Linux의 공통 인수 상태와 MethodChannel 응답을 연결한다.
모든 fallible 검사·목록 할당이 끝난 뒤 같은 lock 아래에서 stamp·serial·인수 ID 목록을 함께
확정하고 submit/status가 그 동일한 결과를 소비하도록 한다. 목록은 기존 최대 8개와 중복 lease
계약을 보존한다. group 경로도 공통 상태의 일관성을 확인한다. Dart의 전체 ID 확인·ambiguous
소유권 보호를 완화하거나 이미 인수한 snapshot을 caller가 다시 해제하도록 바꾸지 않는다.

필수 검증은 다음 범위를 한 단위로 완료한다.

1. 실제 submit/status의 전체 receipt, 여러 페이지 최초 표시와 navigation, 같은 editor owner의
   CPU 출력 대조, close 후 Native 9개 계수 0을 확인한다.
2. 단일 페이지·타일 group 정상 대조군, 중복 ID·stale 거절·교체·종료와 lease 1회 해제를 검사한다.
3. Linux portable·실제 Flutter SDK compile/link·GLES 상태·GTK 표시를 재실행하고 관련 Dart
   계약·analyze를 재검증한다. 변경된 실제 화면을 직접 판독해 정식 Markdown·대표 PNG로 보존한다.

진단 시점의 판정은 **선행 #8869의 실행 검출 결함 때문에 누적 후보 머지 보류**였다. 원시 로그는 ignored
output의 `multipage-capture`와 `multipage-wire-capture`에만 보관한다. 당시는 제품 보정 승인 전이었으며 원격 push·공개 댓글·PR·merge는 수행하지 않았다.

### 메인터너 보정 승인과 집중 검사

작업지시자가 위 원인·제한 보정안의 “메인터너 보정·필수 로컬 검증”을 승인했다. Linux `Shared`
상태에 인수한 고유 snapshot ID 목록을 추가하고, 기존 ownership commit의 같은 lock에서
stamp·serial·목록을 확정했다. 목록 할당은 소유권 이전 전에 완료한다. 공통 `status_value`가
submit과 status 모두에서 이 결과를 직렬화하며 group의 두 member 상태도 같은 union receipt를
갖는다. Dart의 전체 ID 확인·ambiguous 보호와 Mac 제외 범위는 바꾸지 않았다.

Linux portable의 실제 `Shared::submit/status` 검사 18건·292조건과 공통 tile 4건·25조건이
통과했다. 새 관계형 검사는 여러 페이지·중복 placement의 단일 lease·stale 거절 뒤 receipt 보존·
다음 인수로 교체·close 뒤 지연 응답 조정과 한 번 해제를 확인한다. 격리한 임시 사본에서 단일
target의 receipt commit만 제거하면 컴파일은 성공하고 새 receipt assertion이 실제 실패했다.
이 보정 code head를 고정해 실제 SDK·GTK 다중 페이지와 정상 타일 재검증을 계속했고 아래 결과를 확인했다.

### 실제 경로 재검증과 새 devel

보정 `9e02baf1a17fbd8c8b57eb44e3890e5c2325c32d`에서 실제 GTK의 다중 페이지 최초 표시·이동과
CPU 대조, mount를 유지한 종료·명시적 session 복구, 주입한 숨김·재개 신호와 실제 X11 창 resize,
trace ring·producerReady·개인정보 제외, 종료 후 Native 9개 계수 0을 확인했다. 두 타일도 3회
재검증했으며 보정 전후 compositor RGBA 3쌍이 모두 같다. [실제 결과 원장](../assets/seo-rii-20261110/README.md)에
단계·수치·직접 판독 PNG와 플랫폼 한계를 보존한다.

제출 전 조회에서 원격 devel이 `411d4a91fd`(#8899, 원 #8863)로 전진했다. 제품 차이는
`Paragraph` 삭제 전에 원래 중복 글자 모양의 유효 ID를 보존하는 17줄이며 Flutter/Native ABI에는
차이가 없다. 병합 시뮬레이션은 exit 0이다. 이미 실행한 소비 source·Native 입력을 새 Rust source에서
빌드했다고 바꾸지 않았다. 당시 진행 중인 WASM 최적화를 마친 뒤 이 devel을 통합하고, 새 삭제
계약과 해당 입력·출력을 보존하는 최종 검증을 수행했다. 본인 계정의 열린 PR은 조회 결과 0건이다.

### 최종 로컬 완료

최신 devel 통합 뒤 Native·CLI를 실제 `release-test` profile로 재빌드했고, 새로운 Native로
다중 페이지·CPU 대조·명시적 복구·실제 창 resize·두 타일·이동 bundle 및 최소 글꼴의 연동
6개 검사를 완료했다. 새 base의 #8863 정식 집중 회귀 8개와 실제 WASM 모델·저장 API의
잔여 10,456문자 굵기·색상 보존도 통과했다. 최적화 fresh WASM source는 동일한 `857594293f`다.

독립 한컴 Print와 Native/fresh WASM 각각 7쪽의 전쪽 raw 최저는 100.00%다. 같은 배율의
6쪽 gradient와 1쪽 bitmap·회색 계조를 직접 판독했고, 수정 전후 실제 화면과 기본 생산자 입력의
동일성도 확인했다. 전체 TSV 원본·최종 대표 PNG·명령·해시·수치·플랫폼 한계는
[최종 검증 원장](../assets/seo-rii-20261110/README.md)에 보존했다.

새 diff는 Flutter 소비자이며 Rust·조판·WASM 생산자 수정은 없다. 이번에 실행하지 않은 Rust
전체 회귀·광범위 lint를 완료로 보고하거나 원 PR의 CI를 새 제출 head CI로 바꾸지 않는다.
원격 처리 범위는 [구체적인 실행안](../assets/seo-rii-20261110/remote-action-plan.md)의 별도 승인 뒤다.

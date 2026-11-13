# PR #8952 검토 — macOS Metal 두 타일

## 최종 판정

**승인 — 사용자가 결정한 macOS 실기 검증 예외를 적용한다.** Portable·공통 계약과 코드 검토를 완료했다. Apple SDK·Metal/AppKit·ScreenCaptureKit의 실제 실행은 미실행이며 기여자 설명 수용과 구분하여 기록한다.

GitHub 승인 리뷰·댓글·source push·merge는 수행하지 않았다. 원격 처리는 이번 로컬 검토와 별도이며 최종 실행 전 exact head CI와 base를 다시 확인한다.

## 접수와 고정 기준

| 항목 | 값 |
| --- | --- |
| 작성자·base·상태 | seo-rii / devel / Ready |
| 원 head | `3077c620f3ec7c09cd93c2b79e7ea0825be12488` |
| 원 fork·branch | `seo-rii/rhwp` / `task/8701-p117-macos-tile-groups` |
| 검토한 제품 head | `4ab994ee67ff66e35aa9168ac6fae506b6b26122` |
| 기준 devel | `41c889cfd8a869ad23199d6018f8b569879a575c` |
| reviewer | edwardkim 요청·API 재확인 |
| 관련 이슈 | #8701·#536 참조, 전체 로드맵 OPEN 유지 |
| CI | [run 43053929805](https://github.com/edwardkim/rhwp/actions/runs/43053929805), 동일 원 head attempt 2 success |

2026-11-13 조회에서 원 head는 유지됐고 check 33개 success·6개 정상 skip이다. GitHub mergeable은 UNKNOWN으로 재계산 중이며 이전 MERGEABLE 값을 현재 값으로 대신하지 않는다. 최신 devel과 실제 누적 병합·최종 merge-tree는 exit 0이다. [최신 원 CI 원본](assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)을 보존했다.

## 변경과 실제 검토

`MetalTileGroup.hpp`의 실제 공통 admission은 두 owner의 gate·lease·receipt를 준비한 뒤 잠금 안에서 함께 이전한다. `MetalTileGroupMethods.inc`의 create/submit/cover/close/retire는 동일 그룹·generation·full stamp와 두 멤버의 완료 상태를 확인한다. 최대 두 슬롯·한 batch의 고유 snapshot·bounded weak map·shared lease의 단일 release, 부분 생성 실패와 retirement ack를 portable 생산 코드의 1,669개 검사로 확인했다. 정상 실행 및 ASan/UBSan 모두 통과했고 Mac qualification 관련 Python 67개도 통과했다.

이 검사는 실제 CAMetalLayer drawable·consumer queue·ScreenCaptureKit을 실행한 결과가 아니다. 사용자의 이번 명시적 수용 결정을 적용하며 Mac을 제공하라는 질문이나 같은 예외 승인을 반복하지 않는다. 원 기여의 실제 Mac 검증 명령과 제한은 `bindings/Flutter/macos/TILE_GROUPS.md`에 남아 있다.

## 공통 검증과 조판 적용 여부

[검증 원장](assets/seo-rii-20261112/README.md)과 [수행 계획](seo_rii_20261112_review_impl.md)을 따른다. 공통 19개 명령, Python 186개·Metal policy 1,669개(일반/ASan/UBSan), strict analyze·Dart format, 별도 runtime FFI 대조군 3개와 실제 Native Flutter 280개가 통과했다. 숫자들은 서로 다른 범위를 나타내며 합성 검사·실제 SDK·실제 화면을 구분한다.

제품 변경은 Flutter 소비자·도구·검사·해당 가이드이며 Rust parser/IR/조판·Native/WASM 생산자·Cargo·vendor·assets가 기준과 같다. 기존 줄 소속·좌표/높이·pagination/continuation·저장 LineSeg 수용 규칙과 회귀 기대값을 바꾸지 않았다. 이 범위에 조판 재설계·편람 gate·정본 보호 TSV 전체 재산출을 기계적으로 확대하지 않았다. 실제 변경 플랫폼의 검증은 위 성공/실패/미실행 구분을 적용한다.

최신 devel의 동일 공통 생산자로 Native/fresh WASM 각각 8쪽을 독립 Desktop Print와 비교했다. 원 범위 7쪽은 양쪽 최저 100.00%, 새 devel의 zero-size 표 대조군은 Native 99.75%·WASM 99.86%다. 전쪽 TSV와 직접 확대 판독을 [최종 원장](assets/seo-rii-20261112/windows-correction-20261113/README.md)에 보존했다. 새 devel #9025·#9033 기능도 실제 출력·CLI로 확인했다. base를 별도로 빌드했다거나 Mac 실기를 검증했다고 표현하지 않는다.

## 원격 처리 조건

- macOS는 이미 확정한 [사용자 결정](assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)을 적용한다. 실제 Apple SDK 실행은 미실행이다.
- Windows 보정 커밋 `60f5659d5b`·`7e576a18ca`·`4ab994ee67`과 최종 실제 검증은 [진단 기록](seo_rii_20261112_windows_build_finding.md)에 연결했다. 최초 C1083·metadata 실패·종료 fault를 해소했다.
- Draft 6개·부모 #536·#8701은 이번 완료 처리에서 제외하며 원 기여 SHA와 fork branch를 보존한다.
- fork의 push 권한이 없어 upstream 수용 PR 경로를 준비한다. 원격 push·새 PR·승인 리뷰·merge·댓글은 별도 승인과 새 head CI 확인 뒤에 수행한다.

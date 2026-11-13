# PR #8954 검토 — Windows 두 타일·복구 qualification

## 최종 판정

**메인터너 보정 후 수용 가능.** 승인된 Windows CMake 보정과 실제 실행에서 확인한 `.exe` metadata·messenger 종료 경계를 보정했다. 최신 소스의 실제 Windows 빌드·타일 최초/편집/resize/복구·이동 번들·정상 종료 검증을 모두 완료했다. 새 수용 PR head의 CI·최종 self-review는 원격 승인 뒤 진행한다.

GitHub 승인 리뷰·댓글·source push·merge는 수행하지 않았다. 원격 처리는 이번 로컬 검토와 별도이며 최종 실행 전 exact head CI와 base를 다시 확인한다.

## 접수와 고정 기준

| 항목 | 값 |
| --- | --- |
| 작성자·base·상태 | seo-rii / devel / Ready |
| 원 head | `1e14e4b8466b7ce46d9016faf1824bc05c9789de` |
| 원 fork·branch | `seo-rii/rhwp` / `task/8701-p119-windows-tile-consumer` |
| 검토한 제품 head | `4ab994ee67ff66e35aa9168ac6fae506b6b26122` |
| 기준 devel | `41c889cfd8a869ad23199d6018f8b569879a575c` |
| reviewer | edwardkim 요청·API 재확인 |
| 관련 이슈 | #8701·#536 참조, 전체 로드맵 OPEN 유지 |
| CI | [run 43054349242](https://github.com/edwardkim/rhwp/actions/runs/43054349242), 동일 원 head attempt 2 success |

2026-11-13 조회에서 원 head는 유지됐고 check 33개 success·6개 정상 skip이다. GitHub mergeable은 UNKNOWN으로 재계산 중이며 이전 MERGEABLE 값을 현재 값으로 대신하지 않는다. 최신 devel과 실제 누적 병합·최종 merge-tree는 exit 0이다. [최신 원 CI 원본](assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)을 보존했다.

## 변경과 실제 검토

새 collector의 build/collector helper hash·PID/nonce·전체 stamp·공유 snapshot·두 타일 경계·실제 창 크기와 `FlutterView`/Win32 DPR 대조, 편집 revision·resize·명시적 recovery·retirement·Native 9개 자원 계수 검사 경로를 검토했다. Portable Windows tile 21개, continuous 27개, desktop 46개, 공통 tile 4개 검사와 실제 Native를 연결한 Flutter 전체 280개가 통과했다.

실제 Windows Native D3D DLL·MSVC SDK·최종 Flutter runner가 모두 통과했다. 최종 소스에서 두 실제 타일의 최초 표시·편집 revision·창 resize·명시적 recovery와 자원 종료를 실제 CPU oracle과 대조했다. collector는 `windows-two-tile-consumer-passed`, exit 0이며 Native teardown 9개 계수는 모두 0이다. Win32/FlutterView의 실제 DPR 1을 확인했고 두 OS DPR 간 비교는 미실행이다. `actualDprComparison`·`consumerGpuCompletionVerified`·하드웨어 가속·target reuse·driver 메모리 완료를 주장하지 않는다. [최종 실행 증거](assets/seo-rii-20261112/windows-correction-20261113/README.md)를 따른다.

## 공통 검증과 조판 적용 여부

[검증 원장](assets/seo-rii-20261112/README.md)과 [수행 계획](seo_rii_20261112_review_impl.md)을 따른다. 공통 19개 명령, Python 186개·Metal policy 1,669개(일반/ASan/UBSan), strict analyze·Dart format, 별도 runtime FFI 대조군 3개와 실제 Native Flutter 280개가 통과했다. 숫자들은 서로 다른 범위를 나타내며 합성 검사·실제 SDK·실제 화면을 구분한다.

제품 변경은 Flutter 소비자·도구·검사·해당 가이드이며 Rust parser/IR/조판·Native/WASM 생산자·Cargo·vendor·assets가 기준과 같다. 기존 줄 소속·좌표/높이·pagination/continuation·저장 LineSeg 수용 규칙과 회귀 기대값을 바꾸지 않았다. 이 범위에 조판 재설계·편람 gate·정본 보호 TSV 전체 재산출을 기계적으로 확대하지 않았다. 실제 변경 플랫폼의 검증은 위 성공/실패/미실행 구분을 적용한다.

최신 devel의 동일 공통 생산자로 Native/fresh WASM 각각 8쪽을 독립 Desktop Print와 비교했다. 원 범위 7쪽은 양쪽 최저 100.00%, 새 devel의 zero-size 표 대조군은 Native 99.75%·WASM 99.86%다. 전쪽 TSV와 직접 확대 판독을 [최종 원장](assets/seo-rii-20261112/windows-correction-20261113/README.md)에 보존했다. 새 devel #9025·#9033 기능도 실제 출력·CLI로 확인했다. base를 별도로 빌드했다거나 Mac 실기를 검증했다고 표현하지 않는다.

## 원격 처리 조건

- macOS는 이미 확정한 [사용자 결정](assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)을 적용한다. 실제 Apple SDK 실행은 미실행이다.
- Windows 보정 커밋 `60f5659d5b`·`7e576a18ca`·`4ab994ee67`과 최종 실제 검증은 [진단 기록](seo_rii_20261112_windows_build_finding.md)에 연결했다. 최초 C1083·metadata 실패·종료 fault를 해소했다.
- Draft 6개·부모 #536·#8701은 이번 완료 처리에서 제외하며 원 기여 SHA와 fork branch를 보존한다.
- fork의 push 권한이 없어 upstream 수용 PR 경로를 준비한다. 원격 push·새 PR·승인 리뷰·merge·댓글은 별도 승인과 새 head CI 확인 뒤에 수행한다.

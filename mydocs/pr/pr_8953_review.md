# PR #8953 검토 — 이동 가능한 참조앱 번들

## 최종 판정

**메인터너 보정 후 수용 가능.** 승인된 Windows CMake 보정과 실제 실행에서 확인한 `.exe` metadata·messenger 종료 경계를 보정했다. 최신 소스의 실제 Windows 빌드·타일 최초/편집/resize/복구·이동 번들·정상 종료 검증을 모두 완료했다. 새 수용 PR head의 CI·최종 self-review는 원격 승인 뒤 진행한다.

GitHub 승인 리뷰·댓글·source push·merge는 수행하지 않았다. 원격 처리는 이번 로컬 검토와 별도이며 최종 실행 전 exact head CI와 base를 다시 확인한다.

## 접수와 고정 기준

| 항목 | 값 |
| --- | --- |
| 작성자·base·상태 | seo-rii / devel / Ready |
| 원 head | `9cfe2754d92a720135445aed285ff91c0c75dc7e` |
| 원 fork·branch | `seo-rii/rhwp` / `task/8701-p118-reference-app-bundle` |
| 검토한 제품 head | `4ab994ee67ff66e35aa9168ac6fae506b6b26122` |
| 기준 devel | `41c889cfd8a869ad23199d6018f8b569879a575c` |
| reviewer | edwardkim 요청·API 재확인 |
| 관련 이슈 | #8701·#536 참조, 전체 로드맵 OPEN 유지 |
| CI | [run 43053940397](https://github.com/edwardkim/rhwp/actions/runs/43053940397), 동일 원 head attempt 2 success |

2026-11-13 조회에서 원 head는 유지됐고 check 33개 success·6개 정상 skip이다. GitHub mergeable은 UNKNOWN으로 재계산 중이며 이전 MERGEABLE 값을 현재 값으로 대신하지 않는다. 최신 devel과 실제 누적 병합·최종 merge-tree는 exit 0이다. [최신 원 CI 원본](assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)을 보존했다.

## 변경과 실제 검토

빌드 입력의 source/hash 고정, 실행 파일/Native/owner ABI·OS·architecture, 한도·상대 경로·링크/재파싱 거부, `.app` 실행 파일 metadata와 실행 파일에서의 bundle 자동 발견을 검토했다. 런처는 명시적으로 검증한 코드를 실행하며 hash를 서명·sandbox로 주장하지 않는다. Native 빌드는 외부 공급이고 helper 자체가 Native를 다시 빌드했다고 주장하지 않는 계약을 유지한다.

실제 Linux release 참조앱의 bundle을 만든 후 생성 source·runner 사본을 제거했다. 공백이 있는 경로로 옮긴 bundle을 cwd `/tmp`, RHWP 환경 변수 없이 직접 실행했다. 실제 커널 mapping 5개가 이동한 bundle의 파일과 일치했다. 1쪽 및 6쪽 문서 열기·1→6쪽 이동·문서 닫기·정상 exit 0을 확인했다.

Windows는 CMake·metadata·plugin 종료를 보정한 최종 release 번들을 실제로 빌드해 이동 실행했다. 생성 source·runner를 제거하고 별도 cwd, RHWP 환경 변수 없이 1쪽·6쪽 열기·1→6쪽 이동·문서 닫기·WM_CLOSE exit 0을 확인했다. 실제 5개 module은 모두 이동 번들에 있다. [최종 실행 증거](assets/seo-rii-20261112/windows-correction-20261113/README.md)를 따른다. macOS는 사용자의 기여자 설명 수용 결정을 적용한다.

## 공통 검증과 조판 적용 여부

[검증 원장](assets/seo-rii-20261112/README.md)과 [수행 계획](seo_rii_20261112_review_impl.md)을 따른다. 공통 19개 명령, Python 186개·Metal policy 1,669개(일반/ASan/UBSan), strict analyze·Dart format, 별도 runtime FFI 대조군 3개와 실제 Native Flutter 280개가 통과했다. 숫자들은 서로 다른 범위를 나타내며 합성 검사·실제 SDK·실제 화면을 구분한다.

제품 변경은 Flutter 소비자·도구·검사·해당 가이드이며 Rust parser/IR/조판·Native/WASM 생산자·Cargo·vendor·assets가 기준과 같다. 기존 줄 소속·좌표/높이·pagination/continuation·저장 LineSeg 수용 규칙과 회귀 기대값을 바꾸지 않았다. 이 범위에 조판 재설계·편람 gate·정본 보호 TSV 전체 재산출을 기계적으로 확대하지 않았다. 실제 변경 플랫폼의 검증은 위 성공/실패/미실행 구분을 적용한다.

최신 devel의 동일 공통 생산자로 Native/fresh WASM 각각 8쪽을 독립 Desktop Print와 비교했다. 원 범위 7쪽은 양쪽 최저 100.00%, 새 devel의 zero-size 표 대조군은 Native 99.75%·WASM 99.86%다. 전쪽 TSV와 직접 확대 판독을 [최종 원장](assets/seo-rii-20261112/windows-correction-20261113/README.md)에 보존했다. 새 devel #9025·#9033 기능도 실제 출력·CLI로 확인했다. base를 별도로 빌드했다거나 Mac 실기를 검증했다고 표현하지 않는다.

## 원격 처리 조건

- macOS는 이미 확정한 [사용자 결정](assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)을 적용한다. 실제 Apple SDK 실행은 미실행이다.
- Windows 보정 커밋 `60f5659d5b`·`7e576a18ca`·`4ab994ee67`과 최종 실제 검증은 [진단 기록](seo_rii_20261112_windows_build_finding.md)에 연결했다. 최초 C1083·metadata 실패·종료 fault를 해소했다.
- Draft 6개·부모 #536·#8701은 이번 완료 처리에서 제외하며 원 기여 SHA와 fork branch를 보존한다.
- fork의 push 권한이 없어 upstream 수용 PR 경로를 준비한다. 원격 push·새 PR·승인 리뷰·merge·댓글은 별도 승인과 새 head CI 확인 뒤에 수행한다.

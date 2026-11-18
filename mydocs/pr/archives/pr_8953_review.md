# PR #8953 검토 — 이동 가능한 참조앱 번들

## 최종 판정

**메인터너 보정 후 수용 완료.** 승인된 Windows CMake 보정과 실제 실행에서 확인한 `.exe` metadata·messenger 종료 경계를 보정했다. 최신 소스의 실제 Windows 빌드·타일 최초/편집/resize/복구·이동 번들·정상 종료 검증을 모두 완료했다. 수용 PR head의 CI·최종 self-review·머지를 완료했다.

GitHub 수용 PR의 최신 head CI·self-review·merge를 완료했다. 원 PR도 간접 MERGED 상태를 확인했다. 감사 댓글은 이 운영 기록을 devel에 반영한 뒤 아래 계획에 따라 게시한다.

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

제출 전 2026-11-13 조회에서 원 head는 유지됐고 check 33개 success·6개 정상 skip이다. GitHub mergeable은 UNKNOWN으로 재계산 중이며 이전 MERGEABLE 값을 현재 값으로 대신하지 않는다. 최신 devel과 실제 누적 병합·최종 merge-tree는 exit 0이다. [최신 원 CI 원본](../assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)을 보존했다.

## 변경과 실제 검토

빌드 입력의 source/hash 고정, 실행 파일/Native/owner ABI·OS·architecture, 한도·상대 경로·링크/재파싱 거부, `.app` 실행 파일 metadata와 실행 파일에서의 bundle 자동 발견을 검토했다. 런처는 명시적으로 검증한 코드를 실행하며 hash를 서명·sandbox로 주장하지 않는다. Native 빌드는 외부 공급이고 helper 자체가 Native를 다시 빌드했다고 주장하지 않는 계약을 유지한다.

실제 Linux release 참조앱의 bundle을 만든 후 생성 source·runner 사본을 제거했다. 공백이 있는 경로로 옮긴 bundle을 cwd `/tmp`, RHWP 환경 변수 없이 직접 실행했다. 실제 커널 mapping 5개가 이동한 bundle의 파일과 일치했다. 1쪽 및 6쪽 문서 열기·1→6쪽 이동·문서 닫기·정상 exit 0을 확인했다.

Windows는 CMake·metadata·plugin 종료를 보정한 최종 release 번들을 실제로 빌드해 이동 실행했다. 생성 source·runner를 제거하고 별도 cwd, RHWP 환경 변수 없이 1쪽·6쪽 열기·1→6쪽 이동·문서 닫기·WM_CLOSE exit 0을 확인했다. 실제 5개 module은 모두 이동 번들에 있다. [최종 실행 증거](../assets/seo-rii-20261112/windows-correction-20261113/README.md)를 따른다. macOS는 사용자의 기여자 설명 수용 결정을 적용한다.

## 공통 검증과 조판 적용 여부

[검증 원장](../assets/seo-rii-20261112/README.md)과 [수행 계획](seo_rii_20261112_review_impl.md)을 따른다. 공통 19개 명령, Python 186개·Metal policy 1,669개(일반/ASan/UBSan), strict analyze·Dart format, 별도 runtime FFI 대조군 3개와 실제 Native Flutter 280개가 통과했다. 숫자들은 서로 다른 범위를 나타내며 합성 검사·실제 SDK·실제 화면을 구분한다.

제품 변경은 Flutter 소비자·도구·검사·해당 가이드이며 Rust parser/IR/조판·Native/WASM 생산자·Cargo·vendor·assets가 기준과 같다. 기존 줄 소속·좌표/높이·pagination/continuation·저장 LineSeg 수용 규칙과 회귀 기대값을 바꾸지 않았다. 이 범위에 조판 재설계·편람 gate·정본 보호 TSV 전체 재산출을 기계적으로 확대하지 않았다. 실제 변경 플랫폼의 검증은 위 성공/실패/미실행 구분을 적용한다.

최신 devel의 동일 공통 생산자로 Native/fresh WASM 각각 8쪽을 독립 Desktop Print와 비교했다. 원 범위 7쪽은 양쪽 최저 100.00%, 새 devel의 zero-size 표 대조군은 Native 99.75%·WASM 99.86%다. 전쪽 TSV와 직접 확대 판독을 [최종 원장](../assets/seo-rii-20261112/windows-correction-20261113/README.md)에 보존했다. 새 devel #9025·#9033 기능도 실제 출력·CLI로 확인했다. base를 별도로 빌드했다거나 Mac 실기를 검증했다고 표현하지 않는다.

## 원격 처리 조건

- macOS는 이미 확정한 [사용자 결정](../assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)을 적용한다. 실제 Apple SDK 실행은 미실행이다.
- Windows 보정 커밋 `60f5659d5b`·`7e576a18ca`·`4ab994ee67`과 최종 실제 검증은 [진단 기록](seo_rii_20261112_windows_build_finding.md)에 연결했다. 최초 C1083·metadata 실패·종료 fault를 해소했다.
- Draft 6개·부모 #536·#8701은 이번 완료 처리에서 제외하며 원 기여 SHA와 fork branch를 보존한다.
- fork의 push 권한이 없어 upstream 수용 PR 경로를 준비한다. 작업지시자가 원격 처리를 승인했고 수용 PR의 CI·self-review·merge를 완료했다. 댓글과 종료 정리는 정본의 후속 순서에 따라 수행한다.

## 병합과 CI 확정 — 2026-11-13

[수용 PR #9037](https://github.com/edwardkim/rhwp/pull/9037)의 head `728fc70dbd4de0d1fc26a6aaec4a507eb0ddfe5f`를 검증하고 [self-review](https://github.com/edwardkim/rhwp/pull/9037#issuecomment-6840051408)를 게시한 뒤 merge commit `2c7644c53171e21c27b8a532ce4c53f01c40ddff`로 통합했다. 실제 merge tree는 사전 시뮬레이션과 같은 `188efd7a0414605f12ba4bb55e677500f73dd6c9`다. 작업지시자가 CI 성공·self-review 뒤 머지와 후속 처리를 승인했다.

CI 32개 성공·정상 skip 5개, 최종 CI Impact Policy success와 MERGEABLE/CLEAN을 확인했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/43333224593)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43333224573)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43333224506)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43333224421)는 모두 성공했다. 실제 회귀 A–E 로그는 13,236 PASS·플랫폼/feature 정상 skip 45개이며 새 실패는 0개다. CI Frontend의 Studio runner는 2,666 PASS·0 FAIL을 기록했다. 로컬 Native 연결 Flutter 280개와 별개의 실행이다. CI 재실행·workflow·보호 설정 변경은 하지 않았다.

원 PR 3건은 GitHub API에서 모두 `merged=true`를 확인했다. 원 head·fork branch를 보존하고 부모 [#536](https://github.com/edwardkim/rhwp/issues/536)·[#8701](https://github.com/edwardkim/rhwp/issues/8701)은 OPEN을 유지했다. 병합 후 [duration refresh](https://github.com/edwardkim/rhwp/actions/runs/43337460272)는 success이며 검증 CI를 다시 실행하지 않았다. 후속 기록은 소스·test·workflow·golden을 제외한 maintainer 직접 운영 기록(옵션 M)으로 반영한다.

## Merge 후 contributor PR comment 계획

원 기여에 먼저 감사하고 수용 PR·merge SHA·최신 CI·실제 로컬 검증·메인터너 보정 이유를 한국어 존댓말로 게시한다. 원 PR의 해당 범위를 아래 최종 이미지와 연결한다.

- 실제 Windows 이동 bundle 1쪽 입력·6쪽 이동: `https://raw.githubusercontent.com/edwardkim/rhwp/2c7644c53171e21c27b8a532ce4c53f01c40ddff/mydocs/pr/assets/seo-rii-20261112/windows-correction-20261113/windows-relocated-documents.png`.
- 실제 Linux 이동 bundle 1쪽 입력·6쪽 이동: `https://raw.githubusercontent.com/edwardkim/rhwp/2c7644c53171e21c27b8a532ce4c53f01c40ddff/mydocs/pr/assets/seo-rii-20261112/windows-correction-20261113/linux-relocated-documents.png`.

원 범위 7쪽의 Native/fresh WASM 최저 100.00%와 최신 base 대조군 포함 99.75%·99.86%를 구분한다. asset은 실제 merge SHA에서 HTTP 200·PNG signature·SHA-256 일치를 확인했다. 운영 기록 devel 반영 뒤 API 본문·줄바꿈·실제 이미지 로딩을 확인하고, 이번 전용 branch/worktree 및 종료된 임시 산출물 정리를 완료한다.

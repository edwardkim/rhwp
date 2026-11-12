# PR #8953 검토 — 이동 가능한 참조앱 번들

## 최종 판정

**머지 보류 — 실제 Windows runner 빌드의 공통 헤더 경로 실패.** Native DLL·SDK 단위 검사는 통과했지만 실제 Flutter 앱의 C1083을 재현했다. 보정 담당 범위 승인과 실제 Windows 재검증 전에는 수용 완료로 판정하지 않는다.

GitHub 승인 리뷰·댓글·source push·merge는 수행하지 않았다. 원격 처리는 이번 로컬 검토와 별도이며 최종 실행 전 exact head CI와 base를 다시 확인한다.

## 접수와 고정 기준

| 항목 | 값 |
| --- | --- |
| 작성자·base·상태 | seo-rii / devel / Ready |
| 원 head | `9cfe2754d92a720135445aed285ff91c0c75dc7e` |
| 원 fork·branch | `seo-rii/rhwp` / `task/8701-p118-reference-app-bundle` |
| 검토한 제품 head | `1e819721fe789b6b3b5c6db600635245039c9f98` |
| 기준 devel | `a85d3265dd10b316d1a0ae029ca3f67245b38b77` |
| reviewer | edwardkim 요청·API 재확인 |
| 관련 이슈 | #8701·#536 참조, 전체 로드맵 OPEN 유지 |
| CI | [run 43053940397](https://github.com/edwardkim/rhwp/actions/runs/43053940397), 동일 원 head attempt 2 success |

최종 조회에서 원 head는 바뀌지 않았고 MERGEABLE, check 33개 success·6개 정상 skip이다. 본문의 attempt 1 Ubuntu DNS 실패는 현재 실패가 아니다. 최신 devel fetch에서도 위 base를 재확인했고 실제 누적 병합·최종 merge-tree가 모두 exit 0이다. [CI 원본](assets/seo-rii-20261112/final-rollup-8953.json)을 보존했다.

## 변경과 실제 검토

빌드 입력의 source/hash 고정, 실행 파일/Native/owner ABI·OS·architecture, 한도·상대 경로·링크/재파싱 거부, `.app` 실행 파일 metadata와 실행 파일에서의 bundle 자동 발견을 검토했다. 런처는 명시적으로 검증한 코드를 실행하며 hash를 서명·sandbox로 주장하지 않는다. Native 빌드는 외부 공급이고 helper 자체가 Native를 다시 빌드했다고 주장하지 않는 계약을 유지한다.

실제 Linux release 참조앱의 bundle을 만든 후 생성 source·runner 사본을 제거했다. 공백이 있는 경로로 옮긴 bundle을 cwd `/tmp`, RHWP 환경 변수 없이 직접 실행했다. 실제 커널 mapping 5개가 이동한 bundle의 파일과 일치했다. 1쪽 및 6쪽 문서 열기·1→6쪽 이동·문서 닫기·정상 exit 0을 확인했다.

Windows는 같은 builder/공통 plugin CMake 경로가 실제 runner에서 실패한다. 성공한 Linux의 결과로 Windows 성공을 대신하지 않는다. macOS에만 사용자의 기여자 설명 수용 결정을 적용한다.

## 공통 검증과 조판 적용 여부

[검증 원장](assets/seo-rii-20261112/README.md)과 [수행 계획](seo_rii_20261112_review_impl.md)을 따른다. 공통 19개 명령, Python 186개·Metal policy 1,669개(일반/ASan/UBSan), strict analyze·Dart format, 별도 runtime FFI 대조군 3개와 실제 Native Flutter 280개가 통과했다. 숫자들은 서로 다른 범위를 나타내며 합성 검사·실제 SDK·실제 화면을 구분한다.

제품 변경은 Flutter 소비자·도구·검사·해당 가이드이며 Rust parser/IR/조판·Native/WASM 생산자·Cargo·vendor·assets가 기준과 같다. 기존 줄 소속·좌표/높이·pagination/continuation·저장 LineSeg 수용 규칙과 회귀 기대값을 바꾸지 않았다. 이 범위에 조판 재설계·편람 gate·정본 보호 TSV 전체 재산출을 기계적으로 확대하지 않았다. 실제 변경 플랫폼의 검증은 위 성공/실패/미실행 구분을 적용한다.

동일한 공통 생산자의 실제 Native/fresh WASM 전 7쪽은 독립 Print와 쪽수가 같고 각 경로의 전쪽 raw 최저가 100.00%다. 모든 내용 영역을 같은 확대율로 직접 판독했다. base·후보가 같은 생산 입력이므로 기본 출력의 페이지별 하락은 0.00%p이며 base를 별도 빌드했다는 주장은 하지 않는다. macOS 실제 GPU 성공이나 Windows 실패 해소를 이 값으로 대신하지 않는다.

## 남은 조건

- macOS 실제 실행은 [사용자 결정](assets/seo-rii-20261112/macos-maintainer-decision.json)으로 수용하며 실행 성공을 만들지 않는다.
- #8953·#8954의 Windows 실패는 [원인·보정 계획](seo_rii_20261112_windows_build_finding.md)과 제안 patch로 검토 가능하게 준비했다. 아직 제품 소스에는 적용하지 않았다.
- Draft 6개는 이번 수용 대상에서 제외했고 tip ancestry에도 포함하지 않았다.
- 원 기여 SHA·fork branch는 보존한다. 전체 3건을 검증 완료한 수용 PR로 제출한 상태가 아니다.

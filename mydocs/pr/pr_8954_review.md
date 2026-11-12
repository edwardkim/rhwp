# PR #8954 검토 — Windows 두 타일·복구 qualification

## 최종 판정

**머지 보류 — 실제 Windows runner 빌드의 공통 헤더 경로 실패.** Native DLL·SDK 단위 검사는 통과했지만 실제 Flutter 앱의 C1083을 재현했다. 보정 담당 범위 승인과 실제 Windows 재검증 전에는 수용 완료로 판정하지 않는다.

GitHub 승인 리뷰·댓글·source push·merge는 수행하지 않았다. 원격 처리는 이번 로컬 검토와 별도이며 최종 실행 전 exact head CI와 base를 다시 확인한다.

## 접수와 고정 기준

| 항목 | 값 |
| --- | --- |
| 작성자·base·상태 | seo-rii / devel / Ready |
| 원 head | `1e14e4b8466b7ce46d9016faf1824bc05c9789de` |
| 원 fork·branch | `seo-rii/rhwp` / `task/8701-p119-windows-tile-consumer` |
| 검토한 제품 head | `1e819721fe789b6b3b5c6db600635245039c9f98` |
| 기준 devel | `a85d3265dd10b316d1a0ae029ca3f67245b38b77` |
| reviewer | edwardkim 요청·API 재확인 |
| 관련 이슈 | #8701·#536 참조, 전체 로드맵 OPEN 유지 |
| CI | [run 43054349242](https://github.com/edwardkim/rhwp/actions/runs/43054349242), 동일 원 head attempt 2 success |

최종 조회에서 원 head는 바뀌지 않았고 MERGEABLE, check 33개 success·6개 정상 skip이다. 본문의 attempt 1 Ubuntu DNS 실패는 현재 실패가 아니다. 최신 devel fetch에서도 위 base를 재확인했고 실제 누적 병합·최종 merge-tree가 모두 exit 0이다. [CI 원본](assets/seo-rii-20261112/final-rollup-8954.json)을 보존했다.

## 변경과 실제 검토

새 collector의 build/collector helper hash·PID/nonce·전체 stamp·공유 snapshot·두 타일 경계·실제 창 크기와 `FlutterView`/Win32 DPR 대조, 편집 revision·resize·명시적 recovery·retirement·Native 9개 자원 계수 검사 경로를 검토했다. Portable Windows tile 21개, continuous 27개, desktop 46개, 공통 tile 4개 검사와 실제 Native를 연결한 Flutter 전체 280개가 통과했다.

실제 Windows Native D3D DLL과 MSVC plugin 단위 컴파일은 성공했지만, 실제 Flutter runner는 `NativeApi.h:3`의 공통 헤더를 찾지 못해 생성되지 않았다. 실제 GPU sampling·편집·resize·복구·자원 종료와 두 실제 DPR의 비교는 미실행이다. 현재 모니터의 실제 DPI 96(DPR 1) 관측을 runner의 DPR 검증 성공으로 바꾸어 쓰지 않는다. `consumerGpuCompletionVerified`·하드웨어 가속·target reuse·driver 메모리까지 완료됐다는 주장도 하지 않는다.

## 공통 검증과 조판 적용 여부

[검증 원장](assets/seo-rii-20261112/README.md)과 [수행 계획](seo_rii_20261112_review_impl.md)을 따른다. 공통 19개 명령, Python 186개·Metal policy 1,669개(일반/ASan/UBSan), strict analyze·Dart format, 별도 runtime FFI 대조군 3개와 실제 Native Flutter 280개가 통과했다. 숫자들은 서로 다른 범위를 나타내며 합성 검사·실제 SDK·실제 화면을 구분한다.

제품 변경은 Flutter 소비자·도구·검사·해당 가이드이며 Rust parser/IR/조판·Native/WASM 생산자·Cargo·vendor·assets가 기준과 같다. 기존 줄 소속·좌표/높이·pagination/continuation·저장 LineSeg 수용 규칙과 회귀 기대값을 바꾸지 않았다. 이 범위에 조판 재설계·편람 gate·정본 보호 TSV 전체 재산출을 기계적으로 확대하지 않았다. 실제 변경 플랫폼의 검증은 위 성공/실패/미실행 구분을 적용한다.

동일한 공통 생산자의 실제 Native/fresh WASM 전 7쪽은 독립 Print와 쪽수가 같고 각 경로의 전쪽 raw 최저가 100.00%다. 모든 내용 영역을 같은 확대율로 직접 판독했다. base·후보가 같은 생산 입력이므로 기본 출력의 페이지별 하락은 0.00%p이며 base를 별도 빌드했다는 주장은 하지 않는다. macOS 실제 GPU 성공이나 Windows 실패 해소를 이 값으로 대신하지 않는다.

## 남은 조건

- macOS 실제 실행은 [사용자 결정](assets/seo-rii-20261112/macos-maintainer-decision.json)으로 수용하며 실행 성공을 만들지 않는다.
- #8953·#8954의 Windows 실패는 [원인·보정 계획](seo_rii_20261112_windows_build_finding.md)과 제안 patch로 검토 가능하게 준비했다. 아직 제품 소스에는 적용하지 않았다.
- Draft 6개는 이번 수용 대상에서 제외했고 tip ancestry에도 포함하지 않았다.
- 원 기여 SHA·fork branch는 보존한다. 전체 3건을 검증 완료한 수용 PR로 제출한 상태가 아니다.

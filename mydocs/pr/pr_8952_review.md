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
| 검토한 제품 head | `1e819721fe789b6b3b5c6db600635245039c9f98` |
| 기준 devel | `a85d3265dd10b316d1a0ae029ca3f67245b38b77` |
| reviewer | edwardkim 요청·API 재확인 |
| 관련 이슈 | #8701·#536 참조, 전체 로드맵 OPEN 유지 |
| CI | [run 43053929805](https://github.com/edwardkim/rhwp/actions/runs/43053929805), 동일 원 head attempt 2 success |

최종 조회에서 원 head는 바뀌지 않았고 MERGEABLE, check 33개 success·6개 정상 skip이다. 본문의 attempt 1 Ubuntu DNS 실패는 현재 실패가 아니다. 최신 devel fetch에서도 위 base를 재확인했고 실제 누적 병합·최종 merge-tree가 모두 exit 0이다. [CI 원본](assets/seo-rii-20261112/final-rollup-8952.json)을 보존했다.

## 변경과 실제 검토

`MetalTileGroup.hpp`의 실제 공통 admission은 두 owner의 gate·lease·receipt를 준비한 뒤 잠금 안에서 함께 이전한다. `MetalTileGroupMethods.inc`의 create/submit/cover/close/retire는 동일 그룹·generation·full stamp와 두 멤버의 완료 상태를 확인한다. 최대 두 슬롯·한 batch의 고유 snapshot·bounded weak map·shared lease의 단일 release, 부분 생성 실패와 retirement ack를 portable 생산 코드의 1,669개 검사로 확인했다. 정상 실행 및 ASan/UBSan 모두 통과했고 Mac qualification 관련 Python 67개도 통과했다.

이 검사는 실제 CAMetalLayer drawable·consumer queue·ScreenCaptureKit을 실행한 결과가 아니다. 사용자의 이번 명시적 수용 결정을 적용하며 Mac을 제공하라는 질문이나 같은 예외 승인을 반복하지 않는다. 원 기여의 실제 Mac 검증 명령과 제한은 `bindings/Flutter/macos/TILE_GROUPS.md`에 남아 있다.

## 공통 검증과 조판 적용 여부

[검증 원장](assets/seo-rii-20261112/README.md)과 [수행 계획](seo_rii_20261112_review_impl.md)을 따른다. 공통 19개 명령, Python 186개·Metal policy 1,669개(일반/ASan/UBSan), strict analyze·Dart format, 별도 runtime FFI 대조군 3개와 실제 Native Flutter 280개가 통과했다. 숫자들은 서로 다른 범위를 나타내며 합성 검사·실제 SDK·실제 화면을 구분한다.

제품 변경은 Flutter 소비자·도구·검사·해당 가이드이며 Rust parser/IR/조판·Native/WASM 생산자·Cargo·vendor·assets가 기준과 같다. 기존 줄 소속·좌표/높이·pagination/continuation·저장 LineSeg 수용 규칙과 회귀 기대값을 바꾸지 않았다. 이 범위에 조판 재설계·편람 gate·정본 보호 TSV 전체 재산출을 기계적으로 확대하지 않았다. 실제 변경 플랫폼의 검증은 위 성공/실패/미실행 구분을 적용한다.

동일한 공통 생산자의 실제 Native/fresh WASM 전 7쪽은 독립 Print와 쪽수가 같고 각 경로의 전쪽 raw 최저가 100.00%다. 모든 내용 영역을 같은 확대율로 직접 판독했다. base·후보가 같은 생산 입력이므로 기본 출력의 페이지별 하락은 0.00%p이며 base를 별도 빌드했다는 주장은 하지 않는다. macOS 실제 GPU 성공이나 Windows 실패 해소를 이 값으로 대신하지 않는다.

## 남은 조건

- macOS 실제 실행은 [사용자 결정](assets/seo-rii-20261112/macos-maintainer-decision.json)으로 수용하며 실행 성공을 만들지 않는다.
- #8953·#8954의 Windows 실패는 [원인·보정 계획](seo_rii_20261112_windows_build_finding.md)과 제안 patch로 검토 가능하게 준비했다. 아직 제품 소스에는 적용하지 않았다.
- Draft 6개는 이번 수용 대상에서 제외했고 tip ancestry에도 포함하지 않았다.
- 원 기여 SHA·fork branch는 보존한다. 전체 3건을 검증 완료한 수용 PR로 제출한 상태가 아니다.

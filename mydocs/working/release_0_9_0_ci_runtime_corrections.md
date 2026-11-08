# v0.9.0 CI 실행 보정

## 승인과 범위

2026-11-08 작업지시자는 반복 CLI 재링크 제거와 Linux x86_64 runner 고정을 함께 보정하고,
로컬 검증 → PR → 해당 head CI·self-review → 병합 → 승격·배포를 계속하도록 승인했다.
제품 기준은 `1b78dc23fd8add8d2bc61958be5533cbeafdbd20`로 유지한다. 다른 PR·제품 기능은
추가하지 않는다. 현재 변경은 CI 실행 도구·runner와 계약 검사·안내에 한정된다.

기본 경로는 collaborator self-merge, 보조 경로는 intake_and_review, local_validation,
post_merge다. CI 실행 변경의 운영 등급은 O3이며 후속 review·오늘할일은 옵션 2로 기록한다.
현재 문서는 구현·검증 기록이며 아직 생성되지 않은 PR의 self-review가 아니다.

## 원인과 변경

- [Full Renderer Sweep](https://github.com/edwardkim/rhwp/actions/runs/42446197902/job/128625533281)은
  60분 상한으로 취소됐다. Native Skia·WASM·Studio 빌드와 11개 browser render diff는 성공했지만,
  다음 baseline 수집에서 default/native-skia feature가 같은 CLI를 교체하며 19번 장시간
  재링크했다. multi-profile 단계는 35분 23초를 사용했고 후속 surface 검사는 실행되지 않았다.
- `renderer_baseline.py`는 각 feature 구성의 실제 Cargo executable을 한 번 빌드하고 독립 경로에
  보관한다. `dev` profile·feature·원본·쪽·export 인자와 판정 기준은 유지한다. Cargo JSON의
  executable을 확인하며 빌드 실패·출력 누락·실제 CLI 실패는 그대로 실패 처리한다.
- [Release Binary](https://github.com/edwardkim/rhwp/actions/runs/42446213092)는 성공했으나,
  Linux x86_64 archive의 `atan2f`·`acosf`·`remainder`는 GLIBC_2.43을 요구했다. Ubuntu 24.04의
  glibc 2.39에서는 `--version`부터 실패했다. x86_64 runner를 `ubuntu-24.04`로 고정한다.
  ARM은 기존 `ubuntu-24.04-arm`을 유지한다. runner별 cache key로 이전 x86_64 cache와 분리된다.
  다섯 target·Native Skia·실제 archive PNG 검사·게시 권한은 유지한다.

선행 계약은 #612의 CLI 배포, #7818의 Native Skia 포함·archive PNG 검사, #7556의 runner별
SDK cache 분리와 기존 renderer baseline profile·artifact identity를 대조했다. 검사 삭제나
예외·fallback 성공 위장은 하지 않는다. 제품 Rust·Studio 렌더러·입력 manifest는 바꾸지 않았다.

## 로컬 검증

2026-11-08 KST, Ubuntu 24.04 / Rust 1.93.1:

- 기존 `test_canvaskit_parity_batches` 호출에 CLI 재사용 계약 5개를 포함했다. 별도 CI 호출을
  추가하지 않았고 실제 수집·실행을 확인했다.
- batch/CLI·release channel·publish orchestration·workflow wiring 계약 46개 통과.
- 실제 Cargo `dev` default/native-skia 빌드 두 번으로 `lseg-01-basic.hwp`와 `lseg-05-tab.hwp`의
  legacy SVG·PDF·네 profile Layer SVG·Native PNG 출력 20개 성공.
- 기존 source의 release-test 출력과 비교 가능한 SVG·PNG 18개가 모두 동일하다.
  PDF 생성은 성공했지만 생성 메타데이터를 포함한 PDF 바이트 동일성으로 판정하지 않았다.
- 실제 Native CLI `rhwp v0.9.0` 실행 성공, ELF의 최대 GLIBC 요구 버전 2.39.
- [실제 출력 증거](../tech/investigations/release-0.9.0-ci-runtime/cli-reuse-proof.json).

원격 보정 뒤에는 새 Linux archive를 내려받아 Ubuntu 24.04에서 version·PNG 출력을 확인해야
한다. 현재 로컬 성공을 새 GitHub runner의 실행 성공으로 표시하지 않는다.

## 추가 승인된 하네스 계약

같은 source의 전체 baseline은 Native 210개 출력 후 browser 136개에서 실패했다.
`table-in-textbox`·`image-start`는 CanvasKit에 필요한 한컴 글꼴 byte가 없어 제품 계약대로
Canvas2D로 전환됐다. 비교 가능한 폴백 10개는 명시적 Canvas2D와 PNG가 동일하지만,
CanvasKit parity 통과는 아니다. 하네스는 이 전환을 실패로 기록하고, 이후 print용 LayerTree
진단에서 portable metrics 계약을 따르지 않아 중단됐다.

작업지시자는 같은 PR에 제한된 폴백 검사·진단 메트릭 보정과 60→90분 실행 상한을
추가하도록 승인했다. 필수 readiness 8건은 실제 CanvasKit 성공을 그대로 요구한다.
그 밖의 사례는 실제 자원 준비 실패와 독립 font plan이 모두 증명된 경우에만 별도
`fontFallbackGate`에서 명시적 Canvas2D와 동일 픽셀·비어 있지 않은 내용을 요구한다.
알 수 없는 오류·GPU 초기화 실패는 계속 실패하며 CanvasKit 성공 건수로 세지 않는다.
실제 브라우저 검증은 진행 중이며, Full Renderer 또는 릴리즈 승격 완료를 아직 선언하지 않는다. 기존 13개 workflow 중 12개 성공·1개 시간 초과이며,
main PR·태그·실제 게시를 아직 수행하지 않았다.

## 운영 영향과 복구

실행 event·권한·required check·artifact 이름·retention·publish 조건은 유지한다.
CLI 임시 파일은 Git에서 제외된 output에 생성하며 프로세스 종료 시 제거한다.
문제가 있으면 이 보정 commit을 별도 PR에서 revert한다. 실제 적용 이후 같은 head의
CI 결과와 Linux runtime을 확인하며 원격 실패 재실행은 별도 승인 범위를 따른다.

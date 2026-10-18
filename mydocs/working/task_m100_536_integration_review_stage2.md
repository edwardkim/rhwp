# seo-rii 통합 검토 2단계 — 검증 구조 준비

## 메인터너 보정

- 신규 runtime 파일(1,306줄)을 public transport/queue 관리 785줄과 worker 문서 실행 528줄로 분리했다.
  명령 FIFO·취소·보유 바이트·종료 계약과 원 body를 유지한다. worker child의 타입 경로와 진입 visibility만 조정했다.
- 신규 portable geometry 검사(1,251줄)를 원 case 969줄과 공유 fixture 286줄로 나눴다.
  기존 모든 test 함수 이름의 전후 집합 동일성을 확인했다. 실제 실행은 다음 후보 검증에서 확인한다.
- path module 분리의 최초 준비에서는 49/48 target 예산 초과가 드러났다. 기존 support helper와 같은
  `include!` 구조로 같은 case 이름 공간에 포함하여 28 suites + 20 exceptions = 48/48을 유지했다.
  정책 상한·최소 검사 수·예외 목록을 완화하지 않았다.
- 신규 Native host 예제 README를 한국어로 정리하고 공용 target, 실제 라이브러리 경로, 사용 예제,
  입력 보존·버퍼 해제·저장 acknowledge·좌표계·mock/실제 실행 및 미지원 OS 경계를 설명했다.

## 실제 선행 검사

원본 소스만으로 suite를 준비한 뒤 `cargo fmt --all`, format check, 고정 devel 기준 suite 정책 및
unit-tier 정책을 실행해 모두 exit 0을 확인했다. 1,527 sources / 6,744 static test attrs / 48 targets,
source-side 4,103 tests / 296 modules를 확인했다. generated suite·manifest는 commit하지 않는다.
이는 Cargo Clippy·집중/전체 회귀·Native FFI·fresh WASM·시각 검증의 완료를 뜻하지 않는다.

## 시각 기준 조사

같은 원본의 Native/fresh WASM lseg 2문서는 기준 수치를 확보했다. 추가 form/equation 기준의 미달은
[1단계](task_m100_536_integration_review_stage1.md)에 구분했다.

`form-01.hwp`의 5개 폼 모두 원본 `Printable=1`을 조회했으므로 빈 기존 PDF에 맞추려고 출력을 숨기지 않는다.
MCP engine 2020은 원본 OnDocument_Open handler 때문에 무인 변환을 거절했다(job
`790abb53-a486-4a3d-b7da-52b68987c9a5`). 원본 수동 Print PDF를 사용자에게 요청했으며,
기존에 사용자 Print가 있는 HWPX 폼 대조와 다른 검증은 계속한다. 원본에서 script를 제거하거나 다른 입력의
PDF를 이 원본의 증거로 바꾸지 않는다.

`equation-lim.hwp`는 원본 변경 없이 engine 2020 Print job
`85a0902f-4516-48fe-bcdd-fd2b6d66d81e`이 성공했다. 실제 제품 11.0.0.9136,
PrintMethod=0 / hancom2020_pdf_driver_one_up, 전처리 없음, 1쪽, 10,578bytes,
PDF SHA-256 `62b6b47da9c9dde5fb44ed4e025bbb871a048f01e9c7bf51f5e93be4a1a51aae`다.
기존 PDF와 새로운 Print의 차이를 관측하고 최종 비교 기준·변경 사유를 명시한다.

## 다음 단계

이 코드 후보에서 작은 Native/브라우저 출력과 관련 계약을 먼저 확인하고, 필요한 보정을 모은 뒤
필수 lint·workspace·전체 nextest·Native Skia·FFI·frontend와 최종 fresh WASM을 순차 검증한다.
필수 결과가 미완료인 현재 원 PR별 판정은 머지 보류다. PR·승인·통합 완료를 단정하지 않는다.

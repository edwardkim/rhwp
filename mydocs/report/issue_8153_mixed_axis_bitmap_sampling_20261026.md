# #8153: 영역 맞춤 그림의 공통 Print 표본 소비

[계획](../plans/issue_8153_mixed_axis_bitmap_sampling_20261026.md)의 독립 한컴 Print와
공개 표본 계산을 사용한다. 원본 두 문서와 기준 PDF, 임계값 232·이웃 반경 2는 유지한다.

## 실제 수정 전 실패

고정 `89b929b8c0162b5267b8410d0b89d05d0ca3ae40` 라이브러리에 정식 테스트 원문을
연결했다. 빌드 종료 코드 0, 실행 종료 코드 101이며 두 원본 모두 그림 틀 여덟 조각의
소유·누락·중복과 가로·세로 띠 관계를 통과한 뒤, 독립 Print의 모서리 (626,623)
±2px 안의 잉크 누락으로 FAIL했다. 해당 테스트 원문의 SHA-256은
`ee8b0d63b527d25792c4c8e9e28b11ee1d838023fd96eb0f90aef56286ac6cba`다.
호스트의 독점 글꼴 경로를 고정하거나 폰트 파일을 복사하지 않았다.

초안의 소유 검사는 HWP `FitToSize`와 HWPX `Total`을 같은 영역 맞춤으로 취급하지
못해 HWP의 조각 수를 0으로 셌다. 이 실패는 결함 재현으로 세지 않는다. 초안과 잘못된
proof는 ignored `initial-total-only-draft/`에 보존했고, 두 형식의 원래 동일 계약을
반영한 최종 원문으로 두 건의 모서리 FAIL을 다시 확인했다.

## 공통 보정과 보호 범위

`print_area_sampling`이 기존 `device_image::eligible`의 영역 맞춤 전체 원본 계약을
소비하게 한다. 동일한 효과·변환·crop·inset·형식 조건을 따로 구현하지 않는다.
해당 PNG/JPEG 채움은 이미 존재하는 공통 Print 표본 계산으로 전달하며, 일반 JPEG
Picture의 기존 경로도 유지한다. PNG Picture의 팔레트별 재구성, 반복·자연 크기·
Zoom·부분 source 창·회전·효과·명시적 interpolation은 넓히지 않는다.
실제 alpha가 있는 원본은 공통 샘플러의 불투명성 검사에서 거절돼 기존 경로로 돌아간다.
조판 좌표·그림 소속·원본 바이트·표본 수학이나 backend별 수치는 바꾸지 않는다.

기존 Native CI에 연결하려던 새 소스 `cfg(test)` 모듈은 unit-tier 정책에 걸려 제거했다.
소스 테스트 증가 없이 정책을 재실행해 4,100 tests / 296 modules 통과를 확인했다.
새 사례는 정식 `tests/cases/`에만 둔다. Native feature 전용 통합 검사의 실제 CI 수집
누락은 최종 제출 전에 5.3.1 기준에 따라 필요한 최소 호출로 해결하고 실제 실행을 확인한다.

## 검증 상태

공개 표본·단위·크기·alpha 거절의 기존 정상 대조 9건은 수정 전 producer에서 통과했다.
함께 실행한 `verified_print_bitmap_visible_palette.rs`의 두 검사는 같은 수정 전
producer에서도 실패했다. 실제 관측 `imageReconstruction`이 과거 `catmullRom` 표식과
다르다는 실패이며, 이미 앞선 압축 단계에서 관측한 같은 실패다. 이 두 건을 새 보정의
회귀나 정상 통과로 표시하지 않고 수정 후에도 실제 차이를 대조한다. 독립 PDF 검증 없이
기대 표식을 바꾸거나 실패 검사를 제외해 전체 통과로 보고하지 않는다.

새 후보 Native 빌드·두 실제 픽셀 검사·기존 84건과 표본 대조 9건, Native/fresh WASM
시각 비교와 전쪽 검증은 이어서 실행한다. 이 기록 시점에는 새 보정의 PASS·239쪽 해결·
전쪽 100%·PR 준비 완료를 판정하지 않는다.

로그·실행 명령·고정 producer·초안·proof는 ignored 작업 루트
`output/pr-review/issue8153-handbook-raw100-20261022/handbook-mixed-axis-image-candidate/`에 있다.
사용자 지시대로 upstream/devel 동기화와 원격 게시·push·PR 생성은 하지 않는다.

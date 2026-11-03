# PR #8548 CodeQL 후속 보정

## 원인과 기존 해결 이력

제출 head `e68f0e1236`의 Rust 분석 `2087289300`(tested merge `152a6ea4d44b1ef14f4348b361ad19742224f3a3`)은 132개 결과 중 새 경고 세 개를 보고했다.
비밀번호 인자 자체가 로그에 도달한 것은 아니며, `NativeSession::open_bytes → DocumentCore::from_bytes_with_password`에서 열린 문서의 좌표·줄 배치값이 진단 로그로 흐른다.

| alert | 당시 sink | 진단 채널 |
| --- | --- | --- |
| #495 | paragraph_layout.rs:9447 | RHWP_DEBUG_PARA_TAC / TAC_LINE |
| #464 | typeset/section/vpos.rs:308 | RHWP_DIAG_SNAPALL / DIAG_SNAPALL |
| #465 | typeset/section/vpos.rs:323 | RHWP_DIAG_TAC / DIAG_SNAP |

세 sink의 code flow는 각각 네 개다. 모두 반환된 DocumentCore에서 수치 메트릭을 거치는 경로다.
세 로그는 고정 base에도 있었으며 이번 통합으로 과거 보정을 재도입한 것은 아니다.
그러나 기존 #8517의 원 보정 `57ce496ef7370c72ee5c95fc56d5914d61c88c1d`는 환경변수로 켜는 암호 문서의 수치 진단 네 곳도 제거했다.
그 실제 patch와 통합 `c97709b597`을 대조해 같은 정책으로 세 출력을 제거한다. 보안 query·language·path·alert 분류를 변경하지 않는다.

## 보정과 실행 증거

두 source 파일에서 환경변수 진단 블록 세 개, 45행을 제거했다. y·원점·줄·표·모델·저장·paint 계산은 수정하지 않았다.
따라서 앞 단계의 Native/fresh WASM 31입력·71쪽 시각 측정은 같은 조판 계산의 증거로 재사용하고, 해당 수치를 새 security head에서 다시 측정한 것처럼 표시하지 않는다.
이 추가 변경은 진단 출력 보안에 해당하며 새 배치·golden·픽셀 기대값을 추가하지 않는다.

실제 기존 암호 fixture `samples/HWP5-password-123456.hwpx`를 password-stdin으로 열고 세 진단 환경변수를 모두 켜 SVG를 내보냈다.
이전 binary는 DIAG_SNAPALL 362회·DIAG_SNAP 74회·TAC_LINE 110회를 stderr에 썼다. 실제 요청한 SVG 산출물은 정상 생성됐다.
같은 fixture를 사용하는 공개 CLI 관계 회귀를 기존 `tests/hwpx_password_fixture.rs`에 추가했다.
최소 글꼴 격리에서 보존한 이전 binary로 그 새 검사를 실행해 `DIAG_SNAPALL` 채널 때문에 FAIL(exit 101)하는 것을 확인했다.
정상 SVG 내보내기는 유지하면서 해당 세 진단 채널이 stderr에 나타나지 않아야 한다.

보정 후 Rust lint 묶음·workspace build·전체 stock nextest와 새 CLI 회귀를 실행한 뒤 새 head를 제출한다. 이 문서 시점에는 보정 후 검증이 진행 중이다.

## CI 상태의 구분

이전 head의 CodeQL 분석 workflow 자체는 성공했지만 새 alert 세 개 때문에 CodeQL check가 실패했다.
이전 lint는 checkout에 13분 18초를 쓴 뒤 기존 20분 상한으로 WASM32 Clippy가 취소됐다. Native Clippy는 성공했다.
사용자가 승인한 이전 head의 lint 1회 재실행은 아직 호출하지 않았으며, 이 보안 보정으로 후보가 바뀌면 사용하지 않는다.
코드·workflow·검증 기준을 바꾸어 이전 실패를 성공으로 가장하지 않고, 같은 PR의 새 head에서 GitHub CI를 확인한다.

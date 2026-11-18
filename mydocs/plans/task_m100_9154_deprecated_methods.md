# #9154 후속 — deprecated method 호출 정리

## 승인과 기준

2026-11-18 작업지시자의 “deprecated method 들을 모두 수정해야 합니다” 지시에 따라
로컬 구현과 검증을 진행한다. 기존 #9154의 Rust 환경 호환성 조사에 연결하는 별도 후속
브랜치 `task/9154-deprecated-atomic-apis`이며, 기준 devel은
`fbf8081db57c0c6abc9bad418290c5018193e24f`다. 이미 완료한 #9155·#9160을 수정하지 않는다.
원격 push·새 PR·main 반영·게시와 MSRV 상향은 이 로컬 지시에서 확대하지 않는다.

## 확인한 원인과 범위

- Rust 1.99 이상은 `fetch_update`를 deprecated로 표시한다. 대체 이름 `try_update`는
  프로젝트 MSRV 1.93.1에서 `E0658 atomic_try_update`이므로 단순 치환할 수 없다.
- 실제 1.93.1·1.99·1.102 nightly에서 두 API의 컴파일 결과를 확인했다.
- 저장소 Rust source의 `fetch_update` 호출은 13곳이다. PDF 예산 2곳, core Native
  예산·ID·GPU 5곳, Native FFI의 세 GPU 경로·세션·Metal admission 5곳, codec 자원 테스트 1곳이다.
- 이전 Fuzz 실패의 Atomic 경고는 종료 신호의 원인이 아니다. 이번 작업은 지원 환경에서
  경고 없이 같은 동작을 유지하는 후속 수정이며 원격 OOM을 확정하는 작업이 아니다.
- 기존 `#[deprecated]` 공개 호환 API 선언과 그 호환성 시험은 제거 대상과 구분한다.
  실제 compiler 진단과 허용된 내부 호출을 조사하고 다른 활성 deprecated method도 함께 정리한다.

## 구현

1. 기존 최하위 자원 crate `rhwp-codec-resource`에 `core::sync::atomic` 기반 공통 checked
   update 구현을 둔다. `load`와 `compare_exchange_weak` 반복으로 현재 관측값을 갱신하며
   성공·실패 ordering, callback 재평가와 `Ok/Err(previous)` 계약을 보존한다.
2. usize/u64 진입점은 한 macro 본문에서 생성한다. no_std와 해당 target의 atomic 지원
   조건을 보존한다. 새 외부 의존성·Rust 버전 분기·경고 억제를 추가하지 않는다.
3. core와 Native FFI는 이미 사용 중인 이 경량 crate를 직접 연결해 공통 구현을 소비한다.
   PDF backend와 codec 시험도 같은 구현을 소비한다. schema 전용 `rhwp-contracts`의 닫힌
   경계를 범용 utility로 확장하지 않는다. Cargo.lock 변경은 이 직접 의존성 연결만 허용한다.
4. 모든 13개 호출에서 기존 checked_add/sub·상한·ID 고갈·sticky failure와 메모리 순서를
   보존한다. core·FFI의 기존 공개 API·문서 조판·저장 형식·암호화 표현은 변경하지 않는다.
5. 전체 빌드 진단에서 추가 deprecated method가 확인되면 원 API 계약과 기존 해결 이력을
   확인하고 동일 동작의 현행 호출로 수정한다. 의도된 외부 호환 API를 삭제해 경고를 없애지 않는다.

## 검증

- 수정 전 최신 Rust workspace/all-target 진단 목록과 static 호출 목록을 보존한다.
- 공통 구현의 정상 갱신·거절·정수 경계·CAS 실패 후 재평가·경합 시 중복 없는 ID와 예산 상한을
  검사한다. 기존 `tests/cases/` 자동 수집 경로에 계약 시험을 포함하며 CI 개별 호출을 추가하지 않는다.
- MSRV 1.93.1과 최신 Rust에서 core·PDF·Native FFI와 codec 자원 시험을 실행한다.
- Metal admission의 SDK 독립 Rust 모듈 시험과 기존 Native 요청·저장·presentation·자원 한도
  집중 회귀를 실행한다. GPU 장치 구동 성공으로 확대하지 않는다.
- 최종 source에서 필수 Rust lint 묶음·release-test 전체 회귀·WASM32 check를 실행한다.
  회귀는 최소 글꼴 환경을 사용한다. 조판 규칙과 renderer 출력을 바꾸지 않는 동작 보존 수정이므로
  새 시각 baseline이나 fixture를 등록하지 않는다.
- 최신 Rust의 deprecated method 진단과 source의 obsolete 호출이 남지 않는지 확인한다.
  실제 실행한 범위·예외적인 공개 호환 계약·미실행 대상은 결과 보고에 구분한다.

## 완료 기록

구현 code head `efd401f2e665b12666017a885d32019b5ce5f17a`에서 전체 회귀 13,723개와
필수 Rust lint·두 Rust 버전의 계약 시험을 통과했다. 기존 Atomic 호출은 0곳이며 최신
nightly의 deprecated 진단도 0건이다. 실제 명령과 호환 범위는
[결과 보고서](../report/task_m100_9154_deprecated_methods_report.md)에 연결한다.

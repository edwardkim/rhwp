# #9154 후속 — deprecated Atomic method 정리 결과

## 결과

검증 code head `efd401f2e665b12666017a885d32019b5ce5f17a`에서 기존 `fetch_update` 호출 13곳을 모두 교체했다.
최신 nightly의 workspace 전체 target 및 Native Skia 활성화 검사에서 deprecated 진단은 **0건**이다.
수정 전 동일 workspace 검사는 중복 target 진단을 포함해 16건이었다.
MSRV는 **Rust 1.93.1**을 유지했고 새 경고 억제·외부 패키지·CI 정의 변경은 추가하지 않았다.

## 구현과 계약

기존 최하위 `rhwp-codec-resource`의 공통 CAS 구현을 core·PDF backend·Native FFI가 함께 쓴다.
usize/u64 진입점은 같은 macro 본문에서 생성하며 load/실패와 성공의 메모리 순서를 그대로 전달한다.
경합·spurious CAS 실패 시 최신 관측값으로 callback을 다시 평가하고, 거절 시 값을 쓰지 않으며,
성공·거절의 반환값은 이전 값이다. 원래 checked_add/sub·상한·ID 고갈·sticky failure 처리를 보존했다.

Rust 1.93.1의 `try_update`는 불안정 API이므로 이름만 바꾸지 않았다.
[Rust 공식 Atomic 문서](https://doc.rust-lang.org/std/sync/atomic/struct.Atomic.html)의 갱신 계약과
현재 source를 대조했다. [호출 13곳의 도입 commit 연결](../working/assets/issue9154-deprecated-methods/atomic-call-origins.json)을 보존했다.
core와 FFI의 직접 의존성 연결만 Cargo.lock에 추가했으며 registry 패키지 버전·MSRV를 변경하지 않았다.

## 실제 검증

| 검사 | 결과 |
| --- | --- |
| 신규 Atomic 계약 | 실제 source 6개, Rust 1.93.1·nightly 각각 PASS |
| 기존 Metal admission | 실제 source 8개, 두 Rust 각각 PASS; Apple SDK·GPU 구동은 비해당 |
| codec scoped 자원 검사 | 두 Rust 각각 4 PASS, no_std check PASS |
| 최신 nightly | workspace all-targets·Native Skia 모두 PASS, deprecated 진단 0건 |
| MSRV lint | Native·WASM32·workspace all-target Clippy와 workspace build PASS |
| Native FFI | 실제 `host_panic` C ABI 복구 시험 PASS |
| 전체 Rust 회귀 | `Summary [ 533.995s] 13723 tests run: 13723 passed (8 slow), 45 skipped` |
| 시험 수집·tier | 기존 source-side 4,094개 유지, 신규 6개는 기존 `tests/cases/` suite로 자동 수집 |

신규 Atomic 계약 6개가 전체 회귀에서도 실제 실행된 PASS 행을 검증 원장에 보존했다.
전체 회귀와 Native FFI 시험은 bwrap으로 개인 글꼴·Windows 폰트 경로를 차단한 환경에서 실행했다.
노출한 27개 파일은 OS DejaVu와 저장소 공개 글꼴이며 [목록·차단 확인](../working/assets/issue9154-deprecated-methods/minimal-font-manifest.json)에 기록했다.
비공개 문서·글꼴을 추가하지 않았고 renderer·문서 배치·저장 형식을 바꾸지 않아 시각 baseline·fixture를 갱신하지 않았다.

[검증 명령·실행 source·결과](../working/assets/issue9154-deprecated-methods/validation.json),
[실제 계약 시험 출력](../working/assets/issue9154-deprecated-methods/isolated-contracts-final.json),
[수행 계획](../plans/task_m100_9154_deprecated_methods.md)을 연결한다.

## 범위와 남은 단계

외부 호환을 위해 유지하는 공개 `#[deprecated]` API 선언과 그 호환성 시험은 삭제하지 않았다.
기존 2,400바이트 ML-KEM 개인키의 의도된 호환 표현도 변경하지 않았다. 이번 로그의 obsolete Atomic 호출을
경고 억제로 숨기지 않고 실제 대체했으며, 새 source의 compiler deprecated 진단 0건을 확인한 범위다.

이번 사용자 지시는 로컬 수정·검증 범위다. 원격 push·PR·merge·main 적용과 패키지 재게시를 수행하지 않았다.
#9154의 main 환경 반영은 이전 작업과 같이 별도 단계이며 이 로컬 수정으로 issue close를 진행하지 않는다.

최신 원격 devel 재조회와 후보의 실제 병합 시뮬레이션은 충돌 없이 통과했다.
조회 시점 본인 계정의 열린 PR은 Draft를 포함해 0건이었다. 조회 시각·정확한 SHA·tree는
[제출 전 점검](../working/assets/issue9154-deprecated-methods/preflight.json)에 보존했다.

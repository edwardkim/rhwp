---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 58 — 최신 devel 재리베이스 및 PR 게이트

## 목적

Stage 57의 PR 후보를 최신 `upstream/devel` 위에 다시 올리고, 충돌 구간의 최신 renderer
계약을 보존한 정확한 HEAD에서 전체 회귀 게이트를 순차 실행한다. 원격 push와 PR 생성은
사용자 승인 뒤에 수행한다.

## 리베이스 기준

- 이전 작업 HEAD: `f656fc5f4eac95e3d18ddcea39ff548338bba5ee`
- 최신 upstream 기준: `22d06f1ad5ad862024c980cdef3b964a1d150592`
- 리베이스 직후 HEAD: `a65f78fdc06a02cf6984c36d0f653487ba1f5013`
- 결과: ahead 79 / behind 0 / clean

## 충돌 판정

### boxed PUA

`src/renderer/composer.rs`의 일반 U+F02B1–U+F02C4 승격은 제외했다. 최신 devel의
#4139 계약은 issue2007 p2의 일반 boxed PUA를 `TextRun`으로 유지하고, #4158 계약은 실제
`CharOverlap`만 별도 합성한다. 두 경로를 합치면 폭 측정과 페이지네이션이 달라지므로 최신
분리를 보존했다. 이 브랜치가 추가한 U+F02FB 삼각 글머리표 매핑과 fidelity 진단만 유지했다.

### overflow-cell baseline

최신 devel에서 낮아지거나 제거된 baseline 항목은 그대로 보존하고,
`issue3637/regulatory_impact_nested_table_escape.hwpx`만 이 브랜치의 실측값 `19`를 적용했다.
baseline은 신규·증가를 막는 상한 래칫이므로 과거의 더 큰 값으로 되돌리지 않는다.

## 순차 검증 계획

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`과 전용
`CARGO_TARGET_DIR=target/task-3820-stage58-rebased-pr-gate`를 사용한다.

1. release-test 빌드와 충돌 관련 focused 회귀
2. issue2007 p11-p13 PDF 직접 대조
3. release 빌드와 lib 회귀
4. `cargo test --profile release-test --tests` 전체 integration
5. Native Skia 공식 회귀 3종
6. fmt, diff-check, Clippy, rustdoc
7. TypeScript, Studio test, WASM build
8. #4139/#4158 브라우저 E2E와 Markdown 링크 검사

전체 integration은 장시간 걸리는 것이 정상이며 최종 exit code와 test summary까지 대기한다.
다른 작업의 target은 건드리지 않는다.

## 완료 조건

- focused 및 전체 Rust 회귀 실패 0
- overflow-cell baseline 증가 0
- issue2007 17쪽 및 p11-p13 제목 소유권 유지
- Native Skia, fmt, diff-check, Clippy, rustdoc 통과
- TypeScript, Studio, WASM, boxed-PUA 브라우저 회귀 통과
- 최종 코드 SHA와 명령별 결과를 이 문서에 기록

## 단계 전환

위 게이트는 당시 정확한 HEAD
`d5fd69d72c621df291d0da22dca48b75e82a59fd`에서 Rust 전체 integration, Native Skia,
Clippy, Studio, WASM과 브라우저 E2E까지 실패 0으로 완료했다. 그러나 최종
증적을 작성하기 직전 `upstream/devel`이 9커밋 전진했으므로, 이 결과를 새 PR
후보 HEAD의 통과로 간주하지 않는다.

- 새 upstream 기준: `e64c853124a74109e44d5e42499a2825d05c85a2`
- 재리베이스 직후 HEAD: `2558786d746cdc115f6c8a9817949b327e37bce2`
- 결과: 충돌 없음, ahead 80 / behind 0 / clean
- 후속: Stage 59에서 모든 PR 게이트를 새 HEAD 기준으로 처음부터 순차 재실행

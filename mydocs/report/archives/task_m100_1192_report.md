# 최종 결과 보고서 — Task #1192: CI 시간 단축

- **이슈**: #1192 → CLOSED
- **브랜치**: `local/task1192` → devel 머지 (`fe5d306c`)
- **작성일**: 2026-05-31 (수치 정정: 2026-06-01)
- **성격**: CI 인프라 개선 (소스 무변경, `.github/workflows/` 만)

## ⚠️ 수치 정정 안내

초판 보고서는 CI **진행 중**(가장 무거운 `Run tests` 실행 전) 부분 데이터로 "Build & Test
712s→212s(−70%)" 라 적었으나 **오류**였다. CI 완료 후 정확히 재측정한 값으로 아래를 확정한다.
실제 단축은 **712s → 647s (약 −9%)** 이며, B(CodeQL rust 캐시)는 다음 PR 부터 효과가 발생한다.

## 적용 항목

| 항목 | 내용 | 파일 |
|------|------|------|
| A | `Canvas layer parity tests` step 제거 (cargo test 중복). native-skia step 은 유지 | ci.yml |
| C | `Free disk space` 축소 — android/dotnet 만 제거, 느린 정리(apt/docker prune/ghc/CodeQL) 삭제 | ci.yml ×2 job |
| B | CodeQL rust matrix 에 cargo 캐시 추가 (`codeql-rust` key) | codeql.yml |
| F | concurrency 취소 그룹 (cancel-in-progress 는 PR 이벤트 한정) | ci/codeql/render-diff |

## 런타임 검증 결과 (완료된 CI run 비교)

### Build & Test (이전 `6ca2be7f` 26714279649 → 이후 `fe5d306c` 26715868949)

| step | 이전 | 이후 | 비고 |
|------|------|------|------|
| Run tests | 371s | 356s | 본질적, 거의 동일 |
| Free disk space | 76s | **46s** | C: −30s (android/dotnet 삭제 자체가 ~46s) |
| Build | 57s | 59s | — |
| Native Skia tests | 48s | 56s | 유지 (A 대상 아님) |
| Clippy | 41s | 42s | — |
| Check WASM target | 37s | 37s | — |
| Canvas layer parity | 36s | **0s** | A: −36s (제거) |
| Install native Skia pkgs | 11s | 14s | — |
| cache restore | 5s | 8s | — |
| checkout / setup / post | 17s | 25s | — |
| **총계** | **712s (11.9분)** | **647s (10.8분)** | **−65s (−9%)** |

- **A 검증**: `canvas_layer_tree_matches_legacy` 로그 출현(3회) → 별도 step 없이 `Run tests`
  (cargo test)에 포함되어 실행됨. native-skia tests step 도 유지·실행(56s). 테스트 커버리지 손실 0.
- **C 검증**: `df -h` = `/dev/root 72G, 25G Avail, 66% 사용` → 빌드 디스크 여유 충분. 롤백 불필요.
  절감은 −30s 로 당초 기대(−40~60s)보다 작음 — android/dotnet rm 자체가 46s 소요.
- 전체 결과: Run tests(356s)가 여전히 절반 이상으로 지배적. A/C 는 고정비 일부만 제거.

### CodeQL Analyze(rust) (이후 run)

| step | 시간 |
|------|------|
| Perform CodeQL Analysis | 149s (GitHub 분석, 통제 밖) |
| Build Rust (for CodeQL) | 70s |
| Initialize CodeQL | 14s |
| Cache cargo registry & build (rust) | 1s |
| 총계 | 257s (4.3분) |

- **B 검증**: 이번 run 은 첫 실행이라 **cold**(Build Rust 70s). run 종료 후 저장소에
  `Linux-codeql-rust-...` 캐시(**311MB**) 생성 확인 → **다음 PR 부터 warm build**
  (의존성 재컴파일 생략으로 Build Rust 단축 예상). 즉 B 의 시간 효과는 차기 PR 부터 발생.

### F 검증

- ci/codeql/render-diff 3개 워크플로우에 concurrency 적용, `cancel-in-progress` PR 이벤트 한정.
- 본 devel push run 은 단독이라 취소 미발생(정상 — push run 은 보존 대상).
- 연속 push 취소 효과는 후속 PR 의 중복 push 에서 관찰 예정.

## 전체 CI 결과

- **CI run 26715868949: success** / **CodeQL: success** (전부 green).

## 안전성 / 비범위 확인

- 테스트 커버리지 손실 0 (A 는 중복 제거만, native-skia 유지) — feedback_push_full_test_required 준수.
- 디스크 초과 재발 없음 (25G 여유) — C 롤백 불필요.
- PR #1170(진행 중)과 파일 충돌 없음. CodeQL python 매트릭스 제거는 범위 외(유지).

## 평가 및 후속 제언

- 이번 PR 의 직접 효과는 **Build & Test −9% (−65s)** + **CodeQL rust 차기 PR warm 화**.
  당초 기대(−2~3분)보다 작은데, 병목의 본질이 `cargo test`(356s) 자체이기 때문.
- F(concurrency)는 시간 절감보다 **연속 push 시 중복 run 제거**가 본질 — 활발한 컨트리뷰션
  사이클에서 누적 러너 분 절약 효과가 큼.
- **추가 단축 후보(범위 외, 후속 판단)**:
  - `cargo test` 를 `cargo nextest` 로 교체(병렬 실행, 통상 30~50% 단축).
  - `Free disk space` 자체를 제거 가능한지 재검토(현재 25G 여유 → 정리 없이도 빌드 가능 여부 실측).
  - CodeQL python 매트릭스 제거(대상 코드 없음).

## 결론

A/C/B/F 4개 항목 모두 적용·검증 완료, devel CI 전체 green, 안전장치 내 동작.
즉시 효과는 −9% 로 제한적이나 B 는 차기 PR 부터, F 는 중복 제거로 지속 효과. **Task #1192 완료.**
정확한 추가 단축은 후속 제언(nextest 등)으로 별도 판단 권고.

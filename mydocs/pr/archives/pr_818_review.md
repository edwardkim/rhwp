---
PR: #818
제목: perf — release 빌드에 LTO + codegen-units=1 + strip 활성화 (closes #790)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 15번째 시도 (PR #815/#817 close 후 다른 본질)
base / head: devel / contrib/enable-lto-release
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: 결과 부재
변경 규모: +5 / -0, 1 file
커밋: 2
검토일: 2026-05-12
---

# PR #818 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #818 |
| 제목 | perf: release 빌드에 LTO + codegen-units=1 + strip 활성화 |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 (5/11 사이클 15번째 시도, PR #815/#817 close 후) |
| base / head | devel / contrib/enable-lto-release |
| mergeable | CONFLICTING (DIRTY — Cargo.toml 1 파일 충돌) |
| CI | 결과 부재 |
| 변경 규모 | +5 / -0, 1 file (Cargo.toml) |
| 커밋 수 | 2 (1 본질 + 1 strip=debuginfo 정정) |
| closes | #790 |

## 2. 본질 (Issue #790)

`Cargo.toml` 영역 영역 release 프로파일 영역 영역 LTO + codegen-units=1 + strip 활성화 → 바이너리 크기 축소 + 런타임 성능 개선.

### Issue #790 영역 영역 명시
- 제안자 ripgrep profile 패턴 정합
- LTO + CU1 → 8.2 MiB (10.7 MiB → -2.5 MiB, ~23% 감소, 컨트리뷰터 측정)
- Release 프로파일 영역 영역만 활성화 → 개발 빌드 영향 부재

### 컨트리뷰터 PR 본문 측정 (Linux x86_64, Rust 1.87)
| 설정 | 바이너리 크기 | 클린 빌드 시간 |
|------|-------------|--------------|
| 기존 Release | ~10MB | ~58s |
| LTO + CU1 + strip | 7.8MB | ~2m 42s |

→ 빌드 시간 ~3배 증가 (58s → 2m 42s) — CI 시간 영향 점검 필요.

## 3. 정정 본질 — `Cargo.toml` +5/-0

```toml
[profile.release]
lto = true
codegen-units = 1
strip = "debuginfo"
```

### 변경 영역
- **`lto = true`** — Fat LTO 영역 영역 크로스 크레이트 인라이닝 최적화
- **`codegen-units = 1`** — 단일 코드젠 유닛 영역 영역 전역 최적화 극대화
- **`strip = "debuginfo"`** — 디버그 정보 제거 (panic backtrace 보존, 리뷰 반영 commit `cdde2bd6`)

### 리뷰 반영 commit (`cdde2bd6`)
초기 `strip = true` → `strip = "debuginfo"` 정정 — panic backtrace (symbol table) 보존 + 디버그 정보만 제거. 본 환경 영역 영역 panic 시 backtrace 영역 영역 보존 정합.

## 4. 본 환경 점검 영역

### 4.1 WASM 빌드 영향 점검 필수
본 환경 영역 영역 WASM 영역 영역 Docker 영역 영역 `wasm-pack build` 영역 영역 빌드 — `[profile.release]` 영역 영역 영향 받음 + `wasm-opt` 영역 영역 후속 최적화.

**우려 영역**:
- LTO + CU1 영역 영역 WASM 영역 영역 빌드 시간 영역 영역 막대한 증가 (Docker 영역 영역)
- WASM 크기 영역 영역 추가 감소 가능 (이미 wasm-opt 영역 영역 최적화 중) 또는 미세 증가 가능
- 5/11 사이클 영역 영역 WASM 4.5-4.68 MB 영역 영역 기준 → 변화 점검 필요

### 4.2 CI 빌드 시간 영역 영역 영향
- 컨트리뷰터 측정 영역 영역 58s → 2m 42s (~3배 증가)
- 본 환경 영역 영역 GitHub Actions CI 영역 영역 영향 점검 (Build & Test 시간)
- sweep / cargo test 영역 영역 release 빌드 의존 시 영역 영역 누적 영향

### 4.3 panic backtrace 영역 영역 보존 확인
`strip = "debuginfo"` 영역 영역 symbol table 보존 → panic 영역 영역 backtrace 영역 영역 함수 이름 영역 영역 표시 정합 (작업지시자 디버깅 영역 영역 필수).

## 5. 본 환경 충돌 분석

### 5.1 1 파일 충돌 — Cargo.toml
| 파일 | base | our (devel) | their (PR) |
|------|------|-------------|------------|
| `Cargo.toml` | fbc582f9 | 91ba13a2 | 1c3897df |

### 5.2 정합 전략
- devel 측 영역 영역 5/11 사이클 영역 영역 version 0.7.11 변경 (PATCH 릴리즈 후)
- PR 측 영역 영역 파일 끝 영역 영역 `[profile.release]` 5 라인 추가
- → auto-merge 영역 영역 충돌 가능성 (`[lints.rust]` 끝 영역 영역 추가 영역 영역 두 측 모두 변경)
- 정합 영역 영역 양측 모두 보존 (devel 측 version + PR 측 profile.release 추가)

## 6. 영역 좁힘 (회귀 부재 가드)

- **개발 빌드 영향 부재** — `[profile.release]` 영역 영역만 적용
- **panic backtrace 보존** — `strip = "debuginfo"` (Copilot 리뷰 영역 영역 정합)
- 기존 cargo build/test 통과 영역 영역 보장 (PR 본문 명시)
- 기존 clippy `-D warnings` 통과 영역 영역 보장 (PR 본문 명시)

## 7. ⚠️ 우려 영역

### 7.1 WASM 빌드 시간 + 크기 영역 영역 미점검
PR 본문 영역 영역 native 바이너리 (rhwp CLI) 측정만 영역 영역 — **WASM 빌드 측정 부재**. 본 환경 영역 영역 점검 필수:
- WASM 빌드 시간 영역 영역 LTO 적용 시 영역 영역 추가 증가 (Docker 영역 영역 이미 1m 30s)
- WASM 크기 영역 영역 변화 (4.5 MB 기준)
- `wasm-opt` 영역 영역 후속 최적화 영역 영역 추가 효과 영역 영역 점검

### 7.2 CI 시간 영역 영역 영향
- GitHub Actions Build & Test 영역 영역 cargo build --release 영역 영역 시간 증가 (~3배)
- sweep 영역 영역 7 fixture / 170 페이지 영역 영역 누적 영향
- 본 환경 영역 영역 PR cycle 영역 영역 영향

### 7.3 incremental rebuild 영역 영역 영향
- LTO 영역 영역 incremental rebuild 영역 영역 cache miss 영역 영역 잦은 발생
- 본 환경 영역 영역 작업지시자 영역 영역 cargo build --release 영역 영역 자주 수행 — 시간 영향

## 8. 처리 옵션

### 옵션 A — 1 commits cherry-pick + 충돌 수동 해결 + no-ff merge + 결과 측정

```bash
git checkout local/devel
git cherry-pick de0b2a9b cdde2bd6
# Cargo.toml 충돌 수동 해결 (devel version + PR profile.release 양측 보존)
# 빌드 시간 + 바이너리 크기 + WASM 크기 본 환경 측정
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash cherry-pick (단일 commit)

### 옵션 C — 본 환경 영역 영역 측정 후 결정
WASM 빌드 시간 + 크기 영역 영역 측정 후 영역 영역:
- 영향 미세 영역 영역 옵션 A 진행
- 영향 큼 영역 영역 컨트리뷰터 영역 영역 native 영역 영역만 적용 옵션 (e.g., `[profile.release]` 영역 영역 native target 한정) 영역 영역 요청

### 옵션 D — 단계적 적용
1. 먼저 `strip = "debuginfo"` 만 적용 (영향 최소, 즉시 효과)
2. LTO + CU1 영역 영역 별 PR (영향 측정 후 결정)

## 9. 검증 게이트

### 9.1 자기 검증
- [ ] cherry-pick 2 commits + Cargo.toml 충돌 수동 해결
- [ ] cargo build --release (시간 + 바이너리 크기 측정)
- [ ] cargo test --release ALL GREEN
- [ ] cargo clippy --release -- -D warnings
- [ ] **WASM 빌드 (Docker) 시간 + 크기 측정** — 본 환경 영역 영역 핵심 점검
- [ ] 광범위 sweep 7 fixture / 170 페이지 / 회귀 0

### 9.2 정량 점검 표
| 항목 | devel HEAD | PR 적용 | 차이 |
|------|------------|--------|------|
| `target/release/rhwp` 크기 | ? | ? | ? |
| `cargo build --release` 시간 | ? | ? | ? |
| `pkg/rhwp_bg.wasm` 크기 | ? | ? | ? |
| WASM 빌드 시간 (Docker) | ? | ? | ? |

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 15번째 시도, PR #815/#817 close 후) |
| `feedback_image_renderer_paths_separate` | Cargo.toml 영역 영역 빌드 설정 영역 영역 렌더링 경로 무관 |
| `feedback_process_must_follow` | 본 환경 영역 영역 WASM 빌드 영역 영역 측정 필수 — 컨트리뷰터 PR 영역 영역 native 측정만 |
| `feedback_small_batch_release_strategy` | 작은 변경 (+5/-0) + opt-in (release 프로파일 한정) + 명확 효과 — PATCH cycle 머지 정합 |
| `feedback_visual_judgment_authority` | 빌드 설정 영역 영역 시각 판정 면제, 그러나 sweep 결정적 검증 필요 |
| `feedback_pr_supersede_chain` | Issue #790 (외부 제안, OPEN) → **PR #818** (LTO + CU1 + strip 적용) — 본질 정합 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick 2 commits + Cargo.toml 충돌 수동 해결
2. 자기 검증:
   - cargo build --release 시간 + 바이너리 크기 측정
   - cargo test --release + cargo clippy --release -D warnings
   - **WASM 빌드 (Docker) 시간 + 크기 측정** (4.5 MB 기준 영역 영역 변화 확인)
   - 광범위 sweep 7 fixture / 170 페이지 / 회귀 0
3. 측정 결과 영역 영역 작업지시자 영역 영역 보고 + 머지 결정
4. 검증 통과 → no-ff merge + push + archives + 5/12 orders + Issue #790 close
5. PR #818 close

---

작성: 2026-05-12

---
PR: #818
제목: perf — release 빌드에 LTO + codegen-units=1 + strip 활성화 (closes #790)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 15번째 시도 (PR #815/#817 close 후)
처리: 옵션 A — 2 commits cherry-pick + Cargo.toml 충돌 수동 해결 + 정량 측정 + no-ff merge
처리일: 2026-05-12
머지 commit: f5abcf8d
---

# PR #818 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + 충돌 수동 해결 + 정량 측정 + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `f5abcf8d` (--no-ff merge) |
| Cherry-pick commits | 2 (본질 + strip=debuginfo 정정) |
| closes | #790 |
| 시각 판정 | 면제 (빌드 설정 영역, sweep byte-identical 입증) |
| 자기 검증 | cargo test + clippy ALL GREEN + sweep 170/170 same + WASM 4.3 MB |

## 2. 본질 (Issue #790)

`Cargo.toml [profile.release]` 영역 LTO + codegen-units=1 + strip 활성화 → 바이너리 크기 축소 + 런타임 성능 개선. Issue #790 외부 제안 (ripgrep profile 패턴 정합).

## 3. 정정 본질 — `Cargo.toml` +5/-0

```toml
[profile.release]
lto = true
codegen-units = 1
strip = "debuginfo"
```

### 리뷰 반영 commit (`9ccb0c38`)
초기 `strip = true` → `strip = "debuginfo"` 정정 — panic backtrace symbol table 보존 + 디버그 정보만 제거.

## 4. 본 환경 정량 측정 ★ 핵심 결과

| 항목 | devel HEAD (before) | LTO 적용 (after) | 차이 |
|------|---------------------|------------------|------|
| **rhwp CLI 크기** | 14 MB | **10 MB** | **-4 MB (-28%)** |
| **WASM 크기** | 4.6 MB | **4.3 MB** | **-0.3 MB (-6.5%)** |
| cargo build --release (clean) | ~58s | **2m 53s** | +1m 55s (~3배) |
| WASM 빌드 (Docker) | ~1m 30s | **2m 23s** | +53s (+59%) |
| cargo test --release | ALL GREEN | **ALL GREEN** | 회귀 0 |
| cargo clippy -D warnings | 통과 | **통과** | 회귀 0 |
| 광범위 sweep | baseline | **170 same / 0 diff** | byte-identical |

### 효과 분석
- **이득**: rhwp CLI -28% / WASM -6.5% 크기 감소 — 사용자 다운로드 시간 + 메모리 사용량 개선
- **비용**: release 빌드 시간 ~3배 증가 — 그러나 개발 빌드 (cargo build) 영향 부재
- **회귀 부재**: sweep byte-identical 영역 SVG 출력 무영향 입증 + cargo test + clippy 통과

## 5. 본 환경 충돌 수동 해결 — Cargo.toml

| 영역 | devel HEAD | incoming (PR) | 정합 |
|------|------------|---------------|------|
| `[[example]] pr599_png_gateway` (native-skia) | 추가 (devel) | 부재 | 보존 |
| `[profile.release]` 5 라인 | 부재 | 추가 (PR) | 적용 |

→ 양측 모두 보존 (PR #599 example + LTO profile 정합).

## 6. 인프라

- 신규 의존성 도입 부재
- 단순 Cargo.toml 빌드 설정 추가
- panic backtrace 보존 (`strip = "debuginfo"`)

## 7. 영역 좁힘 (회귀 부재 가드)

- **개발 빌드 영향 부재** — `[profile.release]` 영역 영역만 적용
- **panic backtrace 보존** — `strip = "debuginfo"`
- sweep byte-identical 입증 — 출력 무영향
- cargo test/clippy 통과 — 동작 무영향

## 8. CI 결과 부재 (DIRTY 영역)

mergeStateStatus = `DIRTY` 영역 CI 미실행. 본 환경 자기 검증 (cargo test + clippy + sweep) + WASM 빌드 측정 영역 보완.

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 15번째 시도, PR #815/#817 close 후) |
| `feedback_image_renderer_paths_separate` | Cargo.toml 빌드 설정 영역 영역 렌더링 경로 무관 |
| `feedback_process_must_follow` 권위 사례 강화 | 본 환경 영역 영역 WASM 빌드 측정 필수 — 컨트리뷰터 PR 영역 영역 native 측정만, 본 환경 영역 영역 WASM 4.3 MB 추가 측정 (4.6 → 4.3, -6.5%) |
| `feedback_small_batch_release_strategy` 권위 사례 강화 | 작은 변경 (+5/-0) + opt-in (release 한정) + 명확 효과 (-28%/-6.5%) — PATCH cycle 머지 정합 |
| `feedback_hancom_compat_specific_over_general` | 회귀 부재 가드 — `strip = "debuginfo"` (Copilot 리뷰 영역 영역 정정) + sweep byte-identical |
| `feedback_visual_judgment_authority` | 빌드 설정 영역 영역 시각 판정 면제 — sweep 결정적 검증 통과 |
| `feedback_pr_supersede_chain` | Issue #790 (외부 제안, OPEN) → **PR #818** (LTO + CU1 + strip 적용) — 본질 정합 |

## 10. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #790 close 완료
- 차후 v0.7.12+ 릴리즈 시 영역 영역 본 LTO 영역 영역 자동 적용 (PATCH cycle 정합)

---

작성: 2026-05-12

---
PR: #735
제목: Task #613 — export-png VLM 프리셋 확장 (GPT-4V / Gemini / Qwen-VL / LLaVA)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 5번째 PR)
base / head: devel / contrib/vlm-preset-expansion
mergeStateStatus: DIRTY
mergeable: CONFLICTING — `git merge-tree` 충돌 2건 (`src/main.rs` + `src/document_core/queries/rendering.rs`)
CI: ALL SUCCESS (PR #734 머지 영역 영역 후 충돌 발생 영역 영역, CI 영역 영역 통과 영역 영역 base 영역 영역)
변경 규모: +72 / -8, 2 files
검토일: 2026-05-10
---

# PR #735 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #735 |
| 제목 | Task #613 — export-png VLM 프리셋 확장 (GPT-4V / Gemini / Qwen-VL / LLaVA) |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734 후속 영역 5번째) |
| base / head | devel / contrib/vlm-preset-expansion |
| mergeStateStatus | **DIRTY**, mergeable: CONFLICTING — 동일 파일 영역 영역 PR #734 영역 영역 누적 변경 영역 영역 |
| CI | ALL SUCCESS (base 영역 영역 통과 영역 영역) |
| 변경 규모 | +72 / -8, 2 files |
| 커밋 수 | 2 (Task 1 + Copilot 리뷰 반영 1) |
| closes | #613 |

## 2. 결함 본질 (Issue #613)

### 2.1 결함 영역
PR #599 (Task #588 후속) 영역 영역 export-png CLI 영역 영역 VLM 친화 프리셋 영역 영역 추가 — 첫 단계 영역 영역 **Claude Vision** 만 구현 (`--vlm-target claude`). 다른 VLM provider 프리셋 영역 영역 미지원.

### 2.2 채택 접근
6종 VLM provider 영역 영역 프리셋 추가:

| 프리셋 | 한 변 한도 | 최대 픽셀 | 비고 |
|--------|-----------|----------|------|
| `claude` | 1568 px | 1.15 MP | 기존 (변경 없음) |
| `gpt4v-low` | 512 px | 262K px | GPT-4V low detail |
| `gpt4v-high` | 2000 px | 1.54 MP | GPT-4V high detail (별칭: `gpt4v`) |
| `gemini` | 3072 px | 9.44 MP | Google Gemini |
| `qwen-vl` | 2240 px | 5.02 MP | Qwen-VL 28×28 patch (별칭: `qwen`) |
| `llava` | 672 px | 452K px | LLaVA / OSS CLIP backbone |

## 3. PR 의 정정 — 2 영역

### 3.1 `src/document_core/queries/rendering.rs` (+61/-3)

**`VlmTarget` enum 확장** (1 → 6 variants):
```rust
pub enum VlmTarget {
    Claude, Gpt4vLow, Gpt4vHigh, Gemini, QwenVl, Llava,
}
```

**`constraints()` 메서드** 영역 영역 각 provider 별 한도 (edge / pixels) 추가.

**`from_str()` 정규화 + alias** (Copilot 리뷰 반영):
```rust
pub fn from_str(s: &str) -> Option<Self> {
    match s.to_lowercase().replace('-', "_").as_str() {
        "claude" => Some(VlmTarget::Claude),
        "gpt4v_low" => Some(VlmTarget::Gpt4vLow),
        "gpt4v_high" | "gpt4v" => Some(VlmTarget::Gpt4vHigh),
        "gemini" => Some(VlmTarget::Gemini),
        "qwen_vl" | "qwen" => Some(VlmTarget::QwenVl),
        "llava" => Some(VlmTarget::Llava),
        _ => None,
    }
}
```

**`all_names()` 헬퍼** — 에러 메시지 영역 영역 동기화.

**신규 unit tests 2건**:
- `vlm_target_from_str_all_variants` — 12 케이스 (정확 + alias + 대문자 + unknown)
- `vlm_target_constraints_are_sane` — 모든 variant 영역 영역 edge > 0 + pixels > 0 + edge² ≥ pixels 검증

### 3.2 `src/main.rs` (+11/-5)

CLI 도움말 영역 영역 각 프리셋 한 줄 설명 갱신 + 에러 메시지 영역 영역 `VlmTarget::all_names()` 동기화.

### 3.3 Copilot 리뷰 반영 (commit `c6a762c9`)
- `from_str` 정규화 후 dead arm 제거
- 별칭 영역 영역 도움말 동기화

## 4. 충돌 / mergeable

### 4.1 충돌 본질

PR #735 영역 영역 base = `c9dd6f9c` (5/9 시점) — devel HEAD 영역 영역 PR #734 (Task #614 export-png --dpi 옵션) 영역 영역 머지 영역 → **동일 파일** (src/main.rs / src/document_core/queries/rendering.rs) 영역 영역 두 PR 영역 영역 변경 영역 영역 누적 영역 영역 충돌 발생.

### 4.2 충돌 영역
- `src/main.rs` — `print_help()` 영역 영역 `--dpi` 도움말 추가 (PR #734) + `--vlm-target` 도움말 6종 확장 (PR #735) 영역 영역 동일 영역 영역 변경
- `src/main.rs` — `--vlm-target` match 블록 영역 영역 에러 메시지 영역 영역 변경 (PR #735)
- `src/document_core/queries/rendering.rs` — `VlmTarget` enum 부근 영역 영역 PR #734 영역 영역 `inject_png_phys` 영역 영역 추가 영역 영역 코드 위치 영역 영역 영향

### 4.3 충돌 해결 영역 영역 의도

**두 PR 영역 영역 영역 의도 영역 영역 다른 영역 영역** — 수동 통합 영역 영역 가능:
- PR #734 영역 영역 `--dpi` (PNG pHYs) 보존
- PR #735 영역 영역 `--vlm-target` 6종 확장 보존
- 도움말 영역 영역 `--dpi` 영역 영역 `--vlm-target` 위 영역 영역 (현재 순서 보존)

## 5. 본 환경 점검

### 5.1 cherry-pick 전략

**옵션 B (squash cherry-pick + 수동 충돌 해결)** — PR HEAD `pr735-head` 영역 영역 squash cherry-pick 영역 영역 단일 commit 영역 영역 적용 + 충돌 영역 영역 수동 해결 (PR #734 + PR #735 영역 영역 통합).

```bash
git checkout -b local/task613 d985e3af
git cherry-pick --no-commit c9dd6f9c..pr735-head
# 충돌 영역 영역 수동 해결 — PR #734 (--dpi) + PR #735 (VLM 6종) 영역 영역 통합
git commit -m "Task #613: VLM 프리셋 확장 — squash + PR #734 통합"
```

### 5.2 의존성 점검

| 의존성 | 상태 |
|--------|------|
| PR #734 (Task #614 --dpi) 머지 | ✅ 머지 commit `d76e86fc` |
| `VlmTarget` 영역 영역 기존 정의 (PR #599) | ✅ `Claude` variant 영역 영역 보존 |
| `feedback_image_renderer_paths_separate` | native CLI / SkiaLayerRenderer 영역 영역 만 변경 — WASM/CanvasKit 영역 영역 무영향 |

## 6. 영향 범위

### 6.1 변경 영역
- `export-png --vlm-target` 영역 영역 6 provider 프리셋 (Claude + GPT-4V low/high + Gemini + Qwen-VL + LLaVA)
- `from_str` 영역 영역 하이픈/밑줄 정규화 + 축약 별칭 (gpt4v / qwen)
- 도움말 + 에러 메시지 영역 영역 동기화

### 6.2 무변경 영역
- 기존 `--vlm-target claude` 동작 보존
- `--dpi` (PR #734) 동작 보존 — 충돌 수동 해결 영역 영역 통합
- 다른 export-png 옵션 (`--scale` / `--max-dimension` / `--font-path`) 영역 영역 무영향
- HWP3/HWPX 변환본 영역 영역 시각 정합

### 6.3 위험 영역
- **opt-in** — `--vlm-target` 미지정 시 기존 동작 100% 보존
- 충돌 수동 해결 영역 영역 PR #734 영역 영역 통합 — 신중한 점검 필요

## 7. 회귀 가드 (PR 영역 영역 신규)

`rendering.rs::tests` 영역 영역 추가:
- `vlm_target_from_str_all_variants` — 12 케이스 (정확 매칭 + alias + 대소문자 + unknown)
- `vlm_target_constraints_are_sane` — 6 variants 영역 영역 sanity check

## 8. 처리 옵션

### 옵션 A — 2 commits cherry-pick (개별 충돌 가능성 영역 영역 squash 우선) + no-ff merge

```bash
git checkout -b local/task613 d985e3af
git cherry-pick c13a1963 c6a762c9
# 충돌 발생 시 수동 해결 (PR #734 + #735 통합)
git checkout local/devel
git merge --no-ff local/task613
```

### 옵션 B — PR HEAD squash cherry-pick + 수동 통합 (PR #729/#730/#732 동일 패턴)

```bash
git checkout -b local/task613 d985e3af
git cherry-pick --no-commit c9dd6f9c..pr735-head
# 충돌 영역 영역 수동 해결 (도움말 + match 블록 통합)
git commit -m "Task #613: VLM 프리셋 확장 — squash + PR #734 통합"
```

→ **옵션 A 시도 후 충돌 발생 시 옵션 B** 권장.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] 충돌 수동 해결 — `--dpi` + `--vlm-target` 영역 영역 동시 보존 점검
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 신규 2건 PASS + 기존 ALL GREEN
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (CLI 영역 영역 만 변경 영역 영역 SVG 무영향)

### 시각 판정 게이트 — **면제 합리**

본 PR 영역 영역 의 본질 영역 영역 **CLI 옵션 + VlmTarget enum 확장**:
- 결정적 검증 영역 영역 명시 (신규 unit tests 2건)
- SVG 시각 출력 영역 영역 변경 부재 (광범위 sweep 0 회귀 보장)
- 픽셀 수 영역 영역 영역 기존 `--max-dimension` / `--max-pixels` 영역 영역 의 자동 적용 영역 영역 — 메커니즘 동일
- `feedback_visual_judgment_authority` 정합 — 시각 출력 무변경 영역 영역 면제 합리

선택적 영역 영역: 작업지시자 실 sample 영역 영역 `rhwp export-png input.hwp --vlm-target gemini` 등 영역 영역 dimension 점검 가능.

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735 영역 5번째 PR) |
| `feedback_image_renderer_paths_separate` | native CLI / SkiaLayerRenderer 영역 영역 만 변경 — WASM/CanvasKit / 다른 렌더러 영역 영역 무영향 |
| `feedback_process_must_follow` | opt-in 영역 영역 (`--vlm-target` 미지정 시 동작 보존) — 위험 좁힘 + 외부 의존성 부재 |
| `feedback_visual_judgment_authority` | CLI 옵션 + enum 확장 영역 영역 (시각 출력 무영향) → 시각 판정 면제 합리 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 cherry-pick (옵션 A 영역 영역 시도)
2. 충돌 발생 시 옵션 B (PR HEAD squash cherry-pick) + 수동 통합 (PR #734 영역 영역 의 `--dpi` 영역 영역 PR #735 영역 영역 의 VLM 6종 영역 영역 동시 보존)
3. 자기 검증 (cargo build/test/clippy + 광범위 sweep + 신규 2 PASS)
4. 시각 판정 면제 합리 — 결정적 검증 통과 영역 영역 즉시 머지
5. no-ff merge + push + archives 이동 + 5/10 orders 갱신
6. PR #735 close (closes #613 자동 정합)

---

작성: 2026-05-10

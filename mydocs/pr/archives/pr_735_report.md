---
PR: #735
제목: Task #613 — export-png VLM 프리셋 확장 (GPT-4V / Gemini / Qwen-VL / LLaVA)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 5번째 PR)
처리: 옵션 A — 2 commits cherry-pick + 수동 충돌 해결 + no-ff merge
처리일: 2026-05-10
머지 commit: 56fe3a74
---

# PR #735 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + 수동 충돌 해결 + no-ff merge `56fe3a74`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `56fe3a74` (--no-ff merge) |
| Cherry-pick commits | `7d8efd29` (Task #613) + `28837339` (Copilot 리뷰) |
| 충돌 해결 | 수동 (PR #734 영역 `--dpi` + PR #735 영역 VLM 6종 동시 보존) |
| closes | #613 |
| 시각 판정 | 면제 합리 (CLI + enum 확장 영역 영역 픽셀 데이터 무영향) + 통합 검증 ✅ |
| 자기 검증 | cargo test ALL GREEN + VLM unit tests 2 PASS + sweep 170/170 same |

## 2. 정정 본질 — 2 files, +71/-8

### 2.1 `src/document_core/queries/rendering.rs` (+61/-3)

**`VlmTarget` enum 확장** (1 → 6 variants):
```rust
pub enum VlmTarget {
    Claude, Gpt4vLow, Gpt4vHigh, Gemini, QwenVl, Llava,
}
```

**`constraints()` 메서드** 영역 영역 각 provider 한도:

| 프리셋 | edge | pixels |
|--------|------|--------|
| `claude` | 1568 | 1.15 MP |
| `gpt4v-low` | 512 | 262K |
| `gpt4v-high` | 2000 | 1.54 MP |
| `gemini` | 3072 | 9.44 MP |
| `qwen-vl` | 2240 | 5.02 MP |
| `llava` | 672 | 452K |

**`from_str()` 정규화 + alias** (Copilot 리뷰 반영):
- 하이픈/밑줄 통일 (`replace('-', "_")`)
- 축약 별칭 (`gpt4v` → `Gpt4vHigh`, `qwen` → `QwenVl`)
- 대소문자 무시 (`to_lowercase()`)

**`all_names()` 헬퍼** — 에러 메시지 영역 영역 동기화.

**신규 unit tests 2건**:
- `vlm_target_from_str_all_variants` — 12 케이스
- `vlm_target_constraints_are_sane` — 6 variants sanity check

### 2.2 `src/main.rs` (+10/-5)

CLI 도움말 영역 영역 각 프리셋 한 줄 설명 (하이픈/밑줄 허용 명시) + 에러 메시지 영역 영역 `VlmTarget::all_names()` 동기화.

### 2.3 Copilot 리뷰 반영 (commit `28837339`)
- `from_str` 정규화 후 dead arm 제거
- 별칭 영역 영역 도움말 동기화 (하이픈/밑줄 모두 허용 명시)

## 3. 본 환경 cherry-pick + 수동 충돌 해결

### 3.1 충돌 본질
PR #735 base = `c9dd6f9c` (5/9 시점) — devel HEAD 영역 영역 PR #734 (Task #614 --dpi) 영역 영역 머지 영역 → 동일 파일 영역 영역 두 PR 영역 영역 변경 영역 영역 누적 영역 영역 충돌 발생.

### 3.2 수동 통합 결정
**두 PR 영역 영역 영역 의도 영역 영역 다른 영역 영역** (동일 컨트리뷰터 @oksure):
- PR #734 영역 영역 `--dpi` (PNG pHYs) 보존
- PR #735 영역 영역 `--vlm-target` 6종 보존
- 도움말 영역 영역 `--dpi` → `--vlm-target` 순서 보존

### 3.3 cherry-pick 결과
- `7d8efd29` (Task #613 본질) — 충돌 해결 (main.rs 도움말 + rendering.rs 테스트 모듈)
- `28837339` (Copilot 리뷰) — 충돌 해결 (main.rs 도움말 한 줄 동기화)

## 4. 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release --features native-skia` | ✅ 통과 (32.25s) |
| VLM unit tests | ✅ **2 PASS** (`vlm_target_from_str_all_variants` + `vlm_target_constraints_are_sane`) |
| `cargo test --release` (전체) | ✅ ALL GREEN, failed 0 |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |

## 5. 통합 검증 (PR #734 + PR #735)

`samples/통합재정통계(2010.11월).hwp` 영역 영역 통합 동작 검증:

| 패턴 | 동작 |
|------|------|
| `--vlm-target gemini` 단독 | IHDR 폭 794 (1.0× — 페이지 영역 영역 작아 자동 조정 미적용) ✓ |
| `--vlm-target gpt4v --dpi 200` | IHDR 폭 794 + **pHYs ppm 7874 = 200 DPI** ✓ |

**두 옵션 동시 사용 영역 영역 정합** — `--vlm-target` 영역 영역 픽셀 한도 + `--dpi` 영역 영역 메타데이터 영역 영역 독립 동작.

## 6. 영향 범위

### 6.1 변경 영역
- `export-png --vlm-target` 영역 영역 6 provider 프리셋
- `from_str` 영역 영역 하이픈/밑줄 정규화 + 축약 별칭
- 도움말 + 에러 메시지 영역 영역 동기화

### 6.2 무변경 영역 (sweep 170/170 same 영역 영역 입증)
- 기존 `--vlm-target claude` 동작 보존
- `--dpi` (PR #734) 동작 보존
- 다른 export-png 옵션 (`--scale` / `--max-dimension` / `--font-path`) 영역 영역 무영향
- HWP3/HWPX 변환본 영역 영역 시각 정합

### 6.3 위험 영역
- **opt-in** — `--vlm-target` 미지정 시 기존 동작 100% 보존

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735 영역 5번째 PR) |
| `feedback_image_renderer_paths_separate` | native CLI / SkiaLayerRenderer 영역 영역 만 변경 — WASM/CanvasKit / 다른 렌더러 영역 영역 무영향 |
| `feedback_process_must_follow` | opt-in 영역 영역 위험 좁힘 + 외부 의존성 부재 |
| `feedback_visual_judgment_authority` | CLI 옵션 + enum 확장 영역 영역 시각 출력 무영향 → 시각 판정 면제 합리 + 통합 검증 ✅ |

## 8. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- 매뉴얼 (`mydocs/manual/export_png_command.md`) 영역 영역 VLM 프리셋 표 갱신 필요 (현재 claude 1건 만 명시) — 별건 후속

---

작성: 2026-05-10

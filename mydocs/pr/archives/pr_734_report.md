---
PR: #734
제목: Task #614 — export-png --dpi 옵션 PNG pHYs chunk 메타데이터
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 4번째 PR)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: (확정 commit hash 영역 영역 후속 commit 영역 영역 영역)
---

# PR #734 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `d76e86fc` (--no-ff merge) |
| Cherry-pick commits | `b427a68a` (Task #614) + `9c3a6257` (Copilot 리뷰) |
| closes | #614 |
| 시각 판정 | 면제 합리 (PNG 메타데이터 — 픽셀 데이터 무영향) + 작업지시자 실 sample 검증 ✅ |
| 자기 검증 | cargo test ALL GREEN + sweep 170/170 same + 신규 unit tests 2 PASS |

## 2. 정정 본질 — 2 files, +164/-5

### 2.1 `src/main.rs` (+21/-1)
- CLI `--dpi <값>` 옵션 추가 (양수 실수 검증)
- 도움말 갱신
- `PngExportOptions` 영역 영역 `dpi: Option<f64>` 필드 추가

### 2.2 `src/document_core/queries/rendering.rs` (+143/-4)
- **scale 결정 우선순위 갱신** — `--dpi` 만 지정 시 `scale = dpi/96.0` 자동
- **`inject_png_phys` 후처리 함수** — PNG IHDR 직후 + 첫 IDAT 직전 영역 영역 pHYs chunk 삽입 (PNG spec §11.3.5.3)
- **CRC32 자체 구현** — 외부 의존성 없음
- 신규 unit tests 2건: `inject_png_phys_inserts_after_ihdr` + `png_crc32_known_value`

### 2.3 Copilot 리뷰 반영 (commit `9c3a6257`)
- PNG 시그니처 8바이트 검증
- IHDR chunk 타입 (`b"IHDR"`) + data length (=13) 검증
- `ihdr_end` 계산 영역 영역 `checked_add` (오버플로우 방지)

## 3. 본 환경 cherry-pick + 검증

### 3.1 cherry-pick (2 commits)
충돌 0건.

### 3.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 |
| 신규 unit tests | ✅ **2 PASS** |
| `cargo test --release` (전체) | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |

### 3.3 작업지시자 실 sample 검증 ✅

**Sample**: `samples/통합재정통계(2010.11월).hwp` (1 페이지, 47KB)

| 패턴 | pHYs chunk | PPM | 변환 DPI | IHDR 폭 | scale |
|------|-----------|-----|----------|---------|-------|
| baseline (--dpi 미지정) | ❌ 부재 | — | — | 794 | 1.0 (기본) |
| `--dpi 96` | ✅ | 3780 | **96 DPI** ✓ | 794 | 1.0 |
| `--dpi 300` | ✅ | 11811 | **300 DPI** ✓ | 2481 | **3.125** (auto) |
| `--scale 2 --dpi 150` | ✅ | 5906 | **150 DPI** ✓ | 1588 | **2.0** (명시) |

검증 영역 영역:
- `--dpi` 미지정 시 pHYs 부재 (opt-in 정합)
- 모든 DPI 영역 영역 정확 변환 (`ppm = dpi / 0.0254`)
- auto-scale 영역 영역 정합 (`scale = dpi / 96.0`)
- 명시 `--scale` 영역 영역 우선 적용 + `--dpi` 영역 영역 메타데이터 만 영향
- pHYs chunk 영역 영역 IHDR 직후 삽입 (PNG spec §11.3.5.3 정합)
- pHYs chunk 영역 영역 21 bytes 추가 (4 length + 4 type + 9 data + 4 CRC)

## 4. 영향 범위

### 4.1 변경 영역
- `export-png` CLI 영역 영역 `--dpi` 옵션 추가
- PNG raster 영역 영역 pHYs chunk 메타데이터 (선택적 — opt-in)

### 4.2 무변경 영역 (sweep 170/170 same 영역 영역 입증)
- 기존 `export-png` (--scale / --max-dimension / --vlm-target) 영역 영역 동작 보존
- HWP3/HWPX 변환본 영역 영역 시각 정합
- WASM/CanvasKit / 다른 렌더러 경로

### 4.3 위험 영역
- **opt-in 영역 영역** — `--dpi` 미지정 시 영역 영역 기존 동작 100% 보존

## 5. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734 영역 4번째 PR) |
| `feedback_image_renderer_paths_separate` | native CLI / SkiaLayerRenderer 영역 영역 만 변경 — WASM/CanvasKit / 다른 렌더러 영역 영역 무영향 |
| `feedback_process_must_follow` | opt-in 영역 영역 (`--dpi` 미지정 시 동작 보존) — 위험 좁힘 + 외부 의존성 부재 (CRC32 자체 구현) |
| `feedback_visual_judgment_authority` | PNG 메타데이터 영역 영역 (픽셀 데이터 무영향) → 시각 판정 면제 + 작업지시자 실 sample 검증 ✅ |

## 6. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- 다른 VLM 프리셋 (#613) 영역 영역 별건

---

작성: 2026-05-10

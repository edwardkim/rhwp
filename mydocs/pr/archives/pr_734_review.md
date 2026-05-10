---
PR: #734
제목: Task #614 — export-png --dpi 옵션 PNG pHYs chunk 메타데이터
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 4번째 PR)
base / head: devel / contrib/export-png-dpi
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS (Build & Test / Canvas visual diff / CodeQL / Analyze rust+js+py / WASM SKIPPED)
변경 규모: +164 / -5, 2 files (소스 only)
검토일: 2026-05-10
---

# PR #734 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #734 |
| 제목 | Task #614 — export-png --dpi 옵션 PNG pHYs chunk 메타데이터 |
| 컨트리뷰터 | @oksure — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 PR #728/#729/#730 후속 영역 4번째) |
| base / head | devel / contrib/export-png-dpi |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS (Canvas visual diff 포함) |
| 변경 규모 | +164 / -5, 2 files |
| 커밋 수 | 2 (Task 1 + Copilot 리뷰 반영 1) |
| closes | #614 |

## 2. 결함 본질 (Issue #614)

### 2.1 결함 영역
PR #599 (Task #588 후속) 영역 영역 export-png CLI 영역 영역 PNG raster 출력 지원. 본 단계 영역 영역 옵션 영역 영역 **VLM 입력용** (scale / max-dimension 픽셀 수 제어) 영역 한정 영역, **인쇄 워크플로우** 영역 영역 의 DPI 메타데이터 지정 영역 영역 미지원.

### 2.2 채택 접근
PNG `pHYs` chunk 영역 영역 DPI 메타데이터 명시 — 인쇄 시점 크기 힌트. 실제 픽셀 수 영역 영역 영향 부재.

```bash
rhwp export-png input.hwp --dpi 300              # scale 3.125 자동, 300 DPI 메타데이터
rhwp export-png input.hwp --scale 2 --dpi 150    # scale 2 명시, 150 DPI 메타데이터
```

## 3. PR 의 정정 — 2 영역

### 3.1 `src/main.rs` (+21/-1)

CLI 옵션 추가:
```rust
"--dpi" => {
    if i + 1 < args.len() {
        match args[i + 1].parse::<f64>() {
            Ok(d) if d.is_finite() && d > 0.0 => dpi = Some(d),
            _ => { eprintln!("오류: --dpi 값이 올바르지 않습니다 (양수 실수 필요)."); return; }
        }
        i += 2;
    } else {
        eprintln!("오류: --dpi 뒤에 DPI 값이 필요합니다.");
        return;
    }
}
```

도움말 갱신 + `PngExportOptions` 영역 영역 `dpi: Option<f64>` 필드 추가.

### 3.2 `src/document_core/queries/rendering.rs` (+143/-4)

**scale 결정 우선순위 갱신**:
1. 명시 `scale`
2. `max_dimension` / VLM 기반 자동
3. **`--dpi` 만 지정 시 `scale = dpi / 96.0`** (#614 신규)
4. 기본 1.0

**pHYs chunk 후처리 삽입** (`inject_png_phys`):
```rust
const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const IHDR_DATA_LEN: usize = 13;

let ppm = (dpi / 0.0254).round() as u32;
// ... 방어 검증 ...
// pHYs chunk: 4-byte X ppm + 4-byte Y ppm + 1-byte unit(1=meter)
```

PNG 표준 §11.3.5.3 정합 — IHDR 직후 영역 영역 첫 IDAT 직전 영역 영역 삽입.

**CRC32 자체 구현** — 외부 의존성 없음. `png_crc32` 함수 영역 영역 known-value 테스트 (`b"IHDR"` → `0xA8A1_AE0A`).

### 3.3 Copilot 리뷰 반영 (commit `9758fabe`)

**방어 검증 강화**:
- PNG 시그니처 8바이트 검증 추가 (오프셋 0~7)
- IHDR chunk 타입 (`b"IHDR"`) + data length (=13) 검증
- `ihdr_end` 계산 영역 영역 `checked_add` 사용 (오버플로우 방지)
- 불필요한 `mut` 제거 (`pos` 변수)

## 4. 회귀 가드 (PR 영역 영역 신규)

`tests/` 영역 영역 미추가 영역 영역 의 신규 unit tests 영역 영역 `rendering.rs::tests` 영역 영역 추가:
- `inject_png_phys_inserts_after_ihdr` — pHYs 삽입 위치 + PPM 값 검증
- `png_crc32_known_value` — CRC32 known-value 검증

## 5. 본 환경 점검

- merge-base: `c9dd6f9c` (5/9 영역 영역 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 영역 영역 격리: `src/main.rs` (CLI) + `src/document_core/queries/rendering.rs` (PNG 후처리) — 다른 layout/render 경로 영역 영역 무관
- HWP 변환본 영역 영역 시각 정합 영역 영역 무영향 (PNG raster pixel 수 영역 영역 만 자동 scale 영역 영역 영향 영역, 메타데이터 영역 영역 영역 표준 chunk)

## 6. 영향 범위

### 6.1 변경 영역
- `export-png` CLI 영역 영역 `--dpi` 옵션 추가
- PNG raster 영역 영역 pHYs chunk 메타데이터 (선택적 — `--dpi` 미지정 시 영역 영역 영향 부재)

### 6.2 무변경 영역
- 기존 `export-png` (--scale / --max-dimension / --vlm-target) 영역 영역 동작 보존
- HWP3/HWPX 변환본 영역 영역 시각 정합
- WASM/CanvasKit / 다른 렌더러 경로 — 본 변경 영역 영역 native CLI 영역 영역 만

### 6.3 위험 영역
- **opt-in 영역 영역** — `--dpi` 미지정 시 영역 영역 기존 동작 100% 보존
- pHYs chunk 영역 영역 표준 영역 영역 영역 PNG 디코더 영역 영역 무시 영역 영역 가능 (메타데이터 전용)

## 7. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 영역 영역 5/10 사이클 영역 영역 진전, 본 PR 영역 영역 단일 함수 + CLI 영역 영역 충돌 부재

## 8. 처리 옵션

### 옵션 A — 2 commits cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task614 f3314910
git cherry-pick 713774eb 9758fabe
git checkout local/devel
git merge --no-ff local/task614
```

→ **옵션 A 추천**.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN (신규 unit tests 영역 영역 PASS 보장)
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (CLI 영역 영역 만 변경 영역 영역 SVG 영역 영역 무영향 보장)

### 시각 판정 게이트 — **면제 합리**

본 PR 영역 영역 의 본질 영역 영역 **PNG 메타데이터** (pHYs chunk):
- 결정적 검증 영역 영역 명시 (신규 unit tests + CRC32 known-value)
- SVG 시각 출력 영역 영역 변경 부재 (광범위 sweep 0 회귀 보장)
- pHYs chunk 영역 영역 표준 PNG 영역 영역 메타데이터 — 픽셀 데이터 무영향
- `feedback_visual_judgment_authority` 정합 — 시각 출력 무변경 영역 영역 면제 합리

선택적 영역 영역: `rhwp export-png --dpi 300 input.hwp` 영역 영역 결과 PNG 영역 영역 `pHYs` chunk 영역 영역 작업지시자 영역 영역 직접 점검 가능 (도구: `pngcheck` 또는 `hexdump`).

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734 영역 4번째 PR) |
| `feedback_image_renderer_paths_separate` | native CLI / SkiaLayerRenderer 영역 영역 만 변경 — WASM/CanvasKit / 다른 렌더러 영역 영역 무영향 |
| `feedback_process_must_follow` | opt-in 영역 영역 (`--dpi` 미지정 시 동작 보존) — 위험 좁힘 + 외부 의존성 부재 (CRC32 자체 구현) |
| `feedback_visual_judgment_authority` | PNG 메타데이터 영역 영역 (픽셀 데이터 무영향) → 시각 판정 면제 합리 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo build/test/clippy + 광범위 sweep)
3. 시각 판정 면제 합리 — 결정적 검증 통과 영역 영역 즉시 머지 가능
4. no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #734 close (closes #614 자동 정합)

---

작성: 2026-05-10

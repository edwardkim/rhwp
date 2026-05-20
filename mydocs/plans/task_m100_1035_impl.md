# Task #1035 구현 계획서

**Issue**: [#1035 HWP3 vs HWP5 변환본 페이지별 paragraph alignment 차이](https://github.com/edwardkim/rhwp/issues/1035)
**Branch**: `local/task1035` (base = `local/devel` = `a52859de`)
**관련 commit**: PR #1009 의 `71054c51` (Task #1007, closed) — starting point

---

## 1. 사전 단언 (PR #1009 commit 분석 완료)

### 1.1 PR #1009 변경 파일

| 파일 | 변경 라인 | 본 task 활용 |
|------|---------|------------|
| `src/renderer/pagination.rs` | +5 | `PaginationOpts::is_hwp3_variant` 필드 (이미 #1005 영역 가능) |
| `src/renderer/pagination/engine.rs` | +84 | **본 task 의 핵심 fix 위치** — variant vpos reset 감지 |
| `src/renderer/typeset.rs` | +136 | 동일 로직 두 경로 정합 |
| `src/renderer/composer.rs` | CHARS_PER_LINE 45→50 | **재평가 필요** (#1005 이후 이미 #999 효과 정합되었을 수 있음) |
| `src/document_core/queries/rendering.rs` | +4 | `is_hwp3_variant` 전달 |
| `src/parser/cfb_reader.rs` / `model/document.rs` / `parser/mod.rs` / `hwpx/mod.rs` / `cfb_writer/tests.rs` | variant 식별 인프라 | **이미 #1005 머지됨 — 재활용** |

### 1.2 PR #1009 의 핵심 휴리스틱 (engine.rs:155~245 추가)

```rust
// variant 가드 + body_height_hu_for_variant 계산 (engine.rs:43~)
let is_hwp3_variant = opts.is_hwp3_variant;
let body_height_hu_for_variant = if is_hwp3_variant {
    page_def.height - margin_top - margin_bottom - margin_header - margin_footer
} else { 0 };

// engine.rs:170~ — cross-paragraph vpos reset 감지
if is_hwp3_variant && body_height_hu_for_variant > 0 {
    let is_synth = |ls| ls.tag & 0x80000000 != 0;  // synth LineSeg 제외
    let prev_real = ... (이전 real paragraph 의 마지막 LineSeg);
    let curr_real = ... (현 paragraph 의 첫 real LineSeg);

    let prev_end_vpos = prev_real.vertical_pos + prev_real.line_height;
    let curr_first_vpos = curr_real.vertical_pos;
    let high_threshold = body_height_hu × 85 / 100;
    let low_threshold = if curr empty line_segs but non-empty text { 4000 } else { 1500 };

    let main_trigger = prev_end_vpos > high_threshold AND curr_first_vpos < low_threshold;
    let aux_trigger = !main AND ...
        (empty paragraph bridge ≥ 2 AND prev_end > body/2);

    if main_trigger || aux_trigger {
        variant_vpos_reset_break = true;
        force page break
    }
}
```

### 1.3 PR #1009 close 시 발견된 over-split 회귀

메인테이너 sweep:
- 변환본 9개 중 8개: no-effect (휴리스틱 미발동)
- **sample16-hwp5**: devel 64 → with PR 65 (**+1 over-split**)

**의미**: 휴리스틱이 sample16-hwp5 에서 의도된 미정합 페이지 정합 + **의도되지 않은 1 추가 split** 도 trigger 함. 추가 split 발생 paragraph 식별 필요.

---

## 2. Stage 진행 정밀화

### Stage 1 — 진단 + 정답 기준 + over-split case 분석

**Step 1.1 — 한컴 정답 기준 단언** (작업지시자 시각 검증):
- 미정합 페이지 대표 (p3, p4, p5, p6, p9~p12, p29, p31) 의 한컴 한글 viewer 첫 paragraph 단언
- HWP3 vs HWP5 변환본 중 어느 쪽이 한컴 정답에 가까운지

**Step 1.2 — PR #1009 의 over-split case 식별**:
- 임시 cherry-pick `71054c51` (또는 hunk 적용)
- `dump-pages` 로 sample16-hwp5 65 페이지 결과 비교 (devel 64 vs PR 65)
- 추가 split 된 paragraph 식별 — 어떤 pi 에서 over-split trigger 되는지
- 그 paragraph 의 `prev_end_vpos`, `curr_first_vpos`, `high_threshold` 값 측정
- over-split 회피 가드 후보 도출 (예: 추가 조건 — paragraph 종류 제한, height 임계값 등)

**Step 1.3 — 변환본 fixture sweep** — vpos reset 신호 분포:
- sample4/5/10/11/13/14/16/19-hwp5 의 vpos reset paragraph 분포 측정
- 변환본 간 신호 패턴 단언

**Step 1.4 — CHARS_PER_LINE 재평가**:
- PR #1009 의 45→50 변경이 현 devel (a52859de, #1005 머지 후) 에서 필요한지 재평가
- #999/#1005 머지로 이미 정합되었을 수 있음

**산출물**: `mydocs/working/task_m100_1035_stage1.md`

**커밋**: "Task #1035 Stage 1: 진단 + 한컴 정답 + PR #1009 over-split case 분석"

### Stage 2 — Fix 적용 (옵션 A — narrow 가드)

**Step 2.1 — PR #1009 의 engine.rs + typeset.rs 변경 적용 (base)**:
- `71054c51` 의 engine.rs / typeset.rs hunk 적용
- variant 식별 인프라 (cfb_reader.rs / parser/mod.rs / hwpx/mod.rs 등) 는 이미 #1005 머지 — 재활용

**Step 2.2 — over-split 회피 narrow 가드 추가 (Stage 1.2 결과 기반)**:
- 추가 조건 예시 (실제 가드는 Stage 1 결과 따라 결정):
  - `next_paragraph_has_real_text` (다음 paragraph 가 실제 텍스트 있는 경우만 trigger)
  - `prev_paragraph_height > min_height` (이전 paragraph 가 정상 paragraph 인 경우만)
  - `over_split_safeguard`: 누적 split 페이지 수 추적, sample16-hwp5 의 64 한계 단언

**Step 2.3 — composer.rs CHARS_PER_LINE 적용 (Stage 1.4 결과 따라)**:
- 필요 시 45→50
- Stage 1.4 가 #1005 이후 이미 정합 단언 시 skip

**Step 2.4 — 빌드 + 단위 테스트**:
- `cargo build --release`
- `cargo test --release --lib`

**산출물**: `mydocs/working/task_m100_1035_stage2.md`

**커밋**: "Task #1035 Stage 2: PR #1009 휴리스틱 + over-split 회피 narrow 가드"

### Stage 3 — 검증 + 회귀 sweep

**Step 3.1 — alignment 정합률 측정**:
- HWP3 vs HWP5 변환본 64 페이지 첫 paragraph 일치 카운트
- 37.5% → 목표 ≥80%

**Step 3.2 — over-split 회귀 회피 단언** (PR #1009 close 사유):
- **sample16-hwp5 페이지 수 정확히 64** (65 인 경우 회귀 — Stage 2 narrow 가드 재조정)
- sample16 페이지 수 64 (HWP3 native 무영향)

**Step 3.3 — 변환본 + 일반 fixture 회귀 sweep**:
- HWP3 11종 + HWP5/HWPX 변환본 + exam_*/aift/biz_plan
- 페이지 수 회귀 0

**Step 3.4 — 전체 테스트**:
- `cargo test --release --lib`: 1307+ passed
- `cargo test --release --tests`: FAILED 0
- `cargo clippy --release --lib -- -D warnings`: clean
- **`cargo fmt --all -- --check`**: clean (feedback_cargo_fmt_all_required 필수)

**산출물**: `mydocs/working/task_m100_1035_stage3.md`

**커밋**: "Task #1035 Stage 3: 회귀 sweep + alignment 정합률 측정"

### Stage 4 — 최종 보고 + PR

**Step 4.1**: 작업지시자 한컴 viewer 시각 검증 (sample16 + sample16-hwp5 페이지 alignment 비교)
**Step 4.2**: `mydocs/report/task_m100_1035_report.md` 작성
**Step 4.3**: `mydocs/orders/20260520.md` 또는 새 날짜 갱신
**Step 4.4**: WASM 빌드 + rhwp-studio 동기화
**Step 4.5**: PR 생성 (작업지시자 승인 후)

**커밋**: "Task #1035 Stage 4: 최종 보고 + orders 갱신"

---

## 3. 변경 위치 summary (Stage 1 결과 후 확정)

| 항목 | 파일 / 라인 |
|------|------------|
| variant vpos reset 감지 | `src/renderer/pagination/engine.rs:43~245` (PR #1009 + narrow 가드) |
| 동일 로직 두 경로 정합 | `src/renderer/typeset.rs:?~?` (PR #1009 +136) |
| CHARS_PER_LINE (필요 시) | `src/renderer/composer.rs` |
| 단위 테스트 신규 | `tests/issue_1035_alignment.rs` (sample16-hwp5 alignment ≥80% + 페이지 수 64 단언) |

---

## 4. 위험 + 완화 (구체화)

| 위험 | Stage / 완화 |
|------|-------------|
| PR #1009 over-split 회귀 (sample16-hwp5 64→65) 재발 | Stage 3 step 3.2 단언 (페이지 수 64 + narrow 가드 재조정) |
| 변환본 한정 fix 가 다른 변환본에 미적용 또는 회귀 | Stage 1 step 1.3 vpos reset 신호 분포 sweep + Stage 3 step 3.3 변환본 9 종 sweep |
| narrow 가드의 임계값 조정으로 정합률 향상 부족 (≥80% 미달) | Stage 1 결과 따라 가드 재설계 — 분석 깊이 증가 |
| `cargo fmt --all` 미적용으로 CI failure | Stage 3 step 3.4 에서 `cargo fmt --all -- --check` 필수 — feedback_cargo_fmt_all_required 정합 |

---

## 5. 비대상

- 총 페이지 수 (64) — 본 task 아님
- WMF/폰트 fallback / Picture rendering — 다른 영역
- Task #1010 의 micro-overflow (별도 issue 예정)
- WASM 빌드 (Stage 4 최종 1회)

---

## 6. 검증 명령 모음

```bash
# Stage 1
./target/release/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 3 | head
./target/release/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 90 | head -10

# alignment count
for p in $(seq 0 63); do
  H3=$(./target/release/rhwp dump-pages samples/hwp3-sample16.hwp -p $p 2>/dev/null | grep "FullParagraph" | head -1 | grep -oE 'pi=[0-9]+' | head -1)
  H5=$(./target/release/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p $p 2>/dev/null | grep "FullParagraph" | head -1 | grep -oE 'pi=[0-9]+' | head -1)
  [ "$H3" = "$H5" ] && echo "p$p ✓"
done | wc -l   # 정합 페이지 수

# Stage 3 over-split 회귀 단언
PAGES_HWP5=$(./target/release/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp 2>/dev/null | grep -c "^=== 페이지")
[ "$PAGES_HWP5" = "64" ] && echo "✓ 64 유지" || echo "✗ over-split 회귀: $PAGES_HWP5"

# Stage 3 변환본 sweep
for f in samples/hwp3-sample{4,5,10,11,13,14,16,19}-hwp5.hwp samples/hwp3-sample16-hwp5.hwpx; do
  P=$(./target/release/rhwp dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  printf "  %-50s %s\n" "$(basename $f)" "$P"
done

# Stage 3 전체 테스트
cargo test --release --lib
cargo test --release --tests
cargo clippy --release --lib -- -D warnings
cargo fmt --all -- --check    # ← --all 필수!
```

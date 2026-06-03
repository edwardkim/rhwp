# Task #296 구현계획서 — WASM Canvas inline_tabs 존중

## 사전 확인 (Stage 1 완료)

**실측 데이터** (`RHWP_TAB296=1` 네이티브 로그, 문단 0.144 "18.\t\t\t수열..."):

| i | ext[0] (HU) | ext[2] | tab_width_px | high byte |
|---|-------------|--------|--------------|-----------|
| 3 | 132 | 256 (0x0100) | 1.76 | **1 = LEFT** |
| 4 | 671 | 256 (0x0100) | 8.95 | 1 = LEFT |
| 5 | 79 | 256 (0x0100) | 1.05 | 1 = LEFT |

- 네이티브: 3개 탭 합계 ≈ 12px → "수열"이 x≈38 (PDF 일치)
- WASM: inline_tabs 분기 부재 → `auto_tab_right` 폴스루로 420px 이동 → "수열" x≈290.91 (밀림)

**포맷 확정**: `tab_type = (ext[2] >> 8) & 0xFF`. 0/1 = LEFT, 2 = RIGHT, 3 = CENTER, 4 = DECIMAL.

## 구현 범위

### (1) 헬퍼 신규 추가

`src/renderer/layout/text_measurement.rs` 상단 (파일 모듈 수준, pub 불필요):

```rust
/// inline_tabs ext[2] 에서 탭 종류를 추출.
/// HWP tab_extended 포맷: ext[2] 는 high/low byte 합성.
///   high byte = 탭 종류 enum+1 (1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL)
///   low  byte = fill_type
/// PR #292 (Task #290) 에서 실증된 포맷.
#[inline]
fn inline_tab_type(ext: &[u16; 7]) -> u8 {
    ((ext[2] >> 8) & 0xFF) as u8
}
```

### (2) 두 측정기의 4 함수에서 일관 적용

| 측정기 | 함수 | 위치 | 현재 | 수정 |
|--------|------|------|------|------|
| `EmbeddedTextMeasurer` | `estimate_text_width` | L218 부근 | `let tab_type = ext[2];` | `let tab_type = inline_tab_type(ext);` |
| `EmbeddedTextMeasurer` | `compute_char_positions` | L~320 | 동일 버그 | 동일 수정 |
| `WasmTextMeasurer` | `estimate_text_width` | L~530 | inline_tabs 분기 **부재** | `EmbeddedTextMeasurer` 와 동등한 분기 블록 신규 추가 |
| `WasmTextMeasurer` | `compute_char_positions` | L~600 | 동일 부재 | 동등 분기 신규 추가 |

### (3) WASM 측정기 inline_tabs 분기 복사

`EmbeddedTextMeasurer` 의 블록 (L213~234) 을 `WasmTextMeasurer` 의 `\t` 처리부에 이식. 차이점:
- `tab_char_idx` 초기화 필요 (현재 WASM 측정기에는 이 변수 없음)
- `char_width` 는 WASM 내부의 `wasm_internals::measure_char_width_hwp` 사용 (구조는 그대로)

### (4) 단위 테스트

`src/renderer/layout/tests.rs` 하단:

```rust
#[test]
fn task296_inline_tab_type_left() {
    let ext = [132u16, 0, 256, 0, 0, 0, 9]; // 0x0100 → high=1 LEFT
    assert_eq!(super::text_measurement::inline_tab_type(&ext), 1);
}

#[test]
fn task296_inline_tab_type_right() {
    let ext = [100u16, 0, 0x0203, 0, 0, 0, 9]; // 0x0203 → high=2 RIGHT, low=3 fill
    assert_eq!(super::text_measurement::inline_tab_type(&ext), 2);
}

#[test]
fn task296_inline_tab_type_center() {
    let ext = [100u16, 0, 0x0300, 0, 0, 0, 9]; // 0x0300 → high=3 CENTER
    assert_eq!(super::text_measurement::inline_tab_type(&ext), 3);
}

#[test]
fn task296_inline_tab_type_decimal() {
    let ext = [100u16, 0, 0x0400, 0, 0, 0, 9]; // 0x0400 → high=4 DECIMAL
    assert_eq!(super::text_measurement::inline_tab_type(&ext), 4);
}
```

`inline_tab_type` 가시성: `pub(super) fn` 로 변경 (tests 에서 접근 가능해야 함).

## 단계 구성

### Stage 2 (구현)

1. **2-1** `inline_tab_type` 헬퍼 추가
2. **2-2** `EmbeddedTextMeasurer::estimate_text_width` 수정 (1줄: `ext[2]` → `inline_tab_type(ext)`)
3. **2-3** `EmbeddedTextMeasurer::compute_char_positions` 수정 (1줄)
4. **2-4** `WasmTextMeasurer::estimate_text_width` 에 inline_tabs 분기 신규 추가 (약 +25줄)
5. **2-5** `WasmTextMeasurer::compute_char_positions` 동일 (+25줄)
6. **2-6** 단위 테스트 4건 추가
7. **2-7** `cargo test --lib task296` + `cargo clippy` + `cargo check --target wasm32` 로컬 통과

→ Stage 2 보고서 (`task_m100_296_stage2.md`)

### Stage 3 (검증)

1. **3-1** 전체 회귀: `cargo test --lib` (988+4 기대), `svg_snapshot`, `tab_cross_run`
2. **3-2** 네이티브 SVG byte-diff (`git worktree baseline` vs 현재):
   - `exam_math.hwp` 20페이지 / `biz_plan.hwp` / `exam_eng.hwp` / `exam_kor.hwp` / `hwp-3.0-HWPML.hwp` 184페이지 비교
   - 변경 허용 범위: `exam_math_007.svg` 의 inline LEFT 탭 영향으로 미세한 x 좌표 변경 (PDF와 더 가까워져야 함) — 사유 검증
   - **RIGHT inline 탭 샘플**: `hwp-3.0-HWPML.hwp` `저작권\t1` 에서 이번 수정으로 RIGHT 탭 실제 작동하여 x 변경 가능성 → 작업지시자 확인 필요
3. **3-3** WASM Docker 빌드 (`docker compose run --rm wasm`)
4. **3-4** **브라우저 시각 검증** (작업지시자): `samples/exam_math.hwp` p.7 #18 "수열"이 x≈109.80 좌측 정렬 확인
5. **3-5** **진단 로그 제거** (Stage 2 에서 유지했던 `TAB296` 로그)

→ Stage 3 보고서 (`task_m100_296_stage3.md`)

### Stage 4 (마무리)

1. 최종 보고서 `mydocs/report/task_m100_296_report.md`
2. orders 갱신 (`mydocs/orders/20260424.md`)
3. 트러블슈팅 `tab_tac_overlap_142_159.md` 에 "#296 Canvas 경로 inline_tabs 수정" 섹션 추가
4. 이슈 #296 close 준비 (`closes #296` 커밋 메시지)

→ Stage 4 보고서 (`task_m100_296_stage4.md`)

## 변경 파일 최종 목록

| 파일 | 변경 | 예상 라인 |
|------|------|-----------|
| `src/renderer/layout/text_measurement.rs` | 헬퍼 추가 + 4 함수 수정 + Stage 3 에서 진단 로그 제거 | +50 -2 (Stage 3 후) |
| `src/renderer/layout/tests.rs` | 단위 테스트 4건 | +30 |
| `mydocs/plans/task_m100_296.md` | 수행계획서 | 이미 작성 |
| `mydocs/plans/task_m100_296_impl.md` | 구현계획서 | 이 문서 |
| `mydocs/working/task_m100_296_stage{1,2,3,4}.md` | 단계 보고서 4건 | 각 ~80 |
| `mydocs/report/task_m100_296_report.md` | 최종 보고서 | ~140 |
| `mydocs/troubleshootings/tab_tac_overlap_142_159.md` | #296 섹션 추가 | +60 |
| `mydocs/orders/20260424.md` | 종료 섹션 추가 | +10 |

## 리스크 완화 (수행계획서 R1~R3 재검토)

### R1 — 네이티브 SVG 회귀

**위험**: 네이티브 `EmbeddedTextMeasurer` 의 `tab_type = ext[2]` 수정으로 기존 inline LEFT 탭 우연 동작이 깨질 가능성.

**완화**:
- inline LEFT 탭 (0/1): 기존에도 `_ =>` LEFT 분기로 처리되고 있었으므로, 수정 후 `match 0 => LEFT | 1 => LEFT` 가 되어도 **동일 동작** (둘 다 `total = tab_target.max(total)`).
- inline RIGHT (2) / CENTER (3): 기존엔 도달 불가 → 수정 후 실제 RIGHT/CENTER 처리 활성화. **이 경우 기존 SVG 와 결과가 달라질 수 있음**.
- **대응**: Stage 3-2 의 184페이지 byte-diff 에서 RIGHT inline 탭 샘플(`hwp-3.0-HWPML.hwp`) 변경 사항 검증 후 작업지시자 판단 요청.

### R2 — `tab_char_idx` 동기화

**위험**: WASM 측정기의 4 탭 분기(inline/custom/default) 모두에서 `tab_char_idx` 증가 필요. 누락 시 다음 탭 인덱스가 밀림.

**완화**: `EmbeddedTextMeasurer` 의 구조 그대로 복사. 3 분기 모두 `tab_char_idx += 1` 적용.

### R3 — match arm 순서와 기본값

기존 `tab_type = ext[2]` 전체 u16 일 때:
- LEFT (high=1) 은 `256`, `257`, ... 등 256 이상이라 `1`, `2` 와 match 불일치 → `_ =>` → LEFT 처리 (우연 일치)

수정 후 `tab_type = (ext[2] >> 8) & 0xFF` 이면:
- `0` (예외적, 저위 byte만 사용된 경우) → `_ =>` LEFT (유지)
- `1` → `_ =>` LEFT — **match arm 에 `1` 분기 없음이 의도적인가?**

기존 코드 `match tab_type { 1 => RIGHT, 2 => CENTER, _ => LEFT }` 의 의미 재해석:
- 예전 가정: `ext[2]` 가 0=LEFT, 1=RIGHT, 2=CENTER 직접 인코딩
- 실제 (PR #292 실증): high byte = 1=LEFT, 2=RIGHT, 3=CENTER

→ **`match` 분기도 바꿔야 함**:
```rust
let tab_type = inline_tab_type(ext);
match tab_type {
    2 => { /* RIGHT */ }
    3 => { /* CENTER */ }
    _ => { /* LEFT (0, 1, 4=DECIMAL, 기타) */ }
}
```

이는 **단순 1줄 수정이 아니라 match arm 전체 재작성**. 구현계획서 확정 사항으로 반영.

## 승인 요청 사항

1. ✅ Stage 1 실측 결과로 접근 방식 (옵션 A) 타당성 확정
2. **구현계획서의 핵심 변경**:
   - 헬퍼 `inline_tab_type` 추가
   - 4 함수의 **match arm 전체 재작성** (`1 → RIGHT` 가 아닌 `2 → RIGHT, 3 → CENTER`)
   - RIGHT inline 탭 샘플 (`hwp-3.0-HWPML.hwp`) 의 SVG 변경 가능성 인지
3. **진단 로그 처리**: Stage 2~3 는 로그 유지 → Stage 3-5 에서 일괄 제거

## 성공 기준

1. `cargo test --lib` 전체 통과 (988 + 4 신규 = 992+)
2. `cargo clippy` + `cargo check --target wasm32` clean
3. 184페이지 SVG byte-diff 결과를 작업지시자와 공유하고 허용 범위 판정
4. 브라우저에서 `exam_math.hwp` p.7 #18 좌측 정렬 확인 (작업지시자)

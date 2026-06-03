# Task #290 최종 보고서 — cross-run 탭 감지가 inline_tabs 무시

## 1. 배경

작업지시자 제보 (2026-04-24): `samples/exam_math.hwp` 페이지 7 의 **18번 "수열" 문항 첫 줄**이 한컴 PDF 대비 우측 끝으로 밀려 렌더됨.

| | 첫 줄 표시 |
|---|---|
| **PDF (정답)** | `18. 수열 {a_n}이 모든 자연수 n에 대하여` |
| **SVG (버그)** | `18.` ·························· `수열 {a_n}이 모든 자연수 n에 대하여` |

동일 문서의 다른 문항 (19, 20번) 과 본 문항의 2·3째 줄은 정상. 18번 문항 첫 줄에만 국한.

## 2. 원인 분석

### 2.1 IR 관찰 (paragraph 0.144)

```
text     : "18.\t\t\t수열 이 모든 자연수 에 대하여"
tab_def  : [12.0mm(L), 13.3mm(L), 18.0mm(L), 18.6mm(L)]  auto_tab_right=true
inline   : [132,_,256,...] [671,_,256,...] [79,_,256,...]  # ext[2]=0x0100 → LEFT
```

### 2.2 트레이스로 확정한 메커니즘

`paragraph_layout.rs` 의 **cross-run 우측/가운데 탭 감지 블록** (est 측 `:854-868`, render 측 `:1213-1226`) 이 마지막 `\t` 의 종류를 판정할 때 `find_next_tab_stop` 만 호출 (TabDef 전용 경로). 본문 `tab_extended` (inline_tabs) 는 참조하지 않음.

1. Run `"18.\t\t\t"` 의 3 개 \t 는 inline 경로에서 LEFT 로 올바르게 진행 (x=38.24)
2. 그러나 cross-run 감지 블록이 `find_next_tab_stop(abs_before=37.19, ...)` 호출
3. TabDef stops 모두 abs_before 보다 작음 → `auto_tab_right=true` 폴스루 → **type=1 (RIGHT) 반환**
4. `pending_right_tab_render = Some((420.11, 1))`
5. 다음 run `"수열 이 모든 자연수 "` 배치 시 `x = col_area.x + 420.11 - next_w(201.00) = 290.91` 로 역산 → **우측 끝 근처 배치**

### 2.3 ext[2] 포맷 실증 (Stage 1)

RIGHT/CENTER 샘플 탐색으로 `samples/hwp-3.0-HWPML.hwp` 문단 0.39 `"저작권\t1"` (TabDef type=1 RIGHT, fill=3) 확보 → 트레이스:

| 케이스 | ext[2] | 16진 | high | low |
|--------|--------|------|------|-----|
| exam_math 18번 LEFT × 3 | 256 | `0x0100` | 1 | 0 |
| hwp-3.0-HWPML 저작권 RIGHT | 515 | `0x0203` | 2 | 3 |

→ **ext[2] 는 high byte (탭 종류 enum+1) + low byte (fill) 합성 값**. 매핑: 1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL.

## 3. 수정 내용

### 3.1 신규 헬퍼 `resolve_last_tab_pending` (`paragraph_layout.rs`)

```rust
pub(crate) fn resolve_last_tab_pending(
    run_text: &str,
    last_inline_idx: usize,
    tab_extended: &[[u16; 7]],
    text_style: &TextStyle,
    tab_stops: &[TabStop],
    tab_width: f64,
    auto_tab_right: bool,
    available_width: f64,
) -> Option<(f64, u8)> {
    // 1) inline_tabs 가 마지막 \t 를 커버: ext[2] 고바이트로 종류 판정
    if last_inline_idx < tab_extended.len() {
        let inline_type = ((tab_extended[last_inline_idx][2] >> 8) & 0xFF) as u8;
        match inline_type {
            0 | 1 => return None,  // LEFT → pending 없음 (본 수정의 핵심)
            2 | 3 => {}            // RIGHT/CENTER → TabDef 경로 폴스루
            _ => return None,      // 미지 값 (4=DECIMAL 등) → 보수적 LEFT
        }
    }
    // 2) inline 이 LEFT 아님 or inline 없음 → 기존 find_next_tab_stop 경로
    /* ... find_next_tab_stop 호출 ... */
}
```

### 3.2 cross-run 블록 2 곳 교체 + `inline_tab_cursor_*` 도입

- est 측 루프 (`:840` 부근) — `inline_tab_cursor_est: usize = 0` 도입, 기존 블록을 헬퍼 호출로 교체, 루프 말미 + char_overlap `continue` 직전에 cursor 증가
- render 측 루프 (`:1198` 부근) — `inline_tab_cursor_render: usize = 0` 도입, 동일 교체

`composed.tab_extended` 는 parser 에서 `0x0009` (TAB) 마다 1 개씩 push 되므로 `run.text.chars().filter(|c| *c == '\t').count()` 로 cursor 증가가 정확히 일치.

### 3.3 테스트 신규

- **단위 테스트 5 건** (`src/renderer/layout/tests.rs`): LEFT→None / RIGHT→Some / CENTER→Some / inline 없음 폴백 2 건
- **통합 테스트 1 건** (`tests/tab_cross_run.rs`): exam_math.hwp p.7 렌더 후 item 18 "수" glyph `x < 200` 검증

## 4. 검증 결과

### 4.1 테스트

| 항목 | 결과 |
|------|------|
| `cargo test --lib task290` (신규 5) | **5/5 pass** |
| `cargo test --test tab_cross_run` (신규 1) | **1/1 pass** |
| `cargo test --test svg_snapshot` | **3/3 pass** |
| `cargo test --lib` 전체 | 955 pass (선존재 14 fail 은 cfb_writer, 무관) |
| `cargo clippy --lib -- -D warnings` | clean |

### 4.2 회귀 (git worktree 로 Stage 1 baseline 비교)

| 문서 | 변경 / 전체 | 비고 |
|------|------------|------|
| exam_math.hwp | **1 / 20** | p.7 만 (item 18 의도된 수정) |
| biz_plan.hwp | 0 / 6 | |
| exam_eng.hwp | 0 / 11 | |
| exam_kor.hwp | 0 / 25 | |
| hwp-3.0-HWPML.hwp | 0 / 122 | **RIGHT inline tab (저작권\t1) 회귀 없음** |
| **합계** | **1 / 184** | 의도 100%, 의도 외 변화 0 |

### 4.3 수치 실증 (exam_math p.7 item 18 첫 줄)

| 글리프 | before x | after x | 변화 |
|--------|----------|---------|------|
| 수 | 290.91 | 109.80 | **-181.11** |
| 열 | 304.86 | 123.75 | -181.11 |
| ... (14 글자 + 수식 래퍼 모두) | ... | ... | **일관 -181.11** |

`next_w(201.00)` 와 `available_width(420.11)` 의 차이를 일관되게 반영 → cross-run 역산 배치 제거가 정확히 의도대로 작동.

### 4.4 시각 비교 (3 면 PNG, `mydocs/working/task_m100_290_stage3/`)

- `p7_before.png` — 수정 전 (item 18 우측 정렬 버그)
- `p7_after.png` — 수정 후 (PDF 와 동일한 좌측 정렬)
- `p7_pdf.png` — 한컴 PDF 레퍼런스

AFTER ≈ PDF 시각 일치.

## 5. 범위 외로 분리된 후속 과제

### 5.1 inline_tabs RIGHT/CENTER 렌더 경로 (text_measurement.rs)

`ext[2]` 를 전체 u16 으로 해석하는 현재 코드 (`text_measurement.rs:217, 320`) 는 실제 HWP 파일의 합성 값 (최소 256) 과 영원히 매칭되지 않아 **inline RIGHT/CENTER 경로는 도달 불가** — 모든 inline 탭이 LEFT 로 렌더됨.

저작권\t1 케이스가 "그럭저럭 보이는" 이유: RIGHT 탭이 LEFT 처리되어 \t 가 ext[0] 폭(39076 HU = 521 px) 만큼 전진한 자리에 "1" 이 배치 → 우연히 우측 정렬과 유사.

이 범위는 본 타스크 외로 분리 — 후속 이슈 제안:
> inline_tabs (HWP tab_extended) RIGHT/CENTER 렌더가 LEFT 로 폴백되는 문제. ext[2] 를 high/low byte 로 분리 해석 필요 (high=탭 종류, low=fill).

### 5.2 TabDef `/2.0` 스케일

`style_resolver.rs:640` 의 `/2.0` 처리는 #142, #159 에서 "한컴 격자 비교로 확정" 이나 일부 케이스에서 inline_tabs 없이 auto_tab_right 만 있을 때 의도치 않은 영향 가능성 존재. 본 타스크에서는 변경하지 않았음.

## 6. 변경 통계

```
 src/renderer/layout/paragraph_layout.rs | 110 ++++++++++++++++++++++++-------
 src/renderer/layout/tests.rs            |  83 ++++++++++++++++++++++++
 tests/tab_cross_run.rs                  |  52 ++++++++++++++
 3 files changed, 221 insertions(+), 24 deletions(-)
```

문서 (계획서·단계별 보고서·트러블슈팅 등) 는 별도.

## 7. 교훈

1. **"동일 의미의 데이터가 두 경로로 계산될 때 반드시 동기화"** — #142 에서 얻은 교훈이 본 타스크에서도 재확인됨. 이번엔 `estimate_text_width`/`compute_char_positions` 가 아니라 **런 내부 탭 처리 vs cross-run 탭 감지** 가 불일치. 런 내부는 inline_tabs 를 봤지만 cross-run 감지는 안 봤음.
2. **트레이스 기반 원인 확정의 위력** — 임시 `RHWP_TRACE290` 로 양쪽 호출의 입력/출력을 한 번에 관측 → 추측 없이 `pending_right_tab = Some((420.11, 1))` 설정 → `x_after=290.91` (SVG 실제 위치) 로 이어지는 전 경로를 숫자로 연결.
3. **git worktree 활용 baseline diff** — fix 전 커밋의 독립 빌드로 184 페이지 byte-level 회귀를 자동 검증. "변경 페이지 1 / 전체 184" 같은 객관적 지표 제공.
4. **범위 의식적 제어** — inline_tabs RIGHT/CENTER 렌더 버그를 발견했지만 본 타스크 범위에 포함하지 않고 후속 이슈로 분리. 수정 범위가 커지면 회귀 위험도 증가.
5. **작업지시자의 시각 문제 제보 → 데이터 실증 → 1 줄 단위의 정밀 수정** 패턴이 효과적. 한 문장 ("7 page 18. 수열 부분") 에서 출발하여 정확히 해당 문단의 cross-run 탭 감지 로직을 지목한 버그 수정.

## 8. 산출물

### 코드
- `src/renderer/layout/paragraph_layout.rs` — 헬퍼 + 2 곳 교체 + cursor 2 개
- `src/renderer/layout/tests.rs` — 단위 테스트 5 건
- `tests/tab_cross_run.rs` — 통합 테스트 1 건

### 문서
- `mydocs/plans/task_m100_290.md` — 수행계획서
- `mydocs/plans/task_m100_290_impl.md` — 구현계획서
- `mydocs/working/task_m100_290_stage{1,2,3,4}.md` — 단계별 완료 보고서
- `mydocs/working/task_m100_290_stage3/p7_{before,after,pdf}.png` — 시각 비교 3 면
- `mydocs/report/task_m100_290_report.md` — 본 문서
- `mydocs/troubleshootings/tab_tac_overlap_142_159.md` — #290 섹션 추가
- `mydocs/orders/20260424.md` — 종료 항목 이동

### GitHub
- 이슈 [#290](https://github.com/edwardkim/rhwp/issues/290) close

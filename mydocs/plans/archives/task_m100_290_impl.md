---
타스크: #290 cross-run 탭 감지가 inline_tabs 무시
구현계획서
브랜치: local/task290
작성일: 2026-04-24
---

# 구현계획서

## 1. 사전 조사 결과 (수행계획서 이후 추가)

### 1.1 est_x / render 양측 모두 동일 버그

`paragraph_layout.rs` 에 cross-run 우측/가운데 탭 감지가 **두 곳**에 존재. 두 곳 모두 `find_next_tab_stop` 만 사용하고 `composed.tab_extended` 를 참조하지 않음:

| 지점 | 라인 | 용도 | pending 변수 |
|------|------|------|-------------|
| est 측 | `paragraph_layout.rs:854-868` | 줄 전체 폭 선산출 (정렬 계산용) | `pending_right_tab_est` |
| render 측 | `paragraph_layout.rs:1213-1226` | 실제 런 배치 위치 결정 | `pending_right_tab_render` |

두 곳이 불일치하면 정렬 계산과 실제 배치가 어긋나므로 **반드시 동일 로직**으로 수정.

### 1.2 inline_tab 인덱스 추적 필요

`composed.tab_extended` 는 문단 전체의 `\t` 에 대한 평면 리스트. cross-run 감지는 "이 run 의 마지막 `\t` 가 tab_extended 의 어느 인덱스인가" 를 알아야 함. 현재 paragraph_layout 에 inline_tab 인덱스 추적 변수 **없음** → 도입 필요.

- est 측: `comp_line.runs` 순회하며 카운트 누적 (`inline_tab_cursor_est: usize`)
- render 측: `comp_line.runs` 순회하며 카운트 누적 (`inline_tab_cursor_render: usize`)

run 시작 시점의 cursor + `run.text` 내 `\t` 카운트 - 1 = 해당 run 의 마지막 `\t` 의 tab_extended 인덱스.

### 1.3 tab type 매핑 (단계 1 에서 RIGHT 샘플로 검증 예정)

현재 트레이스에서 확인된 값: 모든 `\t` 에 대해 `ext[2] = 256` (=LEFT). `text_measurement.rs:320` 의 `match tab_type` 은 `1 → RIGHT`, `2 → CENTER`, 그 외 (0, 256, 기타) → LEFT 로 동작하므로:

**매핑 규칙 (보수적):**
- `ext[2] == 1` → RIGHT 탭
- `ext[2] == 2` → CENTER 탭
- 그 외 (0, 256, 255 등) → LEFT 탭 → `pending_right_tab_*` 설정하지 않음

단계 1 에서 RIGHT 탭 포함 샘플을 찾으면 해당 ext 값으로 매핑 재검증. 못 찾으면 위 보수적 매핑 유지.

### 1.4 위치 계산 옵션 결정

RIGHT/CENTER 탭일 때 `pending_right_tab_*` 에 넣을 `tab_pos` 는:

- **옵션 A**: 현재처럼 `find_next_tab_stop` 결과 사용 (TabDef + auto_tab_right 로 결정)
- **옵션 B**: `abs_before + ext[0].to_px()` 로 inline 폭 누적

현재 케이스(LEFT 만 존재)에서는 선택이 무관. RIGHT/CENTER 샘플 확보 시 한컴 실제 동작과 일치하는 옵션으로 결정. 없으면 **옵션 A 유지** (TabDef 기반 기존 동작과 일관성 보장).

## 2. 파일 단위 변경

### 2.1 `src/renderer/layout/paragraph_layout.rs` — cross-run 감지 보정 (2 곳)

#### 변경 1: est 측 (`paragraph_layout.rs:786-868` 루프 내)

```rust
// [추가] est 측 inline_tab 커서
let mut inline_tab_cursor_est: usize = 0;
for run in &comp_line.runs {
    // ... (기존 코드) ...

    // run 이 \t 로 끝나면 다음 run 에 우측/가운데 탭 조정 필요
    if run.text.ends_with('\t') {
        let tab_count_in_run = run.text.chars().filter(|c| *c == '\t').count();
        let last_inline_idx = inline_tab_cursor_est + tab_count_in_run - 1;

        // [변경] inline_tabs 우선, 없으면 TabDef 폴백
        let (tp_opt, tt_opt) = resolve_last_tab_type(
            &run.text,
            last_inline_idx,
            &composed.tab_extended,
            &ts,
            &tab_stops,
            tab_width,
            auto_tab_right,
            available_width,
        );
        if let (Some(tp), Some(tt)) = (tp_opt, tt_opt) {
            if tt == 1 || tt == 2 {
                pending_right_tab_est = Some((tp, tt));
            }
        }
    }
    // [추가] run 처리 후 커서 진행
    inline_tab_cursor_est += run.text.chars().filter(|c| *c == '\t').count();
    // ... (기존 run_char_pos_est 업데이트 등) ...
}
```

#### 변경 2: render 측 (`paragraph_layout.rs:1213-1226` 블록)

```rust
// [추가] render 측 inline_tab 커서 (runs 순회 시작 전 선언, run 후 증가)
let mut inline_tab_cursor_render: usize = 0;
// ... (run 순회 내부) ...

if has_tabs && run.text.ends_with('\t') {
    let tab_count_in_run = run.text.chars().filter(|c| *c == '\t').count();
    let last_inline_idx = inline_tab_cursor_render + tab_count_in_run - 1;

    let (tp_opt, tt_opt) = resolve_last_tab_type(
        &run.text,
        last_inline_idx,
        &composed.tab_extended,
        &text_style,
        &tab_stops,
        tab_width,
        auto_tab_right,
        available_width,
    );
    if let (Some(tp), Some(tt)) = (tp_opt, tt_opt) {
        if tt == 1 || tt == 2 {
            pending_right_tab_render = Some((tp, tt));
        }
    }
}
// [추가] run 처리 후 커서 진행
inline_tab_cursor_render += run.text.chars().filter(|c| *c == '\t').count();
```

### 2.2 `src/renderer/layout/paragraph_layout.rs` — 공통 헬퍼 신규

```rust
/// 마지막 `\t` 의 탭 종류·위치 결정.
/// inline_tabs 가 해당 인덱스를 커버하면 그것을 우선 사용, 아니면 TabDef + auto_tab_right 폴백.
/// 반환: (tab_pos, tab_type).
/// tab_type 이 0(LEFT) 또는 3(소수점) 인 경우에도 값을 반환하지만, 호출측은 1/2 일 때만 pending 을 설정한다.
fn resolve_last_tab_type(
    run_text: &str,
    last_inline_idx: usize,
    tab_extended: &[[u16; 7]],
    text_style: &TextStyle,
    tab_stops: &[TabStop],
    tab_width: f64,
    auto_tab_right: bool,
    available_width: f64,
) -> (Option<f64>, Option<u8>) {
    // 1) inline_tabs 우선
    if last_inline_idx < tab_extended.len() {
        let ext = tab_extended[last_inline_idx];
        let tt_raw = ext[2];
        // HWP TAB ext[2] 값 매핑: 1=RIGHT, 2=CENTER, 그 외=LEFT 취급
        let tt = match tt_raw {
            1 => 1u8,
            2 => 2u8,
            _ => 0u8, // LEFT 포함 기타
        };
        if tt == 1 || tt == 2 {
            // 위치는 기존 TabDef + auto_tab_right 기반 (옵션 A)
            // TODO: RIGHT 샘플 확보 시 ext[0] 기반(옵션 B) 로 전환 검토
            if let Some(last_tab_byte) = run_text.rfind('\t') {
                let text_before = &run_text[..last_tab_byte];
                let w_before = estimate_text_width(text_before, text_style);
                let abs_before = text_style.line_x_offset + w_before;
                let tw = if tab_width > 0.0 { tab_width } else { 48.0 };
                let (tp, _, _) = find_next_tab_stop(
                    abs_before, tab_stops, tw, auto_tab_right, available_width,
                );
                return (Some(tp), Some(tt));
            }
        }
        // inline 이 LEFT 이면 pending 없음 — 이것이 본 타스크의 핵심 수정
        return (None, Some(0));
    }
    // 2) inline_tabs 없음 → 기존 TabDef 폴백
    if let Some(last_tab_byte) = run_text.rfind('\t') {
        let text_before = &run_text[..last_tab_byte];
        let w_before = estimate_text_width(text_before, text_style);
        let abs_before = text_style.line_x_offset + w_before;
        let tw = if tab_width > 0.0 { tab_width } else { 48.0 };
        let (tp, tt, _) = find_next_tab_stop(
            abs_before, tab_stops, tw, auto_tab_right, available_width,
        );
        return (Some(tp), Some(tt));
    }
    (None, None)
}
```

### 2.3 `tests/tab_cross_run.rs` — 신규 통합 테스트

`samples/exam_math.hwp` page 7 item 18 첫 줄 렌더 후 "수열" 글리프 위치를 검증.

```rust
#[test]
fn task290_exam_math_p7_item18_sukyul_position() {
    let bytes = std::fs::read("samples/exam_math.hwp").unwrap();
    let doc = rhwp::parse_document(&bytes).unwrap();
    let svg = rhwp::export_svg_page(&doc, 6, &Default::default()).unwrap();
    // "수" 글리프 x 좌표 추출: translate(X,Y)
    let re = regex::Regex::new(r#"translate\(([\d.]+),[\d.]+\).{0,300}>수<"#).unwrap();
    let captures: Vec<_> = re.captures_iter(&svg).collect();
    // 페이지 7 좌측 열 (71.8 ~ 491.8 px) 의 "수" 위치
    let left_col_x: Vec<f64> = captures
        .iter()
        .filter_map(|c| c[1].parse::<f64>().ok())
        .filter(|x| *x >= 71.8 && *x <= 491.8)
        .collect();
    // "수학 영역" 타이틀 "수" 는 페이지 중앙 부근 → 좌측 열에서 "18.수열" 의 "수" 가 맨 앞
    let first_sukyul_x = left_col_x.iter().find(|x| **x > 71.8 && **x < 300.0).copied();
    // 기대: "수" 는 col_area.x(71.8) + ~38 px ≈ 110 px 부근 (한계 < 200)
    let x = first_sukyul_x.expect("item 18 '수' glyph not found");
    assert!(x < 200.0, "item 18 '수' glyph at x={} should be near col start (~110), not right-aligned", x);
}
```

(정확한 API 는 단계 2 에서 확인 후 조정)

### 2.4 단위 테스트 — `resolve_last_tab_type` 경계

`paragraph_layout` 내 `#[cfg(test)] mod tests` 에 추가 (기존 위치 파악 후 결정):

- 케이스 1: `tab_extended[last_idx] = [132, 0, 256, ...]` → (None, Some(0)) (LEFT, pending 없음)
- 케이스 2: `tab_extended[last_idx] = [100, 0, 1, ...]` → (Some(_), Some(1)) (RIGHT)
- 케이스 3: `tab_extended[last_idx] = [100, 0, 2, ...]` → (Some(_), Some(2)) (CENTER)
- 케이스 4: `last_idx >= tab_extended.len()` + TabDef 없음 + auto_tab_right=false → (None, None)
- 케이스 5: `last_idx >= tab_extended.len()` + auto_tab_right=true + 모든 stop 초과 → (Some(avail), Some(1)) (기존 폴백 동작)

## 3. 단계 구성 (4 단계, 수행계획서와 동일)

### Stage 1 — 정밀 진단 + ext[2] 검증 + RIGHT 샘플 확보

**작업**:
- `samples/` 에서 RIGHT/CENTER 탭 포함 HWP 샘플 탐색
  - `rhwp dump -s N -p M` 로 `tab_def` 의 `type=1` 또는 `type=2` 확인
  - 후보: biz_plan.hwp, exam_eng.hwp, exam_kor.hwp 등 기존 샘플
- 찾은 샘플로 `RHWP_TRACE287=1` 트레이스 → inline_tabs 의 `ext[2]` 값 확인
  - `ext[2]` 가 1/2 를 직접 반환하는지, 아니면 다른 인코딩인지 확정
- inline_tab_cursor 추적 변수 도입 위치 확인 (기존 `run_char_pos` / `run_char_pos_est` 와 병행 관리)
- 옵션 A vs B 최종 결정 (한컴 비교 기반)
- [TRACE287] 임시 디버그는 **추가하지 않고** 기존 DEBUG_TAB_POS 만 사용

**산출**: `mydocs/working/task_m100_290_stage1.md` — 진단 결과 + RIGHT 샘플 케이스 + 최종 매핑 규칙 확정

**완료 조건**: ext[2] 매핑이 데이터로 확정 (RIGHT 샘플 찾으면 검증, 못 찾으면 보수적 매핑 유지 + 그 근거 명시)

### Stage 2 — 구현 + 단위 테스트

**작업**:
- `paragraph_layout.rs` 에 `resolve_last_tab_type` 헬퍼 추가
- est 측 + render 측 각각 `inline_tab_cursor` 도입 + 헬퍼 호출로 교체
- 기존 `if has_tabs && run.text.ends_with('\t')` 블록 두 개 삭제 → 헬퍼 호출
- 단위 테스트 5건 추가 (`resolve_last_tab_type` 의 경계 케이스)
- `cargo test --lib` 전체 회귀 0 확인
- `cargo clippy --lib --tests -- -D warnings` clean

**산출**: `mydocs/working/task_m100_290_stage2.md` — 변경 diff 요약 + 테스트 결과

**완료 조건**:
- 신규 테스트 5건 모두 pass
- 기존 `cargo test --lib` 회귀 0 (모든 테스트 통과)
- clippy clean

### Stage 3 — 시각 회귀 검증

**작업**:
- `samples/exam_math.hwp` page 7 SVG 재렌더 → "수" 위치가 col_area.x + ~38 px 로 이동했는지 확인 (PDF 와 일치)
- 통합 테스트 (`tests/tab_cross_run.rs` `task290_exam_math_p7_item18_sukyul_position`) 추가 + pass
- `exam_math.hwp` 20페이지 전체를 PNG 로 재렌더 → 기존 output 과 pixel diff
  - diff 가 p.7 item 18 첫 줄에만 국한되는지 확인
  - 다른 페이지 의도치 않은 변화 없어야 함
- `biz_plan.hwp`, `exam_eng.hwp`, `exam_kor.hwp` 전체 페이지 pixel diff (탭 사용 문서 회귀 방지)
- RIGHT 탭 포함 샘플 (Stage 1 에서 확보 시) 회귀 0 확인
- PDF 레퍼런스와 비교 PNG 생성 (before/after/pdf 3면)

**산출**: `mydocs/working/task_m100_290_stage3.md` — 비교 PNG + diff 통계 + RIGHT 탭 회귀 결과

**완료 조건**:
- exam_math.hwp p.7 item 18 "수" 위치 PDF 일치
- 회귀 샘플 모두 pixel diff 0 또는 허용 범위 내
- 신규 통합 테스트 pass

### Stage 4 — 문서 + 이슈 close

**작업**:
- 최종 결과 보고서 `mydocs/report/task_m100_290_report.md` 작성
- `mydocs/orders/20260424.md` 에 #290 완료 항목 갱신 (신규 등록 → 종료)
- 트러블슈팅 문서 `mydocs/troubleshootings/tab_tac_overlap_142_159.md` 에 #290 섹션 추가 (동일 파일군)
- 이슈 #290 close 코멘트 (merge 커밋 해시 + 재현 방법 + PR 링크)
- 타스크 브랜치 모든 커밋 확인 → local/devel merge 준비

**산출**:
- `mydocs/working/task_m100_290_stage4.md`
- `mydocs/report/task_m100_290_report.md`
- `mydocs/orders/20260424.md` 갱신
- `mydocs/troubleshootings/tab_tac_overlap_142_159.md` 갱신 또는 신규

**완료 조건**:
- 이슈 #290 close 완료
- local/task290 모든 단계 커밋 완료 + `git status` 클린
- 리포트 검토 승인

## 4. 검증 기준 (수행계획서 6 과 동일)

- [ ] `cargo test --lib` 그린 (기존 + 신규 5건)
- [ ] `cargo test --test svg_snapshot` 3 passed
- [ ] `cargo test --test tab_cross_run` 1 passed (신규)
- [ ] `cargo clippy --lib --bins --tests -- -D warnings` clean
- [ ] `samples/exam_math.hwp` p.7 item 18 "수" 위치 PDF 일치
- [ ] `exam_math.hwp` 20p + `biz_plan.hwp` + `exam_eng.hwp` + `exam_kor.hwp` 회귀 0
- [ ] Stage 1 에서 RIGHT 탭 샘플 확보 시 해당 샘플 회귀 0

## 5. 리스크 완화

| 리스크 | 완화 방안 |
|--------|----------|
| RIGHT 탭 샘플 미확보 | 보수적 매핑 (1/2/그 외) 유지 + Stage 4 보고서에 추가 샘플 확보 후속 과제 명시 |
| `inline_tab_cursor` 동기화 오류 | est_cursor 와 render_cursor 가 동일한 값으로 증가하는지 Stage 2 단위 테스트로 확인 |
| 헬퍼 `resolve_last_tab_type` 신규 함수의 회귀 | 기존 로직 100% 재현하는 케이스 (inline_tabs 비었을 때) 가 TabDef 경로와 일치하는지 단위 테스트로 확인 |
| 통합 테스트 API 미정 | Stage 2 구현 시 실제 API 확인 (`rhwp::export_svg_page` 등) 후 조정 |

# Task #287 최종 결과 보고서 — 빈 runs comp_line 의 TAC 수식 인라인 처리 추가

## 타스크

- **이슈**: [#287](https://github.com/edwardkim/rhwp/issues/287) — 수식 SVG 레이아웃: 큰 디스플레이 TAC 수식이 줄 상단(y=col_area.y)으로 올라감 (exam_math_8)
- **마일스톤**: v1.0.0 (M100)
- **브랜치**: `local/task287` (from `devel`)

## 문제

`samples/exam_math_8.hwp` 를 `rhwp export-svg` 로 내보내면, (가) 박스 안의 **큰 cases 수식** `a_{n+1} = { a_n-3 ... | ½a_n ... }` 이 박스 좌상단(body-clip 원점, `(71.80, 147.38)` px)에 찍혀 박스 테두리 및 "(가) 모든 자연수 n에 대하여" 줄과 겹침. 한컴 PDF 에서는 이 수식이 "(가) ..." 줄 다음 줄에 단독으로 배치되어야 함. 같은 문단 내 작은 인라인 수식(`n`, `m`, `3`, `|a_m|=|a_{m+2}|`)은 정상.

## 조사에서 확정된 진짜 원인

수행계획·초기 구현계획서에서 **"`has_tac_shape` 조건 누락 + `.max(y)` clamp"** 로 추정했던 가설은 단계 1 실측에서 틀림으로 확인. 진짜 원인은:

> `src/renderer/composer.rs::compose_lines` 가 큰 수식만 있는 LINE_SEG(ls[1], vpos=1898 HU, lh=4095 HU)를 `runs=[]`(빈 runs)인 `ComposedLine` 으로 만들고,
> `src/renderer/layout/paragraph_layout.rs` 의 TAC 인라인 처리 블록은 **run 루프 안**에 위치해서 빈 runs 줄에서 실행되지 않는다.
> 그 결과 큰 수식은 인라인 경로를 못 타고 `shape_layout.rs:133-182` display 경로로 떨어지며, 거기서 `eq_y = para_y = col_area.y = 147.38` 로 고정된다.

단계 1 덤프:

```
[287] para=0 line=1 y=172.69 raw_lh=54.60 line_h=54.60 baseline=32.76 max_fs=0.00
       tac_offsets_px=[(11,9.8,2,Eq), (17,327.6,3,Eq)] runs=[]
```

(큰 수식 `ci=3` 에 대한 `[287-eq]` 로그 **부재** 로 인라인 경로 미진입 확정)

## 접근 방침 선택

- **(A) vpos 파이프라인 전파** (근본 수정) — `compose_paragraph` 가 vpos 를 완전히 버리고 있어 composer + layout + render_tree + 대량 테스트 스냅샷 수정 필요. 위험·범위 과대.
- **(B) has_tac_shape 확장** (초기 우회안) — 조사 결과 무효. 원인이 다름.
- **(C, 실채택) 빈 runs comp_line 의 TAC 인라인 처리 추가** — 근본적으로 옳고 범위 좁음.

## 변경 사항

### 코드

**`src/renderer/layout/paragraph_layout.rs`** (+63 줄, run 루프 종료 직후)

```rust
// [Task #287] 빈 runs 줄의 TAC 수식 인라인 처리
if comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
    let line_start_char = comp_line.char_start;
    let line_end_char = composed.lines.get(line_idx + 1)
        .map(|l| l.char_start)
        .unwrap_or(usize::MAX);
    let mut inline_x = col_area.x + effective_margin_left;
    for &(tac_pos, tac_w, tac_ci) in &tac_offsets_px {
        if tac_pos < line_start_char || tac_pos >= line_end_char { continue; }
        if let Some(p) = para {
            if let Some(Control::Equation(eq)) = p.controls.get(tac_ci) {
                // ... EquationNode 생성 (기존 인라인 분기와 동일 로직) ...
                let eq_y = (y + baseline - layout_box.baseline).max(y);
                line_node.children.push(eq_node);
                tree.set_inline_shape_position(section_index, para_index, tac_ci, inline_x, eq_y);
                inline_x += tac_w;
            }
        }
    }
}
```

주요 요소:
- 줄 char 범위 `[line_start_char, line_end_char)` 로 이 줄 소유 TAC 만 필터링
- `y`, `baseline`, `line_height` 는 이 줄 컨텍스트에서 이미 정확 → 그대로 사용
- `tree.set_inline_shape_position` 으로 `shape_layout` display 경로의 중복 렌더 차단
- 여러 TAC 있으면 `tac_w` 만큼 `inline_x` 누적

### 샘플

- `samples/exam_math_8.hwp` (18,944 B) — 재현 샘플
- `samples/exam_math_8.pdf` (31,203 B) — 한컴 기준 PDF

## 결과

### 타겟 파일 (`samples/exam_math_8.hwp`)

| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| 큰 수식 transform | `translate(71.80, 147.38)` (col_area 원점 겹침) | `translate(133.27, 188.29)` (line 1 배치) |
| 박스 내부 여부 | ✗ 좌상단 이탈 | ✓ 내부 |
| `shape_layout` display 중복 렌더 | O (버그) | X (제거됨) |
| 작은 인라인 수식 (`n`, `m`, `3`, `|a_m|=|a_{m+2}|`) | 정상 | 정상 (변화 없음) |

### 한계 (Phase 2 로 분리)

| 항목 | 현재 | PDF |
|------|------|-----|
| 큰 수식 x 좌표 | 133.27 (박스 +31.2 px) | ≈162-182 (탭 pos=181 px 반영) |

**약 30-50 px 좌측 치우침**. HWP 문단 구조상 "(가) ... 대하여\n**\t**<수식>" 처럼 탭 제어문자로 들여쓰기된 것으로 추정되나, 현재 빈 runs 줄에서는 탭 처리가 돌지 않음. `compose_lines` 수정 또는 빈 runs 전용 휴리스틱이 필요.

**Phase 2 이슈**: [#288](https://github.com/edwardkim/rhwp/issues/288) 등록.

## 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib renderer` | 291 passed, 0 failed |
| `cargo test --lib equation` | 48 passed, 0 failed |
| `cargo test --test svg_snapshot` | 3 passed, 0 failed |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo test --lib` 전체 | 949/963 (14건 실패는 본 수정 이전부터 존재한 기존 실패: serializer + wasm_api round-trip) |

### 회귀 샘플 (수정 전/후 SVG diff)

| 파일 | 변화 | 해석 |
|------|------|------|
| `exam_math_8.svg` | 큰 수식 이동 (71.80,147.38) → (133.27,188.29) | **본 타스크 목적 달성** |
| `exam_math_008.svg` (page 8) | 큰 수식 이동 (536.12,231.20) → (597.59,272.11) | 동일 "박스 내 큰 수식" 구조 자동 개선 (텍스트 유실 없음, SVG 순서만 변경) |
| `exam_math_012.svg` | `cell-clip-{112→113}` 등 ID shift | `tree.next_id()` 추가 호출로 ID 1씩 증가. 내용·좌표 동일 |
| `equation-lim.svg` | y `132.27 → 133.15` (+0.88 px) | display 경로 → 인라인 경로 전환에 따른 baseline 미세 조정 |

나머지 19개 exam_math 페이지 및 svg_snapshot 파일은 **완전 동일**.

## 교훈

1. **초기 가설에 매달리지 않고 데이터로 검증**: 수행계획과 구현계획 모두 "`has_tac_shape` 조건 확장" 으로 진행하려 했으나, 단계 1 실측 덤프로 진짜 원인이 "빈 runs 줄에서 run 루프 미실행" 임을 확인. 계획을 갱신하고 방향 전환.
2. **(A) 유혹 회피**: vpos 파이프라인 전파는 "올바른" 수정이었지만 조판 엔진 좌표 모델 리팩터링 수준의 범위. 본 타스크에서 무리하게 시도했다면 회귀 폭증과 릴리즈 지연 위험. 별도 타스크로 분리해 현재 버그는 (C) 로 좁게 해결.
3. **완전 일치 vs 근본 해결의 구분**: PDF x 완전 일치는 tab 처리까지 엮여있어 본 타스크 범위 초과. "본래 버그(좌상단 겹침) 해결 + 남은 차이는 Phase 2" 로 분리해 타스크 완결성 확보.

## 산출물

- 계획서: `mydocs/plans/task_m100_287.md`, `task_m100_287_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_287_stage{1,2,3}.md`
- 최종 보고서: `mydocs/report/task_m100_287_report.md` (본 문서)
- 소스: `src/renderer/layout/paragraph_layout.rs` (+63 줄)
- 샘플: `samples/exam_math_8.{hwp,pdf}`
- 후속 이슈: [#288](https://github.com/edwardkim/rhwp/issues/288) (Phase 2)

## 머지 준비

- [x] `cargo test --lib renderer`, `cargo test --test svg_snapshot`, `cargo clippy --lib -- -D warnings` 통과
- [x] 임시 덤프 로그 전부 제거 (단계 3 에서 확인)
- [x] 회귀 샘플 육안 확인 — 모두 의도된 개선 범위
- [x] Phase 2 이슈 등록
- [ ] `devel` merge — 작업지시자 승인 후

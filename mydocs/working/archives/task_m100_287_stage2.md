# Task #287 단계 2 완료 보고서 — 빈 runs comp_line 의 TAC 인라인 처리 추가

## 타스크

[#287](https://github.com/edwardkim/rhwp/issues/287) — 수식 SVG 레이아웃: 큰 디스플레이 TAC 수식이 줄 상단으로 올라감

- 단계 2 목적: 단계 1 에서 확정된 "빈 runs comp_line 이 TAC 인라인 분기를 타지 못하는" 근본 문제를 해결한다.

## 변경 내역

`src/renderer/layout/paragraph_layout.rs` (L1951 부근, run 루프 종료 직후):

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
                // ...EquationNode 생성 (인라인 TAC 분기와 동일 로직)...
                let eq_y = (y + baseline - layout_box.baseline).max(y);
                line_node.children.push(eq_node);
                tree.set_inline_shape_position(section_index, para_index, tac_ci, inline_x, eq_y);
                inline_x += tac_w;
            }
        }
    }
}
```

**추가로**: 단계 1 에서 삽입한 환경변수 기반 임시 덤프(`RHWP_DUMP_287`) 는 단계 3/4 에서도 시각 검증에 사용하기 위해 유지.

## 검증 결과

### 타겟 파일 (`samples/exam_math_8.hwp`)

수정 전 SVG 의 큰 수식:

```xml
<g transform="translate(71.80, 147.38) scale(0.9736,1)">  <!-- col_area 원점 -->
```

수정 후 SVG:

```xml
<g transform="translate(133.27, 188.29) scale(0.9736,1)">  <!-- line 1 y 기반 -->
```

수식이 박스 좌상단에서 이탈해, line 1 의 y(=172.69) + baseline 조정(=188.29) 위치로 정상 배치됨. `shape_layout.rs` 의 display 경로가 중복 렌더하던 노드도 사라짐.

### 단위 테스트

| 테스트 | 결과 |
|--------|------|
| `cargo test --lib renderer` | 291 passed, 0 failed |
| `cargo test --lib equation` | 48 passed, 0 failed |
| `cargo test --test svg_snapshot` | 3 passed |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo test --lib` 전체 | 949 passed, 14 failed |

14건 실패는 **본 수정 이전부터 존재하던 기존 실패** (serializer::cfb_writer + wasm_api round-trip 관련). 렌더러 수정과 무관함. `git stash` 로 수정 제거 후 동일 실패 재현으로 확인.

### 회귀 샘플 비교 (수정 전/후 SVG diff)

| 파일 | diff | 분석 |
|------|------|------|
| `samples/exam_math.hwp` (20페이지 중) | `exam_math_008.svg`: 큰 수식 좌표 `(536.12, 231.20) → (597.59, 272.11)` | page 8 에도 동일 "박스 내 큰 수식" 구조가 있었으며 같이 바르게 고쳐짐 (내용 삭제 없음, 순서만 변경) |
| `samples/exam_math.hwp` | `exam_math_012.svg`: `cell-clip-{112→113}` 등 ID shift | `tree.next_id()` 추가 호출로 ID 번호만 1씩 증가. 내용·좌표 동일. 시각 영향 없음 |
| `samples/equation-lim.hwp` | `translate(113.39, 132.27) → (113.39, 133.15)` (+0.88 px) | 이 수식도 빈 runs 줄에 있어 인라인 경로로 전환. baseline 차이에 따른 미세 조정. PDF 일치도는 단계 3 시각 비교에서 확정 |

19/20 exam_math 페이지, svg_snapshot 테스트 파일들은 **완전히 동일** (diff 없음). 즉 runs 가 있는 일반 문단 및 TAC 표/그림은 영향 없음.

## 위험·미결 사항

1. **equation-lim 의 0.88 px 이동이 PDF 와의 일치도를 높이는지/해치는지 미검증.** 단계 3 에서 시각 비교.
2. **exam_math_008.svg 에서 함께 고쳐진 수식 위치가 PDF 기준으로 맞는지 미검증.** 단계 3 에서 시각 비교.
3. **x 수평 정렬**: 현재 `col_area.x + effective_margin_left` 에서 시작. 한컴 PDF 의 수식 x 와 차이가 나면 단계 3 에서 추가 조정.
4. **임시 덤프(`RHWP_DUMP_287`) 제거**: 단계 3 시각 검증 후 단계 4 에서 제거.

## 산출물

- `src/renderer/layout/paragraph_layout.rs` 수정
- `mydocs/working/task_m100_287_stage2.md` (본 문서)

## 승인 요청

단계 2 완료로 핵심 로직은 동작 확인됨. 단계 3(시각 비교 + x 정렬 미세 조정 필요 시 추가 수정 + 회귀 재확인) 착수해도 될지 승인 부탁드립니다.

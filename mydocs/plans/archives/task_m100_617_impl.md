# Task #617 구현 계획서 — 표 셀 padding 축소 휴리스틱 오작동

## 단계 구성 (4단계)

### Stage 1 — 재현 픽스처와 단위 테스트 (실패 확인)

**목표**: 회귀 방지 테스트 베이스 확보. 수정 전에 실패하고 수정 후에 통과하도록.

**작업**

1. `src/renderer/layout/table_layout.rs` 모듈 안에 `#[cfg(test)] mod shrink_padding_tests` 추가.
2. 테스트 픽스처 헬퍼:
   - `make_composed(line_widths_chars: &[usize])` — 각 줄의 글자 수만큼 가짜 run 을 채운 `ComposedParagraph` 생성.
   - 기본 ResolvedStyleSet (한국어 본문, ratio=95%, spacing=-5%) 픽스처.
3. 테스트 케이스
   - **[A] line_segs filled (정상 IR) → padding 보존**: estimate_text_width 가 임계를 넘는 길이의 줄을 입력하되, 호출자 측에서 line_segs 가 이미 분배되어 있다는 가정. (Stage 2 구현 후 통과)
   - **[B] line_segs empty + 큰 폭 → padding 축소** (이전 동작 유지)
   - **[C] 일반 fit 케이스 → padding 보존** (이전 동작 유지)
   - **[D] 축소 시 최소 padding ≥ 원래의 30%** (Stage 3 구현 후 통과)
4. 시그니처 보강 검토: 현재 `shrink_cell_padding_for_overflow` 는 `composed_paras: &[ComposedParagraph]` 만 받음. line_segs 정보를 호출처에서 넘기려면 `cell.paragraphs` 또는 `&[Paragraph]` 추가 인자가 필요. **시그니처 변경**으로 결정 (호출처 4곳 동시 수정).

**산출물**

- 위 4개 단위 테스트 작성. Stage 1 시점에서는 [B][C] 만 통과, [A][D] 는 실패하는 상태(빨간 줄).
- 호출처 시그니처 갱신 패치(컴파일 그린 보장).

**검증**: `cargo test shrink_padding_tests`.

**커밋**: `Task #617 stage1: 재현 단위 테스트 추가 + 시그니처 보강` + `mydocs/working/task_m100_617_stage1.md`.

---

### Stage 2 — A: line_segs 기반 shrink skip

**목표**: 보고된 16/27/36번 박스 padding 복원의 본 수정.

**작업**

1. `shrink_cell_padding_for_overflow` 진입부에 다음 가드 추가:
   ```rust
   // HWP IR 이 line_segs 로 줄바꿈을 이미 확정한 정상 입력은
   // 셀 가용 폭에 fit 하도록 자간이 분배된 상태. 자연 폭 추정으로
   // 다시 깎으면 오버 페인팅. (Task #617)
   let all_lines_distributed = paragraphs.iter().all(|p| !p.line_segs.is_empty());
   if all_lines_distributed {
       return (pad_left, pad_right);
   }
   ```
2. Stage 1 의 [A] 테스트 통과 확인.
3. 시각 검증
   - `rhwp export-svg samples/exam_kor.hwp -p 5,8,16 -o output/svg/task617/`
   - 16/27/36번 보기 박스 좌·우 padding 시각 복원 확인 (스크린샷 첨부).
   - 25번 박스 회귀 없음 확인.

**산출물**

- `src/renderer/layout/table_layout.rs` 수정.
- 시각 비교 자료(stage2 보고서에 첨부).

**검증**

- `cargo build && cargo test`
- `cargo clippy --all-targets`
- 시각: 16/27/36번 좌·우 padding 약 3 mm 보임. 본문 글자 크기·줄 수 동일.

**커밋**: `Task #617 stage2: line_segs 분배된 정상 IR 에서 padding shrink skip` + stage2 보고서.

---

### Stage 3 — B+C: 잔여 케이스 추정 정확도 및 최소 padding 보정

**목표**: shrink 가 실제로 필요한 비정상 케이스에서도 안전 마진 강화.

**작업**

1. effective width 보정
   - `resolved_to_text_style` 결과의 `ratio` 와 `letter_spacing` 효과를 곱해 effective per-glyph 폭을 산출. (현재 `letter_spacing` 음수 clamp 만 하던 것을 ratio 까지 반영.)
   - 구체 식: `effective_width = estimate_text_width(text, ts_with_clamped_spacing) * (ratio / 100.0)` 또는 ts 의 `font_size` 를 ratio 비율로 미리 축소.
2. 임계 1.15 → 1.30 완화 + 주석 갱신.
3. 최소 padding 정책: 현재 `min_pad = 1.0` → `min_pad = pad_*  * 0.30` (원래 padding 의 30% 하한). 좌·우 각각 별도 적용.
4. Stage 1 의 [D] 테스트 통과 확인. [B][C] 회귀 없음 확인.

**산출물**

- `src/renderer/layout/table_layout.rs` 수정.

**검증**: `cargo test`, 시각 회귀 없음.

**커밋**: `Task #617 stage3: 자연 폭 추정 보정 + 최소 padding 30% 하한` + stage3 보고서.

---

### Stage 4 — 전체 샘플 회귀 검증 + 최종 보고

**목표**: 의도한 박스 외 변경이 없음을 확인.

**작업**

1. 변경 전 baseline 과 변경 후 SVG 비교
   - `samples/` 의 핵심 샘플(특히 KTX 목차 관련, 한컴 고시 샘플, exam_kor) 재렌더.
   - SVG diff 또는 PNG 픽셀 diff (관찰 가능한 차이가 의도한 박스에만 발생).
2. clippy + 전체 cargo test 통과.
3. `mydocs/report/task_m100_617_report.md` 작성.
4. `mydocs/orders/<오늘날짜>.md` Task #617 상태 갱신.

**커밋**: `Task #617 최종 보고서 + orders 갱신`.

**병합 후속**: 작업지시자 승인 시 `local/task617 → local/devel` merge.

---

## 호출처 시그니처 변경 영향

`shrink_cell_padding_for_overflow` 가 `&[Paragraph]` 또는 `line_segs` 정보를 추가로 받도록 변경 시 다음 4곳 동시 수정:

- `src/renderer/layout/table_cell_content.rs:560`
- `src/renderer/layout/table_partial.rs:352`
- `src/renderer/layout/table_layout.rs:1219`
- 그 외 같은 함수 호출처 (Stage 1 에서 grep 으로 재확인)

호출처에서 `cell.paragraphs` 슬라이스를 그대로 넘기면 됨 (이미 컨텍스트에 존재).

## 위험 및 대응

- **위험 1**: line_segs 가 채워졌지만 cell 폭과 line 분배가 실제로는 안 맞는 비정상 IR.
  - 대응: 본 작업은 보고된 케이스 우선. 만약 회귀 발견 시 Stage 2 가드를 "line_segs 분배 + line_seg 의 width 합이 inner_width 의 1.05× 이내" 로 강화.
- **위험 2**: Stage 3 의 임계/하한 변경이 다른 샘플에 영향.
  - 대응: Stage 4 회귀에서 발견 즉시 임계만 롤백.

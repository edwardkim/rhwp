# Stage 1 보고서 — Task #409

## 개요

수정 전 베이스라인을 수집하고 가설(차트 높이 ≈ 31470 HU 이중 반영)을 수치로 재확인.

## 수집 자료

수정 전 자료는 `mydocs/working/task_m100_409_stage1_before/` 디렉토리에 보관:

- `overflow_before.txt` — `export-svg -p 20` 의 LAYOUT_OVERFLOW 로그 전량
- `p21_before.svg` — 21페이지 SVG 출력
- `dump_pages_p21.txt` — 21페이지 단/문단 배치 결과
- `dump_pi172.txt` — pi=172 (차트 anchor 문단) 상세
- `dump_pi173.txt` — pi=173 (차트 다음 빈 문단) 상세
- `dump_pi174.txt` — pi=174 (2x1 표) 상세

## 베이스라인 수치

### 차트 그림 (pi=172, ci=0)
```
[0] 그림: bin_id=19, common=48190×31470 (170.0×111.0mm), tac=false
    위치: 가로=단 오프셋=0.0mm(0), 세로=문단 오프셋=0.0mm(0)
    배치: 위아래, 글자처럼=false
```
- `text_wrap = TopAndBottom (위아래)`
- `vert_rel_to = Para (문단 오프셋)`
- 높이: 31470 HU = **419.6 px (96 dpi)**

### vpos 값 (HU)
| 문단 | vpos | 차이 |
|------|------|------|
| pi=172 | 1275685 | — |
| pi=173 | 1307155 | +31470 (= 차트 높이) |
| pi=174 | 1308915 | +1760 (=lh 1100 + ls 660) |

→ 한컴이 차트 높이를 후속 문단 vpos에 반영했음을 확인.

### LAYOUT_OVERFLOW 로그 (수정 전)
```
LAYOUT_OVERFLOW: page=20, col=0, para=174, type=Table, y=1049.7, bottom=1028.0, overflow=21.8px
LAYOUT_OVERFLOW: page=20, col=0, para=175, type=FullParagraph, y=1063.1, overflow=35.1px
... (pi=176~191 모두 overflow, +14.7px씩 누적)
LAYOUT_OVERFLOW: page=20, col=0, para=191, type=FullParagraph, y=1296.4, overflow=268.4px
LAYOUT_OVERFLOW: page=20, col=0, para=192, type=Table, y=1549.6, overflow=521.7px
```

총 19개 LAYOUT_OVERFLOW (pi=174~192).

### 회귀 테스트 베이스라인
- `cargo test --lib --release`: **1023 passed**, 0 failed, 1 ignored
- `cargo test --release --test svg_snapshot`: **6 passed**, 0 failed

## 가설 재확인

### vpos 보정 흐름 검증
1. `Shape pi=172` 처리 후 y_offset = 528.8 (= 94.5 col_top + 14.7 paragraph + 419.6 chart, 또는 y_offset+total_height 패턴)
2. `vpos_page_base = None; vpos_lazy_base = None` 무효화 (line 1490-1491)
3. `FullParagraph pi=173` vpos 보정 진입:
   - `prev_pi = 172`, `vpos_end = 1275685 + 1100 + 660 = 1277445`
   - `y_delta_hu ≈ (528.8 - 94.5) × 75 = 32574` (≈ 31470 차트 + 1100 paragraph)
   - `lazy_base = 1277445 - 32574 = 1244871`
4. `Table pi=174` vpos 보정:
   - `vpos_end_173 = 1308915`
   - `end_y = 94.5 + (1308915 - 1244871) × 96/7200 = 94.5 + 853.92 = 948.4 px`
   - 이후 표 자체 높이 + line_seg 보정으로 최종 y_offset ≈ 1049.7 (LAYOUT_OVERFLOW 로그 일치)

→ 가설 확정: **lazy_base 산출 시 prev_pi의 텍스트 vpos_end 와 차트 바닥 y_offset 의 불일치로 차트 높이만큼 base가 낮게 산출됨**.

### 기존 가드의 한계 재확인
`src/renderer/layout.rs:1366-1370`:
```rust
matches!(c, Control::Shape(s) if matches!(s.common().text_wrap,
    TextWrap::InFrontOfText | TextWrap::BehindText))
```
- `Control::Picture` 미검사 → 본 케이스의 그림이 가드를 통과하지 않음
- `TopAndBottom` 미포함 → vert=Para + TopAndBottom 케이스 처리 누락

## 다른 샘플의 TopAndBottom 그림 분포

추후 회귀 검증 시 우선 점검 대상 (Stage 3에서 활용):
- `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` — 본 타스크 대상 (다수의 TopAndBottom 차트)
- 기타: `cargo test --test svg_snapshot` 의 골든 SVG 가 회귀 검출 1차 방어선

## 결론

- 가설 확정: **vpos lazy_base 산출에서 차트 높이가 이중 반영되어 후속 문단/표가 차트 높이만큼 추가 점프**
- 수정 방향 확정: **`prev_has_overlay_shape` 가드를 Picture (non-TAC) + TopAndBottom/vert=Para 까지 확장**
- 베이스라인: 1023 lib + 6 svg_snapshot 통과, LAYOUT_OVERFLOW 19건 (pi=174~192)

Stage 2 진행 승인 요청.

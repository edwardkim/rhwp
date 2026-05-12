# Stage 1 보고 — Task #846 (M100) — 진단 및 column-band 확장 설계

상태: **진단 완료 — 소스 수정 없음**.

## 1. SVG 렌더 경로 확정

- `render_page_svg_native` → `src/document_core/queries/rendering.rs` → 페이지네이션은 **`src/renderer/typeset.rs::TypesetEngine`** (`pagination/engine.rs` 아님). 이슈 "주의" 항 해소.
- `pagination/engine.rs` 는 본 타스크에서 손대지 않는다.

## 2. shortcut.hwp 페이지 3 끝 배치 추적

`rhwp dump-pages -p 2`(페이지 3) / `-p 3`(페이지 4) + `rhwp dump -s 0` 결과:

- pi=81 `[쪽나누기]` "보기" 헤더 + 1×1 TAC 표 → 페이지 3 상단.
- pi=82 `[다단나누기]`(MultiColumn, 새 ColumnDef col_count=2) → 2단 zone 시작.
- 2단 zone 본문: col 0 = pi=82~87 (6행, used 80px / hwp≈106.7px), col 1 = pi=88~93 (6행, used 80px / hwp≈106.7px).
- **pi=94 `[단나누기]`(ColumnBreakType::Column)** — col 1(마지막 단) 을 다 채운 직후 등장.
- pi=95 "화면 이동 ⟶ Ctrl+W,N" (컨트롤 없음).
- pi=96 `[다단나누기]` "입력" 헤더 + 1×1 TAC 표.

현 동작: 페이지 3 = pi=81~93 까지. pi=94/95/96~ 는 **페이지 4 로 밀림** (`dump-pages -p 3` 의 단 0 = {pi=94, pi=95}).
한컴 PDF(`pdf/basic/shortcut-2022.pdf`): pi=94(좌)/pi=95(우) 가 **페이지 3 의 기존 6행 밴드 아래 새 2단 밴드**로, 이후 "입력" 헤더(페이지 3 하단까지)도 페이지 3 에 들어감. PDF 총 7페이지(rhwp baseline 8).

## 3. 원인

`typeset.rs::paginate`:
```rust
if para.column_type == ColumnBreakType::Column {
    if has_diff_col_def { ... }
    else if !st.current_items.is_empty() { st.advance_column_or_new_page(); }
}
```
`advance_column_or_new_page`:
```rust
self.flush_column();
if self.current_column + 1 < self.col_count { self.current_column += 1; self.current_height = ...; }
else { self.push_new_page(); }   // ← pi=94: current_column=1, col_count=2 → 여기로
```
→ 마지막 단에서 명시적 `Column` break 를 만나면 무조건 새 페이지.

## 4. 핵심 발견 — "동일 zone 다중 밴드" 메커니즘은 이미 존재

`dump-pages -p 3`(페이지 4) 는 `zone_y_offset` 이 0 → 18.7 → 78.6 → … 로 누적되며 한 페이지에 **밴드 여러 개가 적층**됨. 이 적층은 `process_multicolumn_break` 가 담당:
- `flush_column()`, 직전 문단의 마지막 LINE_SEG `vpos_end` → px 변환 → `current_zone_y_offset += vpos_zone_height`, `current_column = 0`, `current_height = 0.0`, 새 ColumnDef 로 `col_count`/`layout` 갱신.

즉 **`[다단나누기]`(새 ColumnDef 동반) 에 대해서만** 같은 페이지 새 밴드를 만들고, **`[단나누기]`(마지막 단) 는 새 페이지**로 보낸다. 한컴은 마지막 단에서 `[단나누기]` 를 만나면 — 새 ColumnDef 가 없어도 — **같은 col_count 로 같은 페이지에 새 밴드**를 시작한다 (≈ 닫힌 #768). 이 경로가 누락.

## 5. 모델 확장 설계

`paginate` 의 명시적 `Column` break 경로에서, 마지막 단(`current_column + 1 >= col_count` 이고 `col_count > 1`) 이면 — `advance_column_or_new_page`(→push_new_page) 대신 신규 `start_new_column_band`:

1. `flush_column()`
2. 다음 밴드(= `para_idx` ~ 다음 나누기/새 ColumnDef 직전)에 떠다니는(글자처럼 취급 아닌) 표·그림·그리기 개체가 있으면 `push_new_page` 후 종료.
3. 방금 닫힌 밴드의 높이 = 그 밴드 각 단의 마지막 문단 `vpos_end` 중 최댓값 (`process_multicolumn_break` 의 vpos 기반 산출을 단별 max 로 확장).
4. `available_height() - 밴드높이 >= 이_문단_첫줄_높이` 이면 새 밴드: `current_zone_y_offset += 밴드높이`, `current_column = 0`, `current_height = 0.0`, `on_first_multicolumn_page = true`. col_count/layout 은 유지. 그렇지 않으면 `push_new_page`.

새 밴드가 본문 하단을 넘는 경우는 다음 문단 배치 시 기존 높이 초과 → 새 페이지로 자동 처리(별도 검사 추가 안 함).

## 6. `layout.rs` 연동점

`renderer/layout.rs` 의 `ColumnContent` → 단 영역 x/y 좌표 산출은 이미 `zone_y_offset` 을 기준으로 함 (페이지 4 의 다중 밴드가 정상 렌더되는 것으로 검증됨). 본 변경은 `start_new_column_band` 가 `zone_y_offset` 을 진행시키는 것뿐 → **`layout.rs` 무수정 예상**.

## 7. 단일 룰 판정

"마지막 단 + 명시적 `ColumnBreakType::Column` → (들어갈 공간이 있으면) 같은 페이지 새 밴드 / 없으면 새 페이지" — 분기는 "들어갈 공간 여부" 단일 판정뿐, 허용오차·fallback 없음. 휴리스틱 아님 (메모리 `feedback_rule_not_heuristic` 자문 불요).

## 8. 회귀 대상 (Stage 3)

본 변경은 **`col_count > 1` zone 에서 마지막 단에 명시적 `[단나누기]` 가 오는 경우**에만 동작이 바뀐다. 그래도 광역 검증: shortcut.hwp 1~8페이지, exam_* 류, k-water-rfp, 다단+TopAndBottom 표, 다단+목차, 다단+각주, 단일 단 샘플 일부, `cargo test` 전건.

## 9. 닫힌 #768 흔적

`mydocs/` 전수 검색 — #768 관련 분석 문서 없음 (이슈 본문의 "동일 사안" 언급뿐).

# Task #1748 1단계 완료 보고 — 원인 조사·확정

## 결론 (요약)

p6 하단 +13px 의 원인은 수행계획서의 가설 ①~③(행 컷/grace 판정)이 **아니다**.
행 컷과 페이지네이터 회계는 이미 정확하다. 결함은 **컷 행에 걸친(straddling)
rowspan 셀의 렌더 처리**다:

- 컷 페이지(p6): 걸친 rowspan 셀이 **가시범위 제한·클리핑 없이 전체 줄을 렌더** →
  셀 박스 아래로 3줄 흘러넘침 (+13px 잉크).
- 연속 페이지(p7): 같은 셀이 **처음부터 전체 재렌더** → 내용 중복 + 아래 행 영역 침범.
- 한글 2024 PDF 는 컷 페이지에 들어가는 줄까지만 보여주고 나머지를 연속 페이지에
  **이어그리기** 한다 (중복·넘침 없음).

## 조사 경로 (계측 코드 추가 없음 — 기존 진단·CLI 만 사용)

1. **재현**: `dump-pages -p 5` → `PartialTable pi=9 rows=100..140 end_cut=[1,2]` (이슈와 동일).
   pymupdf 96 DPI dark-pixel bbox: p6 Δbot **+13** (rhwp 1094 vs 한글 1081), p5 Δbot −6.
2. **회계 확인**: `RHWP_TABLE_DRIFT=1` →
   `cursor_row=100 end_row=140 consumed=981.1 avail_for_rows=988.3 partial_h=1002.0 fits=true`
   — 페이지네이터는 예산 내. **가설 ①~③ 기각** (grace/tolerance 는 발동 자체가 문제가 아님).
3. **render tree 대조**: 표 박스 h=1002.0(회계와 일치), bottom=1077.6. 그런데 래스터
   잉크는 1094 — **셀 박스 밖 텍스트**.
4. **초과 잉크의 정체**: x=466.7 셀(y=1001.4, h=76.2, bot=1077.6, rowspan)의 TextLine 이
   y=1074.5/1091.8/1109.2 (bot 1087.8/1105.1/1122.5)까지 렌더. row_span==1 셀 2개는
   end_cut=[1,2] 대로 1줄/2줄만 렌더(정상).
5. **p7 대조**: 같은 셀이 y=96.5 h=59.1 박스에서 **전체 7줄을 처음부터 재렌더**
   (텍스트 bot 217.5 > 셀 bot 155.6 — 아래 행 침범). 한글 p7 은 p6 마지막 줄
   ("세제곱미터 미만인 경우") **다음 줄부터** 4줄만 렌더.

## 근본 원인 (코드)

`src/renderer/layout/table_partial.rs` 셀 루프(비블록 분할 경로):

```rust
let is_split_end_row = !end_cut.is_empty() && cell_row == end_row.saturating_sub(1);
```

- 컷 행(139)에 **걸쳐 있기만 한** rowspan 셀(cell_row=138, span 이 139 를 포함)은
  `cell_row != end_row-1` → `is_in_split_row=false` → `clip:false`, `cut_units=None`
  → 전체 줄 무제한 렌더.
- 컷 부기(end_cut)는 `advance_row_cut` 이 컷 행의 `row_span==1` 셀만 담으므로
  걸친 rowspan 셀에는 애초에 컷 항목이 없다.
- 연속 페이지에서도 같은 이유로 컷 없음 → 전체 재렌더(중복).
- 페이지 경계가 rowspan 블록 내부를 per-row 분할하는 경로(#1022,
  `rowbreak_rowspan_row_splittable`)에서만 발생. 블록 분할(`is_block_split`) 경로는
  rowspan 셀도 `block_cut_index` 로 컷 부기가 있어 정상.

## 수정 설계 (2단계에서 구현)

페이지네이션(행 컷·회계)은 불변. **렌더러(table_partial.rs)만** 수정:

1. **걸친 rowspan 셀 판별**: 비블록 경로에서 `cell_end_row > end_row`(끝 걸침) /
   `cell_row < start_row`(시작 걸침, 연속분).
2. **높이 기반 유닛 컷**: 셀의 `cell_units` 를 프래그먼트-가시 높이 예산으로 잘라
   su/eu 산출 → 기존 `cell_line_ranges_from_cut` 재사용으로 줄 범위 제한 + `clip:true`.
   - 끝 걸침(p6): 예산 = 프래그먼트 내 span 행 높이 합(=셀 박스 h) − 패딩.
   - 시작 걸침(p7): 이전 프래그먼트 소비 높이를 **결정적으로 재계산** — 온전 행은
     `row_cut_content_height(r, &[], &[])`, 분할 행(139)은
     `row_cut_content_height(r, &[], start_cut)` (p6 이 end_cut 으로 계산한 값과 동일 식)
     → su. 이로써 p6.eu == p7.su 가 산술적으로 일치(상태 전달 불필요).
3. 컷 적용 셀은 top-flow 로 배치(기존 분할 셀 줄범위 렌더와 동일 경로) —
   현재 걸친 셀은 valign 이 전체 콘텐츠 기준이라 첫 줄도 어긋난다.

### 예상 효과

- p6: rowspan 셀이 박스 안 줄까지만 렌더 → 잉크 bottom ≈ 표 경계(1078) → **Δbot ≈ −3**.
- p7: 이어그리기 (중복 제거 + 아래 행 침범 해소) — 한글 정합.

## 게이트 영향 재평가

행 컷·페이지네이션 불변이므로 쪽수 게이트(giant 42, byeolpyo 4/27) 회귀 위험은
수행계획서 예상보다 낮다. 다만 rowspan 걸침 렌더는 블록 분할·중첩 표와 인접하므로
`issue_rowbreak_chart_overlap` 등 분할 표 시각 테스트 전체를 게이트에 유지한다.

## 계획 대비 변경

- 수행계획서의 "수정 방향(grace 축소)" 폐기 → 구현계획서 2단계를 본 설계로 갱신
  (`task_m100_1748_impl.md` v2 수정).
- 수정 대상 파일: `src/renderer/layout/table_partial.rs` (+ 필요 시
  `table_layout.rs` 헬퍼 추가). `typeset.rs` 는 손대지 않는다.

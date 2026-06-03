# Task #287 단계 1 완료 보고서 — 파이프라인 실측 및 수정 지점 확정

## 타스크

[#287](https://github.com/edwardkim/rhwp/issues/287) — 수식 SVG 레이아웃: 큰 디스플레이 TAC 수식이 줄 상단으로 올라감

- 단계 1 목적: `exam_math_8.hwp` 의 실제 `ComposedLine` 구조와 인라인 TAC 분기 진입 여부를 데이터로 확정하고, 수정 지점을 결정한다.

## 측정 방법

`src/renderer/layout/paragraph_layout.rs` 두 지점에 환경변수(`RHWP_DUMP_287`) 기반 임시 덤프 로그 삽입:

1. L730 부근 (comp_line 루프 시작) — 줄별 y, raw_lh, line_height, baseline, max_fs, has_tac_shape, tac_offsets_px, runs
2. L1584 부근 (인라인 수식 분기) — sec, para, ci, y, baseline, layout_box.baseline, eq_y, eq_h, tac_w, x

이후:
```
cargo build --release
RHWP_DUMP_287=1 ./target/release/rhwp export-svg samples/exam_math_8.hwp -o output/svg/exam_math_8/
```

## 측정 결과

본문 페이지(section=0) 의 주요 로그:

```
[287] para=0 line=0 y=147.39 raw_lh=15.33 line_h=15.33 baseline=13.04 max_fs=15.33 has_tac_shape=false
       tac_offsets_px=[(11,9.8,2,Eq), (17,327.6,3,Eq)]
       runs=["(가) 모든 자연수 에 대하여"]
[287-eq] para=0 ci=2  y=147.39 baseline=13.04 lb.baseline=11.73 eq_y=148.69 eq_h=14.67 tac_w=9.83 x=217.07
[287] para=0 line=1 y=172.69 raw_lh=54.60 line_h=54.60 baseline=32.76 max_fs=0.00 has_tac_shape=false
       tac_offsets_px=[(11,9.8,2,Eq), (17,327.6,3,Eq)]
       runs=[]
[287] para=0 line=2 y=237.27 raw_lh=15.33 line_h=15.33 baseline=13.04 max_fs=15.33 has_tac_shape=false
       tac_offsets_px=[(11,9.8,2,Eq), (17,327.6,3,Eq)]
       runs=["이다."]
[287] para=1 line=0 y=267.91 raw_lh=18.00 line_h=18.00 baseline=13.04 max_fs=15.33 has_tac_shape=false
       tac_offsets_px=[(4,99.0,0,Eq), (10,13.8,1,Eq), (17,8.8,2,Eq)]
       runs=["(나) 인 자연수 의 최솟값은 이다."]
[287-eq] para=1 ci=0  y=267.91 ... eq_y=269.21  (|a_m|=|a_{m+2}|)
[287-eq] para=1 ci=1  y=267.91 ... eq_y=269.21  (m)
[287-eq] para=1 ci=2  y=267.91 ... eq_y=269.21  (3)
```

## 핵심 관찰

1. **큰 수식(para=0, ctrl=3) 에 대한 `[287-eq]` 로그가 존재하지 않음.**
   → 큰 수식은 `paragraph_layout.rs:1574-1620` 의 인라인 TAC 분기를 **전혀 타지 않음**.

2. **para=0 line=1 은 `runs=[]`(빈 줄).**
   → `for run in &comp_line.runs { ... }` 루프가 1회도 돌지 않음 → TAC 인라인 처리 블록이 실행되지 않음.
   → max_fs=0.00 (runs 없음), line_h=54.60 (ls[1].line_height 그대로) 은 의도된 "플레이스홀더" 줄 형태.

3. **y 누적은 정확함.**
   → line 0 y=147.39 → line 1 y=172.69 → line 2 y=237.27. 간격 25.30 / 64.58 은 line_height+spacing 누적으로 각 ls 의 vpos 증가분(1898/4843 HU ≈ 25.3/64.6 px) 과 부합. 파이프라인의 y 는 잘못되지 않았다.

4. **작은 수식들은 모두 정상.**
   → para=0 ctrl=2 (작은 `n`), para=1 ctrl=0/1/2 (|a_m|=|a_{m+2}|, m, 3) 모두 `[287-eq]` 로그에 나타나며 해당 줄의 y 에 baseline 정렬되어 `eq_y` 가 올바르게 계산됨.

## 가설 수정 — 진짜 원인

**수행계획·구현계획서에서 추정했던 "`has_tac_shape` 조건 누락 + `.max(y)` clamp" 가설은 틀림.**

진짜 원인:

> `compose_lines` 가 큰 수식만 있는 ls(line_segs[1]) 를 `runs=[]` 로 만들고,
> `layout_paragraph` 의 TAC 인라인 처리가 run 루프 안에 있어서 빈 runs 줄에서 실행되지 않는다.
> 그 결과 큰 수식은 인라인 경로를 못 타고 `shape_layout.rs:133-182` display 경로로 떨어져,
> 거기서 `eq_y = para_y = col_area.y = 147.38` 로 고정된다.

## 수정 지점 (단계 2 이후)

**핵심 수정**: `paragraph_layout.rs` 의 comp_line 루프에서,

- `runs` 가 비어있어도 **이 줄이 소유한 TAC 수식**(`composed.tac_controls` 중 해당 `char` 범위에 속한 항목)을 인라인 렌더 경로로 처리한다.
- 줄의 char 범위는 `comp_line.char_start` 와 다음 comp_line 의 `char_start`(마지막이면 문단 끝)를 이용한다.
- 이 줄의 `y`, `baseline`, `line_height` 는 이미 정확히 계산돼 있으므로 그대로 사용.
- `shape_layout.rs:133-182` 가 동일 수식을 중복 렌더하지 않도록 `tree.set_inline_shape_position` 등록.

**부수 수정**:

- 큰 수식에서 `eq_y = (y + baseline - layout_box.baseline).max(y)` clamp 는 원안대로 유지해도 문제없음 (line 1 의 y=172.69 에서 시작하므로 `.max(y)` 의 부작용 없음). 단, 큰 수식의 `eq_h = layout_box.height` 가 `line_height=54.60` 과 정합한지는 단계 3에서 재확인.
- x 정렬: 현재 인라인 커서 x 는 빈 runs 줄에서 `col_area.x + margin_left`(= line 시작 x) 가 될 것. 한컴 PDF 에서의 x(박스 내 중앙 근처)와 비교해 단계 3 에서 결정. 1차 범위는 "인라인 TAC 진입" 까지.

**단계 2 구현 계획 (조정):**

- "has_tac_shape 조건 확장" → **"빈 runs comp_line 에서의 TAC 처리 추가"** 로 수정
- 구현계획서 `task_m100_287_impl.md` 단계 2 본문 업데이트 필요

## 산출물

- `mydocs/working/task_m100_287_stage1.md` (본 문서)
- 임시 덤프 로그: `paragraph_layout.rs` 두 지점 (단계 3/4 에서 제거)

## 검증 체크

- [x] 큰 수식이 인라인 분기를 타는가? → **타지 않음** (`[287-eq] para=0 ci=3` 부재로 확정)
- [x] line 1 의 y 가 정확한가? → **정확** (y=172.69, ls[1] vpos 와 일치)
- [x] runs=[] 빈 줄이 원인인가? → **그렇다** (원인 확정)
- [x] 수정 지점 확정 → `paragraph_layout.rs` comp_line 루프 + runs=[] 분기 추가

## 승인 요청

단계 1 결과로 원인이 당초 가설과 다르게 확정되어, 단계 2 의 구체 구현 방침을 **"빈 runs comp_line 의 TAC 인라인 처리 추가"** 로 조정합니다. 구현계획서 단계 2 섹션을 이에 맞춰 갱신하고 단계 2 착수해도 될지 승인 부탁드립니다.

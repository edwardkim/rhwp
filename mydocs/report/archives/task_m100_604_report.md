# Task #604 — 최종 결과 보고서

## 1. 본질

### 1.1 시작 — Issue #604 결함

PR #589 (Task #511 v2 + #554) 머지 후 시각 판정 중 발견:
`hwp3-sample5.hwp` page 4 (HWP3 native): pi=74 그림 (User level programs/Kernel/Hardware
다이어그램, 126.4×94.5mm, Square wrap) 의 우측에 wrap text (pi=75, "커널의 가장 밑바탕은...")
가 정상 배치되지 않고 **그림 좌측 + 그림 위 + 그림 아래** 에 산재 — 그림과 텍스트 겹침.

### 1.2 본질 진단

데이터 검증 (`rhwp dump`):
```
--- 문단 0.74 (anchor) ---
  ls[0]: cs=35460, sw=15564

--- 문단 0.75 (wrap text) ---
  ls[0~2]: cs=0, sw=0     ← ❌ wrap zone 미설정
  ls[3~]:  cs=35460, sw=15564
```

근본 원인 두 가지:
1. **HWP3 파서 결함**: `src/parser/hwp3/mod.rs:1399-1407` 의 wrap zone pgy 범위 검사가
   양방향 가드 (`pgy >= pgy_start && pgy < pgy_end`) 라 wrap text 문단의 첫 줄 pgy 가
   anchor 의 pgy_start 미만 인 경우 cs/sw=0 설정.
2. **Document IR 표준 부재**: `Paragraph.wrap_precomputed: bool` 플래그가 HWP3 휴리스틱
   을 IR 에 누설 (PR #589 보완6 도입). LineSeg 필드의 단위/원점/0 의미가 명문화되지 않음.

## 2. 정정 방향 (옵션 C → R3)

작업지시자 결정: 옵션 C (Document IR 표준 정합화). 단순 결함 정정이 아니라 IR 부채 청산
+ 본질적 메커니즘 도입. 진행 중 옵션 C (cs/sw 단독 판정) 가 본질 부적합으로 판명되어
**R3 (typeset 출력 메타데이터 채널)** 로 진화.

## 3. 4 Stage 진행

### Stage 1 — IR 표준 + helper (commit `40739ae`)

- `mydocs/tech/document_ir_lineseg_standard.md` 신설 (+150 LOC) — LineSeg 필드의
  단위/원점/0 의미 명시. HWP5/HWPX/HWP3 각 파서 인코딩 책임 명시.
- `src/model/paragraph.rs` — LineSeg 필드 doc 정합 + `is_in_wrap_zone(col_w_hu)` helper 추가.
- 분석 자료 3 파일 `mydocs/tech/` 로 이동 (git 추적 영역).
- 검증: `cargo test --lib` 1130 passed (회귀 0).

### Stage 2a — 옵션 C (단독 판정) 시도 ❌ revert

`is_in_wrap_zone(col_w_hu)` 단독 판정으로 wrap_precomputed 검사 교체 시도. `test_547`
회귀 발견 — HWP5 native passage box 본문 LineSeg `cs=852/sw=30184` 가 false-positive
판정. 본질 한계: cs/sw 만으로는 wrap zone 과 box inset/cell padding 구분 불가능.
전체 변경 revert. R3 본질 채택.

### Stage 2 — typeset 출력 메타데이터 채널 (commit `b255540`)

- `src/renderer/pagination.rs`: `WrapAnchorRef` struct + `ColumnContent.wrap_anchors`
  HashMap 필드 추가.
- `src/renderer/typeset.rs` + `pagination/state.rs`: TypesetState/PaginationState 에
  `current_column_wrap_anchors` 필드 + flush 시 take.
- `src/renderer/typeset.rs:495~`: wrap_around 매칭 시 wrap_anchors 등록.
- `src/renderer/layout.rs`: ColumnItemCtx 필드 + layout_column_item 시그니처 정합화.
- `src/renderer/layout/paragraph_layout.rs`: 3 시그니처 (layout_paragraph,
  layout_partial_paragraph, layout_composed_paragraph) 에 `wrap_anchor` 인자 추가.
  `wrap_precomputed` 검사 3곳 → `wrap_anchor.is_some()` 검사로 교체.
- 21 호출처 모두 정합화 (PageItem 처리는 ctx.wrap_anchors.get, 그 외는 None).
- 검증: lib 1130 passed, test_547 PASS, Task #546 정합.

### Stage 2b — IR 부채 마무리 (commit `d71f944`)

- `src/renderer/typeset.rs:495~` 매칭 분기 본질화: anchor 종류 (Picture vs Table) 기반
  검사로 `Paragraph.wrap_precomputed` 의존성 제거.
- `src/model/paragraph.rs`: `wrap_precomputed` 필드 제거.
- `src/parser/hwp3/mod.rs`: PR #589 보완6/8 후처리 30 LOC 청산.
- 잔존 주석 정리 (Task #460 보완6 인용 → Task #604 Stage 2 인용).
- 검증: lib 1130 passed, clippy 0, 통합 31 통과.
- LOC: -53 / +30 (-23 net) 소스.

### Stage 3 — HWP3 파서 cs/sw 인코딩 정정 (commit `d96320d`)

- `src/parser/hwp3/mod.rs:1399-1407`: `pgy_start` 가드 제거 (옵션 3a). `pgy_end` 만
  검사하는 단방향 가드. wrap text 문단의 모든 줄이 anchor 그림 우측에 정합 배치.
- pi=75 모든 LineSeg `cs=35460, sw=15564` 정합 (이전: 첫 3 줄 cs=0/sw=0).
- 검증: lib 1130 passed, Task #546/#554 통과, HWP3 native 페이지 회귀 0.
- 시각 정합: page 4 x=529 분포 20 → 23개 (pi=75 첫 3 줄 추가 정합).

### Stage 4 — 광범위 회귀 검증 + 최종 보고 (본 보고서)

- `cargo build --release` 통과
- `cargo test --lib --release` **1130 passed** / 0 failed / 2 ignored
- `cargo test --release` 통합 31 모두 통과
- `cargo clippy --lib -- -D warnings` 0건

## 4. 결정적 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` + `cargo build --release` | ✅ 통과 |
| `cargo test --lib --release` | ✅ **1130 passed** / 0 failed / 2 ignored |
| `cargo clippy --lib -- -D warnings` | ✅ 0건 |
| `cargo test --test issue_546` (Task #546) | ✅ 1 passed (exam_science 4페이지) |
| `cargo test --test issue_554` (HWP3 변환본) | ✅ 12 passed |
| `cargo test svg_snapshot` | ✅ 6/6 |
| `cargo test` 통합 31 | ✅ 모두 통과 |

## 5. 회귀 영역 검증

### 5.1 HWP3 native 페이지 수

| 파일 | 페이지 수 | 베이스라인 (PR #589) | 회귀 |
|------|---------|--------------------|------|
| `hwp3-sample.hwp` | 16 | 16 | ✅ 0 |
| `hwp3-sample5.hwp` | 64 | 64 | ✅ 0 |

### 5.2 HWP5/HWPX 변환본 페이지 수 (Task #554)

| 파일 | 페이지 수 | 베이스라인 | 회귀 |
|------|---------|----------|------|
| `hwp3-sample-hwp5.hwp` | 15 | 15 (Task #554 -1 over-correct) | ✅ 0 |
| `hwp3-sample-hwpx.hwpx` | 15 | 15 | ✅ 0 |
| `hwp3-sample4-hwp5.hwp` | 36 | 36 | ✅ 0 |
| `hwp3-sample5-hwp5.hwp` | 64 | 64 | ✅ 0 |
| `hwp3-sample5-hwpx.hwpx` | 64 | 64 | ✅ 0 |

### 5.3 Task #546 (exam_science.hwp) 회귀 검증

- 페이지 수: 4 (정합)
- p2 단 0 items: 37 / used=1133.6px (정합)

## 6. 시각 판정 자료

`output/svg/task604_after/hwp3-sample5/hwp3-sample5_{004,008,016,022,027}.svg`

| 페이지 | x>380 분포 (상위 3개) | 정합 |
|------|---------------------|------|
| 4 | (725, 27), **(529, 23)**, (713, 19) | ✅ pi=75 23개 — Stage 3 정정 (PR #589: 20개) |
| 8 | (725, 23), (713, 13), **(384, 13)** | ✅ Pattern A wrap text |
| 16 | (725, 15), (713, 13), (701, 8) | ✅ |
| 22 | **(407, 23)**, (725, 21), (419, 18) | ✅ |
| 27 | (725, 18), (713, 15), (701, 14) | ✅ |

## 7. 본질 정정 효과

### 7.1 Issue #604 결함 본질 정정

`hwp3-sample5.hwp` page 4 의 pi=75 모든 LineSeg `cs=35460/sw=15564` 정합 — 그림 우측
정합 배치. PR #589 의 잔존 결함 (첫 3 줄 cs=0/sw=0) 청산.

### 7.2 Document IR 표준 정합화

| 영역 | 본질 |
|------|------|
| **IR 표준 문서화** | `mydocs/tech/document_ir_lineseg_standard.md` 신설 — 단위/원점/0 의미 명시 |
| **IR 부채 청산** | `Paragraph.wrap_precomputed` 필드 제거 — 포맷 독립성 회복 |
| **HWP3 파서 정합화** | 후처리 30 LOC 청산. LineSeg cs/sw 정합 인코딩 책임 |
| **typeset 출력 메타데이터 채널** | `ColumnContent.wrap_anchors` HashMap — anchor ↔ wrap text 컨텍스트 보존 |
| **layout 정합화** | wrap zone 판정이 IR 의존성 제거 — `wrap_anchor.is_some()` |
| **anchor 종류 기반 분기** | typeset 매칭 분기 = Picture vs Table 기반 본질 판정 |

### 7.3 CLAUDE.md HWP3 파서 규칙 정합

- HWP3 휴리스틱의 IR 누설 청산
- HWP3 전용 분기를 typeset/layout 에서 모두 제거 (anchor 종류 기반 본질 분기로 대체)
- IR 의 포맷 독립성 회복

## 8. LOC 합계

| 영역 | 변경 |
|------|-----|
| Stage 1 — IR 표준 + helper | +27/-10 (paragraph.rs) + 150 LOC (표준 문서) |
| Stage 2 — wrap_anchors 메타데이터 | +83 LOC (pagination + typeset + layout + 호출처) |
| Stage 2b — IR 부채 청산 | -53/+30 (-23 net) |
| Stage 3 — HWP3 cs/sw 인코딩 정정 | +14/-2 (mod.rs) |
| **소스 합계** | **+76 LOC net (소스), +1100 LOC 문서** |

## 9. 잔존 영역 (별도 후속 task 권고)

### 9.1 HWP3 폰트 크기 / 줄 간격 처리

`hwp3-sample5.hwp` page 4 의 시각 정합이 본 task 의 cs/sw 정정만으로는 완전 도달 안 함:
- HWP3 native: 폰트 13.0pt 로 표시
- HWP5 변환본 (`hwp3-sample5-hwp5-v2024.hwp`): 폰트 9.0pt 로 표시 (한컴 변환 시 정정)

폰트 크기 차이로 wrap zone 좁은 영역 (sw=15564=207px) 안에 글자가 안 들어가는 결함.
**별도 후속 task 권고**: HWP3 → HWP5 IR 변환 시 char_shape font_size / line_height 처리.

### 9.2 HWP3 LineSeg vertical_pos 누적 계산

`mydocs/tech/document_ir_lineseg_standard.md` §"HWP3" 에 명시: HWP3 파서가 vertical_pos
를 항상 0 으로 채움 (HWP3 spec 본질). 본 표준 (페이지 상단 기준 누적 절대값) 미정합.
**별도 후속 task 권고**: HWP3 파서가 LineSeg 누적 계산.

### 9.3 Task #525 본질 재검토

`Task #525` 가 제거한 `layout_wrap_around_paras` 호출이 본 task 의 wrap_anchors
메커니즘 도입 후에도 유효한지 재검토. dead code 가능성.

## 10. 작업지시자 승인 요청

본 Task #604 (Document IR 표준 정합화 + HWP3 wrap zone cs/sw 인코딩 정정) 완료 보고.

**누적 commits:**
1. `40739ae` Stage 1 — IR 표준 + helper
2. `b255540` Stage 2 — wrap_anchors 메타데이터 채널
3. `d71f944` Stage 2b — IR 부채 마무리
4. `d96320d` Stage 3 — HWP3 cs/sw 인코딩 정정

본 보고서 + 오늘할일 갱신 commit 후 PR 진행 또는 task close 결정 영역.

## 11. 참조

### 관련 문서
- 수행계획서: `mydocs/plans/task_m100_604.md`
- 구현계획서: `mydocs/plans/task_m100_604_impl.md`
- LineSeg 표준: `mydocs/tech/document_ir_lineseg_standard.md`
- 단계별 보고서: `mydocs/working/task_m100_604_stage{1,2,2b,3}.md`

### 분석 자료
- `mydocs/tech/document_ir_parser_relationship_analysis.md` (16KB)
- `mydocs/tech/hwp5_wrap_precomputed_analysis.md`
- `mydocs/tech/document_ir_wrap_zone_standard_review.md`

### 시각 판정 자료
- `output/svg/task604_after/hwp3-sample5/hwp3-sample5_{004,008,016,022,027}.svg` — Stage 1~6 영역
- `output/svg/task604_stageD2/hwp3-sample5/hwp3-sample5_004.svg` — Stage A+D + D-2 영역
- `output/svg/task604_stageD2/hwp3-sample4/hwp3-sample4_021.svg` — sample4 picture anchor 영역

### 관련 task / PR / 이슈
- **Issue #604** — 본 task 의 결함 보고
- **PR #589** (Task #511 v2 + #554) — wrap_precomputed IR 플래그 도입 (본 task 정정 대상)
- **Task #460 보완6** (`bdb51a4`) / 보완8 (`ff64387`) — 본 task Stage 2b 청산 대상
- **Task #546** — exam_science.hwp 회귀 정정 (본 task 회귀 0 보존)
- **Task #525** — Picture Square wrap 호스트 중복 emit 정정 (잔존 검토 영역)
- **Task #489** — Picture/Shape Square wrap LINE_SEG.cs/sw 적용 (anchor 문단)

## 3. Stage A + D + D-2 보완 영역 (본 task 의 본질 영역 도달)

### 3.1 Stage A — HWP5 spec 정밀 재진단 영역

HWP5 v2024 변환본 (`hwp3-sample5-hwp5-v2024.hwp`) page 4 LineSeg 정밀 분석으로
**Document IR 표준 미명시 영역** 발견:

1. **paragraph 내 LineSeg cs/sw 전환** — wrap zone 끝 시 같은 paragraph 안에서도 cs=0 변경
2. **paragraph 간 vpos 연결** — `next.vpos = prev.last.vpos + lh + ls`
3. **paragraph 내 line wrap 시 vpos reset** — `pgy < prev.pgy` 시 새 페이지 시작 영역

`mydocs/tech/document_ir_lineseg_standard.md` 영역 갱신.

### 3.2 Stage D + D-2 — HWP3 파서 IR 표준 완전 정합 인코딩

`src/parser/hwp3/mod.rs` 정정 영역 8개 (Stage A+D 4개 + D-2 보완 4개):

| # | 영역 | 본질 |
|---|------|------|
| 1 | `is_page_break` 영역 보강 | `prev_para_had_flags_break` + `first_pgy_here=0` 케이스 |
| 2 | lh/ls HWP5 분리 인코딩 | `lh = th, ls = th * (ratio - 100) / 100` (시각 줄 높이 정합) |
| 3 | `break_flag` → `tag` bit 누설 제거 | `tag = 0x00060000` 고정 |
| 4 | pgy-based `column_type=Page` 설정 제거 | 자연 wrap 은 typeset 책임 |
| 5 | wrap zone cs/sw 정합 인코딩 | `active_wrap_cs_sw` outer state + 후속 paragraph 정합 채움 |
| 6 | paper-top anchor `acc_vpos` reset | paper-relative + body top 영역 anchor 의 vpos=0 정합 |
| 7 | line_info.break_flag 0x8001 → `column_type=Page` | HWP3 한글97 layout 의 페이지 경계 신호 영역 |
| 8 | paragraph 내 line wrap vpos reset | `pgy[i] < pgy[i-1]` 시 acc_vpos = 0 reset (HWP5 ls[i].vpos=0 영역 정합) |

### 3.3 본질 정합 결정적 검증

**HWP3 / HWP5 / HWPX 3개 포맷 변환본 모두 동일 페이지 수 정합**:

| 파일 | baseline | 정정 결과 | 정합 |
|------|---------|-----------|------|
| `hwp3-sample.hwp` | 16 | **16** | ✅ |
| `hwp3-sample4.hwp` | HWP5 36 | **36** | ✅ + paragraph 시퀀스 정합 |
| `hwp3-sample5.hwp` | HWP5 64 | **64** | ✅✅ **HWP5 완전 정합** |
| `exam_science.hwp` | 4 | **4** | ✅ Task #546 정합 |

**LineSeg 영역의 본질 정합** (sample5):

| 영역 | 본 환경 (정정 후) | HWP5 v2024 | 정합 |
|------|------------------|-----------|------|
| pi=74 ls[0] vpos | 0 | 0 | ✅ |
| pi=74 picture page | 4 | 4 | ✅ |
| pi=75 ls[0..18] cs/sw | 35460 / 15564 | 37164 / 13860 | ✅ wrap zone 안 |
| pi=75 ls[19..20] cs/sw | 0 / 0 | 0 / 51024 | ✅ wrap zone 끝 전환 |
| pi=1213 ls[0] vpos | 72000 | 72000 | ✅ |
| **pi=1213 ls[1] vpos** | **0** | **0** | ✅ **paragraph 내 페이지 reset** |
| pi=1213 ls[2..3] vpos | 1440, 2880 | 1440, 2880 | ✅ |

### 3.4 결정적 회귀 영역 (rebase 후)

| 영역 | 결과 |
|------|------|
| `cargo build` | ✅ |
| `cargo test --lib` | ✅ **1131 passed** (Task #568 흡수) |
| `cargo test --test svg_snapshot` | ✅ 6/6 (HWP5 native 회귀 0) |
| `cargo test --test issue_546` | ✅ Task #546 정합 |
| `cargo test --test issue_554` | ✅ **12/12** (모든 fixture 정합) |
| `cargo clippy` (lib) | ✅ warning 0 |

### 3.5 본 task 영역의 본질 영역 도달

본 stage 로 **Document IR 표준 정합화 본질 영역 완전 도달**:

1. HWP3 / HWP5 / HWPX 3개 포맷 변환본 모두 동일 페이지 수
2. paragraph 시퀀스 정합 (sample4 page 33+34 HWP5 변환본 정합)
3. LineSeg cs/sw/vpos 영역 HWP5 정합 (pi=74/75/1213)
4. wrap zone 영역 정합 (anchor + 후속 paragraph + 영역 끝 전환)
5. orphan 페이지 0
6. 모든 회귀 영역 정합

### 3.6 잔존 영역 (별도 task scope)

**조판부호 보기 영역의 시각 차이** (한컴 변환기 휴리스틱 영역):

dump 분석 영역 (sample4 pi=960):

| 영역 | HWP3 native | HWP5 변환본 |
|------|-------------|-------------|
| TAB 개수 | 4 | 2 |
| ParaShape indent | 660 HU | 1320 HU (× 2) |

한컴 변환기 (HWP3 → HWP5) 가 paragraph 시작 leading TAB 시퀀스 중 일부를 ParaShape
indent 영역으로 흡수하는 휴리스틱. 본 환경 양쪽 파서 모두 raw binary 영역 정합 표현 —
일반 보기 시각 가로 위치는 정합. 시각 차이 (조판부호 영역의 TAB 마커 개수) 는
raw binary 자체 영역의 차이 영역.

본 영역의 본질 — 한컴 변환기 paragraph indent 변환 휴리스틱 reverse engineering 영역
— **별도 task 영역** (본 task 의 Document IR 표준 정합화 본질 영역 외).

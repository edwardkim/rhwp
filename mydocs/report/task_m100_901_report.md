# Task #901 최종 보고서 — pic2.hwp 페이지 1+2 한컴 정합

**이슈**: [edwardkim/rhwp#901](https://github.com/edwardkim/rhwp/issues/901)
**브랜치**: `local/task901`
**Base**: `upstream/devel @ 3efbaeda`
**Stage**: 1+2+3+5+6+7+8+10+11 (9 commits)
**상태**: 완료 ✅

## 1. 증상 (Original Issue)

`samples/pic2.hwp` (HWP5, 2 페이지) 의 layout 이 한컴 viewer 와 매우 다름:
- paragraph 0 "우리나라" 위치 좌측 (잘못) — 한컴: 우측 세로
- paragraph 1 "대한민국" 위치 좌측 — 한컴: 우측
- 본문 paragraph 들의 vertical drift
- 의자 그림 배치 차이
- paragraph 22 "올해 확정·지급된 PS" overflow

## 2. ROOT CAUSE 다단계 분석

진단을 통해 5 개의 독립된 ROOT CAUSE 식별:

### 2.1 wrap zone cs/sw 분기 미발동 (Stage 1+2+3)
- `paragraph_layout.rs` 의 `effective_col_x/w` 분기가 `avail < col_w - 200` 조건만 사용
- paragraph 0 의 8 line_seg 중 일부 (cs=39123 sw=3397, avail=col_w) 가 분기 미진입
- `typeset.rs` 의 wrap_anchor 매칭이 sw 정확 일치 요구 → paragraph 1 (cs=24470 sw=18050) 미매칭

### 2.2 wrap zone phantom line 의 y advance (Stage 5+6+7)
- paragraph 0 의 8 line_segs 중 4개 (cs=24470 LEFT narrow zone) 는 한컴이 텍스트 미배치 — empty/whitespace-only runs
- paragraph_layout 가 이 phantom line 도 y advance → vertical 2배 누적
- pagination engine 의 height 계산도 동일 문제 → page break 잘못

### 2.3 TopAndBottom anchor paragraph 위치 (Stage 8)
- iris (TopAndBottom wrap) 의 anchor paragraph 19 가 picture 아래에서 시작
- 한컴: paragraph 19 가 picture 위에 위치, 후속 paragraph 가 picture 아래
- `calculate_shape_reserved_heights` 가 anchor paragraph 의 fit 검증 없이 무조건 jump

### 2.4 flow-around 후 vpos correction 간섭 (Stage 10)
- iris Shape item 처리가 paragraph 19-20 사이 `vpos_lazy_base = None` reset
- post_jump 의 base 조정 무효화 → paragraph 22 가 page 2 boundary 까지 push

### 2.5 vpos correction base 계산 불일치 (Stage 11)
- anchor_first_vpos 를 base 로 사용 시 paragraph 22 가 +130 px 어긋남
- 정확한 base = `next_para_vpos - (bottom_y - col_area.y) * 7200/dpi`

## 3. Fix 적용 — 9 Stage Sequential

| Stage | Commit | 영역 | 핵심 변경 |
|-------|--------|------|-----------|
| 1 | `bac1dca5` | paragraph_layout.rs | `cs_significant` 분기 조건 추가 |
| 2+3 | `d973e9ff` | typeset.rs | `cs_only_match` wrap_anchor 매칭 |
| 5 | `d72def4d` | paragraph_layout.rs | empty-runs y advance skip |
| 6 | `58c94169` | paragraph_layout.rs | whitespace-only skip 확장 |
| 7 | `eb8cdafe` | typeset.rs | format_paragraph 동일 skip 정합 |
| 8 | `51890d12` | layout.rs + shape_layout.rs | TopAndBottom flow-around (anchor above picture) |
| 10 | `6b5e5954` | layout.rs | vpos_lazy_base flow-around 정합 |
| 11 | `fd45a691` | layout.rs | base 정밀 계산 (next_para_vpos peek) |

## 4. 최종 결과

### 4.1 시각 정합 비교

pic2.hwp 정합 progress (Baseline → 최종):

| 항목 | Baseline | 최종 |
|------|----------|------|
| paragraph 0 "우/리/나/라" | 좌측 좌측 좌측 좌측 (잘못) | **우측 세로** (한컴 정합) ✅ |
| paragraph 0 line gap | 119 px | **60 px** (한컴 정합) ✅ |
| paragraph 1 "대한민국" | 좌측 | **우측** (한컴 정합) ✅ |
| paragraph 7 "SK하이닉스" y | 788 | 571 |
| paragraph 11 "올해 확정" | page 2 | **page 1** ✅ |
| paragraph 19 page 2 | iris 아래 | **iris 위** ✅ |
| paragraph 22 page 2 y | 1045 (overflow) | **545** (iris 직하, 한컴 정합) ✅ |
| 의자 그림 배치 | 잘못 | 정합 ✅ |

### 4.2 회귀 검증

- ✅ `cargo test --release --all-targets`: **1402 passed, 0 failed** (전 Stage 동일)
- ✅ pic2.hwp 페이지 수 유지 (2 페이지)
- ✅ pic2-2018.hwp (한컴오피스 2018 재저장) 동일 결과
- ✅ 모든 sample SVG 페이지 수 유지

## 5. 리버스 엔지니어링 발견

수행 중 발견된 HWP 파일 포맷 인사이트 (mydocs/working/task_m100_901_stage2.md, stage5.md):

- HWPTAG_PARA_LINE_SEG 의 `flags` bit 에 wrap zone 표시 비트 없음 (한글5.0 스펙 표 62)
- HWPX `<hp:pic>` 의 `textWrap="SQUARE"` 도 wrap zone 좌표 없음
- 한컴 (2010 / 2018 / 2022) 모든 버전이 paragraph 7 같은 self-picture wrap 의 wrap zone 을 사전 인코딩하지 않음
- HWPX/owpml 도 동일 — wrap zone 은 viewer 의 **runtime 계산** 영역

이 인사이트로 paragraph 7 본문 wrap 영역에 대한 fix 방향이 바뀜:
- 초기 진단: runtime wrap engine 신규 구현 (수일~수주)
- 실제: file format 의 line_seg cs/sw 가 이미 wrap zone 인코딩 (paragraph 8~15 의 cs=0 sw=26140), Stage 2+3 cs_only_match 로 활용 가능

## 6. 잔존 차이

- ~10 mm 공통 vertical offset (`col_area.y` 의 picture vertOffset 영향) — 별도 issue 후보
- HWPX parser bug (compose_lines packing all chars to line 0) — 별도 issue 후보

이들은 본 Task #901 의 scope 를 벗어나는 layout engine 의 기존 영역이며, 향후 별도 task 로 분리 처리 가능.

## 7. 산출물

- **수행 계획서**: `mydocs/plans/task_m100_901.md`
- **구현 계획서**: `mydocs/plans/task_m100_901_impl.md`, `task_m100_901_impl_v2.md`
- **Stage 보고서**: `mydocs/working/task_m100_901_stage{1,2,5,6,7,10,11}.md`
- **최종 보고서**: 본 파일
- **fixture**: `samples/pic2.hwp`, `samples/pic2.hwpx`, `samples/pic2-2018.hwp`, `samples/pic2.owpml`
- **권위 자료**: `pdf/pic2.pdf` (한컴 2022 PDF — 시각 정합 권위)

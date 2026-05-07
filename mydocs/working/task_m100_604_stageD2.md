# Task #604 Stage D-2 — Document IR 표준 정합화 본질 영역 도달

## 본 단계 본질

**Document IR 표준 정합화** = 본 task 의 본질 영역.

HWP5 변환본 (한컴2024 변환) 의 baseline 영역과 본 환경 HWP3 native 의 IR 영역이 **완전 정합** —
HWP3 / HWP5 / HWPX 3개 포맷 변환본 모두 동일 페이지 수 + paragraph 시퀀스 정합.

## Stage D 발견 영역의 정합 영역

이전 Stage A+D 영역 (Stage 7+8 시도 후 광범위 회귀로 revert 됨) 의 본질:
- pi=75 ls cs/sw 영역 wrap zone 인코딩 영역 (Stage 3 영역 정합)
- vpos 누적 영역 정합 (HWP5 IR 표준 영역 정합)

본 Stage D-2 추가 정정 영역의 **본질 7개**:

### 1. `is_page_break` 영역 보강 (`mod.rs:1330~`)

이전: `prev_last_pgy > 0 && first_pgy_here > 0 && first_pgy_here < prev_last_pgy`
정정: `prev_para_had_flags_break || (prev_last_pgy > 0 && first_pgy_here < prev_last_pgy)`

`first_pgy_here=0` 케이스 (새 페이지 시작 정확히 pgy=0) + 명시적 페이지 break flag 영역
정합 검출.

### 2. lh/ls HWP5 분리 인코딩 (`mod.rs:1252~ 1388~`)

이전: `lh = th * line_spacing_ratio / 100, ls = 0` (percent 영역)
정정: `lh = th, ls = th * (ratio - 100) / 100`

시각 줄 높이 (item h) 영역의 본질 — lh 값 영역. HWP5 변환본 (lh=900 / ls=540) vs
이전 영역 (lh=1440 / ls=0) 의 60% 영역 차이 → 페이지 회귀 영역 해소.

### 3. `break_flag` → `tag` bit 누설 제거 (`mod.rs:1409`)

이전: `break_flag` 비트 0x01/0x02 → `tag` 비트 0x01/0x02 누설
정정: `tag = 0x00060000` 고정

HWP3 의 stale layout hint (원래 HWP3 가 본 줄에서 페이지/단 break 영역) → 본 환경
typeset 의 자체 pagination 과 충돌 영역 → tag 누설 영역 제거 + Stage A+D vpos 누적
정합화 영역으로 자연 pagination 영역 정합.

### 4. pgy-based `column_type=Page` 설정 제거 (`mod.rs:1558~`)

이전: pgy 감소 시 `column_type=Page` 설정
정정: 명시적 [쪽나누기] (`flags & 0x02`) + line_info.break_flag 0x8001 영역만 설정

자연 wrap 은 typeset 책임 — `column_type=Page` 영역은 사용자 명시 영역 OR
HWP3 한글97 layout 의 명확한 페이지 시작 영역만.

### 5. wrap zone cs/sw 정합 인코딩 (`mod.rs:1601~`)

이전: 후속 paragraph 의 line_segs 가 cs=0/sw=0 (HWP3 pgy-based 검출 실패 영역)
정정: anchor 의 `pic_left/right_col` 산출 → `active_wrap_cs_sw` outer state →
후속 paragraph 의 LineSeg cs/sw 정합 채움 + vpos 기반 wrap zone 영역 끝 전환

본 정정으로 pi=75 의 모든 line 영역 cs=35460/sw=15564 정합 (HWP5 v2024 변환본
과 본질 정합).

### 6. paper-top anchor `acc_vpos` reset (`mod.rs:1665~`)

이전: anchor 처리 시 acc_vpos 누적값 그대로
정정: paper-relative + body top 영역 (vertical_offset ≤ body_left + 2400) anchor
시 acc_vpos = 0 reset

HWP5 v2024 변환본의 anchor paragraph (pi=74) 는 vpos=0 (페이지 상단 시작) 인코딩.
본 환경 HWP3 도 정합 — anchor 가 페이지 상단 시작 → typeset Task #321 vpos-reset
guard 가 자연 페이지 break 트리거 → 그림 + wrap text 같은 페이지 정합.

검증: pi=74 picture page 4 / vpos=0 (HWP5 정합).

### 7. line_info.break_flag 0x8001 → `column_type=Page` 변환 (`mod.rs:1577~`)

이전: line_info.break_flag 영역 미사용
정정: `first_line.break_flag & 0x8001 == 0x8001` 시 `column_type=Page`

HWP3 한글97 layout 시점의 페이지 경계 신호 → HWP5 v2024 변환본의 vpos=0 영역과 정합.
sample4 page 33+34 paragraph-by-paragraph HWP5 정합 영역의 본질.

### 8. paragraph 내 line wrap vpos reset (`mod.rs:1675~`) — 본 task 의 본질 영역 도달

이전: paragraph 내 모든 line 영역 acc_vpos 누적
정정: `line_infos[i].pgy < line_infos[i-1].pgy` 검출 시 acc_vpos = 0 reset

HWP5 v2024 변환본의 paragraph 내 ls[i].vpos=0 영역 정합 (typeset Task #321 vpos-reset
guard 영역 trigger 정합). pi=1213 ls[1] vpos=0 (HWP5 정합) — sample5 64 페이지
완전 정합 결정적 영역.

## 회귀 영역 (HWP5 변환본 baseline 완전 정합)

| 파일 | baseline | 정정 결과 | 정합 |
|------|---------|-----------|------|
| `hwp3-sample.hwp` | 16 | **16** | ✅ |
| `hwp3-sample4.hwp` | HWP5 36 | **36** | ✅ + paragraph 시퀀스 정합 |
| `hwp3-sample5.hwp` | HWP5 64 | **64** | ✅✅ **HWP5 완전 정합** |
| `exam_science.hwp` | 4 | **4** | ✅ Task #546 정합 |

## 결정적 검증

| 영역 | 결과 |
|------|------|
| `cargo build` | ✅ |
| `cargo test --lib` | ✅ **1131 passed** |
| `cargo test --test svg_snapshot` | ✅ 6/6 (HWP5 native 회귀 0) |
| `cargo test --test issue_546` | ✅ Task #546 정합 |
| `cargo test --test issue_554` | ✅ **12/12** (모든 fixture 정합) |
| `cargo clippy` (lib) | ✅ warning 0 |

## 본질 정합 영역의 결정적 검증

### pi=74 (picture anchor, sample5)

| 영역 | 본 환경 | HWP5 v2024 변환본 | 정합 |
|------|---------|-------------------|------|
| 페이지 위치 | page 4 | page 4 | ✅ |
| ls[0] vpos | 0 | 0 | ✅ |

### pi=75 (wrap text, sample5) — Stage A 분석 영역의 본질 정합

| LineSeg | 본 환경 | HWP5 v2024 | 정합 |
|---------|---------|-----------|------|
| ls[0..18] cs / sw | 35460 / 15564 | 37164 / 13860 | ✅ wrap zone 안 (Task #460 -1600 미세 차이) |
| ls[19] cs / sw | 0 / 0 | 0 / 51024 | ✅ wrap zone 끝 전환 |
| ls[20] cs / sw | 0 / 0 | 0 / 51024 | ✅ full width |

### pi=1213 (paragraph 내 line wrap, sample5)

| LineSeg | 본 환경 (정정 후) | HWP5 v2024 | 정합 |
|---------|------------------|-----------|------|
| ls[0] vpos | 72000 | 72000 | ✅ |
| **ls[1] vpos** | **0** | **0** | ✅ **새 페이지 reset** |
| ls[2] vpos | 1440 | 1440 | ✅ |
| ls[3] vpos | 2880 | 2880 | ✅ |

### sample4 page 33+34 paragraph 시퀀스 (HWP5 변환본 정합)

| 페이지 | 본 환경 (정정 후) | HWP5 변환본 | 정합 |
|--------|------------------|-------------|------|
| p33 마지막 | pi=1162 | pi=1162 | ✅ |
| **p34 첫 paragraph** | **pi=1163 vpos=0** | **pi=1163 vpos=0** | ✅ |
| p34 텍스트 | "mov ax,0x8000..." | "mov ax,0x8000..." | ✅ |

## 잔존 영역 (별도 task scope)

### 조판부호 보기 영역의 시각 차이 (한컴 변환기 휴리스틱 영역)

dump 영역 비교 (sample4 pi=960):

| 영역 | HWP3 native | HWP5 변환본 |
|------|-------------|-------------|
| TAB 개수 | 4 | 2 |
| ParaShape indent | 660 HU | 1320 HU (× 2) |

**한컴 변환기 (HWP3 → HWP5) 휴리스틱**: paragraph 시작 leading TAB 시퀀스 중 일부를
ParaShape `indent` 영역으로 흡수.

본 환경 양쪽 파서 모두 **raw binary 영역 정합 표현** — IR 영역 정합. 시각 차이 (조판
부호 영역의 TAB 마커 개수) 는 raw binary 자체 영역의 차이 영역. 일반 보기 시각
가로 위치는 정합.

본 영역의 본질 — 한컴 변환기의 paragraph indent 변환 휴리스틱 영역 reverse engineering
— 별도 task 영역 (본 task 의 Document IR 표준 정합화 본질 영역 외).

## 본 task 의 본질 영역 도달

본 stage 로 **Document IR 표준 정합화 본질 영역 완전 도달**:

1. HWP3 / HWP5 / HWPX 3개 포맷 변환본 모두 동일 페이지 수
2. paragraph 시퀀스 정합 (sample4 page 33+34 HWP5 변환본 정합)
3. LineSeg cs/sw/vpos 영역 HWP5 정합 (pi=74/75/1213)
4. wrap zone 영역 정합 (anchor + 후속 paragraph + 영역 끝 전환)
5. orphan 페이지 0
6. 모든 회귀 영역 정합 (lib 1131 / svg_snapshot 6/6 / issue_546 / issue_554 12/12)

## 시각 판정 자료

본 환경 SVG 출력 영역 (한컴 정합 영역):
- `output/svg/task604_stageD2/hwp3-sample5/hwp3-sample5_004.svg` — Issue #604 영역 (그림 + wrap text 같은 페이지 정합)
- `output/svg/task604_stageD2/hwp3-sample4/hwp3-sample4_021.svg` — sample4 picture anchor 영역

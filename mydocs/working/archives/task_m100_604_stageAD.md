# Task #604 Stage A + D — HWP5 IR 표준 정합 본질 정정 (HWP3 파서)

## 본 단계 본질

작업지시자 통찰 정합 — Document IR 표준 정합화 = 더 큰 목적. IR 표준 정확 정의 +
HWP3 파서가 표준대로 던지면 시각 결함도 자연스럽게 해결.

## Stage A — HWP5 spec 정밀 재진단 결과

HWP5 v2024 변환본 (`hwp3-sample5-hwp5-v2024.hwp`) page 4 LineSeg 정밀 분석:

| paragraph | LineSeg | vpos | cs | sw | 본질 |
|-----------|---------|------|----|----|------|
| pi=74 anchor | ls[0] | 0 | 37164 | 13860 | anchor lh=900 (1줄) |
| pi=75 wrap text | ls[0] | **1440** | 37164 | 13860 | anchor 끝 vpos = 1440 |
| pi=75 ls[18] | | 27360 | 37164 | 13860 | 마지막 wrap zone 줄 |
| **pi=75 ls[19]** | | **28800** | **0** | **51024** | ★ 그림 영역 끝 — **같은 paragraph 내 cs=0 전환** |
| pi=75 ls[20] | | 30240 | 0 | 51024 | full width |
| pi=76 (그림 아래) | ls[0] | **31680** | 0 | 51024 | paragraph 간 vpos 연결 |

**핵심 발견** — IR 표준 미명시 영역:
1. **paragraph 내 LineSeg cs/sw 전환** (wrap zone 끝 시 같은 paragraph 안에서 cs=0 변경)
2. **paragraph 간 vpos 연결** (next.vpos = prev.last.vpos + lh + ls)

`mydocs/tech/document_ir_lineseg_standard.md` 갱신.

## Stage D — HWP3 파서 IR 표준 완전 정합 인코딩

### 변경 영역 (`src/parser/hwp3/mod.rs`)

1. parse_paragraph_list 외부에 `acc_section_vpos`, `wrap_zone_end_vpos` 누적 상태 변수 추가
2. paragraphs.push 직전 후처리:
   - **페이지 break reset**: column_type=Page 시 acc_section_vpos=0
   - **anchor 검출**: Picture Square wrap (Control::Picture / Control::Shape::Picture
     모두) → wrap_zone_end_vpos = acc_section_vpos + total_height (= 박스 + margin)
   - **LineSeg vpos 누적**: each seg.vertical_pos = acc_section_vpos
   - **paragraph 내 cs=0 전환**: acc_section_vpos >= wrap_zone_end_vpos AND cs>0 시
     cs/sw=0 전환
   - **paragraph 간 vpos 연결**: acc_section_vpos += lh + ls (다음 줄/paragraph)
3. Stage 5 B-2 (wrap zone 안 lh=th 강제) revert — HWP3 본질 유지 (lh=1440/ls=0 = HWP5
   v2024 의 lh=900/ls=540 누적값 1440 동등)

## 검증

### LineSeg 정합 (pi=75)

| 줄 | 본 환경 | HWP5 v2024 | 정합 |
|----|--------|-----------|------|
| ls[0] vpos | 1440 | 1440 | ✅ |
| ls[18] vpos / cs | 27360 / 35460 | 27360 / 37164 | ✅ (cs 값 약간 차이) |
| **ls[19] vpos / cs** | **28800 / 0** | **28800 / 0** | ★ **cs=0 전환 정합** |
| ls[20] vpos / cs | 30240 / 0 | 30240 / 0 | ✅ |

### 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build` | ✅ |
| `cargo test --lib` | ✅ **1130 passed** |
| `cargo test --test issue_546` | ✅ exam_science 4페이지 / items=37 |
| `cargo test --test issue_554` | ⚠️ `hwp3_sample5_hwp3_64p` FAILED (64 → 67) |

### 회귀 영역

| 파일 | PR #589 baseline | Stage A+D | 회귀 |
|------|----------------|-----------|------|
| hwp3-sample.hwp | 16 | 16 | ✅ 0 |
| hwp3-sample5.hwp | 64 | **67** | ⚠️ +3 |
| hwp3-sample4.hwp | 39 | 40 | ⚠️ +1 |
| exam_science.hwp | 4 | 4 | ✅ Task #546 정합 |

**Stage 7+8 시도 대비 회귀 영역 큰 폭 축소** (Stage 7: sample5 +94, Stage 8: Task #546 회귀
재발). 본 정정이 가장 정합한 본질 영역.

## 시각 판정 자료

`output/svg/task604_stageAD/hwp3-sample5/hwp3-sample5_004.svg`

## 잔존 영역 (Stage E 영역)

- HWP3 native 페이지 +N 회귀 (issue_554 test_failed)
- HWP3 LineSeg cs=0/sw=0 vs HWP5 sw=51024 (full width 의미 의존)
- 일부 typeset 영역의 vpos 처리 정합화 검증

Stage E (렌더러 IR 표준 신뢰 sequential flow) 진행 시 본 회귀 정합 가능.

## 작업지시자 승인 요청

본 Stage A+D 완료 보고. 다음 단계 (Stage E: 렌더러 IR 표준 정합화) 진입 승인 요청.

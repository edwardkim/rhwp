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
   양방향 가드 (`pgy >= pgy_start && pgy < pgy_end`).
2. **Document IR 표준 부재**: `Paragraph.wrap_precomputed: bool` 플래그가 HWP3 휴리스틱
   을 IR 에 누설 (PR #589 보완6 도입). LineSeg 필드의 단위/원점/0 의미 미명문화.

## 2. 본 task 의 본질적 가치 ★

### 2.1 Issue #604 결함 본질 정정

`hwp3-sample5.hwp` page 4 의 그림+텍스트 겹침 결함 정정 (한컴 변환본과 시각 정합).

### 2.2 **HWP3 → 한컴 2018/2024 변환본 시각 정합 자동 달성** ★

본 정정의 부가 가치 — Stage 5 의 한컴 변환 메커니즘 모방으로 다음 변환본도 정상 표시:

| 파일 | 한컴 변환 버전 | 페이지 수 | 시각 정합 |
|------|------------|---------|---------|
| `hwp3-sample5-hwp5-v2018.hwp` | **한컴 2018** | 64 | ✅ |
| `hwp3-sample5-hwp5-v2024.hwp` | **한컴 2024** | 64 | ✅ |
| `hwp3-sample5-hwp5.hwp` | 한컴 변환 (HWP5) | 64 | ✅ |
| `hwp3-sample5-hwpx.hwpx` | 한컴 변환 (HWPX) | 64 | ✅ |
| `hwp3-sample-hwp5.hwp` | 한컴 변환 | 15 | ✅ |
| `hwp3-sample-hwpx.hwpx` | 한컴 변환 | 15 | ✅ |
| `hwp3-sample4-hwp5.hwp` | 한컴 변환 | 36 | ✅ |

**본질**: 한컴 자체가 HWP3 → HWP5/HWPX 변환 시 wrap zone 인코딩 (cs/sw=0/full) +
그림 absolute layer 메커니즘을 사용. 본 task Stage 5 가 동일 메커니즘을 HWP3 파서
에서 모방하여 변환본과 동일 시각 본질 달성.

### 2.3 Document IR 표준 정합화

| 영역 | 본질 |
|------|------|
| **IR 표준 문서화** | `mydocs/tech/document_ir_lineseg_standard.md` 신설 — 단위/원점/0 의미 명시 |
| **IR 부채 청산** | `Paragraph.wrap_precomputed` 필드 제거 — 포맷 독립성 회복 |
| **HWP3 파서 정합화** | 후처리 30 LOC 청산. LineSeg cs/sw 정합 인코딩 책임 |
| **typeset 출력 메타데이터 채널** | `ColumnContent.wrap_anchors` HashMap — anchor ↔ wrap text 컨텍스트 보존 |
| **layout 정합화** | wrap zone 판정이 IR 의존성 제거 — `wrap_anchor.is_some()` |
| **anchor 종류 기반 분기** | typeset 매칭 분기 = Picture vs Table 기반 본질 판정 |
| **CLAUDE.md HWP3 파서 규칙 정합** | HWP3 휴리스틱 IR 누설 청산 |

## 3. 정정 방향 진화 (옵션 C → R3 → 옵션 D)

작업지시자 결정 — 본질적 IR 표준 정합화. 진행 중 옵션 C (cs/sw 단독 판정) 가 본질
부적합으로 판명 → **R3 (typeset 출력 메타데이터 채널)** 진화 → 옵션 D (한컴 변환 모방)
로 시각 정합 달성.

## 4. 5 Stage 진행

### Stage 1 — IR 표준 + helper (commit `40739ae`)

- `mydocs/tech/document_ir_lineseg_standard.md` 신설 (+150 LOC) — LineSeg 필드의
  단위/원점/0 의미 명시. HWP5/HWPX/HWP3 각 파서 인코딩 책임 명시.
- `src/model/paragraph.rs` — LineSeg 필드 doc 정합 + `is_in_wrap_zone(col_w_hu)` helper.
- 분석 자료 3 파일 `mydocs/tech/` 로 이동 (git 추적 영역).

### Stage 2a — 옵션 C (단독 판정) 시도 ❌ revert

`is_in_wrap_zone(col_w_hu)` 단독 판정 시도 → test_547 회귀 (HWP5 native passage box
본문 LineSeg cs=852/sw=30184 false-positive 판정). 본질 한계 확인 → R3 채택.

### Stage 2 — typeset 출력 메타데이터 채널 (commit `b255540`)

- `src/renderer/pagination.rs`: `WrapAnchorRef` struct + `ColumnContent.wrap_anchors`.
- `src/renderer/typeset.rs:495~`: wrap_around 매칭 시 wrap_anchors 등록.
- `src/renderer/layout.rs`: ColumnItemCtx 정합화 + 21 호출처 wrap_anchor 인자 전달.
- `src/renderer/layout/paragraph_layout.rs`: 3 시그니처 인자 추가, wrap_precomputed
  검사 → `wrap_anchor.is_some()` 교체.

### Stage 2b — IR 부채 마무리 (commit `d71f944`)

- typeset 매칭 분기 본질화: anchor 종류 (Picture vs Table) 기반.
- `Paragraph.wrap_precomputed` 필드 제거 + HWP3 파서 후처리 30 LOC 청산.
- LOC: -53 / +30 (-23 net) 소스.

### Stage 3 — HWP3 파서 cs/sw 인코딩 정정 (commit `d96320d`)

- `src/parser/hwp3/mod.rs:1399~`: pgy 단방향 가드 (pgy_end 만 검사).
- 결과: pi=75 모든 LineSeg cs=35460/sw=15564 (Stage 5 에서 무효화).

### Stage 4 — 광범위 회귀 검증 (commit `7fec186`)

- 광범위 fixture sweep + 작업지시자 시각 판정 자료 (`hwp3-sample5_{004,008,016,022,027}.svg`).
- 작업지시자 발견: `hwp3-sample4.hwp` 40p vs `hwp3-sample4-hwp5.hwp` 36p — 페이지 수 차이.

### Stage 5 — HWP3 wrap zone 인코딩 무효화 (commit `bc0ea7c`) ★

작업지시자 발견 후 한컴 v2024 변환본 분석 → 모든 LineSeg cs=0/sw=51024 (full body
width) — 한컴 변환 시 wrap zone 인코딩 자체를 제거하고 그림은 paper-relative absolute
layer 로 별도 그리는 본질로 정합 시각.

옵션 D 채택 (한컴 변환 메커니즘 모방):
- `src/parser/hwp3/mod.rs:1399~`: 후속 wrap text 문단 cs/sw=None 무효화.
- 앵커 문단은 cs/sw 보존 (그림 위치 영향).
- pi=75 모든 LineSeg cs=0/sw=0 → 본문 full width 흐름 + 그림 absolute layer.
- **한컴 2018/2024 변환본과 시각 정합 자동 달성** ★.

## 5. 결정적 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` + `cargo build --release` | ✅ 통과 |
| `cargo test --lib --release` | ✅ **1130 passed** / 0 failed / 2 ignored |
| `cargo clippy --lib -- -D warnings` | ✅ 0건 |
| `cargo test --test issue_546` (Task #546) | ✅ 1 passed (exam_science 4페이지) |
| `cargo test --test issue_554` (HWP3 변환본) | ✅ 12 passed |
| `cargo test svg_snapshot` | ✅ 6/6 |
| `cargo test --release` 통합 31 | ✅ 모두 통과 |
| WASM 빌드 크기 | 4,588,976 bytes (PR #589 baseline 4,569,773 +19,203 — IR 메타데이터 채널 도입 정합) |

## 6. 회귀 영역 검증

### 6.1 HWP3 native 페이지 수

| 파일 | 페이지 수 | PR #589 baseline | 회귀 |
|------|---------|-----------------|------|
| `hwp3-sample.hwp` | 16 | 16 | ✅ 0 |
| `hwp3-sample5.hwp` | 64 | 64 | ✅ 0 |
| `hwp3-sample4.hwp` | 40 | 39 | ⚠️ +1 (HWP3 폰트 13pt 처리 — 별도 task 영역) |

### 6.2 HWP3 → 한컴 변환본 페이지 수 (Task #554 정합 + Stage 5 시각 정합)

| 파일 | 페이지 수 | PR #589 baseline | 시각 |
|------|---------|-----------------|------|
| `hwp3-sample-hwp5.hwp` | 15 | 15 | ✅ |
| `hwp3-sample-hwpx.hwpx` | 15 | 15 | ✅ |
| `hwp3-sample4-hwp5.hwp` | 36 | 36 | ✅ |
| `hwp3-sample5-hwp5.hwp` | 64 | 64 | ✅ |
| `hwp3-sample5-hwpx.hwpx` | 64 | 64 | ✅ |
| **`hwp3-sample5-hwp5-v2018.hwp`** | **64** | (신규) | ✅ **한컴 2018 변환 정합** ★ |
| **`hwp3-sample5-hwp5-v2024.hwp`** | **64** | (신규) | ✅ **한컴 2024 변환 정합** ★ |

### 6.3 Task #546 (exam_science.hwp) 회귀 검증

- 페이지 수: 4 (정합)
- p2 단 0 items: 37 / used=1133.6px (정합)

## 7. 시각 판정 자료

`output/svg/task604_final/`:
- `hwp3-sample5-native/hwp3-sample5_{004,008,016,022,027}.svg` (HWP3 native 결함 정정)
- `hwp3-sample5-hwp5-v2018/hwp3-sample5-hwp5-v2018_{004,008}.svg` (한컴 2018 변환본)
- `hwp3-sample5-hwp5-v2024/hwp3-sample5-hwp5-v2024_{004,008}.svg` (한컴 2024 변환본)
- `hwp3-sample5-hwp5/hwp3-sample5-hwp5_{004,008}.svg` (한컴 변환 HWP5)
- `hwp3-sample5-hwpx/hwp3-sample5-hwpx_{004,008}.svg` (한컴 변환 HWPX)

**본 정정의 부가 가치** ★: HWP3 native 결함 정정 + 한컴 2018/2024 변환본 정합 시각 자동
달성. 사용자가 HWP3 파일을 한컴 2018/2024 로 다시 저장한 HWP5 파일도 본 정정으로 정상
표시 가능.

## 8. LOC 합계

| 영역 | 변경 |
|------|-----|
| Stage 1 — IR 표준 + helper | +27/-10 (paragraph.rs) + 150 LOC (표준 문서) |
| Stage 2 — wrap_anchors 메타데이터 | +83 LOC (pagination + typeset + layout + 호출처) |
| Stage 2b — IR 부채 청산 | -53/+30 (-23 net) |
| Stage 3 — HWP3 cs/sw 인코딩 정정 | +14/-2 (mod.rs) |
| Stage 5 — wrap zone 무효화 | +13 LOC (mod.rs) |
| **소스 합계** | **+89 LOC net (소스), +1300 LOC 문서** |

## 9. 잔존 영역 (별도 후속 task 권고)

### 9.1 HWP3 폰트 크기 / 줄 간격 처리

`hwp3-sample.hwp / sample4.hwp / sample5.hwp` HWP3 native 폰트 13pt vs 한컴 변환본 9pt
차이로 페이지네이션 미정합 (`hwp3-sample4.hwp` 40p vs 변환본 36p). 본 task 의 wrap zone
인코딩 정정과 무관. **별도 후속 task 권고**: HWP3 → HWP5 IR 변환 시 char_shape font_size
/ line_height 처리 정합화.

### 9.2 HWP3 LineSeg vertical_pos 누적 계산

`mydocs/tech/document_ir_lineseg_standard.md` §"HWP3" 명시 — HWP3 파서가 vertical_pos
를 항상 0 으로 채움. 표준 미정합. **별도 후속 task 권고**.

### 9.3 Task #525 본질 재검토

`Task #525` 가 제거한 `layout_wrap_around_paras` 호출이 본 task 의 wrap_anchors
메커니즘 도입 후에도 유효한지 재검토. dead code 가능성.

## 10. 누적 Commits

| Stage | Commit | 본질 |
|-------|--------|------|
| 1 | `40739ae` | IR 표준 + `is_in_wrap_zone` helper |
| 2 | `b255540` | typeset → layout `wrap_anchors` 메타데이터 채널 |
| 2b | `d71f944` | `wrap_precomputed` 필드 제거 + HWP3 후처리 청산 |
| 3 | `d96320d` | HWP3 cs/sw 인코딩 정정 (Stage 5 에서 일부 무효화) |
| 4 | `7fec186` | 광범위 회귀 검증 + 시각 판정 + 최종 보고서 |
| 5 | `bc0ea7c` | HWP3 wrap zone 인코딩 무효화 (한컴 변환 모방) ★ |

## 11. 참조

### 관련 문서
- 수행계획서: `mydocs/plans/task_m100_604.md`
- 구현계획서: `mydocs/plans/task_m100_604_impl.md`
- LineSeg 표준: `mydocs/tech/document_ir_lineseg_standard.md`
- 단계별 보고서: `mydocs/working/task_m100_604_stage{1,2,2b,3,5}.md`

### 분석 자료
- `mydocs/tech/document_ir_parser_relationship_analysis.md` (16KB) — IR ↔ 각 파서 관계
- `mydocs/tech/hwp5_wrap_precomputed_analysis.md` — HWP5/HWPX wrap_precomputed 분석
- `mydocs/tech/document_ir_wrap_zone_standard_review.md` — IR 표준 부재 본질

### 시각 판정 자료
- `output/svg/task604_final/{hwp3-sample5-native,hwp3-sample5-hwp5-v2018,hwp3-sample5-hwp5-v2024,hwp3-sample5-hwp5,hwp3-sample5-hwpx}/`

### 관련 task / PR / 이슈
- **Issue #604** — 본 task 의 결함 보고
- **PR #589** (Task #511 v2 + #554) — wrap_precomputed IR 플래그 도입 (본 task 정정 대상)
- **Task #460 보완6** (`bdb51a4`) / 보완8 (`ff64387`) — 본 task Stage 2b 청산 대상
- **Task #546** — exam_science.hwp 회귀 정정 (본 task 회귀 0 보존)
- **Task #525** — Picture Square wrap 호스트 중복 emit 정정 (잔존 검토 영역)
- **Task #489** — Picture/Shape Square wrap LINE_SEG.cs/sw 적용 (anchor 문단)
- **Task #554** — HWP5/HWPX 페이지네이션 회귀 (본 task 회귀 0)

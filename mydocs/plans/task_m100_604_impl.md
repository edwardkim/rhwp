# Task #604 — 구현계획서 (Stage 1~4)

수행계획서 승인 후 작성. 본 문서는 각 Stage 의 구체 변경 위치, 알고리즘, 검증 명령을 명시한다.

## Stage 1 — IR 표준 + LineSeg helper

### 1.1 변경 영역

#### A. `mydocs/tech/document_ir_lineseg_standard.md` 신규 생성

```markdown
# Document IR — LineSeg 필드 표준

## 본 문서의 본질

`LineSeg` 는 `Document` IR 의 핵심 줄 단위 레이아웃 필드. HWP5 가 IR origin
이므로 HWP5 LineSeg 인코딩이 표준. HWP5/HWPX/HWP3 모든 파서가 본 표준 의미로
LineSeg 를 채워야 한다.

## 단위

모든 i32 필드는 **HWPUNIT** (1 inch = 7200 HWPUNIT, 1 inch = 25.4mm).

## 필드별 표준

| 필드 | 단위 | 원점 | 의미 |
|------|------|------|------|
| `text_start` | UTF-16 code unit | 문단 시작 | 본 줄이 차지하는 텍스트 시작 위치 |
| `vertical_pos` | HWPUNIT | 페이지 상단 | 페이지 내 흐름 y 좌표 (누적 절대값) |
| `line_height` | HWPUNIT | (없음) | 줄 높이 (line_spacing 포함) |
| `text_height` | HWPUNIT | (없음) | 텍스트 부분의 높이 |
| `baseline_distance` | HWPUNIT | 줄 시작 | 베이스라인까지 거리 |
| `line_spacing` | HWPUNIT | (없음) | 줄간격 |
| `column_start` | HWPUNIT | 단(column) 좌측 | wrap zone x 오프셋. **0 = wrap 없음** |
| `segment_width` | HWPUNIT | (없음) | 줄 너비. **단 너비와 같으면 wrap 없음** |
| `tag` | 비트 플래그 | (없음) | 첫 줄 / 첫 단 등 |

## wrap zone 판정 (포맷 무관)

```rust
impl LineSeg {
    pub fn is_in_wrap_zone(&self, column_width_hu: i32) -> bool {
        self.column_start > 0
            || (self.segment_width > 0 && self.segment_width < column_width_hu)
    }
}
```

## 각 파서의 인코딩 책임

- **HWP5 파서** (`src/parser/body_text.rs`): HWP5 PARA_LINE_SEG 바이너리 레코드 1:1 매핑
- **HWPX 파서** (`src/parser/hwpx/section.rs`): `<hp:lineseg>` XML 속성 매핑
- **HWP3 파서** (`src/parser/hwp3/mod.rs`): HWP3 → HWP5 IR 변환
  - `vertical_pos`: 누적 계산 (Stage C 별도 task 권장 — 본 task 범위 외)
  - `column_start/segment_width`: wrap zone 영역의 모든 줄에 정확히 인코딩

## 본 표준의 정합성 검증

- HWP5 native fixture (form-002, issue-147 등): 인코더 동작과 정합 (회귀 0)
- HWPX native fixture: 동일
- HWP3 native fixture (hwp3-sample.hwp 등): 본 task Stage 3 정정 후 정합

## 변경 이력

- 2026-05-05: Task #604 Stage 1 신규 작성 (Document IR 표준 정의)
```

#### B. `src/model/paragraph.rs` 변경

**B-1.** `LineSeg` 필드 doc 정합 (struct 내 모든 필드 doc 주석 갱신):

```rust
pub struct LineSeg {
    /// 본 줄이 차지하는 텍스트 시작 위치 (UTF-16 code unit, 문단 시작 기준)
    pub text_start: u32,
    /// 페이지 내 흐름 y 좌표 (HWPUNIT, 페이지 상단 기준 누적 절대값)
    pub vertical_pos: i32,
    /// 줄 높이 (HWPUNIT, line_spacing 포함)
    pub line_height: i32,
    /// 텍스트 부분의 높이 (HWPUNIT)
    pub text_height: i32,
    /// 베이스라인까지 거리 (HWPUNIT, 줄 시작 기준)
    pub baseline_distance: i32,
    /// 줄간격 (HWPUNIT)
    pub line_spacing: i32,
    /// wrap zone x 오프셋 (HWPUNIT, 단 좌측 기준). 0 = wrap 없음
    pub column_start: i32,
    /// 줄 너비 (HWPUNIT). 단 너비와 같으면 wrap 없음
    pub segment_width: i32,
    /// 비트 플래그 (첫 줄 / 첫 단 등)
    pub tag: u32,
}
```

**B-2.** `LineSeg::is_in_wrap_zone(col_w_hu)` helper 추가 (impl 블록):

```rust
impl LineSeg {
    /// 페이지의 첫 줄인지 여부 (기존)
    pub fn is_first_line_of_page(&self) -> bool {
        self.tag & 0x01 != 0
    }

    /// 본 줄이 wrap zone (그림/표 옆) 안에 있는지 (포맷 무관 표준).
    /// 표준: `mydocs/tech/document_ir_lineseg_standard.md`
    ///
    /// `col_w_hu`: 단 너비 (HWPUNIT). 본 줄의 segment_width 와 비교.
    pub fn is_in_wrap_zone(&self, col_w_hu: i32) -> bool {
        self.column_start > 0
            || (self.segment_width > 0 && self.segment_width < col_w_hu)
    }
}
```

### 1.2 검증

```bash
cargo build               # 빌드 통과
cargo test --lib          # 1129+ passed (helper 만 추가, 호출처 없음)
cargo clippy --lib -- -D warnings  # 0건
```

### 1.3 단계별 보고서

`mydocs/working/task_m100_604_stage1.md` 작성:
- 변경 파일 목록 + LOC
- 검증 결과
- 다음 단계 (Stage 2) 진입 승인 요청

### 1.4 commit 메시지

```
Task #604 Stage 1: Document IR LineSeg 표준 정의 + is_in_wrap_zone helper

- mydocs/tech/document_ir_lineseg_standard.md 신규 — LineSeg 필드 표준 명시
  (단위 HWPUNIT, vertical_pos 누적 절대값, column_start 0 = wrap 없음 등)
- src/model/paragraph.rs — LineSeg 필드 doc 정합 + is_in_wrap_zone(col_w_hu) helper 추가
- 호출처 변경 없음 (Stage 2 에서 일괄 적용)
```

---

## Stage 2a — wrap_precomputed → is_in_wrap_zone 교체 시도 ❌ revert

옵션 C (cs/sw 만으로 wrap zone 판정) 본질 부적합 — test_547 회귀.
HWP5 native passage box 본문 LineSeg `cs=852, sw=30184` 가 false-positive 판정.
**전체 변경 revert. R3 (Stage 2 신규 본질) 채택.**

---

## Stage 2 — typeset 출력 메타데이터 도입 (R3 본격 진행)

### 2.0 본질

`src/renderer/typeset.rs:478-513` 의 wrap_around state machine 이 anchor ↔ wrap text
매칭 본질 메커니즘. 현재 매칭 결과가 layout 시점에 전달 안 됨 → wrap_precomputed 우회.
**R3**: 매칭 결과를 `ComposedParagraph` 또는 `PageItem::FullParagraph` 메타데이터로
보존 → layout 이 본 메타데이터로 wrap zone 판정 + 정합 렌더.

### 2.1 변경 영역

#### A. ComposedParagraph (또는 PageItem) 에 `wrap_anchor` 필드 추가

위치: `src/renderer/composer.rs` 의 `ComposedParagraph` struct 또는
`src/renderer/pagination.rs` 의 `PageItem::FullParagraph` variant.

```rust
pub struct WrapAnchorRef {
    /// anchor 문단 인덱스 (그림/표를 가진 문단)
    pub anchor_para_index: usize,
    /// anchor 의 wrap zone cs (HWPUNIT)
    pub anchor_cs: i32,
    /// anchor 의 wrap zone sw (HWPUNIT)
    pub anchor_sw: i32,
}

pub struct ComposedParagraph {
    // ... 기존 필드 ...
    /// 본 문단이 wrap text (anchor 그림/표 옆) 인 경우 anchor 참조
    pub wrap_anchor: Option<WrapAnchorRef>,
}
```

직렬화 영향 없음 (typeset 출력은 휘발성 — Document IR 저장 영역 외).

#### B. `src/renderer/typeset.rs:478-513` wrap_around 매칭 시 메타데이터 설정

기존:
```rust
if !para.wrap_precomputed {
    // 어울림 문단 흡수
    st.current_column_wrap_around_paras.push(...);
    continue;
}
// pre-computed: fall through to normal FullParagraph rendering
```

변경:
```rust
// Task #604 Stage 2: wrap_precomputed → is_in_wrap_zone 표준 적용
let col_w_hu = page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
let any_seg_in_wrap = para.line_segs.iter().any(|s| s.is_in_wrap_zone(col_w_hu));
if !any_seg_in_wrap {
    // 어울림 문단 흡수 (cs/sw 모두 0 — wrap zone 없음)
    st.current_column_wrap_around_paras.push(...);
    continue;
}
// any_seg_in_wrap=true: FullParagraph path 가 LineSeg cs/sw 로 직접 처리
```

#### B. `src/renderer/layout/paragraph_layout.rs` 3곳 교체

**B-1.** Line 862 (line_cs_offset 계산):
```rust
// 기존: if para.map(|p| p.wrap_precomputed).unwrap_or(false) { ... }
// 변경: if para.and_then(|p| p.line_segs.get(line_idx))
//          .map(|s| s.is_in_wrap_zone(col_area_w_hu)).unwrap_or(false) { ... }
```

**B-2.** Line 883 (BoundingBox x):
```rust
// 기존: if para.map(|p| p.wrap_precomputed).unwrap_or(false) { col_area.x + ... + line_cs_offset }
// 변경: if para.and_then(|p| p.line_segs.get(line_idx))
//          .map(|s| s.is_in_wrap_zone(col_area_w_hu)).unwrap_or(false) { col_area.x + ... + line_cs_offset }
```

**B-3.** Line 1208 (x_base 계산):
```rust
// 기존: if para.map(|p| p.wrap_precomputed).unwrap_or(false) { col_area.x + ... + line_cs_offset }
// 변경: if para.and_then(|p| p.line_segs.get(line_idx))
//          .map(|s| s.is_in_wrap_zone(col_area_w_hu)).unwrap_or(false) { col_area.x + ... + line_cs_offset }
```

본 위치는 모두 `line_idx` 가 알려진 컨텍스트이므로 line 단위 판정으로 자연스러움.

#### C. `src/renderer/layout.rs` 주석 갱신

- Line 2957: "Task #460 보완6의 wrap_precomputed IR 플래그로" → "Task #604 Stage 2의 LineSeg::is_in_wrap_zone 표준으로"
- Line 3345: 동일 갱신

#### D. `src/model/paragraph.rs` 필드 제거

- 라인 52~55 `wrap_precomputed: bool` 필드 doc + 선언 제거
- 라인 599 `Paragraph::default()` 의 `wrap_precomputed: false,` 제거

#### E. `src/parser/hwp3/mod.rs:1556~` 후처리 제거

기존 보완6 + 보완8 의 `wrap_precomputed = true` 후처리 블록 (약 30 LOC) 제거.

→ 본 task Stage 3 에서 cs/sw 인코딩 자체를 정정하므로 wrap_precomputed 후처리 불필요.

### 2.2 col_w_hu 인자 산출

`paragraph_layout.rs` 의 호출 컨텍스트에는 이미 `col_area_w_hu` 변수 존재 (line 696):
```rust
let col_area_w_hu = px_to_hwpunit(col_area.width, self.dpi);
```
→ 그대로 활용.

`typeset.rs` 의 호출 컨텍스트:
- `page_def.width / margin_left / margin_right` 모두 i32 HWPUNIT 으로 접근 가능

### 2.3 검증

```bash
cargo build
cargo test --lib                  # 1129+ passed
cargo test --test issue_546       # 1 passed (Task #546 회귀 0)
cargo test --test issue_554       # 12 passed
cargo test                        # 통합 모두 통과
cargo clippy --lib -- -D warnings # 0건
```

`samples/exam_science.hwp` 페이지 수 확인:
```bash
cargo run --bin rhwp -- dump-pages samples/exam_science.hwp 2>&1 | grep -c "^=== 페이지"
# 4 (Task #546 정합)
```

### 2.4 단계별 보고서

`mydocs/working/task_m100_604_stage2.md` 작성. 다음 단계 (Stage 3) 진입 승인 요청.

### 2.5 commit 메시지

```
Task #604 Stage 2: 렌더러 wrap_precomputed → is_in_wrap_zone 교체 + 필드 제거

- src/renderer/typeset.rs: 흡수 조건을 line_segs.any(is_in_wrap_zone) 으로 표준화
- src/renderer/layout/paragraph_layout.rs (3곳): line_cs_offset / BoundingBox.x / x_base
  계산 시 wrap_precomputed → seg.is_in_wrap_zone(col_w_hu) 교체
- src/renderer/layout.rs: 주석 Task #604 인용 갱신
- src/model/paragraph.rs: wrap_precomputed 필드 제거 (HWP3 휴리스틱 누설 청산)
- src/parser/hwp3/mod.rs: wrap_precomputed 후처리 30 LOC 제거 (보완6/8 무력화)

Stage 3 에서 HWP3 cs/sw 인코딩 정정으로 본질 결함 정정 예정.
```

---

## Stage 3 — HWP3 파서 cs/sw 인코딩 정정

### 3.1 결함 본질 진단

`src/parser/hwp3/mod.rs:1399-1407`:
```rust
let line_cs_sw = current_zone.and_then(|(cs, sw, pgy_start, pgy_end)| {
    if pic_wrap_zone.is_some() || (linfo.pgy >= pgy_start && linfo.pgy < pgy_end) {
        Some((cs, sw))
    } else {
        None
    }
});
```

**가드 분석:**
- `pic_wrap_zone.is_some()`: 본 문단 자체가 그림 anchor (pi=74) → 모든 줄에 cs/sw 적용 ✓
- `linfo.pgy >= pgy_start && linfo.pgy < pgy_end`: 본 문단의 줄별 pgy 가 wrap zone 범위 안 → 적용

pi=75 의 ls[0~2] 가 실패하는 이유:
- pi=75 자체는 anchor 아님 → `pic_wrap_zone = None`
- 첫 3 줄의 `linfo.pgy` 가 pi=74 anchor 의 wrap zone `pgy_start..pgy_end` 보다 **작음** (그림 시작 y 보다 위에 있는 것으로 판정됨)

### 3.2 진단 도구

```bash
# pi=75 의 line별 pgy 확인
cargo run --bin rhwp -- dump samples/hwp3-sample5.hwp -s 0 -p 75 2>&1 | grep "ls\["
# pi=74 anchor 의 pgy_start/pgy_end 확인 — 코드 추가 필요 (디버그 출력)
```

먼저 진단 출력으로 정확한 pgy 값 비교 후 본질 정정 방향 결정.

### 3.3 정정 방향 (3가지 옵션)

#### 옵션 3a — pgy_start 가드 완화

후속 wrap text 문단의 첫 N 줄 pgy 가 pgy_start 보다 작아도, 같은 페이지 내이고 첫 anchor 직후 문단이면 적용.

```rust
let line_cs_sw = current_zone.and_then(|(cs, sw, pgy_start, pgy_end)| {
    if pic_wrap_zone.is_some()
        || (linfo.pgy < pgy_end)  // pgy_end 만 검사 (pgy_start 가드 완화)
    {
        Some((cs, sw))
    } else {
        None
    }
});
```

**위험**: pgy_end 만 검사하면 wrap zone 시작 전 페이지의 줄에도 cs 적용 가능.
**완화**: `is_anchor_continuation` 가드 추가 — 직전 문단이 anchor (pi-1 == anchor_pi) 인 경우만 완화.

#### 옵션 3b — anchor 직후 첫 wrap text 문단의 pgy 보정

pi=74 anchor 의 wrap zone 시작 이후 첫 wrap text 문단 (pi=75) 의 모든 줄에 cs/sw 적용 (per-line pgy 무시).

```rust
let force_apply_zone = if let Some((anchor_pi, anchor_zone)) = active_wrap_zone_info {
    // 본 문단이 anchor 직후 첫 wrap text 문단인지
    para_idx == anchor_pi + 1 && anchor_zone == current_zone
} else {
    false
};
let line_cs_sw = if force_apply_zone {
    current_zone.map(|(cs, sw, _, _)| (cs, sw))
} else {
    // 기존 per-line pgy 검사
    current_zone.and_then(|(cs, sw, pgy_start, pgy_end)| {
        if pic_wrap_zone.is_some() || (linfo.pgy >= pgy_start && linfo.pgy < pgy_end) {
            Some((cs, sw))
        } else { None }
    })
};
```

**장점**: anchor 직후 wrap text 문단의 모든 줄에 일관 적용.
**위험**: anchor 직후 wrap text 문단이 길어져 wrap zone pgy_end 를 넘는 경우, 마지막 줄들에도 cs 적용되어 결함 가능. → `pgy_end` 가드 유지.

#### 옵션 3c — wrap zone 의 pgy 범위 자체를 정정

본질 진단: pi=74 anchor 의 `pic_wrap_zone` 의 pgy_start 가 어떻게 계산되는지 추적 (`mod.rs:1266` 부근). 만약 pgy_start 가 부정확하다면 산출 로직 정정.

**장점**: 본질적 정정.
**단점**: HWP3 spec 의 pgy 의미 깊은 이해 필요. 다른 wrap zone 시나리오 회귀 가능.

### 3.4 결정 기준

진단 출력 (3.2) 결과로 결정:
- pgy_start 가 pi=74 그림의 paper_y_offset 정합 → 옵션 3a/3b (가드 완화)
- pgy_start 가 부정확 → 옵션 3c (계산 정정)

### 3.5 검증

```bash
cargo run --bin rhwp -- dump samples/hwp3-sample5.hwp -s 0 -p 75 2>&1 | grep "ls\["
# pi=75 모든 ls 가 cs=35460, sw=15564 정합 확인

cargo test --lib
cargo test --test issue_546
cargo test --test issue_554
cargo test
cargo clippy --lib -- -D warnings
```

광범위 fixture sweep 추가:
```bash
for f in samples/hwp3-sample*.hwp; do
    echo "=== $f ==="
    cargo run --bin rhwp -- dump-pages "$f" 2>&1 | grep -c "^=== 페이지"
done
# HWP3 native pagination 회귀 0 확인
```

### 3.6 단계별 보고서

`mydocs/working/task_m100_604_stage3.md` 작성. 다음 단계 (Stage 4) 진입 승인 요청.

### 3.7 commit 메시지

```
Task #604 Stage 3: HWP3 파서 wrap zone cs/sw 인코딩 정정

- src/parser/hwp3/mod.rs:1399-1407 wrap zone pgy 범위 검사 정정 (옵션 3X 채택 본질)
- pi=75 (그림 anchor pi=74 의 wrap text) 의 첫 3 줄 cs=0/sw=0 결함 정정
  → 모든 줄 cs=35460, sw=15564 정합

검증:
- pi=75 cs/sw 인코딩 정합 (dump 비교)
- HWP3 native pagination 회귀 0
- Task #546 회귀 0
- Task #554 회귀 0
```

---

## Stage 4 — 광범위 회귀 검증 + 시각 판정

### 4.1 결정적 검증

```bash
cargo build --release
cargo test --lib --release        # 1129+ passed
cargo test --release              # 통합 모두 통과
cargo clippy --lib -- -D warnings # 0건
```

### 4.2 회귀 영역 검증

```bash
# Task #546 정합
cargo run --bin rhwp -- dump-pages samples/exam_science.hwp 2>&1 | grep -c "^=== 페이지"
# 4 (정합)
cargo run --bin rhwp -- dump-pages samples/exam_science.hwp -p 1 2>&1 | grep "단 0"
# 단 0 (items=37, ...) (정합)

# Task #554 정합
cargo test --test issue_554       # 12 passed

# HWP3 native 페이지 수 (PR #589 baseline)
for f in hwp3-sample.hwp hwp3-sample5.hwp; do
    cargo run --bin rhwp -- dump-pages "samples/$f" 2>&1 | grep -c "^=== 페이지"
done
# 16, 64 (PR #589 정합)
```

### 4.3 시각 판정 SVG 출력

```bash
mkdir -p output/svg/task604_after/hwp3-sample5/
for p in 3 7 15 21 26; do
    cargo run --bin rhwp -- export-svg samples/hwp3-sample5.hwp -p $p \
        -o output/svg/task604_after/hwp3-sample5/
done

# HWP5 변환본 (부가 가치 영역)
mkdir -p output/svg/task604_after/hwp3-sample5-hwp5/
for p in 3 7 15 21 26; do
    cargo run --bin rhwp -- export-svg samples/hwp3-sample5-hwp5.hwp -p $p \
        -o output/svg/task604_after/hwp3-sample5-hwp5/
done
```

작업지시자 시각 판정 ★ 영역:
- `hwp3-sample5_004.svg` page 4 — pi=75 wrap text 그림 우측 정합 (한컴뷰어 정합)
- 동일 패턴 page 8/16/22/27
- `hwp3-sample5-hwp5_004.svg` (HWP5 변환본) — 부가 가치, 동일 정합 기대

### 4.4 광범위 fixture sweep

```bash
# golden SVG 회귀 검사 (form-002, issue-147, issue-157, issue-267, table-text)
cargo test --test svg_snapshot
# 6/6 정합 (PR #589 baseline)
```

### 4.5 최종 보고서

`mydocs/report/task_m100_604_report.md` 작성:
- Stage 1~4 완료 본질 요약
- 변경 파일 + LOC 합계
- 결정적 검증 결과
- 회귀 영역 검증 결과
- 시각 판정 자료 (SVG 경로)
- IR 표준 도입 의의 (포맷 일관성, 부채 청산, 신규 컨트리뷰터 진입 장벽 감소)
- 별도 후속 task 권고:
  - HWP3 파서 vertical_pos 누적 계산 (별도 task)
  - Task #525 본질 재검토 (별도 task)
  - HWP5/HWPX wrap text 추가 광범위 검증 (별도 task)

### 4.6 orders 갱신

`mydocs/orders/20260505.md` 의 Task #604 entry 상태 → **완료** 갱신.

### 4.7 commit 메시지

```
Task #604 Stage 4: 광범위 회귀 검증 + 시각 판정 + 최종 보고서

- 결정적 검증: cargo test --lib 1129+ / svg_snapshot 6/6 / clippy 0 / release build
- 회귀 영역 검증: Task #546 정합 (exam_science 4 페이지) / Task #554 12 / HWP3 64 페이지
- 시각 판정 자료: output/svg/task604_after/{hwp3-sample5,hwp3-sample5-hwp5}/
- 최종 보고서: mydocs/report/task_m100_604_report.md
- orders 갱신: mydocs/orders/20260505.md Task #604 완료
```

---

## 위험 매트릭스 (구현 단계별)

| Stage | 위험 | 완화 |
|-------|------|------|
| 1 | helper 시그니처 부적합 | col_w_hu 만 인자로 받음 (호출 컨텍스트에서 자연스러움) |
| 2 | wrap_precomputed 호출처 누락 | grep -rn "wrap_precomputed" 으로 일괄 식별 |
| 2 | typeset.rs col_w_hu 산출 부정확 | page_def.width - margin_left - margin_right (HWPUNIT 단위 통일) |
| 2 | HWP3 wrap_precomputed 후처리 제거로 일시 회귀 | Stage 3 에서 cs/sw 인코딩 정정으로 즉시 회복 |
| 3 | 옵션 3a/3b/3c 결정 — 진단 결과에 의존 | Stage 3.2 진단 단계 명시 + 결정 기준 명시 (3.4) |
| 3 | pgy 가드 완화로 false-positive | `is_anchor_continuation` 가드 추가 |
| 4 | 광범위 fixture 회귀 발견 | 회귀 영역별 분리 정정 (별도 stage 추가 가능) |

## 작업지시자 승인 요청 사항

본 구현계획서 (Stage 1~4 구체 알고리즘 + 검증 명령 + 위험 매트릭스) 에 대한 승인 요청.

승인 후 Stage 1 부터 진행. 각 Stage 완료 시 단계별 보고서 작성 → 추가 승인 → 다음 stage.

## 참조

- 수행계획서: `mydocs/plans/task_m100_604.md`
- Issue: #604
- 분석 자료: `mydocs/tech/document_ir_parser_relationship_analysis.md` (16KB)

# Document IR — LineSeg 필드 표준

## 본 문서의 본질

`LineSeg` 는 `Document` IR (`src/model/paragraph.rs`) 의 핵심 줄 단위 레이아웃 필드.
`Document` IR 은 HWP5 형식 기반으로 설계되었으며 (`CLAUDE.md` 명시), HWP5 가 IR origin
이므로 HWP5 LineSeg 인코딩이 표준. **HWP5/HWPX/HWP3 모든 파서가 본 표준 의미로
LineSeg 를 채워야 한다.**

본 문서는 Task #604 (Document IR 표준 정합화) 에서 작성. PR #589 머지 후 시각 판정 중
`Paragraph.wrap_precomputed` 플래그가 HWP3 휴리스틱을 IR 에 누설했음이 발견되어, IR 표준
명문화로 부채 청산 + 포맷 일관성 확보가 본 문서의 목적.

## 단위

모든 i32 필드는 **HWPUNIT** (1 inch = 7200 HWPUNIT, 1 inch = 25.4mm).

```
hwpunit_to_px(hu, dpi=96) = hu * 96 / 7200
```

## 필드별 표준

| 필드 | 타입 | 단위 | 원점 | 의미 |
|------|------|------|------|------|
| `text_start` | u32 | UTF-16 code unit | 문단 시작 | 본 줄이 차지하는 텍스트 시작 위치 |
| `vertical_pos` | i32 | HWPUNIT | 페이지 상단 | 페이지 내 흐름 y 좌표 (누적 절대값) |
| `line_height` | i32 | HWPUNIT | (없음) | 줄 높이 (line_spacing 포함) |
| `text_height` | i32 | HWPUNIT | (없음) | 텍스트 부분의 높이 |
| `baseline_distance` | i32 | HWPUNIT | 줄 시작 | 베이스라인까지 거리 |
| `line_spacing` | i32 | HWPUNIT | (없음) | 줄간격 |
| `column_start` | i32 | HWPUNIT | 단(column) 좌측 | wrap zone x 오프셋. **0 = wrap 없음** |
| `segment_width` | i32 | HWPUNIT | (없음) | 줄 너비. **단 너비와 같으면 wrap 없음** |
| `tag` | u32 | 비트 플래그 | (없음) | 첫 줄 / 첫 단 등 |

### vertical_pos 의 본질

페이지 시작 (= body 영역 시작) 을 0 으로 한 누적 절대값. 페이지 break 가 발생하면 다음
페이지의 첫 줄은 다시 0 또는 작은 값으로 reset (Task #321 vpos-reset 본질).

- HWP5: 인코더가 누적 계산하여 저장 → 파서는 1:1 매핑
- HWPX: 동일 (XML `vertpos` 속성)
- **HWP3**: 본 표준에 정합하지 않음 (현재 항상 0). 향후 별도 task 에서 누적 계산 정정 권고

### column_start / segment_width 의 본질

본 줄이 그림/표 옆에 있는 wrap zone 텍스트인지 판정하는 핵심 필드.

- `column_start = 0` AND `segment_width = column_width`: 본 줄은 단 전체 너비 사용 (wrap 없음)
- `column_start > 0`: 본 줄은 단 좌측에서 `column_start` HWPUNIT 떨어진 위치에서 시작 (wrap zone)
- `segment_width > 0` AND `segment_width < column_width`: 본 줄의 너비가 단 너비보다 작음 (wrap zone)

### paragraph 내 LineSeg cs/sw 전환 (Stage A 본질 발견)

본 표준의 핵심 본질 — **wrap zone 안 줄과 wrap zone 끝 (그림 영역 끝) 시 같은
paragraph 안에서도 LineSeg.cs/sw 가 전환된다**.

HWP5 v2024 변환본 (`hwp3-sample5-hwp5-v2024.hwp`) page 4 pi=75 실측 데이터:

| LineSeg | vpos | cs | sw | 본질 |
|---------|------|-----|------|------|
| ls[0~18] | 1440~27360 | **37164** | 13860 | wrap zone 안 (그림 옆 좁은 영역) |
| **ls[19]** | **28800** | **0** | **51024** | ★ **그림 영역 끝 — full width 전환** |
| ls[20] | 30240 | 0 | 51024 | full width (그림 아래) |

본질:
- 같은 paragraph (pi=75) 안에서 LineSeg cs/sw 가 줄 단위로 전환
- wrap zone 끝 줄 (그림 영역 vpos 끝) 부터 cs=0 / sw=column_width 로 변경
- 다음 paragraph (pi=76) 의 첫 vpos = 직전 paragraph 마지막 vpos + lh + line_spacing

### paragraph 간 vpos 연결 (Stage A 본질 발견)

다음 paragraph 의 첫 `LineSeg.vpos`:
```
next_para.line_segs[0].vpos = prev_para.line_segs.last().vpos
                              + prev_para.line_segs.last().line_height
                              + prev_para.line_segs.last().line_spacing
```

HWP5 v2024 page 4 실측:
- pi=75 마지막 ls[20] vpos=30240, lh=900, ls=540 → next vpos = 30240 + 900 + 540 = 31680
- pi=76 ls[0] vpos=**31680** ✓ (정합 연결)

본 paragraph 간 vpos 연결이 paginate / layout 의 sequential flow 의 본질. HWP3 파서가
LineSeg.vpos = 0 채우면 본 연결이 끊김 → 렌더러 vpos 기반 로직 우회 → 시각 결함.

### HWP3 파서의 IR 표준 정합 인코딩 책임 (Stage A 진단)

HWP3 → HWP5 IR 변환 시 다음 모두 정합화 필요:

1. **anchor paragraph LineSeg lh = 1줄** (HWP5 표준 — 그림 height 가 아님)
2. **wrap text paragraph LineSeg vpos = section 누적 절대값** (현재 항상 0 — 위배)
3. **paragraph 내 cs/sw 전환** — wrap zone 안 줄 (cs>0) → 그림 영역 끝 시 cs=0/sw=full
   (현재 미인코딩 — 위배)
4. **paragraph 간 vpos 연결** — anchor 끝 다음 paragraph 의 vpos 누적 (현재 모두 0 — 위배)

## wrap zone 판정 (포맷 무관)

`LineSeg::is_in_wrap_zone(col_w_hu)` helper 가 본 표준의 정합한 구현.

```rust
impl LineSeg {
    /// 본 줄이 wrap zone (그림/표 옆) 안에 있는지 (포맷 무관 표준).
    ///
    /// `col_w_hu`: 단 너비 (HWPUNIT)
    pub fn is_in_wrap_zone(&self, col_w_hu: i32) -> bool {
        self.column_start > 0
            || (self.segment_width > 0 && self.segment_width < col_w_hu)
    }
}
```

본 helper 는:
- **포맷 무관**: HWP3/HWP5/HWPX 모두 동일 의미
- **상태 무관**: `Paragraph` 의 다른 필드 (예: `wrap_precomputed`) 에 의존하지 않음
- **per-line**: 각 LineSeg 가 독립 판정 (한 문단 내 일부 줄만 wrap zone 인 케이스 정합)

## 각 파서의 인코딩 책임

### HWP5 파서 (`src/parser/body_text.rs:422`)

HWP5 PARA_LINE_SEG 바이너리 레코드 (36바이트) 를 LineSeg 에 1:1 매핑. **변형 없음**.
HWP5 가 IR origin 이므로 자연스러움.

### HWPX 파서 (`src/parser/hwpx/section.rs:497`)

HWPX `<hp:lineseg>` XML 속성을 LineSeg 에 매핑.

| XML 속성 | LineSeg 필드 |
|----------|-------------|
| `textpos` | `text_start` |
| `vertpos` | `vertical_pos` |
| `vertsize` | `line_height` |
| `textheight` | `text_height` |
| `baseline` | `baseline_distance` |
| `spacing` | `line_spacing` |
| `horzpos` | `column_start` |
| `horzsize` | `segment_width` |
| `flags` | `tag` |

### HWP3 파서 (`src/parser/hwp3/mod.rs:1409`)

HWP3 → HWP5 IR 변환. 다음을 정합화 책임:

- `text_start`: HWP3 char index → UTF-16 code unit 변환
- `line_height/text_height/baseline_distance/line_spacing`: HWP3 line height (1바이트 * 4) → HWPUNIT 비례 계산
- `column_start/segment_width`: wrap zone 영역의 모든 줄에 정확히 인코딩 (Task #604 Stage 3 정정 영역)
- `vertical_pos`: 현재 항상 0 → **본 표준 미정합** (향후 별도 task 에서 누적 계산 정정 권고)

## 본 표준 도입 이전의 부채 (Task #604 청산 대상)

### `Paragraph.wrap_precomputed` 플래그 (PR #589 보완6 도입, 본 task Stage 2 제거)

- 본질: HWP3 파서의 `all(vertical_pos==0) && any(column_start>0)` 검출 결과를 IR 에 저장
- 위배: HWP3 휴리스틱이 IR 에 누설 → IR 의 포맷 독립성 위배
- 정합 방향: `LineSeg::is_in_wrap_zone(col_w_hu)` 로 모든 포맷 일관 판정

### HWP3 LineSeg `vertical_pos` 항상 0 (별도 task 권고)

- 본질: HWP3 spec 에 vpos 개념 없음 → 파서가 0 으로 채움
- 위배: 본 표준 (페이지 상단 기준 누적 절대값) 미정합
- 영향: 렌더러의 vpos 기반 로직 (Task #321 vpos-reset, Task #332 vpos correction, Task #412
  vpos_page_base 등) 이 HWP3 에 대해 우회 동작
- 정합 방향: HWP3 파서가 LineSeg 누적 계산 — 첫 줄=0, 다음 줄=prev.vpos+prev.lh+prev.ls

## 본 표준의 정합성 검증

본 표준 도입 후 회귀 검증 영역:

| 영역 | fixture | 명령 |
|------|---------|------|
| HWP5 native | form-002, issue-147, issue-157, issue-267, table-text | `cargo test --test svg_snapshot` |
| Task #546 회귀 | exam_science.hwp | `cargo test --test issue_546` |
| Task #554 정합 | hwp3-sample{,4,5}-hwp5.hwp / .hwpx | `cargo test --test issue_554` |
| HWP3 native | hwp3-sample.hwp, hwp3-sample5.hwp | `cargo run --bin rhwp -- dump-pages` 페이지 수 |
| HWP3 시각 정합 | hwp3-sample5.hwp page 4/8/16/22/27 | `cargo run --bin rhwp -- export-svg` + 한컴뷰어 비교 |

## 변경 이력

- **2026-05-05**: Task #604 Stage 1 신규 작성 (Document IR LineSeg 표준 정의)
- 향후 변경 시 본 영역에 추가

## 참조

### 관련 task / PR / 이슈
- **Task #604** (본 표준 정의 task) — Document IR 표준 정합화
- **Issue #604** — 본 표준 부재로 인한 hwp3-sample5.hwp page 4 시각 결함
- **PR #589** (Task #511 v2 + #554) — `wrap_precomputed` 플래그 도입 (본 표준 정정 대상)

### 분석 자료
- `mydocs/tech/document_ir_parser_relationship_analysis.md` (16KB) — IR ↔ 각 파서 관계 종합 분석

### 권위 자료
- `mydocs/tech/한글문서파일형식_5.0_revision1.3.md` — HWP5 PARA_LINE_SEG spec
- `mydocs/tech/한글문서파일구조3.0.md` — HWP3 line layout spec

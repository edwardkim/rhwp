# Document IR wrap zone 표현 표준 부재 — 본질 재검토

## 발견 일시

2026-05-04, PR #589 (Task #511 v2 + #554) 검토 중 작업지시자 지적

## 지적 요지

> document IR 쪽에서 표준만 잡혀있으면 hwp3 포맷 파서에서 document IR 로 던져주기만 하면 되는 문제 아님? 현재 document IR 이 잘못되어 있는것 아님?

본질: **Document IR 이 wrap zone 표현 표준을 명확히 정의하지 않은 상태**. 본 세션 보완6 의 `wrap_precomputed` 플래그가 IR 에 새어 들어간 휴리스틱 — 포맷 독립성 위배.

## 현재 Document IR 상태

### `src/model/paragraph.rs`

```rust
pub struct LineSeg {
    pub text_start: u32,
    pub vertical_pos: i32,      // ← HWP3는 항상 0, HWP5는 누적값
    pub line_height: i32,
    pub text_height: i32,
    pub baseline_distance: i32,
    pub line_spacing: i32,
    pub column_start: i32,      // ← cs (wrap zone x 오프셋)
    pub segment_width: i32,     // ← sw (wrap zone 너비)
    pub tag: u32,
}

pub struct Paragraph {
    // ...
    pub wrap_precomputed: bool, // ← 본 세션 보완6에서 추가, 의미 모호
    // ...
}
```

### IR 표준 모호성 매트릭스

| 필드 | HWP3 파서 | HWP5 파서 | HWPX 파서 | 표준 정의? |
|------|---------|---------|----------|----------|
| `LineSeg.vertical_pos` | 항상 0 (의미 없음) | 누적 절대값 (HWPUNIT) | 누적 절대값 | ❌ 정의 없음 |
| `LineSeg.column_start` | wrap zone 오프셋 (HWPUNIT) | wrap zone 오프셋 (HWPUNIT) | 동일 | ⚠️ 의미는 같지만 일부 0 |
| `LineSeg.segment_width` | wrap zone 너비 (HWPUNIT) | wrap zone 너비 (HWPUNIT) | 동일 | ⚠️ 일부 0 |
| `Paragraph.wrap_precomputed` | true/false (HWP3 vpos=0 패턴 검출) | 미설정 (false 유지) | 미설정 | ❌ 의미 모호 |

## 본질적 문제

`wrap_precomputed` 자체가 **HWP3 파서의 휴리스틱이 IR 에 새어 들어간 것** — IR 표준이 아니라 "HWP3 특수 case 표시 플래그" 가 됨. 이는 다음을 위배:

1. **IR 의 포맷 독립성** — Document IR 은 모든 포맷의 공통 표현이어야 함
2. **단일 책임** — IR 필드는 명확한 의미를 가져야 하고, 파서가 어떻게 채웠는지 추측하지 않아야 함
3. **CLAUDE.md 정합** — HWP3 전용 분기를 렌더러에서 제거한다고 했으나, IR 에 HWP3 휴리스틱 플래그가 남음

## 정합한 IR 표준 설계 옵션

### 옵션 IR-1: `vertical_pos` 의미 통일

- 표준: `vertical_pos` = 줄 시작 y의 절대값 (HWPUNIT, 문단 시작 기준 누적)
- HWP3 파서가 vpos=0 → 누적값으로 변환 후 IR 에 저장
- 렌더러는 `vertical_pos` 만 신뢰
- `wrap_precomputed` 플래그 불필요 (cs/sw 만으로 wrap zone 식별)

**장점**: vertical_pos 의 의미 명확화. 렌더러가 일관 처리.
**단점**: HWP3 파서가 LineSeg 누적 계산 추가 필요. 렌더러 vpos 기반 로직 정합화 필요 (현재 vpos 누적값을 신뢰하지 않는 코드 경로 다수).

### 옵션 IR-2: `LineSeg` 의 `wrap_zone` 명시 필드 추가

```rust
pub struct LineSeg {
    // ...
    pub column_start: i32,
    pub segment_width: i32,
    /// 이 줄이 wrap zone 안에 있는지 (전체 단 너비를 사용하지 않음)
    pub is_in_wrap_zone: bool,
}
```

- 모든 파서가 `is_in_wrap_zone` 일관 설정
- `Paragraph.wrap_precomputed` 제거
- 렌더러는 `LineSeg.is_in_wrap_zone` 으로 판정

**장점**: 명시적 표준. 파서 별 검출 로직 분리.
**단점**: 새 필드 추가. derived 정보 (cs/sw 로 계산 가능) 를 명시적으로 저장하는 것은 redundant.

### 옵션 IR-3: `column_start>0 OR segment_width<col_width` 로 wrap zone 정의 (암묵 표준) — 권장

- IR 에 새 필드 없음
- 모든 파서가 cs/sw 를 정확히 인코딩하도록 표준화
- 렌더러가 cs/sw 값 자체로 wrap zone 판정
- `wrap_precomputed` 제거

**장점**:
1. **포맷 독립성** — cs/sw 는 이미 모든 포맷이 동일 의미로 사용 (HWPUNIT)
2. **추가 필드 불필요** — `wrap_precomputed` / `is_in_wrap_zone` 등 derived 필드 제거
3. **파서 단순화** — 각 파서는 LineSeg 의 cs/sw 를 본 포맷 데이터에서 정확히 인코딩만 하면 됨
4. **렌더러 단순화** — `LineSeg.column_start > 0 || LineSeg.segment_width < col_w` 면 wrap zone 처리

**단점**: 렌더러가 `col_w` 를 알아야 segment_width 와 비교 가능. 단 이미 col_area.width 는 알고 있으므로 추가 부담 없음.

## 권장 방향 — 옵션 IR-3

### 본 세션 변경사항 정정 방향

| 파일 | 정정 |
|------|------|
| `src/model/paragraph.rs` | `wrap_precomputed: bool` 필드 **제거** (IR 표준 위배) |
| `src/parser/hwp3/mod.rs` | wrap_precomputed 후처리 제거. 단, HWP3 LineSeg 의 vpos=0 인코딩 → 정합한 cs/sw 만 보존 |
| `src/parser/body_text.rs` | (변경 없음) — HWP5 LineSeg cs/sw 그대로 |
| `src/parser/hwpx/section.rs` | (변경 없음) |
| `src/renderer/typeset.rs` | 흡수 조건을 `LineSeg.column_start > 0 OR segment_width < col_w` 로 변경 |
| `src/renderer/layout/paragraph_layout.rs` | `wrap_precomputed` 검사 → `LineSeg.column_start > 0 \|\| segment_width < col_w` 검사 |

### Document IR 표준 문서화

`mydocs/tech/document_ir_lineseg_standard.md` 신설:

```markdown
# Document IR — LineSeg 표준

## vertical_pos
- 의미: 줄 시작 y 위치 (HWPUNIT, 문단 시작 기준 누적)
- HWP3 파서: 0 → 누적값으로 정합화 (라인 인덱스 × line_height 누적)
- HWP5 파서: 원본 누적값 그대로
- HWPX 파서: 원본 누적값 그대로

## column_start (cs)
- 의미: wrap zone x 오프셋 (HWPUNIT, body_left 기준)
- 0 = wrap 없음 (full width)
- > 0 = 그림/표 옆에 텍스트 배치

## segment_width (sw)
- 의미: wrap zone 텍스트 너비 (HWPUNIT)
- == column_width (col_w) = wrap 없음
- < column_width = wrap zone 너비 (그림 옆 좁은 영역)

## wrap zone 판정 (렌더러)
한 줄이 wrap zone 안에 있는지:
  is_in_wrap_zone = (column_start > 0) OR (segment_width > 0 AND segment_width < col_w)

## 파서 책임
- 각 파서는 본 포맷의 wrap zone 데이터를 cs/sw 로 정확히 인코딩
- 추가 derived 필드 (wrap_precomputed 등) 는 IR 에 두지 않음
```

## 본 PR (#589) 처리 방향

| 옵션 | 처리 | 장점 | 단점 |
|------|------|------|------|
| **재PR 정정** | PR #589 close → IR 표준 정정 + 모든 파서 정합화 + 렌더러 정합화 후 신규 PR | IR 설계 부채 즉시 해결. 본질 깔끔. | PR scope 확대. 회귀 검증 영역 광범위. 시간 소요. |
| **본 PR 유지 + 후속 task** | PR #589 머지 (page 4/8 결함 정정) + Task X 분리 ("Document IR LineSeg 표준 통일 + wrap_precomputed 제거") | 점진적 개선. 본 PR 의 page 4/8 정정 즉시 가치. | IR 부채 잔존. 후속 task 까지 wrap_precomputed 플래그 살아있음. |

### 점진적 개선의 위험

`wrap_precomputed` 가 IR 에 살아있는 상태로 머지되면:
- 후속 PR (HWP5/HWPX wrap_precomputed 후처리) 도 IR 부채 강화
- IR 표준 정정 시 광범위 영향 (호출처 다수)
- 다른 컨트리뷰터가 `wrap_precomputed` 의미를 오해할 위험

### 즉시 정정의 본질

옵션 IR-3 정정은 본질적으로:
- 추가 코드 거의 없음 (필드 제거 + 조건식 변경)
- 회귀 위험 낮음 (cs/sw 자체는 이미 IR 에 인코딩되어 있음)
- 렌더러 변경 영역 좁음 (typeset.rs 흡수 조건 + paragraph_layout.rs 보완6 영역)
- HWP3/HWP5/HWPX 모든 변환본 일관 처리

## 검증 영역 (옵션 IR-3 + 재PR 시)

- `cargo test --lib` 1124+ passed (회귀 없음)
- `cargo test --test issue_546` exam_science.hwp 4페이지 정합
- `cargo test --test issue_554` 12 passed
- 광범위 fixture sweep (form-002, issue-147, issue-157, issue-267, table-text 등 golden SVG 회귀 없음)
- HWP3 native 시각 판정 (page 4/8/16/22/27)
- HWP5 변환본 시각 판정 (`hwp3-sample5-hwp5.hwp` page 4)
- HWPX 변환본 시각 판정 (`hwp3-sample5-hwpx.hwpx`)
- 일반 HWP5 fixture (Square wrap 그림 포함) 회귀 검증

## 별도 후속 task 권고

### Task Y — Document IR LineSeg 표준 통일

위 옵션 IR-3 정정 + `mydocs/tech/document_ir_lineseg_standard.md` 신설.

### Task Z — Task #525 본질 재검토

`Task #525 가 layout_wrap_around_paras 호출을 모두 제거한 본질 (중복 emit 결함, 7 샘플 37 페이지)` 이 IR 표준 정정 후에도 유효한지 재검토:
- IR 표준화로 흡수/렌더 path 가 일관되면 layout_wrap_around_paras 자체가 dead code 가능성
- 또는 Task #525 가드를 IR 기반으로 재구현

## 참조

### 관련 문서
- [mydocs/tech/hwp5_wrap_precomputed_analysis.md](mydocs/tech/hwp5_wrap_precomputed_analysis.md) — HWP5/HWPX wrap_precomputed 미적용 결함 분석 (선행 문서)
- `CLAUDE.md` § 파일 포맷별 파서 구조 (Document IR 본질)
- `mydocs/troubleshootings/square_wrap_pic_bottom_double_advance.md`

### 관련 task / PR
- **Task #460 보완6/8** (본 세션, PR #589) — HWP3 wrap_precomputed 도입 (IR 부채 발생 지점)
- **Task #525** — Picture Square wrap 호스트 텍스트 중복 emit 정정 (layout_wrap_around_paras 호출 제거)
- **Task #489** — Picture/Shape Square wrap LINE_SEG.cs/sw 적용 (anchor 문단)

## 결론

본질적으로 **옵션 IR-3 (cs/sw 만으로 wrap zone 정의) + Document IR 표준 문서화** 가 정합. 본 PR 의 page 4/8 정정 가치를 보존하면서 IR 부채를 정정하는 방향으로:

1. **재PR 정정** (권장): PR #589 close → 옵션 IR-3 적용 + 모든 파서 정합화 + 렌더러 단순화
2. **본 PR 유지 + 후속 task** (점진): PR #589 머지 + Task Y, Z 분리

작업지시자 결정 영역.

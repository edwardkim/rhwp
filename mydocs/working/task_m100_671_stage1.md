# Task #671 Stage 1 단계별 보고서 — 본질 진단

## 진단 결과 요약

`samples/계획서.hwp` 표 셀 줄겹침 결함의 본질은 **`compose_lines` fallback 분기의 단일 ComposedLine 압축**으로 식별.

## 1. HWP 파싱 결과 — line_segs 비어있음 확인

진단 도구 (`examples/inspect_task671.rs`) 로 `samples/계획서.hwp` 1페이지 표 (17행×15열, 69개 셀) 의 모든 셀 paragraph 검사:

```
셀[0] r=0,c=0  text="연구·개발 계획서"        line_segs=0
셀[13] r=3,c=1 text="탈레스 HSM 관리 시스템..." line_segs=0  ← 결함 발현
셀[21] r=5,c=1 text="탈레스 HSM 을 관리..."    line_segs=0  ← 결함 발현
... (전체 69개 셀 paragraph 모두 line_segs=0)
```

**모든 셀 paragraph 의 `line_segs.len() = 0`**.

## 2. 다른 HWP5 파일 비교 — line_segs 정상 인코딩

`samples/exam_kor.hwp` 셀 paragraph 검사:

```
text="A 단계는 확산 모델..."      line_segs=4  ← 정상
text="○○ 인터넷 카페..."        line_segs=6  ← 정상
text="조선 후기의 가사나..."     line_segs=8  ← 정상
text="(다)에서 편지는..."         line_segs=9  ← 정상
```

**다른 정상 HWP5 파일은 셀 paragraph 에 PARA_LINE_SEG 인코딩됨**.

→ 결론: `samples/계획서.hwp` 만 한컴이 셀 paragraph 의 PARA_LINE_SEG 를 인코딩하지 않은 특이 케이스.

## 3. 본 환경 HWP5 파서 — 정상 동작 확인

`src/parser/body_text.rs:152-154`:

```rust
tags::HWPTAG_PARA_LINE_SEG => {
    para.line_segs = parse_para_line_seg(&record.data);
}
```

PARA_LINE_SEG 레코드가 존재하면 정상 파싱. 본 파일은 레코드 자체가 없으므로 `para.line_segs` 가 빈 Vec 으로 유지됨 (파서 결함 아님).

## 4. compose_lines fallback — 본질 결함 위치

`src/renderer/composer.rs:296-323`:

```rust
fn compose_lines(para: &Paragraph) -> Vec<ComposedLine> {
    if para.line_segs.is_empty() {
        // LineSeg가 없으면 전체 텍스트를 하나의 줄로
        if para.text.is_empty() {
            return Vec::new();
        }
        ...
        return vec![ComposedLine {
            runs: split_runs_by_lang(...),
            line_height: 400,        // ← 고정값
            baseline_distance: 320,
            segment_width: 0,        // ← 0
            column_start: 0,
            line_spacing: 0,
            has_line_break: false,
            char_start: 0,
        }];
    }
    ...
}
```

**fallback 경로**: `line_segs.is_empty()` 시 **단일 ComposedLine** 으로 전체 텍스트를 한 줄에 압축. 셀 너비를 초과하는 텍스트도 한 줄로 처리되어 줄바꿈이 일어나지 않음.

## 5. layout 결과 — vpos 누적 안 됨

`src/renderer/layout/paragraph_layout.rs:898-903`:

```rust
let mut line_node = RenderNode::new(
    line_id,
    RenderNodeType::TextLine({
        let vpos = para.and_then(|p| p.line_segs.get(line_idx)).map(|ls| ls.vertical_pos).unwrap_or(0);
        TextLineNode::with_para_vpos(line_height, baseline, section_index, para_index, line_idx as u32, vpos)
    }),
    ...
);
```

`para.line_segs.get(line_idx)` 가 `None` (line_segs 비어 있음) → `vpos = 0`. 단일 ComposedLine 만 생성되어 layout 한 번만 호출 → 다중 줄 누적 자체가 발생하지 않음.

## 6. 결함 발현 메커니즘 (정리)

```
HWP5 파일 (계획서.hwp)
  │
  └─ 셀 paragraph: PARA_LINE_SEG 레코드 부재 (한컴 인코딩 안 함)
     │
     └─ 본 환경 파서: para.line_segs = [] (Vec 빈 상태)
        │
        └─ composer::compose_lines fallback:
           단일 ComposedLine (전체 텍스트 한 줄 압축)
           │
           └─ layout_composed_paragraph:
              한 번 layout, vpos=0
              │
              └─ SVG 렌더: 셀 너비 초과 텍스트가 한 줄에 그려짐
                 (또는 폰트/path 변환 시 자체 줄바꿈, vpos 누적 X)
                 → 시각 결함 (줄겹침)
```

## 7. 본질 정정 방향 제안

### 7.1 정정 위치 — `compose_lines` fallback 분기

**가장 본질적이고 회귀 위험이 낮은 위치**.

현재 fallback (단일 ComposedLine) 대신, 셀 컨텍스트를 받아 자동 줄바꿈 ComposedLine 다중 생성. 단, compose_lines 는 paragraph 단독 처리이고 셀 너비를 모르므로:

**옵션 A**: compose_lines 시그니처 변경하여 cell_width 파라미터 추가 (호출자에서 전달)
**옵션 B**: 별도 fallback 함수 생성 (`compose_lines_with_width`) — 셀 컨텍스트만 호출
**옵션 C**: 파서 후처리로 line_segs 비어 있는 paragraph 에 자동 layout (HWP5 import 시 한 번)

### 7.2 회귀 위험 영역 좁힘

- 케이스 가드: `para.line_segs.is_empty()` + 셀 컨텍스트만 새 로직 진입
- 다른 영역 (정상 line_segs 인코딩된 paragraph) 기존 로직 그대로
- 본문 paragraph 영향 X (본문도 line_segs 부재 시 동일 fallback 이지만 다른 경로 가능성)

### 7.3 휴리스틱 vs 본질 정정

본 결함은 한컴이 인코딩 안 한 케이스 → layout 엔진이 자체 layout 해야 정합. 한컴 layout 알고리즘을 정확히 모방하기는 어려우나, 폭 측정 + 단어 단위 줄바꿈은 본질적 layout 으로 인정 (`feedback_rule_not_heuristic` 영역).

## 8. Stage 2 정정 방향 제안

### 8.1 우선 옵션 — 옵션 B (별도 fallback 함수)

**이유**:
- 시그니처 변경 영향 최소화
- 셀 layout 호출자에서만 새 fallback 호출
- 본문 paragraph fallback 은 기존 그대로 (회귀 0)

### 8.2 구현 방안

1. `src/renderer/composer.rs` 에 `compose_paragraph_with_width(para, width_hu, styles)` 추가
2. 셀 paragraph layout 호출 경로 (table_layout.rs / table_cell_content.rs) 에서 line_segs 비어 있는 paragraph 에 한해 새 함수 호출
3. 새 함수 내부:
   - 텍스트 폭 측정 (estimate_text_width 헬퍼 재사용)
   - 셀 가용 폭 초과 시 단어 경계 또는 글자 단위 줄바꿈
   - 각 ComposedLine 의 line_height/line_spacing 설정 (ParaShape 기반)
   - segment_width = cell_width 로 설정

### 8.3 검증 절차

- 결정적 검증: cargo test --lib --release 회귀 0
- 시각 검증: 계획서.hwp 셀 [13]/[21] 정상 줄바꿈 시각 확인
- 다른 HWP5 fixture 영향 0 (기존 fallback 경로 보존)

## 9. Stage 1 산출물

- `examples/inspect_task671.rs` — 진단 도구 (셀 paragraph line_segs 검사)
- 본 단계별 보고서 (`mydocs/working/task_m100_671_stage1.md`)

## 10. Stage 2 진행 승인 요청

본질 진단 결과 + Stage 2 정정 방향 (옵션 B: `compose_paragraph_with_width` 신규 함수 + 셀 layout 호출 경로 가드) 승인 요청.

승인 후 Stage 2 본질 정정 진행.

# Task #671 최종 결과 보고서

## 1. 요약

**Issue #671**: 표 셀 내부 paragraph 줄바꿈 시 다중 LINE_SEG 줄 겹침 — 같은 y 좌표 그려짐

**결과**: 본 task 본질 영역 (셀 paragraph `line_segs` 부재 시 단일 ComposedLine 압축으로 인한 줄겹침) **정정 완료**. 회귀 0. 시각 판정 통과.

**잔존 결함 영역 (별개 본질)**: TAC 표 비례 축소 → **Issue #672** 별도 등록.

## 2. 본질 진단 (Stage 1)

`samples/계획서.hwp` 의 모든 셀 paragraph 가 `line_segs.len() == 0` 상태:

```
셀[13] r=3,c=1 text="탈레스 HSM 관리 시스템 및 REST API..." line_segs=0
셀[21] r=5,c=1 text="탈레스 HSM 을 관리하기위한 CCC..."   line_segs=0
... (전체 69개 셀 paragraph 모두 line_segs=0)
```

다른 정상 HWP5 (`exam_kor.hwp`) 는 셀 paragraph 에 PARA_LINE_SEG 정상 인코딩. 본 파일만 한컴이 의도적으로 PARA_LINE_SEG 를 인코딩하지 않음.

**결함 메커니즘**:

```
HWP5 파일 → 셀 paragraph PARA_LINE_SEG 부재 (한컴 인코딩 안 함)
  ↓
파서 정상 → para.line_segs = [] (빈 Vec)
  ↓
composer::compose_lines fallback (composer.rs:296-323):
  단일 ComposedLine 으로 전체 텍스트 압축 ⚠️
  ↓
layout 한 번 호출, vpos=0
  ↓
시각 결함 (한 줄 압축, 텍스트 겹침)
```

## 3. 본질 정정 (Stage 2)

### 3.1 신규 함수 (`src/renderer/composer.rs`)

```rust
pub fn recompose_for_cell_width(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
)
```

**3 중 가드 (회귀 0 보장)**:

1. `para.line_segs.is_empty()` — 한컴 인코딩 부재만
2. `composed.lines.len() == 1` — fallback 단일 ComposedLine 만
3. 측정 폭 > `cell_inner_width_px` — 너비 안에 들어가면 분할 불필요

3 중 가드 미충족 시 `composed` 무변경.

**분할 전략**: 단어 경계 (공백) 우선, 단일 단어가 width 초과 시 글자 단위 break (CJK 안전).

### 3.2 호출 위치 (6곳)

| 파일:줄 | 역할 | 영향 |
|---------|------|------|
| `composer.rs` | 신규 함수 추가 | — |
| `table_layout.rs:1226-1234` | 셀 layout 렌더링 경로 | 핵심 본질 정정 |
| `table_layout.rs:614/678/700` | `resolve_row_heights` 측정 fallback (caller + callee) | 측정/렌더링 일관성 |
| `table_partial.rs:94, 358` | 분할 표 측정 + layout | 분할 표 일관성 |
| `height_measurer.rs:527, 712` | MeasuredTable 핵심 측정 | row_heights 정합 |

### 3.3 회귀 위험 영역 좁힘 원칙

- **본문 paragraph 무영향**: `compose_paragraph` 기존 함수 변경 없음 (호출 자체 없음)
- **HWPX 영역 무영향**: 셀 layout 호출 경로만 영향
- **정상 line_segs 인코딩된 셀 무영향**: 1차 가드 미충족 시 무변경
- **단일 줄 텍스트 무영향**: 3차 가드 (측정 폭 ≤ width) 미충족 시 무변경

`feedback_rule_not_heuristic` + `feedback_hancom_compat_specific_over_general` 정합.

## 4. 결정적 검증

| 검증 영역 | 결과 |
|----------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 0 warnings |

## 5. 광범위 페이지네이션 회귀 sweep (Stage 3)

`samples/` 폴더 전체 187 fixture 페이지 수 BEFORE/AFTER 비교:

| 영역 | 결과 |
|------|------|
| BEFORE (devel) | 187 fixtures / **2013 pages** |
| AFTER (Task #671) | 187 fixtures / **2013 pages** |
| **차이** | **0** ✅ |

**핵심 fixture 개별 검증**: aift / exam_kor / exam_math / exam_science / synam-001 / footnote-01 / hwp3-sample / hwp3-sample4 / hwp3-sample5 / 계획서 — **모두 차이 0**.

## 6. 시각 판정 게이트웨이

`samples/계획서.hwp` 1페이지 표 (PNG 시각 확인):

| 셀 | BEFORE (정정 전) | AFTER (정정 후) | 판정 |
|----|-----------------|-----------------|------|
| [13] r=3,c=1 "탈레스 HSM 관리 시스템 및 REST API" | 1줄 압축 (글자 겹침) | **2줄 정상 분리** | ✅ 통과 |
| [21] r=5,c=1 "탈레스 HSM 을 관리하기위한 CCC..." | 1줄 압축 (글자 겹침) | **3줄 분리** (마지막 줄 일부 클립 - Issue #672) | ✅ 본질 통과 |

본 task 의 본질 영역 (줄겹침) **정정 완료** ✅.

## 7. 잔존 결함 영역 (별도 Issue)

본 task 정정 후 노출된 잔존 영역:

- 셀 [21] 3번째 줄 마지막 ~0.88px 클립
- 셀 [52] 3번째 paragraph 클립

**본질 진단**: `height_measurer.rs:822-830` TAC 표 (treat_as_char=true) 비례 축소 메커니즘 — 측정 row_heights 합 > common.height 시 모든 row_heights 비례 축소.

**별도 Issue**: [#672](https://github.com/edwardkim/rhwp/issues/672) — "TAC 표 비례 축소 시 셀 콘텐츠 클립 — common.height vs measured row_heights 불일치"

본 task #671 본질 영역 (line_segs 부재) 과 다른 본질 영역 (TAC 표 권위 영역 불일치) 으로 분리 — `feedback_hancom_compat_specific_over_general` 정합.

## 8. 권위 자료

`samples/계획서.hwp` (143,360 bytes, HWP 5.x):
- 본 task 의 권위 재현 영역
- 한컴 PARA_LINE_SEG 인코딩 부재 케이스의 권위 영역
- Issue #672 의 권위 재현 영역도 됨

→ git tracked 영구 보존 영역 등록 (작업지시자 결정 영역).

## 9. 진단 도구

- `examples/inspect_task671.rs` — 셀 paragraph line_segs 검사 도구
- `examples/inspect_task671_v2.rs` — recompose 결과 정밀 진단 도구

## 10. 최종 산출물

| 영역 | 파일 |
|------|------|
| 코드 정정 | `src/renderer/composer.rs` (+신규 함수), `src/renderer/layout/table_layout.rs`, `src/renderer/layout/table_partial.rs`, `src/renderer/height_measurer.rs` |
| 진단 도구 | `examples/inspect_task671.rs`, `examples/inspect_task671_v2.rs` |
| 권위 자료 | `samples/계획서.hwp` (작업지시자 결정 영역) |
| 수행계획서 | `mydocs/plans/task_m100_671.md` |
| 구현계획서 | `mydocs/plans/task_m100_671_impl.md` |
| 단계별 보고서 | `mydocs/working/task_m100_671_stage1.md`, `_stage2.md`, `_stage3.md` |
| 최종 보고서 | `mydocs/report/task_m100_671_report.md` (본 문서) |

## 11. 정합 패턴 정리

- `feedback_rule_not_heuristic`: 단어 경계 + 글자 단위 break 본질 룰
- `feedback_hancom_compat_specific_over_general`: 3 중 가드로 영역 명시 좁힘
- `feedback_visual_judgment_authority`: 작업지시자 시각 판정 게이트웨이 통과
- `feedback_close_issue_verify_merged`: Issue close 시 정정 코드 devel 머지 검증
- `project_dtp_identity`: 조판 엔진 정합성 강화
- `reference_authoritative_hancom`: 한컴 권위 영역 인지 + 별도 결함 분리

## 12. 후속 영역

1. **Issue #672 정정**: TAC 표 비례 축소 결함 (별도 task 영역)
2. **유사 패턴 점검**: 다른 fixture 에서 셀 paragraph line_segs 부재 케이스 추가 발견 시 본 task 의 정정 영역으로 자연 처리됨

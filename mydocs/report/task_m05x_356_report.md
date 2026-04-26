# 최종 보고서 — Task #356 페이지 분기 오버플로 (vpos 권위값/spacing 누적 오차)

- **이슈**: [#356](https://github.com/edwardkim/rhwp/issues/356)
- **마일스톤**: M05x (v0.5.x)
- **브랜치**: `local/task356` (base: `devel`)
- **샘플**: `samples/2022년 국립국어원 업무계획.hwp`
- **단계**: 5/5 (최종 보고)

## 1. 배경 및 증상

`samples/2022년 국립국어원 업무계획.hwp` 페이지 3 에서 본문이 페이지 영역(933.5px)을 넘어 푸터 `- 1 -` 가 SVG 영역 밖으로 밀려나는 증상이 보고됨.

권위값 분석:
- pi=39 누적 vpos = 68,681 HU
- body_area = 70,012 HU (남은 1,331 HU = 17.7 px)
- pi=40 spacing_before(1,000 HU) + 첫 줄 line_height(1,600 HU) 만으로 한계 초과
- HWP 가 자체 인식하여 pi=40 의 첫 LINE_SEG `vertical_pos` 를 500 HU 로 리셋(=새 페이지 상단)
- rhwp 는 px 누적 평가만 사용해 이 신호를 놓치고 pi=40 ~ 42 를 같은 페이지에 무리하게 채워 overflow 발생

같은 패턴이 페이지 29 → 30 사이에도 존재(pi=572 → pi=573 vpos=500).

## 2. 해결

### 핵심 변경

페이지네이터 메인 루프에 **인접 문단 간 vpos 리셋 감지** 로직을 추가하여, HWP 권위값 신호로 강제 페이지 분기를 적용.

```rust
pub fn detect_inter_paragraph_vpos_reset(prev: &Paragraph, cur: &Paragraph) -> bool {
    // 둘 다 line_segs 보유 + 같은 column_start +
    // cur.first.vertical_pos < prev.first.vertical_pos
}
```

### 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/pagination/engine.rs` | 헬퍼 함수 추가 (pub) + 메인 루프 통합 + 단위 테스트 6개 |
| `src/renderer/pagination.rs` | `pub use engine::detect_inter_paragraph_vpos_reset` |
| `src/renderer/typeset.rs` | Task #321 위치에 보조 트리거로 통합 (이중 트리거) |

### 통합 전략

- **pagination engine** (`paginate_with_measured_opts`): `process_page_break` 직후 `current_items` 비어있지 않은 경우 헬퍼 호출, 트리거 시 `advance_column_or_new_page()`.
- **typeset engine** (기본 사용 경로): 기존 `cv==0 && pv>5000` strict 트리거를 유지하고 헬퍼를 보조 트리거로 추가. 다단 column 변경(column_start 다름) 케이스에서 헬퍼는 false 반환하므로 기존 cv==0 트리거가 그 영역을 담당.

### 헬퍼 정의의 핵심 결정

- 비교 기준을 `prev.last.vpos_end` 대신 `prev.first.vertical_pos` 로 설정.
  - 합성 데이터(lh>0/vpos=0) 회귀 방지
  - "cur 이 prev 시작점보다 위 = 새 페이지/단" 의 더 강한 의미적 신호
- `column_start` 일치 체크로 다단 단 변경 오탐 방지
- `current_items.is_empty()` 게이팅으로 페이지 첫 문단 제외 (pagination 측), `para_idx > 0 && !st.current_items.is_empty()` (typeset 측)

## 3. 검증

### 본 샘플

| 항목 | Before | After |
|------|--------|-------|
| 페이지 3 마지막 문단 | pi=42 (overflow) | pi=39 (정상) |
| 페이지 29 마지막 문단 | pi=575 (overflow) | pi=572 (정상) |
| LAYOUT_OVERFLOW 경고 | 5+건 | **0건** ✅ |
| SVG 페이지 수 | 35 | 35 |

SVG diff: 35개 중 4개 파일만 변경(page 3/4, 29/30) — vpos 리셋 두 지점과 정확히 일치. 나머지 31 페이지 바이트 단위 동일.

### 다중 샘플 회귀

| 샘플 | 베이스라인 | After | 평가 |
|------|-----------|-------|------|
| 2022년 국립국어원 업무계획.hwp | 35p / 5+ | 35p / 0 | ✅ 100% 해결 |
| aift.hwp | 74p / 30 | 86p / 16 | ✅ +12p, overflow 47% 감소 |
| exam_eng.hwp (다단) | 8p / 0 | 8p / 0 | ✅ 회귀 없음 |
| exam_math.hwp | 20p / 0 | 20p / 0 | ✅ 회귀 없음 |
| 2010-01-06.hwp | 6p / 0 | 6p / 0 | ✅ 회귀 없음 |

### 테스트

| 테스트 | 결과 |
|--------|------|
| `cargo test --release` (lib 1014 + integration 14+25+6+1+1) | **PASS** (전체 1061, 회귀 0) |
| 신규 단위 테스트 `inter_para_vpos_reset_tests` | 6/6 PASS |
| 골든 SVG (`tests/golden_svg/`) — form-002, issue-147/157/267, table-text | **6/6 PASS** |

## 4. 잔여 과제 (본 이슈 외)

- **aift.hwp 잔여 16건 overflow**:
  - 12건이 `PartialTable`/`Table` (표 행 분할 — `split_table_rows` 경로, 본 fix 무관)
  - 4건이 page 35 의 그림(Shape) + 빈 문단 패턴
  - → 별도 이슈로 분리 검토 권장

- **PDF 페이지 수 일치(35→37)**: 본 fix 로 LAYOUT_OVERFLOW 0건 달성했으나 SVG 페이지 수는 35로 유지. 본 이슈 명시 증상에는 포함되지 않으며, 꼬리말/빈 페이지/표 분할 등 별도 요인. 후속 추적.

## 5. 산출물

### 코드
- `src/renderer/pagination/engine.rs` (헬퍼 + 단위 테스트 + 통합)
- `src/renderer/pagination.rs` (재내보내기)
- `src/renderer/typeset.rs` (보조 트리거)

### 문서
- `mydocs/plans/task_m05x_356.md` — 수행계획서
- `mydocs/plans/task_m05x_356_impl.md` — 구현계획서
- `mydocs/working/task_m05x_356_stage1.md` — 재현·정량 진단
- `mydocs/working/task_m05x_356_stage2.md` — 헬퍼 + 단위 테스트
- `mydocs/working/task_m05x_356_stage3.md` — 페이지네이션 통합
- `mydocs/working/task_m05x_356_stage4.md` — 통합 검증
- `mydocs/report/task_m05x_356_report.md` — 본 보고서

### 커밋
```
8cf216c Task #356 단계 1: 재현 및 정량 진단
94c66e9 Task #356 단계 2: 인접 문단 vpos 리셋 감지 헬퍼 + 단위 테스트
da229ff Task #356 단계 3: 인접 문단 vpos 리셋 페이지네이션 통합
a50b84e Task #356 단계 4: 통합 검증 및 회귀 측정
(본 단계 커밋: 단계 5 + 오늘할일 갱신)
```

## 6. 머지 권장

본 fix 는 다음을 만족:
- 명시 증상 100% 해결
- 회귀 0 (단위 테스트, 골든 SVG, 다중 샘플)
- 다단 케이스(exam_eng) 별도 처리로 안정성 확보

`local/task356` → `local/devel` merge 권장.

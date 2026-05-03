# Task #544 구현 계획서

**제목**: passage 박스 (paragraph border) 위치/크기 PDF 정합
**브랜치**: `local/task544`
**이슈**: https://github.com/edwardkim/rhwp/issues/544
**수행계획서**: `mydocs/plans/task_m100_544.md`
**Stage 0 보고서**: `mydocs/working/task_m100_544_stage0.md`

---

## 1. Stage 0 진단 결과 요약

| 차이 | 원인 | fix 위치 |
|------|------|---------|
| 박스 left +11.5 px | `box_x = col_area.x + box_margin_left` | `paragraph_layout.rs:2697` |
| 박스 width -22.7 px | `box_w = col_area.width - margin_left - margin_right` | `paragraph_layout.rs:2697` |
| 박스 top -9.4 px | `bg_y_start = y_start (sequential)`, trailing-ls 716 HU 누락 | `paragraph_layout.rs:786` + 박스 top 의 IR vpos 기반 산출 |

## 2. 변경 대상 (예상)

| 파일 | 변경 | LOC |
|------|------|-----|
| `src/renderer/layout/paragraph_layout.rs` | bg_y_start IR vpos 기반 산출 + box_x/box_w 산식 정정 | +20 |
| `src/renderer/layout/integration_tests.rs` | TDD 통합 테스트 (3건+) | +180 |
| `mydocs/working/task_m100_544_stage{1,2,3}.md` | 단계별 보고서 | 신규 |
| `mydocs/report/task_m100_544_report.md` | 최종 보고서 | 신규 |

## 3. 단계 분할

### Stage 1 — TDD + fix 위치 정밀 진단 (코드 수정 없음)

목적: TDD 테스트로 회귀 측정 + fix 위치 확정. **코드 수정 없음, 회귀 위험 0**.

1. **테스트 추가** (`integration_tests.rs`):
   - `test_544_passage_box_left_x_matches_pdf` — 페이지 4 [7~9] col 0 박스 left x = 117.0 (±2 px). 현재 SVG=128.5 이므로 RED.
   - `test_544_passage_box_width_matches_pdf` — 페이지 4 [7~9] col 0 박스 width = 425.1 (±2 px). 현재 SVG=402.5 이므로 RED.
   - `test_544_passage_box_top_y_matches_pdf` — 페이지 4 [7~9] col 0 박스 top y = 233.8 (±2 px). 현재 SVG=224.4 이므로 RED.

2. **fix 위치 진단** (코드 수정 없음, 분석만):
   - `paragraph_layout.rs:2683-2697` box_x/box_w 산식 분석
   - `paragraph_layout.rs:786` bg_y_start 산식 분석
   - prev paragraph trailing-ls 보정을 어디서 적용할지 확정 (push 시점 vs paragraph_layout 호출 시 y_start 보정 vs 별도 변수)
   - `border_box_override` (wrap=Square 호스트) 케이스 영향 분석
   - 셀 내부 (`cell_ctx.is_some()`) 케이스 영향 분석

3. **광범위 영향 사전 평가**:
   - synam-001 / 복학원서 / exam_math/eng/kor/science 등의 paragraph border 박스가
     PDF 와 일치하는지/안 하는지 확인 (현재 SVG 출력 vs 각 샘플 PDF 비교)
   - 위 결과로 fix 범위 (A/B/C) 결정 입력 받기

4. Stage 1 보고서 + 커밋.

### Stage 2 — Fix 적용 (작업지시자 fix 범위 결정 후)

작업지시자가 fix 범위 (A/B/C) 결정 후 진행.

1. 결정된 범위에 따라 산식 변경:
   - **A안 (광범위)**: 모든 paragraph border 에 적용 — `box_x = col_area.x`, `box_w = col_area.width`, `bg_y_start = prev_pi.IR_vpos_end` (또는 trailing-ls 보정).
   - **B안 (빈 paragraph 직후만)**: Task #540 가드 활용. 빈 paragraph (text=∅, controls=∅) 직후 paragraph 의 border 만 정정.
   - **C안 (특정 ParaShape)**: ps_id=11 또는 특정 패턴 (margin_left ≥ 1700 등) 만 정정. heuristic 비권장.
2. 단위 테스트 통과 (Stage 1 RED → GREEN).
3. Stage 2 보고서 + 커밋.

### Stage 3 — 광범위 회귀 검증

1. `cargo test --release --lib` 전체 통과.
2. **21_언어_기출 9개 passage 박스 모두 PDF 일치 검증** (Stage 1 추가 테스트 케이스 확장).
3. **광범위 샘플 회귀 검증**:
   - synam-001.hwp (음수 ls 57건 + 다양한 paragraph border)
   - 복학원서.hwp / exam_math/eng/kor/science.hwp / 2010-01-06.hwp
   - 셀 내부 paragraph border 가 있는 샘플 (있다면)
   - wrap=Square 호스트 박스 (있다면)
4. 회귀 발견 시:
   - 가드 정밀화 (예: 빈 paragraph 직후 케이스만 정정으로 축소)
   - 또는 본 task 정정 보류 + 별도 분석
5. Stage 3 보고서 + 최종 보고서 + 커밋.

## 4. 위험 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| paragraph border 좌표 변경이 다른 샘플 박스 회귀 | 매우 큼 | Stage 1 광범위 사전 평가 + Stage 3 다중 샘플 회귀 검증 |
| Task #537 / #540 fix 와 충돌 | 큼 | 기존 1120 단위 테스트 통과 검증 |
| 셀 내부 / wrap host 케이스 회귀 | 큼 | cell_ctx / border_box_override 분기 보존 |
| PDF 비교 결과 절대 기준 아님 [feedback_pdf_not_authoritative] | 중간 | 한컴 2010/2020 다중 환경 검증 (작업지시자 입력) |

## 5. 검증 명령

```bash
cargo build --release
cargo test --release --lib

# Stage 1: 신규 테스트 RED 확인
cargo test --release --lib test_544 2>&1 | tail -10

# Stage 3: 광범위 회귀 (PDF vs SVG 박스 좌표 비교)
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -o /tmp/diag544/21
./target/release/rhwp export-svg samples/synam-001.hwp -o /tmp/diag544/synam
./target/release/rhwp export-svg samples/복학원서.hwp -o /tmp/diag544/bok
# ... 그 외 샘플
python3 mydocs/scripts/compare_box_pdf_svg.py  # 신규 스크립트
```

## 6. 커밋 단위

- Stage 1: \"Task #544 Stage 1: TDD 통합 테스트 (RED) + fix 위치 정밀 진단\"
- Stage 2: \"Task #544 Stage 2: paragraph border 좌표 산식 정정 (범위: A/B/C)\"
- Stage 3: \"Task #544 Stage 3: 광범위 회귀 검증 + 최종 보고서\"

`closes #544` 는 Stage 3 마지막.

## 7. Stage 1 진행 시 작업지시자 입력 사항

Stage 1 광범위 사전 평가 결과를 보고 후 작업지시자에게:
1. fix 범위 (A/B/C) 결정 입력
2. (선택) 한컴 2020 / 한컴독스 환경 검증 결과 입력
3. (선택) HWP 표준 명세 paragraph border 룰 입력

위 입력 수신 후 Stage 2 진행.

---

승인 후 Stage 1 시작합니다.

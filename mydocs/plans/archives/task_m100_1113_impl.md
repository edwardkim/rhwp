# Task M100-1113 — 구현 계획서

- 이슈: [#1113](https://github.com/edwardkim/rhwp/issues/1113)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1113`
- 일시: 2026-05-29
- 수행 계획서: [`task_m100_1113.md`](task_m100_1113.md)

## 1. 본질 (수행계획 + 페이지번호 경로 조사 확정)

ODD(홀수쪽) 머리말 글상자는 **autoNum(페이지번호) 단독** (빈 텍스트). 글상자 width=4252(좁음) + paraPr 53(RIGHT/BREAK_WORD). 한컴 에디터가 이 좁은 글상자 안에서 페이지번호를 줄나눔하고 글상자 높이를 키운다. EVEN(짝수, 정상)은 fwSpace+텍스트+autoNum, width=15941(넓음), paraPr 64(JUSTIFY/KEEP_WORD).

본 task = 한컴이 ODD 글상자 페이지번호 줄나눔/높이 계산에 사용하는 **저장 전용 contract** 식별 + 단일 정정. **글자 좌표 사후 보정 금지** (이슈 명시).

## 2. Stage 구성 (3 단계)

### Stage 1 — 진단 (trigger contract 확정)

**목표**: 정답지 vs 저장본의 ODD 머리말 글상자 영역 record byte diff → 한컴 줄나눔 trigger contract 단일 식별.

**작업**:
1. 진단 도구 `examples/dump_odd_header_1113.rs` (이전 세션 패턴 재구성) — ODD/EVEN head subtree record byte 정밀 dump
2. 정답지 vs 저장본 record-by-record byte 비교:
   - SHAPE_COMPONENT (글상자 도형, 239 bytes) — current size / 렌더링 매트릭스
   - 글상자 LIST_HEADER — text margin / max_width
   - 글상자 내부 PARA_HEADER / PARA_TEXT / PARA_LINE_SEG
   - PARA_SHAPE (paraPr 53 lowering) — align / break / line_seg
3. ODD vs EVEN 비교 (EVEN 정상이므로 contract 차이가 trigger 단서)
4. #1110 stage28 oracle inventory / needle32 / shape bundles 교차 확인

**판정 기준**:
- 정답지에만 있고 저장본에 없는 (또는 상이한) 단일 contract = trigger
- AutoNumber 문단 주변 byte 일치 (이슈 확인) → SHAPE_COMPONENT / LIST_HEADER / PARA_SHAPE 우선

**산출**: `mydocs/working/task_m100_1113_stage1.md` — trigger contract 확정 + 가설

### Stage 2 — 정정 + 시각 판정

**정정 영역 후보** (Stage 1 확정):
- `src/document_core/converters/hwpx_to_hwp.rs` (어댑터) — ODD 글상자 contract materialize
- `src/serializer/control.rs` (글상자 LIST_HEADER / SHAPE_COMPONENT 직렬화)
- 단일 함수 정정 우선 (case-specific, 메모리 룰 `feedback_hancom_compat_specific_over_general`)

**검증**:
1. `output/poc/issue_1113/exam_social-round1.hwp` 산출
2. `examples/dump_odd_header_1113.rs` 로 정정 후 byte 정합 확인
3. EVEN / 다른 페이지 회귀 없음 (byte)
4. **작업지시자 한컴 한글 2020/2022 시각 판정** — 3페이지 홀수쪽 머리말 정상 배치 (정답지 등급)

**산출**: `mydocs/working/task_m100_1113_stage2.md`

### Stage 3 — 종결

- 회귀 가드: `tests/issue_1113_*.rs` — ODD 글상자 contract byte 검증
- 트러블슈팅 문서 (필요 시): `mydocs/troubleshootings/`
- 최종 보고서: `mydocs/report/task_m100_1113_report.md`
- commit/merge + 이슈 close (작업지시자 승인 후)

## 3. 위험 분석

| 위험 | 평가 |
|------|------|
| 이전 round1 (LIST_HEADER tail) 시각 실패 재발 | 이슈 본문 AutoNumber 주변 byte 일치 명시 → SHAPE_COMPONENT/PARA_SHAPE 우선, byte 정합 + 시각 판정 게이트 |
| storage-only contract spec 미문서화 | 정답지 byte 정합 + 한컴 시각 판정 |
| 정정이 EVEN/다른 글상자 영향 | ODD 한정 + byte 회귀 가드 + 시각 판정 |
| 시각 판정 단일 round 실패 | 좁은 scope — 재시도 비용 낮음. 실패 시 다른 contract 후보 재진단 |

## 4. 작업지시자 승인 요청

1. 본 구현 계획 (3 단계) 승인 여부
2. Stage 1 진단 도구 (`dump_odd_header_1113.rs` ODD/EVEN byte dump) 권장 수용 여부
3. Stage 2 시각 판정 게이트 (한컴 2020/2022) 권장 수용 여부

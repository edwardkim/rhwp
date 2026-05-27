# Task #1147 구현계획서 — typeset.rs TopAndBottom 표 host_spacing 보정

수행계획서: [task_m100_1147.md](./task_m100_1147.md)

## 0. 사전 측정 (devel HEAD 기준 baseline)

- 본 페이지: `dump-pages` 페이지 X (page_num=5) `used=931.2px` / body=941.1px, 마지막 한 줄 문단 (31.7px) overflow
- `TABLE_DRIFT` 진단: 비-TAC TopAndBottom 표 단독 `host_sp=24.7`, HWP vpos delta 와 비교 시 +27.6px 과잉
- 같은 문서 다른 페이지 (TAC 1x1 표 단독) 도 hwp_used vs used 드리프트 **+42.9px** 재현
- 모든 비-TAC TopAndBottom 표가 `text_len=0` (빈 앵커 문단) 패턴

## 1. 정밀 트리거 조건

### Stage 1 트리거 (`typeset.rs:2638`)

```
suppress_sb = !is_tac
           && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
           && para.text.is_empty()
           && !is_column_top
```

**근거**: HWP 의 빈 앵커 문단 vpos 는 직전 문단 종료 vpos 와 동일 (갭 0). 따라서 `spacing_before` 를 별도 가산하면 안 됨. 본 문서 모든 비-TAC TopAndBottom 표가 이 패턴이며, 측정한 `host_sp` 값에 일관되게 sb 가 +13.3 (1000 HU) 또는 +8.0 (600 HU) 포함됨.

### Stage 2 트리거 (`typeset.rs:3201-3203`)

TAC 표가 빈 앵커 문단을 라인으로 차지하는 경우 LINE_SEG.lh 에 표 본체가 이미 포함됨. 그런데 `fmt.height_for_fit = sb + lines + sa - trailing_ls` 가 sb 를 추가 가산 → 과잉.

```
suppress_sb_tac = para.text.is_empty()
               && fmt.line_heights.len() == 1
               && (line_height - table_body_height_with_margin).abs() < ε
```

**구현 옵션**:
- A. `fmt.height_for_fit` 사용 시 위 조건에서 `sb` 만큼 차감
- B. 별도 height 산출 (LINE_SEG.lh 만 사용)

옵션 선택은 Stage 2 진행 중 단위 측정으로 결정.

## 2. 단계 (4단계)

### Stage 1 — 비-TAC TopAndBottom 표의 빈 앵커 sb 가산 억제

**변경 파일**: `src/renderer/typeset.rs` (산식 1)

**구현**:
- `format_table()` 의 `before` 계산 분기 확장 (현재 2635-2639)
- 신 트리거 조건 추가 → suppress 시 `before = outer_top` 사용 (현재 wrap=1 분기와 동일)

**검증**:
- `cargo build` (warnings 0)
- `RHWP_TABLE_DRIFT=1 rhwp dump-pages <비공개샘플>` 으로 본 페이지 `host_sp` 측정값 변화 (예상: 24.7 → 11.4)
- 본 페이지 `used` 값 변화 (예상: 931.2 → ~918)
- pi=127 placement 확인 (여전히 다음 페이지로 가는지)
- `cargo test --lib` (typeset 관련 단위 테스트)

**단계별 보고서**: `mydocs/working/task_m100_1147_stage1.md`

### Stage 2 — TAC 표 sb 중복 가산 보정

**변경 파일**: `src/renderer/typeset.rs` (산식 2)

**Stage 1 후 본 페이지 overflow 가 해소되지 않는 경우 진행** (Stage 1 만으로 21.8px 이상 줄지 않으면 필요).

**구현**:
- `typeset_tac_table()` 의 `fmt.height_for_fit` 사용 분기 (3201-3206) 에 trigger 조건 추가
- 트리거 충족 시 `height_for_fit - sb_px` 또는 LINE_SEG.lh 만 사용

**검증**:
- 본 페이지 `used` 추가 감소
- pi=127 placement: 본 페이지에 들어가는지 확인 (최종 목표)
- TAC 1x1 표 일관성 검증 (다른 페이지 drift 추적)
- `cargo test --lib`

**단계별 보고서**: `mydocs/working/task_m100_1147_stage2.md`

### Stage 3 — Golden SVG 회귀 분석

**작업**:
- `cargo test` 전수 실행
- 변경된 golden SVG 별 diff 분류:
  - **의도된 변경** (drift 감소 방향): 표 인접 문단 위치 조정, 페이지 경계 이동
  - **회귀 의심**: 라인 위치 ±3px 이상 이동, 페이지 수 변동
- 의심 케이스별 추가 조사 → 필요 시 Stage 1/2 보정 미세 조정

**검증**:
- `RHWP_OUTPUT_PARITY` (있다면) 검증
- 공개 가능한 wrap=TopAndBottom 표 포함 샘플 (예: `samples/` 의 공개 hwp/hwpx 다수) SVG 비교
- `RHWP_TABLE_DRIFT` 전체 통계 (drift 분포가 더 작아졌는지)

**단계별 보고서**: `mydocs/working/task_m100_1147_stage3.md`

### Stage 4 — 본 페이지 검증 + 잔여 정리 + 문서

**작업**:
- 본 페이지 SVG 재출력 → 권위 PDF 와 시각 정합 확인
- `pdf/` 권위 자료 가진 공개 샘플들로 추가 시각 정합 확인
- `RHWP_OUTPUT_PARITY` / `RHWP_GOLDEN_RTL` 등 옵션 회귀 확인
- cargo fmt (변경 파일만)
- 최종 결과보고서 작성

**산출물**: `mydocs/report/task_m100_1147_report.md`

## 3. 작업 외 범위

- HWP3 파서 (다른 경로)
- layout.rs paint/builder (렌더 좌표 계산은 별도)
- 머리말/꼬리말/각주
- 다른 wrap 종류 (Square, Tight 등)

## 4. 위험 / 롤백

- **회귀 위험 (높음)**: `typeset.rs:2638` 산식은 모든 비-TAC TopAndBottom 표 페이지에 영향. 골든 SVG 다수 변경 가능.
- **롤백**: 각 stage 는 독립 커밋 → `git revert` 가능. Stage 1 만 적용 후 본 페이지 안 풀려도, Stage 1 자체가 drift 감소에 의미 있으면 유지하고 Stage 2 만 보강하는 옵션도 열어둠.

## 5. 비공개 샘플 취급

- 본 문서 / 단계 보고서 / 최종 보고서 / 커밋 메시지에서 비공개 샘플 파일명·식별 가능 콘텐츠 사용 금지
- 검증 로그도 일반화 표현 (예: "내부 검증용 A4 HWPX, page_num=5") 사용
- 회귀 분석 다음에는 가능한 공개 샘플로 같은 패턴 재현하여 그것을 회귀 케이스로 기록

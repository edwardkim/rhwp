# Task #671 구현 계획서

## 개요

Issue #671 "표 셀 내부 paragraph 줄바꿈 시 다중 LINE_SEG 줄 겹침" 정정 구현. 3단계 진행.

## Stage 1: 본질 진단

### 목표

표 셀 [13]/[21] 의 다중 LINE_SEG vpos 누락 결함의 본질적 위치를 코드 수준에서 정확히 식별.

### 진단 절차

1. **IR 덤프 정밀 분석**
   - `samples/계획서.hwp` 의 셀 [13]/[21] 의 paragraph 정보 확인
   - 셀 paragraph 의 `line_segs` 배열 정보 확인 (vpos / lh / th / bl / ls / cs / sw)
   - 단일 paragraph인데 다중 LINE_SEG가 존재하는지, 각 vpos 값 분포

2. **코드 path 추적**
   - `src/renderer/layout/table_cell_content.rs` — 표 셀 내부 paragraph 진입점
   - `src/renderer/layout/paragraph_layout.rs` — paragraph 라인 단위 layout
   - `src/renderer/typeset.rs` — LINE_SEG vpos 계산 (HWP5 native)
   - vpos 누적/계산이 누락되는 정확한 위치 식별

3. **rhwp-studio (web editor) vs SVG 출력 양쪽 비교**
   - SVG: `rhwp export-svg samples/계획서.hwp -p 0`
   - WASM: rhwp-studio 에서 동일 페이지 시각 확인
   - 양쪽 모두 같은 결함 발현인지 (동일 layout 경로) / 다른지 (분리 경로) 확인
   - 동일 발현 → 공통 layout 영역 결함 / 다른 발현 → renderer-specific

### 진단 산출물

`mydocs/working/task_m100_671_stage1.md` 단계별 보고서 — 본질 진단 결과 + 정정 위치 후보 + Stage 2 정정 방향 제안.

### 승인 요청

Stage 1 진단 결과 보고 + Stage 2 정정 방향 승인 요청.

## Stage 2: 본질 정정

### 목표

Stage 1 진단 결과에 따라 식별된 위치에 케이스별 명시 가드 적용. 표 셀 내부 paragraph 영역만 영향, 다른 영역 무영향.

### 정정 원칙

1. **케이스별 명시 가드** — 표 셀 내부 paragraph 다중 LINE_SEG 영역만 명시 정정
2. **휴리스틱 금지** — 본질 정정 (rule, not heuristic)
3. **신규 단위 변환 코드 0줄** — 기존 헬퍼 재사용 (가능한 경우)
4. **다른 영역 무영향** — 본문 paragraph / HWPX 영역 byte-identical 보장

### 검증 절차

1. **cargo test --lib --release** 회귀 0 (1155+ passed 유지)
2. **svg_snapshot 6/6** + **issue_546 1/1** + **issue_554 12/12**
3. **cargo clippy --release** 신규 경고 0
4. **계획서.hwp SVG output 비교** — 셀 [13]/[21] 정상 줄바꿈 시각 확인
5. **광범위 페이지네이션 회귀 sweep (167 fixture)** — 차이 0 (Stage 3 에서 본격 진행)

### 정정 산출물

- 영향 코드 변경 (`src/renderer/layout/*.rs` 또는 `src/renderer/typeset.rs`)
- `mydocs/working/task_m100_671_stage2.md` 단계별 보고서

### 승인 요청

Stage 2 정정 결과 + 검증 결과 보고 → Stage 3 광범위 회귀 sweep 진행 승인.

## Stage 3: 광범위 회귀 sweep + 최종 검증

### 목표

광범위 페이지네이션 회귀 sweep 으로 회귀 위험 영역 좁힘 입증 + 시각 판정 게이트웨이 통과.

### 검증 절차

1. **광범위 페이지네이션 회귀 sweep**
   - `samples/` 폴더 전체 fixture (167+ HWP/HWPX) BEFORE/AFTER 페이지 수 차이 측정
   - 차이 0 보장 (회귀 위험 영역 좁힘 입증)
2. **결정적 검증 (release 모드)**
   - cargo test --lib --release 1155+ passed
   - cargo test --release 전체 GREEN
   - cargo clippy --release 신규 경고 0
   - cargo build --release
3. **WASM 빌드 검증**
   - Docker WASM 재빌드 성공
   - rhwp-studio npm run build 통과
4. **시각 판정 게이트웨이 (작업지시자)**
   - rhwp-studio 에서 `samples/계획서.hwp` 1페이지 시각 판정
   - 셀 [13]/[21] 줄겹침 해소 확인
   - 다른 페이지/표 영역 회귀 0 확인

### 산출물

- `mydocs/report/task_m100_671_report.md` 최종 보고서 — Stage 1~3 통합 + 결정적 검증 + 광범위 sweep + 시각 판정 결과
- `mydocs/orders/20260507.md` Task #671 상태 갱신
- `samples/계획서.hwp` git tracked 등록 (작업지시자 결정 영역)

### 승인 요청

최종 보고서 + 시각 판정 게이트웨이 통과 후 작업지시자 승인 → devel merge 영역.

## 회귀 위험 영역 좁힘 원칙

- **수정 영역 명시** — 표 셀 내부 paragraph 다중 LINE_SEG 영역만 정정
- **다른 영역 byte-identical** — 본문 paragraph, HWPX 영역, 단일 LINE_SEG paragraph 영역 무영향
- **케이스별 명시 가드** — `feedback_hancom_compat_specific_over_general` 정합

## 충돌 가능 PR 영역 인지

본 task가 수정 영역(`paragraph_layout.rs` / `table_cell_content.rs`)이 OPEN PR 영역과 겹치므로 충돌 위험 인지:

- PR #629 (Task #628): `paragraph_layout.rs` / `table_cell_content.rs` 수정 중
- PR #620 (Task #618): `paragraph_layout.rs` / `table_cell_content.rs` 수정 중 (이미 cherry-pick 완료)
- PR #621 (Task #617): `table_cell_content.rs` 수정 중
- PR #650 (Task #518): `paragraph_layout.rs` 수정 중

→ 본 task의 정정 영역이 좁고 명시적이라면 충돌 위험 낮음. 단계별 진행 중 OPEN PR 영역 변경 발생 시 rebase 영역.

## 최종 결과 영역

본 task 완료 후:
- Issue #671 close (closes #671 키워드 또는 수동 close)
- Document IR 표준 영역 추가 명문화 가능성 (다중 LINE_SEG vpos 누적 표준)
- 후속 task 영역 — 유사 결함 다른 fixture 영역 (동일 패턴 재현 가능성 점검)

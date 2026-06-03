# Task #435: exam_kor.hwp 24페이지 → 20페이지 정합 — 구현 계획서

> **이슈**: [#435](https://github.com/edwardkim/rhwp/issues/435)
> **수행계획서**: `mydocs/plans/task_m100_435.md`
> **브랜치**: `local/task435` (base: `local/devel` @ `d1a4058`)
> **작성일**: 2026-04-29

---

## 단계 분할 개요

5 단계로 분해. 각 단계는 **(1) 진단/구현 → (2) 단위 검증 → (3) 회귀 검증 → (4) 단계별 보고서** 사이클로 진행.

| 단계 | 목표 | 측정 가능한 통과 조건 | 예상 변경 영역 |
|---|---|---|---|
| 1 | 베이스라인 측정 + 진단 도구 정비 | 페이지별 컬럼 사용 / reserve 값 / typeset cur_h 데이터 확보 | 코드 변경 없음 (덤프/계측만) |
| 2 | #393 (1) col 1 reserve 정정 | exam_kor 24 → 22 페이지, page 2 / 15 의 orphan 해소 | `typeset.rs` 또는 `pagination/engine.rs` |
| 3 | (2) 표/도형 후 컬럼 잔여 공간 정정 | exam_kor 22 → 20 페이지, 일반 페이지 \|diff\| < 100px | `pagination/engine.rs`, `layout/shape_layout.rs` |
| 4 | (3) Square wrap 표 over-fill 보호 (조건부) | page 16 단 0 ≤ 본문 높이, 단 1 hwp_used 근접 | `pagination/engine.rs:702-711` |
| 5 | 회귀 테스트 + 최종 검증 + 보고서 | 모든 회귀 케이스 통과, 최종 보고서 등록 | `tests/`, `mydocs/report/` |

---

## Stage 1 — 베이스라인 측정 + 진단 도구 정비

### 목표

코드 수정 없이 진단 데이터 확보. 후속 단계의 기준선 (pre-fix baseline) 으로 사용.

### 작업 항목

1. **exam_kor.hwp 페이지별 컬럼 사용 덤프** (24 페이지 전체)
   - `dump-pages` 출력에서 `단 N (used, hwp_used, diff)` 레코드 수집
   - CSV 형태로 정리: `output/debug/task435/exam_kor_baseline.csv`

2. **회귀 비교 케이스 베이스라인 측정**
   - `exam_eng.hwp`: 페이지 수 + 페이지별 diff
   - `k-water-rfp.hwp`: 페이지 수
   - `hwpspec.hwp`: 페이지 수
   - `synam-001.hwp`: 페이지 수 (#431 별도, 회귀 감지용)

3. **`RHWP_TYPESET_DRIFT=1` 환경변수 진단 출력 수집**
   - `pi=0.30` (page 1 → 2 split) 의 typeset cur_h vs HWP vpos 추적
   - `pi=1.25` (page 14 → 15 split) 동일 추적
   - 출력은 `output/debug/task435/typeset_drift_baseline.txt` 에 저장

4. **`compute_body_wide_top_reserve_for_para` 호출 추적**
   - 함수 위치 식별 (`grep` 으로 정의 / 호출처 매핑)
   - exam_kor 의 다단 진입 지점에서 reserve 산정 값 로그 (코드 임시 println, Stage 2 에서 제거)

### 통과 조건

- [ ] `output/debug/task435/exam_kor_baseline.csv` 생성 — 페이지별 단별 used/hwp_used/diff
- [ ] 회귀 대상 4개 문서 페이지 수 기록
- [ ] `pi=0.30`, `pi=1.25` 의 typeset cur_h 진행 로그 캡처
- [ ] reserve 산정 값 (page 1 단 0 → 단 1 전환 시점) 수치 확보

### 위험 / 주의

- 임시 디버그 println 추가 시 Stage 2 진입 전 반드시 제거
- 측정 결과는 단계별 보고서에 첨부

---

## Stage 2 — #393 (1) col 1 reserve 정정

### 목표

`exam_kor.hwp` 의 page 1 단 1 over-fill (+54.5px) 과 page 14 단 1 over-fill (+106.2px) 을 해소하여 page 2, 15 orphan 발생 차단.

### 가설 (#393 옵션 A)

`compute_body_wide_top_reserve_for_para` 의 다단 분기에서 reserve 값이 표 높이 자체 (147.4px) 가 아닌 ~306px 으로 과대 산정. 다음 중 하나가 원인:
- 표 높이 + spacing_before/after 중복
- VertRelTo (Paper-rel) 좌표 변환 누락
- pi=0 의 Shape 컨트롤 다중 (BehindText/InFrontOfText) 누적

### 작업 항목

1. **Stage 1 의 reserve 산정 로그 분석**
   - 산정값 = 306.1, 기대값 = 147.4 (표 높이) 또는 94.5 (HWP col 1 시작 vpos px) 의 차이 어디에서 발생하는지 코드 경로 추적

2. **수정 적용** (가설 검증 후)
   - 표 높이만 reserve 로 사용, spacing 중복 제거 또는
   - Shape 컨트롤이 BehindText/InFrontOfText 인 경우 reserve 미포함 또는
   - 다단 진입 시 reserve = max(0, 표 영역 - col 1 시작 vpos)

3. **단위 검증**
   - `rhwp dump-pages -p 0` → page 1 단 1 used ≤ hwp_used + 마진
   - `rhwp dump-pages -p 13` → page 14 단 1 동일
   - `rhwp export-svg samples/exam_kor.hwp` → 22 페이지 (page 2, 15 사라짐)

4. **회귀 검증**
   - exam_eng.hwp 8 페이지 유지
   - k-water-rfp.hwp 27 페이지 유지
   - hwpspec.hwp 페이지 수 유지
   - 다단 + body-wide TopAndBottom 표 케이스 (`grep -l "wrap=TopAndBottom" samples/`)

### 통과 조건

- [ ] exam_kor.hwp 22 페이지 (page 2, 15 사라짐)
- [ ] page 1 단 1 used: 1207.6px → ~1153px 근접 (`|diff| < 20px`)
- [ ] page 14 단 1 동일 (`|diff| < 20px`)
- [ ] 회귀 대상 4 문서 페이지 수 베이스라인 일치
- [ ] `cargo test` 통과

### 위험 / 주의

- 다단 reserve 변경은 Task #391, #386 의 누적 결과와 충돌 가능 → 두 task 의 회귀 테스트 결과 확인
- 본 단계에서 Stage 3 / 4 영역은 변경하지 않음 (영향 분리)

---

## Stage 3 — (2) 표/도형 후 컬럼 잔여 공간 정정

### 목표

페이지 4, 5, 7, 8, 10, 12, 13, 19 의 누적 -100~-300px 부족 해소. exam_kor 22 → 20 페이지 도달.

### 가설

표/도형 (Shape, wrap=TopAndBottom 또는 BehindText) 배치 후 컬럼의 `current_height` 누적 산정이 한컴 대비 작게 잡혀 다음 paragraph 가 일찍 다음 컬럼/페이지로 밀림. (1) 의 over-fill 과 정반대 방향이며 별도 경로.

### 작업 항목

1. **Stage 1 의 페이지별 diff 데이터에서 패턴 식별**
   - Shape/Table 항목이 있는 페이지의 diff 가 일반 페이지보다 큰 음수인지 확인
   - 특정 wrap 모드 (Square / TopAndBottom / BehindText) 와 diff 의 상관관계 분석

2. **`pagination/engine.rs` 의 Shape/Table 처리 후 height 누적 로직 검토**
   - `process_controls` (`engine.rs:933`)
   - `paginate_table_control` (`engine.rs:1055`)
   - `place_table_fits` (`engine.rs:1310`)
   - 표/도형 배치 후 `current_height` 가 어떻게 갱신되는지

3. **수정 적용**
   - 누락된 spacing / wrap 영역 height 가 있다면 보정
   - 또는 누적 산정이 각 line_advance 의 합으로 정확히 일치하도록 정정

4. **단위 검증**
   - `rhwp dump-pages` 페이지 4, 5, 12, 13 의 \|diff\| < 100px
   - exam_kor 20 페이지 도달

5. **회귀 검증**
   - exam_eng, k-water-rfp, hwpspec, synam-001 페이지 수 유지

### 통과 조건

- [ ] exam_kor.hwp 20 페이지 (마지막 페이지 푸터 `20/20`)
- [ ] 일반 페이지 \|diff\| < 100px (단 0, 단 1 모두)
- [ ] 회귀 대상 4 문서 페이지 수 유지
- [ ] `cargo test` 통과

### 위험 / 주의

- height 누적 변경은 다단 / 단단 양쪽에 영향. Stage 2 의 reserve 정정과 상호작용 가능 → Stage 2 결과를 깨지 않는 방향으로 적용

---

## Stage 4 — (3) Square wrap 표 over-fill 보호 (조건부)

### 목표

Stage 3 종료 후 `rhwp dump-pages -p 15` 의 page 16 단 0 used 가 본문 높이 (1211.3px) 를 초과하면 진입. 그렇지 않으면 **Stage 4 생략하고 Stage 5 로 직행**.

### 가설

`paginate_text_lines` (`engine.rs:702-711`) 의 표 직후 trailing line_spacing 제외 로직이 단 0 over-fill 을 통과시킴.

```rust
let overflow_threshold = if prev_is_table {
    let trailing_ls = mp.line_spacings.get(end_line.saturating_sub(1)).copied().unwrap_or(0.0);
    cumulative - trailing_ls   // <- trailing 제외
} else {
    cumulative
};
```

prev_is_table 분기에서 trailing 을 제외해 통과시킨 결과, 단 0 이 본문 높이를 초과하는 사례가 발생.

### 작업 항목

1. **상태 확인**: Stage 3 후 page 16 단 0 used 측정
2. **상한 검사 추가** (필요 시): `cumulative - trailing_ls` 이 본문 높이 + 일정 마진 이내인지 추가 검증
3. **단위 검증**: page 16 단 0 ≤ 1211.3px, 단 1 hwp_used 근접
4. **회귀 검증**: 동일

### 통과 조건 (진입 시)

- [ ] page 16 단 0 used ≤ 본문 높이 (1211.3px)
- [ ] page 16 단 1 \|diff\| < 100px
- [ ] 회귀 대상 4 문서 페이지 수 유지

### 위험 / 주의

- 본 분기는 표 trailing 처리의 핵심 로직 → 다른 표 + 텍스트 혼합 케이스 회귀 위험 → 추가 회귀 검증 케이스 필요

---

## Stage 5 — 회귀 테스트 추가 + 최종 검증 + 보고서

### 목표

수정 결과 영구 회귀 방지 + 최종 결과 정리.

### 작업 항목

1. **회귀 테스트 추가**
   - `tests/` 또는 `src/renderer/pagination/tests.rs` 에 exam_kor 20 페이지 보장 테스트 추가
   - page 1 단 1 used (~1153px), page 14 단 1 used (~1087px) 의 분기 분포 확인 테스트
   - 회귀 대상 4 문서 페이지 수 보장 테스트 (이미 있다면 확인)

2. **전체 회귀 검증**
   - `cargo test` 전체 통과
   - `samples/` 의 주요 문서 (exam_kor, exam_eng, k-water-rfp, hwpspec, synam-001) export-svg 후 페이지 수 일치
   - clippy 통과 (`cargo clippy --release -- -D warnings`)

3. **최종 보고서 작성**
   - `mydocs/report/task_m100_435_report.md`
   - 단계별 결과 요약, 측정값 변화 (24 → 20), 회귀 검증 결과, 코드 변경 요약

4. **오늘할일 갱신**
   - `mydocs/orders/20260429.md` (또는 종료일 기준) 에 #435 완료 기록

### 통과 조건

- [ ] `cargo test` 전체 통과
- [ ] `cargo clippy` warning 없음
- [ ] 회귀 테스트 추가 + 통과
- [ ] 최종 보고서 작성
- [ ] 오늘할일 갱신

---

## 단계별 커밋 규칙

| 단계 | 커밋 메시지 (예시) |
|---|---|
| 1 | `Task #435 Stage 1: 베이스라인 측정 + 진단 데이터 수집` |
| 2 | `Task #435 Stage 2: col 1 reserve 정정 (#393 옵션 A)` |
| 3 | `Task #435 Stage 3: 표/도형 후 컬럼 잔여 공간 정정` |
| 4 | `Task #435 Stage 4: Square wrap 표 over-fill 보호 (조건부)` |
| 5 | `Task #435 Stage 5: 회귀 테스트 + 최종 보고서` |

각 단계의 단계별 보고서 (`task_m100_435_stage{N}.md`) 도 해당 단계 커밋에 포함.

---

## 의존성 / 외부 영향

- **#393 (open)** — Stage 2 가 #393 의 옵션 A 를 구현하므로 본 task 완료 시 #393 close 가능 (작업지시자 결정 필요)
- **#391 (closed)** — Stage 2 의 reserve 정정이 #391 의 다단 누적 공식 분기와 상호작용. 회귀 검증 필요
- **#386 (closed)** — Paper-rel 좌표 가드. Stage 2 의 VertRelTo 처리 시 참고
- **#431 (open, synam-001)** — Stage 별 회귀 검증으로 영향 감지. 회귀 발견 시 별도 task

---

## 작업 규칙 준수

- 각 단계 시작 전 작업지시자 승인
- 각 단계 완료 후 단계별 보고서 작성 → 승인
- 단계 진행 중 가설이 깨지면 즉시 중단 + 보고
- 코드 수정은 단계별 별도 커밋
- 임시 디버그 println 은 단계 종료 전 제거

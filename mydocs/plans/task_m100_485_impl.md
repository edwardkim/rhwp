# 구현 계획서 — Task #485

**관련**: `task_m100_485.md` (수행계획서, 승인 완료)
**브랜치**: `local/task485`

---

## 단계 구성 (4단계)

수행계획서의 우선순위 (B → A → D) 를 단계별로 적용. 각 단계 종료 시 회귀 검증 후 승인 요청.

### Stage 1 — 본질 정밀 측정

**목적**: 결함 페이지의 `line_end_pos vs abs_limit` 정밀 차이 측정. epsilon 임계값 결정 근거 확보.

**작업**:
1. `compute_cell_line_ranges` 에 임시 디버그 trace 삽입 (혹은 별도 분석 스크립트) — p15·p20·p21 의 분할 셀 마지막 줄 `line_end_pos`, `abs_limit`, 차이값 측정
2. line_h 분포 (셀 안 모든 줄의 line_h 통계) 측정
3. typeset 의 `split_end_limit` 산정 입력 (avail_for_rows, padding) 과 layout 의 `cum` 누적 출력 차이 비교
4. epsilon 후보 비교:
   - 고정 0.5px / 1.0px / 2.0px
   - line_h 비례 0.05·line_h / 0.1·line_h
   - 각 후보가 p15·p20·p21 결함 해소 + 회귀 발생 여부 (kps-ai.hwp 분할 표 페이지 비교) 예측

**산출물**: `working/task_m100_485_stage1.md`
- 측정 결과표 (페이지별 line_end_pos, abs_limit, 차이)
- epsilon 후보 비교표
- 권장 값 + 근거

**커밋**: 디버그 코드는 산출물 작성 후 revert. trace 코드 자체는 커밋하지 않음.

---

### Stage 2 — 정정 구현 (후보 B)

**목적**: layout 의 `compute_cell_line_ranges` break 조건에 epsilon 마진 적용.

**대상**: `src/renderer/layout/table_layout.rs:2304`

**변경**:
```rust
// before
if has_limit && line_end_pos > abs_limit { break; }

// after (epsilon 마진 적용)
const SPLIT_LIMIT_EPSILON: f64 = <stage1 권장값>;
if has_limit && line_end_pos > abs_limit - SPLIT_LIMIT_EPSILON { break; }
```

**부수 작업**:
1. 상수는 모듈 상단에 `const` 로 정의 + 주석 (Task #485 본질 명시)
2. `calc_visible_content_height_from_ranges` 가 동일 epsilon 을 사용해야 하는지 검토 — height 계산이 break 결정과 정합되어야 함
3. 단위 테스트: 경계 케이스 (line_end_pos == abs_limit / abs_limit - eps / abs_limit + eps)

**검증**:
```bash
cargo build --release
./target/release/rhwp export-svg samples/synam-001.hwp -p 14 -o /tmp/v485/
./target/release/rhwp export-svg samples/synam-001.hwp -p 19 -o /tmp/v485/
./target/release/rhwp export-svg samples/synam-001.hwp -p 20 -o /tmp/v485/
./target/release/rhwp dump-pages samples/synam-001.hwp -p 14
./target/release/rhwp dump-pages samples/synam-001.hwp -p 19
./target/release/rhwp dump-pages samples/synam-001.hwp -p 20
cargo test --release
```

PDF 대조 (`samples/synam-001.pdf` p15/p20/p21) 로 시각 판정 — 마지막 줄 클립 해소 + 줄 적재 일치 확인.

**산출물**: `working/task_m100_485_stage2.md`
- 변경 요약 + diff
- 결함 페이지 SVG 시각 판정 결과
- cargo test 결과
- (필요 시) Stage 3 으로 회귀 점검 진행 판정

**커밋**: `Task #485: layout break 조건 epsilon 마진 적용 (1차)`

---

### Stage 3 — 회귀 점검 + 보강

**목적**: Stage 2 변경의 회귀 영향 점검 + (필요 시) 후보 A·D 추가.

**작업**:
1. **회귀 시각 검증** (PDF 비교):
   - `samples/kps-ai.hwp` — Task #362 정정 페이지 (p56/p67/p68-70/p72-73) 보존 확인
   - `samples/synam-001.hwp` — Task #431 의 빈 페이지 미발생 (전 페이지 일관성)
   - `samples/synam-001.hwp` p15 직전·직후 페이지 (p14/p16) 콘텐츠 흐름 일관성
2. **회귀 발생 시 후보 A 추가**: `engine.rs` 4곳의 `split_end_limit = avail_content` 에 동일 epsilon 차감
   - engine.rs:1708, 1718, 1759, 1770
   - 헬퍼 `apply_split_limit_epsilon(avail_content)` 도입 검토
3. **회귀 발생 시 후보 D 추가**: layout 의 vpos correction 단계 본문 영역 침범 시 드롭 (선행 분석에서 collapse 위험 — `typeset_layout_drift_analysis.md` 참조 — 신중)
4. 전체 cargo test 재실행

**산출물**: `working/task_m100_485_stage3.md`
- 회귀 검증 결과
- (필요 시) 후보 A/D 적용 내용 + 재검증

**커밋**: 변경 발생 시 `Task #485: 회귀 보강 (후보 A/D)`

---

### Stage 4 — 최종 보고

**작업**:
1. `report/task_m100_485_report.md` 작성
   - 본질·정정 영역·검증 결과 요약
   - 변경 파일 목록 + diff 요점
   - 회귀 점검 결과
   - 잔여 위험 (epsilon 임의성, layout drift 본질 통일은 별도 이슈로 보관)
2. `mydocs/orders/<오늘날짜>.md` 갱신 — Task #485 상태 갱신
3. 이슈 #485 본문에 후속 검증 케이스 (페이지 20·21) 추가 코멘트

**산출물**: `report/task_m100_485_report.md`

**커밋**: `Task #485: 최종 보고서 + orders 갱신`

**merge 전 점검**: `git status` 로 미커밋 파일 없는지 확인.

---

## 변경 파일 (예상)

| 단계 | 파일 | 변경 |
|------|------|------|
| 2 | `src/renderer/layout/table_layout.rs` | break 조건 epsilon 마진 (2304) + 상수 정의 |
| 2 | (테스트 파일) | 경계 케이스 단위 테스트 |
| 3 (조건부) | `src/renderer/pagination/engine.rs` | 4곳 split_end_limit 산정 epsilon 차감 |
| 4 | `mydocs/working/task_m100_485_stage{1,2,3}.md` | 단계별 보고서 |
| 4 | `mydocs/report/task_m100_485_report.md` | 최종 보고서 |
| 4 | `mydocs/orders/<오늘>.md` | 상태 갱신 |

---

## 위험 / 주의

1. **epsilon 임의성**: Stage 1 의 측정 데이터로 정량 근거를 확보. 임의값 회피.
2. **layout drift 본질 통일은 본 이슈 밖**: `typeset_layout_drift_analysis.md` 의 "단일 모델 통합" 은 별도 이슈로 유지.
3. **calc_visible_content_height_from_ranges 정합**: break 조건 변경 시 height 합산도 동일 규칙으로 동작해야 함 (Stage 2 작업 §2 항).
4. **vpos correction 의 collapse 회귀**: 선행 분석 (`typeset_layout_drift_analysis.md` §회귀 원인 체인 6항) 에서 vpos correction 양방향 시 col 1 의 pi=10/pi=11 이 같은 y 로 collapse 되었음. 후보 D 는 신중 적용.

---

## 승인 요청

1. 단계 구성 (4단계, B → 회귀 검증 → 조건부 A/D → 보고) 가 적절한가?
2. Stage 1 의 측정 방식 (디버그 trace 후 revert) 이 적절한가? 별도 측정 도구 (`dump` 확장 등) 가 더 나은가?
3. epsilon 정의 방식 — 고정값 vs line_h 비례 — 작업지시자 선호?
4. Stage 4 의 이슈 #485 코멘트 추가 (페이지 20·21 검증 케이스 추가) 진행 여부?

승인 후 Stage 1 (본질 정밀 측정) 시작합니다.

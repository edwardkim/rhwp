# Task #291 최종 결과 보고서

## 이슈

[#291](https://github.com/edwardkim/rhwp/issues/291) — KTX.hwp 2단 구성에서 오른쪽 단 TAC 표가 왼쪽 단 영역에 밀려 렌더링

## 결론

✅ **해결 완료** — TAC 표 분기에 ParaShape `alignment` 반영 추가로 `align=Right/Center` TAC 표가 한컴 기대 위치로 정렬됨.

## 처리 절차

1. ✅ 트러블슈팅 폴더 사전 검색 (memory 규칙)
2. ✅ Stage 1: 좌표 측정 + ParaShape 비교 + 코드 위치 식별
3. ✅ Stage 2: `src/renderer/layout.rs` 수정 + 빌드/테스트 회귀 0
4. ✅ Stage 3: 5샘플 byte-diff 회귀 스캔 + WASM 브라우저 시각 검증
5. ✅ Stage 4: 트러블슈팅 등록 + 최종 보고서

## 근본 원인

`src/renderer/layout.rs::layout_columns` 의 TAC 표 분기 (`tbl_inline_x` 계산):

- 비-TAC + Square wrap 분기는 Task #295 (PR #298) 에서 `t.common.horz_align` 반영 추가됨
- **TAC 분기는 ParaShape `alignment` 무시** → `align=Right/Center` 인 TAC 표가 단/문단 좌측에 강제 정렬

KTX.hwp pi=31/32 의 경우:
- TAC 표 (treat_as_char=true, wrap=TopAndBottom)
- ParaShape align=Right
- 한컴: 단 우측 정렬 (x=518.56)
- rhwp: 단 좌측 강제 (x=494.10)
- **차이 24.46px (6.5mm)**

## 변경 내역

### 코드 (1파일, ~+15줄)

`src/renderer/layout.rs` — TAC 분기에 alignment 매치 추가:
- `align=Right` → 단 우측 끝 - 표 폭 - margin_right
- `align=Center` → 단 중앙
- 기타 (`Justify/Left/...`) → 기존 base_x 유지
- `.max(base_x)` 안전장치로 leading 보존

### 문서

- `mydocs/plans/task_m100_291{,_impl}.md`
- `mydocs/working/task_m100_291_stage{1,2,3}.md`
- `mydocs/report/task_m100_291_report.md` (이 문서)
- `mydocs/troubleshootings/tac_table_para_align.md` (재발 방지 자료)
- `mydocs/orders/20260425.md` 갱신

## 검증 결과

### 단위/통합 테스트

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **992 passed / 0 failed / 1 ignored** |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 무회귀) |
| `cargo test --test issue_301` | ✅ 1 passed |
| `cargo test --test tab_cross_run` | ✅ 1 passed |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |

### KTX.hwp 좌표 변화

| 표 | Before | After | 한컴 기대 | 오차 |
|----|--------|-------|-----------|------|
| pi=29 (비-TAC) | 744.71 | 744.71 | (변화 없음) | ✅ 영향 없음 |
| pi=31 (TAC, Right) | 494.10 | **518.16** | 518.56 | 0.4px |
| pi=32 (TAC, Right) | 494.10 | **517.95** | 518.56 | 0.6px |

### 5샘플 회귀 스캔

| 샘플 | 변경/전체 | 평가 |
|------|-----------|------|
| KTX.hwp | 1/1 | 의도 (이슈 목표) |
| exam_math.hwp | 0/20 | 무영향 |
| 21_언어_기출_편집가능본.hwp | 0/19 | 무영향 |
| **aift.hwp** | **18/74** | **모두 의도된 개선** (Center/Right TAC) |
| biz_plan.hwp | 1/6 | 의도된 개선 (Center TAC) |

### 브라우저 시각 검증

- WASM Docker 빌드: 11:59 재생성
- 작업지시자 직접 확인: **문제 없음** (회귀 없음 확인)

## 부가 성과

KTX.hwp 외에도 다음 샘플의 TAC 표 정렬이 함께 개선됨 (제보 안 된 잠재 회귀):
- **aift.hwp 18페이지**: Center/Right TAC 표 9건 + 9건
- **biz_plan.hwp 1페이지**: Center TAC 표

총 **20개 페이지의 TAC 표 정렬 개선**.

## 후속 이슈 후보 (별도 분리)

- **`t.common.horz_align` 와 ParaShape `alignment` 우선순위 정립**: 본 수정은 ParaShape 만 고려. 두 값이 충돌하는 표 발견 시 정의 필요.
- **TAC 가 아닌 inline 표의 align 처리**: Square wrap 외 케이스 검토.

## 교훈

### 1. 데이터 기반 좌표 검증의 가치

이슈 등록 시 추정한 "정적 SVG 정상, WASM Canvas 만 회귀" 가설이 작업지시자 재검증 결과 사실과 다름. 실제로는 **두 경로 모두 동일하게 좌측 밀림**이었으나, 정확한 좌표 측정 (x=494.10 vs 기대 518.56) 으로 본질을 파악할 수 있었다.

### 2. 비-TAC Square wrap 의 분기 존재가 힌트

Task #295 (PR #298) 에서 비-TAC Square wrap 분기에 `horz_align` 반영을 이미 추가한 상태였음. **TAC 분기에 같은 처리가 없는 것이 비대칭** 임을 인지하는 것이 핵심. 비대칭 패턴 → 대칭화로 해결한 케이스.

### 3. 회귀 스캔의 다층 의미

byte-diff 가 변경된 페이지는 **회귀 의심** 으로 보일 수 있으나, 본 케이스에서는 모두 **의도된 개선**. ParaShape align 분포를 분석하여 사례별 판단함. "변경 = 회귀" 가 아닌 "변경 → align 패턴 → 의도 판단".

## 관련

- 이슈: [#291](https://github.com/edwardkim/rhwp/issues/291)
- 트러블슈팅: `mydocs/troubleshootings/tac_table_para_align.md`
- 관련 작업: Task #295 (PR #298) — 비-TAC Square wrap 표의 horz_align 반영

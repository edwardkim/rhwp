# Task M100 #1378 — 1단계 완료 보고서

## 단계 목표

게이트 강화(`diff_documents` 에 본문 문단 char_shapes 시퀀스 비교 추가) + 전수 측정 +
실패 샘플 임시 xfail 등록으로 "작업 전 실패 가시화" — 구현계획서 1단계.

## 구현 내역

### 1. `src/serializer/hwpx/roundtrip.rs` — 게이트 강화

- `IrDifference::ParagraphCharShapes { section, paragraph, expected, actual }` 추가.
- `diff_documents()` 가 각 섹션의 본문(top-level) 문단별 `char_shapes` 시퀀스
  `(start_pos, char_shape_id)` 를 전체 비교. 셀·글상자 내부 문단 재귀 비교는
  계획대로 3단계에서 확장.
- 단위 테스트 4건 추가: 동일 시퀀스 무차이 / 평탄화 검출 / start_pos 변형 검출 /
  기존 케이스 무회귀.

### 2. `tests/hwpx_roundtrip_baseline.rs` — 임시 xfail

- `XFAIL_1378_RUN_SPLIT` 상수(37건) 신규 — 사유 doc 주석 포함, #1378 2~3단계에서
  해소하며 제거.
- `xfail_1378_entries_still_fail` 테스트 신규 — 해소되어 통과하게 되면 테스트가
  실패하므로 목록 제거(=baseline 복귀)가 강제된다.
- `run_baseline` 의 "검사 대상 없음" 가드를 eligible 기준으로 정비 — LARGE 3건
  전부가 임시 xfail 인 동안 오탐하지 않도록.

## 전수 측정 결과 (`samples/hwpx` 54건)

- 검사 대상 52건 (B등급 xfail 1, 제외 1) 중 **강화 게이트 실패 37건 / 통과 15건**
- char_shapes 불일치 총 **1,483건** — 비 char_shapes 차이는 0건 (구조 뼈대는 모두 보존)

### 검출 양상 분류 (모두 본문 경로 `write_section` 기인)

| 양상 | 건수 | 내용 | 해소 단계 |
|------|------|------|----------|
| ① run 평탄화 | 1,438 | 다중 char_shapes 문단이 `char_shapes[0]` 단일 run 으로 출력 | 2단계 |
| ② 섹션 첫 문단 run 구조 | 45 | 전부 각 섹션 p0. 템플릿 secPr run 의 `charPrIDRef="0"` 고정으로 재파싱 시 `(0,0)` entry 추가, 텍스트 run id 가 dedup 에 휩쓸려 소실되는 사례 포함 (예: `expected=[(0,7)] actual=[(0,0),(16,7)]`, `expected=[(0,63)] actual=[(0,0)]`) | 2단계 |

②는 수행계획서 위험 항목 "템플릿 anchor 치환 취약성"의 실증이다. 게이트가 비교하지
않던 동안 **단일 run 문단조차 섹션 첫 문단에서는 id 가 왜곡**되고 있었다.
구현계획서 1.3(텍스트 run 전체 1회 치환 + secPr run id 정비)으로 함께 해소한다.

### 샘플별 측정표 (실패 37건, diff 건수 내림차순)

| 샘플 | diff | ① 평탄화 | ② 첫 문단 구조 |
|------|------|---------|---------------|
| exam_kor.hwpx | 338 | 338 | 0 |
| aift.hwpx | 237 | 234 | 3 |
| mel-001.hwpx | 199 | 198 | 1 |
| [2027] 온새미로 1 본교재.hwpx | 125 | 122 | 3 |
| exam-kor-4p.hwpx | 68 | 68 | 0 |
| k-water-rfp.hwpx | 61 | 59 | 2 |
| exam-kor-3p.hwpx | 53 | 53 | 0 |
| 2025년 1분기 해외직접투자 보도자료f.hwpx | 40 | 38 | 2 |
| 2025년 2분기 해외직접투자 (최종).hwpx | 40 | 38 | 2 |
| hwpx-h-02.hwpx | 40 | 38 | 2 |
| hwpx-h-03.hwpx | 40 | 38 | 2 |
| 2024년 2분기 해외직접투자 보도자료ff.hwpx | 38 | 36 | 2 |
| 2024년 연간 해외직접투자 보도자료 _ ff.hwpx | 38 | 36 | 2 |
| 2024년 1분기 해외직접투자 보도자료 ff.hwpx | 37 | 35 | 2 |
| hwpx-h-01.hwpx | 37 | 35 | 2 |
| exam-kor-2p.hwpx | 26 | 26 | 0 |
| exam-kor-1p.hwpx | 13 | 13 | 0 |
| hy-001.hwpx | 9 | 8 | 1 |
| hy-002.hwpx | 7 | 6 | 1 |
| hcar-001.hwpx | 6 | 3 | 3 |
| issue_1133.hwpx | 5 | 4 | 1 |
| el-school-001.hwpx | 4 | 3 | 1 |
| exam_social-p1.hwpx | 3 | 2 | 1 |
| issue_157.hwpx | 3 | 2 | 1 |
| form-01.hwpx | 2 | 1 | 1 |
| form-02.hwpx | 2 | 1 | 1 |
| hwpx-02.hwpx | 2 | 0 | 2 |
| 143E433F503322BD33.hwpx | 1 | 1 | 0 |
| eq-002.hwpx | 1 | 0 | 1 |
| expense_report.hwpx | 1 | 1 | 0 |
| footnote-01.hwpx | 1 | 0 | 1 |
| form-002.hwpx | 1 | 0 | 1 |
| issue_241.hwpx | 1 | 0 | 1 |
| landscape-001.hwpx | 1 | 0 | 1 |
| math-001.hwpx | 1 | 1 | 0 |
| table-text.hwpx | 1 | 0 | 1 |
| tb-org-02.hwpx | 1 | 0 | 1 |
| **계** | **1,483** | **1,438** | **45** |

> 본 게이트는 본문 문단만 비교하므로 셀·글상자 내부 평탄화는 아직 집계되지 않는다
> (3단계 게이트 재귀 확장에서 추가 검출 예상).
> 분류 범위 판정: 37건 전부 본 타스크(#1378) 수정 대상 — 범위 밖 원인 없음.

## 2단계 단위 테스트로 고정할 경계 케이스 (구현계획서 1.2)

1. 경계와 컨트롤 슬롯이 같은 위치 — 경계 먼저 cut, 컨트롤은 새 run 소속
2. 경계가 슬롯 사이(텍스트 중간) — 텍스트 분할
3. 연속 동일 id 경계 skip (파서 dedup 왕복 정합)
4. 탭/lineBreak 를 포함한 run 분할
5. 빈 문단·char_shapes 빈 경우 (단일 run id 0 유지)
6. 필드 begin/end 와 경계 교차
7. 섹션 첫 문단 — secPr run id 와 텍스트 run id 의 dedup 상호작용 (양상 ②)

## 검증 결과

- `cargo test --lib` — 1663 passed / 0 failed
- `cargo test --tests` — 123개 바이너리 전부 ok (EXIT=0, baseline 게이트 + 임시 xfail 포함)
- `cargo fmt --check` — 통과
- `cargo clippy --lib --tests -- -D warnings` — 통과

## 다음 단계

2단계 — 본문 경로 `render_runs()` 전환 + `write_section` 치환 단순화 + 경계 케이스
단위 테스트. 완료 조건: 본문 기인 임시 xfail 해소(37건 중 셀·글상자 기인이 없으므로
전건 해소 목표 — 단, 셀·글상자 내부 평탄화가 본문 외 IrDiff 를 만들지 않는 한).

# Task M100 #1384 구현계획서 — borderFillIDRef 등록 축 정정 (#1381 통합)

- 수행계획서: `mydocs/plans/task_m100_1384.md` (승인 완료)
- 브랜치: `local/task1384`
- 작성일: 2026-06-14
- 단계: 3단계

## 0. 사전 조사 확정 (1단계 측정 완료)

### 0.1 결함 본체 — borderFill 등록 축 off-by-one

| 경로 | 축 | 위치 |
|------|-----|------|
| 방출 | `idx + 1` (1-based) | `header.rs:300` |
| 참조 (charPr/paraPr borderFillIDRef) | 1-based 원본 보존 | parser·serializer 무변환 |
| 인라인 등록 (표/셀) | `tbl.border_fill_id` = 1-based IR 값 | `context.rs:135~140` (정상) |
| **doc_info 등록** | **`idx`** (0-based) ← 결함 | `context.rs:117` |

→ doc_info 등록만 0-based라 마지막 id 미등록. **인라인 등록은 1-based(정상)**이므로,
doc_info 등록을 1-based로 맞추면 두 등록 경로가 통일된다.

### 0.2 회귀 안전성 전수 확인

- `border_fill_ids` 사용처: 등록 5곳(doc_info 1 + 인라인 4), 참조 3곳(table.rs),
  검사 1곳(`unresolved`). doc_info 1지점만 0-based — 나머지는 1-based.
- charShape/paraShape는 원본 id가 0-based라 `idx` 등록과 일치(무수정).

### 0.3 numbering 잠재 불일치 (범위 밖 — 기록)

- `write_numbering`도 `id + 1`(1-based) 방출인데 등록은 `idx`(0-based) —
  borderFill과 동형 불일치. **단 numbering은 `reference()` 호출이 없어**
  (`unresolved` 항상 빈집합) SERIALIZE_FAIL을 안 낸다. 표면화 안 됨.
- 본 타스크 범위(borderFill 표면화 결함)에 미포함. 최종 보고서에 잠재 결함으로
  기록 → 별도 이슈 등록 제안 여부 작업지시자 판단.

### 0.4 가설 검증 (임시 적용 → 되돌림)

`context.rs:117` → `(idx + 1)`: 4샘플(exam_social/exam_kor/exam_social-p1/issue_1133)
전부 PASS + k-water-rfp 회귀 0.

## 1단계 — (측정 완료) 보고

0절이 1단계 측정 결과. 보고서에 결함 축·회귀 안전성·numbering 잠재 기재 → 승인.
(코드 수정 없음)

## 2단계 — 수정 + xfail 승격

### 2.1 등록 축 정정

- `context.rs:117`: `register(idx as u16)` → `register((idx + 1) as u16)`.
- doc 주석에 사유 명기 (방출 id+1·인라인 1-based와 통일, #1384).

### 2.2 baseline xfail 승격

- `tests/hwpx_roundtrip_baseline.rs` `XFAIL`에서 4건(exam_social/exam_kor/
  exam_social-p1/issue_1133) 제거 → `xfail_entries_still_fail` 가드가 승격 강제.
- `ORACLE_UNFIT`(exam_kor 등 복합 실문서)은 **유지** — 시각 oracle 부적합은 별개.

### 2.3 단위 테스트

- doc_info borderFill 1-based 등록 + 1-based borderFillIDRef resolved 확인
  (수제 Document: border_fills N개 + charShape borderFillIDRef=N → SERIALIZE 성공).
- 회귀 가드: 등록이 0-based로 되돌아가면 마지막 id 미등록 실패.

### 2.4 보고: spot 배치 결과 + xfail 승격, `_stage2.md`

## 3단계 — 전수 검증 + 문서

1. `hwpx-roundtrip --batch samples/hwpx` 전수 → `output/poc/task1384/`
   (SERIALIZE_FAIL 4 → 0, PASS 49 → 53)
2. baseline + CI급 (release-test + fmt + clippy)
3. 매뉴얼 갱신 (#1384 해소 + 등급 현황 A=53/B=0)
4. 최종 보고서 + numbering 잠재 결함 기록

## 위험 관리

| 위험 | 단계 | 대응 |
|------|------|------|
| 1-based 전환이 다른 소비처 회귀 | 2·3 | 0.2 전수 확인(1지점) + 전수 배치 회귀 검출 |
| ORACLE_UNFIT 중복 처리 누락 | 2 | exam_kor 등 ORACLE_UNFIT 유지, 매뉴얼 명시 |
| numbering 잠재 결함 미기록 | 3 | 최종 보고서 명시 + 이슈 제안 |
| #1381/#1384 중복 클로즈 | 3 후 | 두 이슈 동시 close + 상호 참조 |

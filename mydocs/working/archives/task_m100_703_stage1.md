# Task #703 Stage 1 — TDD RED 완료 보고서

**Issue**: #703
**브랜치**: `local/task703`
**작성일**: 2026-05-08
**구현계획서**: `mydocs/plans/task_m100_703_impl.md`

---

## 1. 작업 내용

결함 재현 + 검증 기준 확립을 위한 RED 단계 테스트 작성.

### 1.1 통합 테스트 (`tests/issue_703.rs`, 신규 +57 줄)

3개 샘플 파일에 대한 페이지 수 == 1 검증:

| 테스트 | 샘플 | 기대 | 현재 (RED) |
|--------|------|------|------------|
| `issue_703_calendar_year_single_page` | `samples/basic/calendar_year.hwp` | 1 | 2 |
| `issue_703_tonghap_2010_11_single_page` | `samples/통합재정통계(2010.11월).hwp` | 1 | 2 |
| `issue_703_tonghap_2011_10_single_page` | `samples/통합재정통계(2011.10월).hwp` | 1 | 2 |

### 1.2 단위 테스트 (`src/renderer/typeset.rs#mod tests`, 신규 +66 줄)

`test_typeset_703_behind_text_table_no_flow_advance`:

- BehindText 1×1 wrapper Table (높이 60000 HU ≈ 800 px) 캐리어 빈 paragraph + 후속 5 단락 fixture
- paginator (engine.rs reference) 와 typeset (현재 메인) 결과 비교
- **paginator**: 1 페이지 (BehindText 가드 정상 작동) — 검증 1 통과
- **typeset**: 2 페이지 (현재 결함) — 검증 2 RED

## 2. 실행 결과

### 통합 테스트

```
$ cargo test --release --test issue_703
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

세 케이스 모두 `left: 2, right: 1` 로 실패 (기대: 1 페이지, 실제: 2 페이지).

### 단위 테스트

```
$ cargo test --lib --release -- test_typeset_703_behind_text_table_no_flow_advance
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1159 filtered out
```

panic 메시지:
```
[BUG #703] typeset 도 1 페이지여야 함. 결함 시 BehindText 표 height ≈800 px 가
cur_h 에 가산되어 후속 paragraph 가 다음 페이지로 밀림 (RED)
left: 2
right: 1
```

## 3. RED 의미

- **정정 위치 확정**: `typeset_block_table` → `place_table_with_text` (line 1539-1635) 의 `cur_h += pre_height + table_total_height` 가 BehindText/InFrontOfText 표에 대해 가산되는 것이 결함의 직접 원인.
- **paginator vs typeset 비교**: paginator 는 동일 fixture 에서 1 페이지로 정상 동작. 즉 engine.rs:976-981 의 BehindText/InFrontOfText 가드 시멘틱이 정답지.
- **회귀 안전망**: 단위 테스트는 작은 fixture 로 빠르게 핵심 결함 검출. 통합 테스트는 실제 샘플 파일로 end-to-end 검증.

## 4. 다음 단계

Stage 2 (GREEN): `typeset_table_paragraph` 의 Control::Table 분기 (line 1369 부근) 에 BehindText/InFrontOfText 가드 추가 → RED 4 테스트 모두 GREEN 전환.

## 5. 변경 파일

| 파일 | 변동 |
|------|------|
| `tests/issue_703.rs` | 신규 +57 줄 |
| `src/renderer/typeset.rs` | +66 줄 (mod tests) |
| `mydocs/plans/task_m100_703.md` | 신규 (수행계획서) |
| `mydocs/plans/task_m100_703_impl.md` | 신규 (구현계획서) |
| `mydocs/working/task_m100_703_stage1.md` | 신규 (본 보고서) |
| `mydocs/report/svg_vs_pdf_diff_20260508.tsv` | 신규 (196 샘플 비교 데이터) |

## 6. 작업지시자 승인 요청

Stage 1 RED 확인 + Stage 2 (GREEN) 진행 승인 부탁드립니다.

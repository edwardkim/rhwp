# Task #470 최종 결과 보고서

## 이슈

- 번호: #470
- 제목: 21_언어_기출 1p 좌측 단 pi=10 cross-column vpos-reset 미인식 (cv != 0)
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task470`

## 증상

`samples/21_언어_기출_편집가능본.hwp` 페이지 1 좌측 단(col 0) 하단에 pi=10 ("적합성 검증이란…") 의 처음 2줄이 배치되어 본문 영역을 56.2px 넘침. PDF 원본은 pi=10 전체가 우측 단(col 1) 상단에 위치.

```
LAYOUT_OVERFLOW: page=0, col=0, para=10, type=PartialParagraph, y=1492.3, bottom=1436.2, overflow=56.2px
```

dump-pages: col 0 hwp_used≈159px, 실제 used=1202.7px — HWP 의도와 큰 격차.

## 근본 원인

`src/renderer/typeset.rs:415` (Task #321) cross-paragraph vpos-reset 검출이 `cv == 0` (정확히 0) 만 인정:

```rust
if cv == 0 && pv > 5000 {
    st.advance_column_or_new_page();
}
```

본 케이스:
- pi=9 last vpos = 90426 (col 0 바닥 근처)
- pi=10 first vpos = **9014** (col 1 컬럼 헤더 오프셋 — pi=0 의 first vpos 와 동일)

`9014` 는 col 1 의 컬럼 헤더("언어이해" 글상자 + 세로 구분선) 만큼 비우기 위한 col 1 시작 오프셋. `cv < pv` (9014 < 90426) 이지만 `cv == 0` 가드 때문에 reset 미감지 → pi=10 partial 이 col 0 에 강제 삽입 → overflow.

## 수정

### 1차 시도 → 회귀 발견

`cv == 0` → `cv < pv` 로 단순 완화. 21_언어_기출 통과했으나 `tests/issue_418.rs` 실패.

#### 회귀 원인

`hwpspec.hwp` 단일 단 섹션 2:
- pi=78 (block 표 anchor, RowBreak 분할) last vpos = 56052
- pi=79 (text) first vpos = 27872

partial-table split 의 LAYOUT 잔재로 cv < pv 발생. HWP 의도는 페이지 reset 이 아닌 "표 분할 후 같은 페이지 후속 paragraph". 잘못 트리거 → image-bearing paragraphs (pi=83/86/89) 가 다음 페이지로 이동 → issue_418 fail.

### 최종 수정 — 다단/단일 단 분기

`src/renderer/typeset.rs:415, 439`:

```rust
let trigger = if st.col_count > 1 {
    cv < pv && pv > 5000   // 다단: 컬럼 헤더 오프셋 (cv != 0) 도 인정
} else {
    cv == 0 && pv > 5000   // 단일 단: partial-split 잔재 회피 (Task #321 보수적 기준 유지)
};
```

근거:
- HWPUNIT vpos 는 컬럼 내 단조 증가하지만, 단일 단에서 partial-table 분할은 `cv < pv` 형태의 LAYOUT 잔재를 만들 수 있음.
- 다단 섹션은 partial-table 분할이 일반적이지 않고, `cv < pv` 가 명확한 컬럼 reset 시그널.
- L439 look-ahead 로직 (Task #359 단독 항목 페이지 차단) 도 동일 패턴 적용.

## 추가된 테스트

`src/renderer/layout/integration_tests.rs::test_470_cross_paragraph_vpos_reset_with_column_header_offset`

- `samples/21_언어_기출_편집가능본.hwp` 페이지 1 의 `dump_page_items` 결과 검증
- 단 0 블록에 `pi=10` 미포함, 단 1 블록에 `pi=10` 포함
- 수정 전: FAIL → 수정 후: PASS

## 검증 결과

| 항목 | 결과 |
|------|------|
| 신규 단위 테스트 | PASS |
| `tests/issue_418.rs` (hwpspec page 20 image count) | PASS |
| 전체 cargo test (1122건) | 1122 / 1122 PASS |
| 21_언어_기출 OVERFLOW | 13 → 10 (-3) ✓ |
| exam_science OVERFLOW | 5 → 0 (-5) ✓ |
| exam_social OVERFLOW | 4 → 1 (-3) ✓ |
| hwpspec OVERFLOW | 45 → 45 (회귀 0) |
| exam_kor / exam_eng / exam_math OVERFLOW | 회귀 0 |

다단 샘플 11건 추가 해소. 단일 단 샘플 회귀 0.

## 영향 범위

- 다단(`col_count > 1`) 섹션의 cross-paragraph vpos-reset 검출에 한정.
- 단일 단 섹션은 기존 Task #321 보수적 기준 유지 → issue_418 등 회귀 차단.
- `pagination/engine.rs` (opt-in `RHWP_USE_PAGINATOR=1`) 는 별도 follow-up task 로 분리.

## 산출물

- `src/renderer/typeset.rs` (2개 분기 수정)
- `src/renderer/layout/integration_tests.rs` (테스트 1건 추가)
- `mydocs/plans/task_m100_470.md` (수행계획서)
- `mydocs/plans/task_m100_470_impl.md` (구현계획서)
- `mydocs/working/task_m100_470_stage{1,2,3}.md` (단계별 보고서)
- `mydocs/report/task_m100_470_report.md` (본 문서)

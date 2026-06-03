# Task M100-1113 — Stage 2 정정 보고서

- 이슈: [#1113](https://github.com/edwardkim/rhwp/issues/1113)
- 일시: 2026-05-29
- 단계: Stage 2 (정정 + round1) — **byte 정합 완료, 한컴 시각 판정 대기**

## 1. 정정 영역

`src/document_core/converters/hwpx_to_hwp.rs` — `materialize_master_page_autonum_placeholder`

### 변경 내용

```rust
// 변경 전
if context != ParagraphContext::MasterPage {
    return;
}

// 변경 후 [Task #1113]
if !matches!(
    context,
    ParagraphContext::MasterPage | ParagraphContext::HeaderFooter
) {
    return;
}
```

바탕쪽(MasterPage) 한정이던 AutoNumber-only 문단 placeholder 공백(U+0020) 제거 로직을 **머리말/꼬리말(HeaderFooter) context** 에도 확장.

### 정정 근거

- HWPX 원본은 페이지번호 단독 (`<hp:t/>` 빈 텍스트). rhwp 파서가 HWP5 PARA_TEXT 조립 시 placeholder 공백 U+0020 합성.
- 어댑터가 바탕쪽에서만 제거 → 머리말 글상자(HeaderFooter)는 누락 = Trigger A.
- 정답지(한컴)도 머리말 페이지번호를 AutoNumber-only 로 저장 (visible 공백 없음).

### 회귀 안전성

조건 `para.text != " "` (단독 공백) + `controls.len()==1` + autonum 단독 유지.
→ 짝수쪽 머리말 (fwSpace+텍스트+autoNum) 은 `text != " "` 에서 자동 제외. 일반 머리말 텍스트 문단도 제외.

## 2. round1 byte 정합 검증

산출본: `output/poc/issue_1113/exam_social-round1.hwp` (427008 bytes)

### 2.1 홀수쪽(apply_type=2) 페이지번호 문단 — 정정 성공

| record | baseline (정정 전) | round1 (정정 후) | 정답지 |
|--------|-------------------|------------------|--------|
| PARA_HEADER #154 | char_count=10 (불일치) | char_count=9 | char_count=9 ✅ |
| PARA_TEXT #155 | size=20 (U+0020 추가, 불일치) | size=18 | size=18 ✅ |

→ **페이지번호 글상자 문단(rec #154/#155) 정답지와 byte 완전 일치.** Trigger A 제거.

### 2.2 회귀 검증 — 짝수쪽(apply_type=1) 영향 없음

apply_type=1 (짝수) subtree diff 는 baseline 과 **동일** (PARA_HEADER/PARA_TEXT 공백 항목 원래 없음). 정정이 짝수쪽에 영향 없음.

### 2.3 잔존 차이 (Stage 1 분류 — trigger 후보 아님)

- [7] TABLE byte[3], [8] LIST_HEADER byte[7], [9] PARA_HEADER byte[7] — 홀/짝 공통 (정상)
- [10] PARA_TEXT — fixed-width space 0x001f vs 0x2007 (Trigger B, 텍스트 폭)
- [14] SHAPE_COMPONENT — 도형 렌더링 매트릭스 미세값
- (짝수쪽 [15] LIST_HEADER tail — 이전 round1 영역, 짝수쪽이므로 무관)

→ 이 차이들은 한컴 시각 무해 (짝수쪽에도 동일 발생, 짝수쪽 정상). Trigger A 단독 정정 → 시각 판정.

## 3. 자체 검증

- `cargo test --release --lib hwpx_to_hwp` → **32 passed, 0 failed**
- `hwpx_h_03_draw_text_*` 등 글상자 관련 테스트 통과 (회귀 없음)

## 4. 한컴 시각 판정 요청 (게이트)

작업지시자 한컴 한글 2020/2022 시각 판정 항목:

1. **3페이지 홀수쪽 머리말** 글상자 — 페이지번호 앞 공백/줄바꿈 **없음** + 글상자 높이 정상 ✅ 인지
2. **짝수쪽 머리말** — 정상 (회귀 없음) ✅ 인지
3. 다른 페이지 — 정상 (회귀 없음) ✅ 인지

산출본: `output/poc/issue_1113/exam_social-round1.hwp`
비교 자료: 정답지 `samples/exam_social.hwp`, baseline `output/poc/issue_1113/exam_social-current.hwp`

## 5. Stage 3 (승인 후)

- 회귀 가드: `tests/issue_1113_*.rs` — 머리말 글상자 AutoNumber-only byte 검증
- 최종 보고서: `mydocs/report/task_m100_1113_report.md`
- commit/merge + 이슈 close

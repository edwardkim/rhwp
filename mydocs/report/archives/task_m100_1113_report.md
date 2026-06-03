# Task M100-1113 최종 보고서 — exam_social.hwpx 홀수쪽 머리말 글상자 페이지번호 줄바꿈/높이 정합

- 이슈: [#1113](https://github.com/edwardkim/rhwp/issues/1113)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1113`
- 작성일: 2026-05-29
- 선행 task: #1110 (파일손상/문서변조 판정 제거)

## 1. 본질

HWPX→HWP 저장본을 한컴 에디터에서 열면 **3페이지 홀수쪽 머리말 글상자** 안 페이지번호 앞에 공백/줄바꿈이 생기고 글상자 높이가 비정상적으로 커졌다. rhwp-studio 와 짝수쪽 머리말은 정상.

원인: HWPX 머리말 글상자는 `<hp:autoNum>` + 빈 `<hp:t/>` 로 페이지번호만 담는데, rhwp 파서가 HWP5 PARA_TEXT 조립 시 AutoNumber placeholder 공백(U+0020)을 합성한다. 어댑터의 `materialize_master_page_autonum_placeholder` 가 이 공백을 제거하지만 **바탕쪽(MasterPage) 에만 적용**되어, 머리말(HeaderFooter) 홀수쪽 글상자(폭 4252)에서 잉여 공백이 남았다. 한컴이 이 공백을 페이지번호와 함께 좁은 글상자에서 줄나눔 → 높이 증가.

## 2. 진단 (Stage 1)

### 2.1 apply_type 매핑 확정 (이전 세션 오류 정정)

HWP5 apply_type **1=Even(짝수), 2=Odd(홀수)** (`serializer/control.rs:634`). 이전 세션 round1 은 apply_type=1 을 홀수로 착각하여 짝수쪽 LIST_HEADER tail 을 정정 → 시각 무효. 본 진단으로 **apply_type=2 = 홀수쪽** 확정.

### 2.2 trigger 확정 (record byte diff)

진단 도구 `examples/dump_odd_header_1113.rs` 로 정답지 vs 저장본 머리말 subtree byte 비교:

| record | 정답지 | 저장본 (정정 전) |
|--------|--------|------------------|
| PARA_HEADER #154 | char_count=9 | char_count=10 |
| PARA_TEXT #155 | size=18: `[0x0012 autonum][...][0x000d]` | size=20: **`[0x0020 공백]`** `[0x0012]...` |

→ 저장본이 페이지번호 앞에 공백 U+0020 합성. HWPX 원본·정답지 모두 페이지번호 단독.

## 3. 정정 (Stage 2)

`src/document_core/converters/hwpx_to_hwp.rs` — `materialize_master_page_autonum_placeholder`:

```rust
// 변경 전: 바탕쪽 한정
if context != ParagraphContext::MasterPage { return; }
// 변경 후 [Task #1113]: 바탕쪽 + 머리말/꼬리말
if !matches!(context, ParagraphContext::MasterPage | ParagraphContext::HeaderFooter) {
    return;
}
```

AutoNumber-only 문단 placeholder 공백 제거 로직을 머리말/꼬리말 context 에도 확장.

조건 `para.text != " "` + `controls.len()==1` + autonum 단독 유지 → 짝수쪽(fwSpace+텍스트+autoNum) 자동 제외, 회귀 없음.

## 4. 검증

### 4.1 byte 정합 (round1)

- 홀수쪽 페이지번호 문단(rec #154/#155): 정정 후 정답지와 byte 완전 일치 (char_count=9, PARA_TEXT size=18)
- 짝수쪽(apply_type=1): baseline 과 동일 (영향 없음)

### 4.2 한컴 시각 판정 (게이트) — 통과

작업지시자 한컴 에디터 확인: **3페이지 홀수쪽 머리말 정상** (페이지번호 앞 공백/줄바꿈 없음, 글상자 높이 정상).

### 4.3 자동 검증

- `cargo test --release --lib hwpx_to_hwp` → 32 passed, 0 failed
- 회귀 가드 `tests/issue_1113_header_autonum_placeholder.rs` → 1 passed
- `cargo test --release --tests` → (회귀 없음)
- `cargo fmt --all -- --check` → 정합
- `cargo clippy --lib --release -- -D warnings` → warnings 0

## 5. 변경 파일

- `src/document_core/converters/hwpx_to_hwp.rs` — `materialize_master_page_autonum_placeholder` context 확장 (소스)
- `tests/issue_1113_header_autonum_placeholder.rs` — 회귀 가드 (신규)
- `examples/dump_odd_header_1113.rs`, `examples/repro_1113_save.rs`, `examples/repro_1113_round1.rs` — 진단 도구
- 문서: `mydocs/plans/task_m100_1113*.md`, `mydocs/working/task_m100_1113_stage{1,2}.md`, 본 보고서

## 6. 잔존 차이 (한컴 시각 무해)

머리말 subtree 의 다른 byte 차이 (TABLE byte[3], LIST_HEADER byte[7], SHAPE_COMPONENT 렌더링 매트릭스 미세값, 본문 fixed-width space 0x001f vs 0x2007) 는 홀·짝 공통 발생 (짝수쪽 정상) → trigger 아님. 시각 판정 통과로 무해 확인.

## 7. 메모리 룰 정합

- `feedback_self_verification_not_hancom` — rhwp-studio 정상 ≠ 한컴 호환 (본 task 의 본질)
- `feedback_pdf_not_authoritative` — 한컴 시각 판정 게이트 통과
- `feedback_hancom_compat_specific_over_general` — text==" " + autonum 단독 조건의 case-specific 가드
- `feedback_diagnosis_layer_attribution` — apply_type 매핑 정정으로 trigger 본질 위치 정확 귀속 (이전 세션 실패 교훈)
- `feedback_push_full_test_required` — cargo test --tests + fmt --check 점검

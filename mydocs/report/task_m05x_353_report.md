# 최종 보고서 — Issue #353 쪽번호 처리

GitHub Issue: https://github.com/edwardkim/rhwp/issues/353
브랜치: `local/task353` (base: devel)

## 한 일

NewNumber 컨트롤이 모든 후속 페이지에 매번 재적용되어 page_number 가 사실상 1로 고정되던 회귀 버그를 해결.

| 영역 | 변경 |
|------|------|
| `src/renderer/page_number.rs` (신규) | `PageNumberAssigner` — consumed set + 단조 증가 카운터 |
| `src/renderer/mod.rs` | `pub mod page_number;` |
| `src/renderer/typeset.rs` | TypesetEngine::finalize_pages 의 NewNumber 블록 → Assigner 호출 |
| `src/renderer/pagination/engine.rs` | Paginator::finalize_pages 의 NewNumber 블록 → Assigner 호출 |
| `tests/page_number_propagation.rs` (신규) | 회귀 테스트 2건 |

## 핵심 변경 로직

기존 (버그):
```rust
if let Some(fp) = first_para {
    for &(nn_pi, nn_num) in new_page_numbers {
        if nn_pi <= fp {           // 매 페이지마다 참
            page_num = nn_num as u32;
        }
    }
}
```

수정:
```rust
let mut assigner = PageNumberAssigner::new(new_page_numbers, 1);
for page in pages.iter_mut() {
    let page_num = assigner.assign(page);  // NewNumber 1회만 적용 + 자동 +1
    ...
}
```

`assign()` 내부:
1. consumed 에 없는 NewNumber 중 그 컨트롤의 소유 문단이 페이지에서 처음 등장하면 적용 + consumed 마킹
2. 카운터 증가

"처음 등장" 판정:
- `FullParagraph` / `Table` / `Shape`: 항상 인정
- `PartialParagraph`: `start_line == 0` 일 때만
- `PartialTable`: `is_continuation == false` 일 때만

## 검증

| 항목 | 결과 |
|------|------|
| 단위 테스트 (renderer::page_number) | 6/6 통과 |
| 회귀 테스트 (page_number_propagation) | 2/2 통과 (기본 경로) |
| 회귀 테스트 (RHWP_USE_PAGINATOR=1) | 2/2 통과 |
| 전체 cargo test | 1006 + 49 (통합) 모두 통과, 회귀 0 |
| clippy | 신규 경고 0 |
| 시각 검증 | PDF 페이지 5/10/15/20/30/35 푸터 SVG 와 정확 일치 |
| 다른 샘플 spot check | exam_kor / exam_math / 21_언어 모두 1..N 단조 |

## 성과

`samples/2022년 국립국어원 업무계획.hwp`:
- 시퀀스: `[1, 2, 3, 1, 1, ..., 1]` → `[1, 2, 1, 2, 3, ..., 33]`
- 푸터 페이지 10: `- 1 -` → `- 8 -` (PDF 와 일치)
- 푸터 페이지 30: `- 1 -` → `- 28 -` (PDF 와 일치)

## 명시적 제외 (별도 이슈로 추적)

- 전체 페이지 수 차이 (PDF 37 vs SVG 35) — 섹션 경계 빈/꼬리 페이지 누락 별도 이슈 필요
- HWPPAGE_ODD/EVEN 강제 시작 — 이번 문서는 미해당, 별도 이슈

# 구현 계획서 — Issue #353 쪽번호 처리

수행 계획서: `mydocs/plans/task_m05x_353.md`

## 설계 요지

쪽번호 할당을 두 finalize_pages 구현 (`typeset.rs`, `pagination/engine.rs`) 에서 분리하여
공통 함수 `assign_page_numbers` 로 통합한다. 핵심 규칙:

1. NewNumber 컨트롤은 그 컨트롤의 **소유 문단이 페이지에서 처음 등장하는** 페이지에서 1회만 적용
2. 적용된 NewNumber 는 `consumed: HashSet<usize>` 로 추적하여 재적용 방지
3. 그 외 페이지는 직전 페이지 page_number + 1
4. PageHide.hide_page_num=true 라도 카운터는 계속 흘러야 함 (표시만 생략)

"페이지에서 처음 등장" 판정:
- `FullParagraph { para_index = nn_pi }`
- `PartialParagraph { para_index = nn_pi, start_line = 0 }`
- `Table { para_index = nn_pi }`
- `PartialTable { para_index = nn_pi, continuation_index = 0 }`
- `Shape { para_index = nn_pi }` (control_index 까지 일치)

## 새 모듈

`src/renderer/page_number.rs`:

```rust
pub(crate) struct PageNumberAssigner<'a> {
    new_page_numbers: &'a [(usize, u16)],
    consumed: std::collections::HashSet<usize>,
    counter: u32,
}

impl<'a> PageNumberAssigner<'a> {
    pub fn new(new_page_numbers: &'a [(usize, u16)], initial: u32) -> Self {
        Self { new_page_numbers, consumed: HashSet::new(), counter: initial }
    }

    /// 페이지에 쪽번호를 할당하고, 다음 페이지를 위해 카운터를 1 증가시킨다.
    pub fn assign(&mut self, page: &PageContent) -> u32 {
        for (idx, &(nn_pi, nn_num)) in self.new_page_numbers.iter().enumerate() {
            if self.consumed.contains(&idx) { continue; }
            if Self::para_first_appears(page, nn_pi) {
                self.counter = nn_num as u32;
                self.consumed.insert(idx);
                break; // 한 페이지에서는 최대 1개의 NewNumber 만 적용
            }
        }
        let assigned = self.counter;
        self.counter += 1;
        assigned
    }

    fn para_first_appears(page: &PageContent, target_pi: usize) -> bool {
        page.column_contents.iter().any(|col| col.items.iter().any(|item| match item {
            PageItem::FullParagraph { para_index } => *para_index == target_pi,
            PageItem::PartialParagraph { para_index, start_line, .. } =>
                *para_index == target_pi && *start_line == 0,
            PageItem::Table { para_index, .. } => *para_index == target_pi,
            PageItem::PartialTable { para_index, continuation_index, .. } =>
                *para_index == target_pi && *continuation_index == 0,
            PageItem::Shape { para_index, .. } => *para_index == target_pi,
        }))
    }
}
```

(`PartialTable` 의 `continuation_index` 가 실제 모델에 있는지 단계 1 에서 확인 — 없으면 시그니처 조정)

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/page_number.rs` | 신규 — `PageNumberAssigner` |
| `src/renderer/mod.rs` | `pub(crate) mod page_number;` 추가 |
| `src/renderer/typeset.rs` | `finalize_pages` 의 NewNumber 블록을 `PageNumberAssigner::assign` 호출로 치환 |
| `src/renderer/pagination/engine.rs` | 동일하게 치환 |
| `tests/page_number_propagation.rs` | 신규 — 회귀 테스트 |

## 단계

### 단계 1 — 모델 확인 + 회귀 테스트 작성 (TDD)

- `PageItem::PartialTable` 의 필드 확인 (continuation_index 또는 split_index 등 실명)
- `tests/page_number_propagation.rs` 신설:
  - 케이스 A: NewNumber 1개 (para 5에 NewNumber=1) — 페이지가 5장일 때 page_num 시퀀스 = [1, 1, 2, 3, 4] (NewNumber 가 page 2 에서 적용됐다고 가정) **혹은** 실제 mini HWP 로드
  - 케이스 B: 실제 `samples/2022년 국립국어원 업무계획.hwp` 로딩 후 dump-pages 와 동일한 검증으로 page_num 시퀀스가 [1,2,3,4,...,35] 인지 (PageHide 무시)
  - 케이스 C: NewNumber 없음 — [1,2,3,...]

이 시점에는 **테스트가 실패한다** (현재 버그).

### 단계 2 — `PageNumberAssigner` 구현

- 위 모듈 신설
- 단위 테스트 (`page_number.rs` 안에 `#[cfg(test)] mod tests`):
  - new_page_numbers 비어 있으면 1, 2, 3, ...
  - NewNumber 1개 + 트리거 페이지 = 해당 페이지에서 적용, 이후 +1
  - NewNumber 2개 (예: 첫 구역 시작 page=1, 별첨 시작 page=1) 가 각각 1회만 적용
  - 트리거 문단이 PartialParagraph(start_line=0) 인 경우 적용
  - 트리거 문단이 PartialParagraph(start_line>0) 인 경우 미적용 (이미 이전 페이지에서 적용됨)

### 단계 3 — typeset.rs / engine.rs 양쪽에 어플라이

- 두 finalize_pages 의 NewNumber 처리 부분만 `PageNumberAssigner` 호출로 치환
- 머리말/꼬리말 carry 로직, PageHide 처리는 그대로 유지

### 단계 4 — 회귀 검증

- `cargo test page_number_propagation` 통과
- `cargo test` 전체 통과 (회귀 0건)
- `dump-pages samples/2022년 국립국어원 업무계획.hwp` 로 페이지 1~35 의 page_num 시퀀스가 1..=35 임을 확인
- `cargo clippy --all-targets` 0 warning

### 단계 5 — 시각 검증

- `rhwp export-svg samples/2022년 국립국어원 업무계획.hwp -o output/svg/gugeo_after/`
- PDF 와 페이지 5/10/20/30 푸터 비교 (PNG side-by-side)
- 다른 샘플 회귀 spot check: `samples/exam_kor.hwp`, `samples/exam_math.hwp`, `samples/21_언어_기출_편집가능본.hwp` (각각 10페이지 미만, 머리말 쪽번호 패턴 검증)

### 단계 6 — 문서화

- `working/task_m05x_353_stage{1..5}.md` (단계별 보고서)
- `report/task_m05x_353_report.md` (최종 보고서)
- `orders/{오늘}.md` 갱신

## 완료 기준

| 항목 | 기준 |
|------|------|
| 회귀 테스트 | `tests/page_number_propagation.rs` 통과 |
| 전체 테스트 | `cargo test` 회귀 0건 |
| 시각 | PDF 페이지 5/10/20/30 푸터와 SVG 푸터 일치 |
| Clippy | warning 0 |
| 다른 샘플 | exam_kor / exam_math 등 spot check 회귀 0건 |

## 명시적 제외 (별도 이슈)

- 전체 페이지 수 부족 (PDF 37 vs SVG 35) → 새 이슈
- HWPPAGE_ODD/EVEN 강제 시작 → 새 이슈
- 머리말 안 쪽번호 위치(format/position) 추가 케이스 → 발견 시 별도 이슈

## 승인 요청

위 구현 계획대로 단계 1 (회귀 테스트 작성) 부터 진행해도 되는지 확인 부탁드립니다.

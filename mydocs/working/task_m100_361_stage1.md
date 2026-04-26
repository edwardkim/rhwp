# Task #361 Stage 1 — 결함 origin 정량 분석

## Task 1.1 — page_num 결함 origin 식별

### 진단 방법
`typeset_section` 의 `finalize_pages` 호출 전후로 임시 디버그 출력을 추가하고 `dump-pages` 실행.

### 진단 결과 (k-water-rfp.hwp)

```
T361 BEFORE finalize_pages: section=0 pages.len()=2
T361 AFTER finalize_pages: section=0, page_numbers=[1, 2]   ← 정상
T361 BEFORE finalize_pages: section=1 pages.len()=26
T361 AFTER finalize_pages: section=1, page_numbers=[1, 1, 1, 1, 1, ..., 1]  ← 모두 1
```

- **section=0**: 정상 갱신 (NewNumber 컨트롤 영향 없음)
- **section=1**: 모든 26 페이지 page_num=1

### 결함 위치 (`src/renderer/typeset.rs:1697-1715`)

```rust
for page in pages.iter_mut() {
    let first_para = page.column_contents.first()
        .and_then(|col| col.items.first())
        .map(|item| ...);

    if let Some(fp) = first_para {
        for &(nn_pi, nn_num) in new_page_numbers {
            if nn_pi <= fp {
                page_num = nn_num as u32;  // ← 매 페이지마다 재설정
            }
        }
    }
    // ...
    page.page_number = page_num;
    page_num += 1;
}
```

**문제**:
- `nn_pi <= fp` 조건만 체크
- section=1 의 NewNumber 컨트롤이 첫 문단 (pi=0) 에 있다면 **모든 후속 페이지의 first_para >= 0** 이므로 매 페이지마다 page_num 을 nn_num (=1) 으로 강제 재설정
- 직전의 `page_num += 1` 가 다음 iteration 에서 다시 1 로 덮어쓰여짐

### 정상 시멘틱 (`src/renderer/pagination/engine.rs:1850-1856`)

```rust
for (para_idx, new_num) in new_page_numbers {
    if *para_idx > prev_page_last_para || i == 0 {
        if *para_idx <= page_last_para {
            page_num_counter = *new_num as u32;
        }
    }
}
```

조건 (Paginator 의 정상 시멘틱):
- `para_idx > prev_page_last_para`: NewNumber 가 이전 페이지에 이미 적용되지 않았음
- `para_idx <= page_last_para`: NewNumber 가 이 페이지 안에 있음
- → NewNumber 는 한 페이지에서 한 번만 적용 → 후속 페이지는 +1 정상 증가

### 수정 방향

TypesetEngine 의 finalize_pages 에 동일 시멘틱 적용:
1. `prev_page_last_para` 추적 변수 추가 (초기값: -1 또는 None)
2. NewNumber 적용 조건을 `nn_pi > prev_page_last_para && nn_pi <= page_last_para` 로 변경
3. 페이지 끝 문단 (`page_last_para`) 계산은 finalize_pages 에 이미 있음 (line 1717-1727 의 `.max()`)

## Task 1.2 — vpos / hwp_used 누적 origin 식별

### 진단 방법
TypesetEngine vs Paginator (RHWP_USE_PAGINATOR=1) 의 dump-pages 비교.

### 진단 결과

```
=== TypesetEngine k-water-rfp p4 ===
  단 0 (items=12, used=897.1px, hwp_used≈1783.5px, diff=-886.4px)
    FullParagraph  pi=32  vpos=67200..69280   ← 누적값
    FullParagraph  pi=33  vpos=71360..80000

=== Paginator (RHWP_USE_PAGINATOR=1) k-water-rfp p4 ===
  단 0 (items=13, used=899.2px, hwp_used≈1811.7px, diff=-912.5px)
    PartialParagraph  pi=32  lines=1..2  vpos=69280   ← 누적값
    FullParagraph  pi=33  vpos=71360..80000
```

### 핵심 사실

1. **vpos 누적은 HWP 원본 데이터** — Paginator 도 동일하게 누적값 표시
   - `LINE_SEG.vertical_pos` 가 section 내 절대 좌표 (페이지 reset 없음)
   - dump 출력에서 vpos 가 누적되는 것은 결함이 아니라 원본 그대로

2. **hwp_used 누적은 `compute_hwp_used_height` 의 알고리즘 한계**
   - 알고리즘: vpos-reset 발견 시 reset 직전 vpos + line_height, 미발견 시 마지막 항목의 vpos + line_height
   - section 내 vpos-reset 이 없는 페이지에서는 절대 vpos 를 반환 → 누적값으로 보임
   - Paginator 도 동일 함수 사용 → 동일 결과
   - **본 task 범위 외**: dump 도구의 표시 결함 (별도 task 후보)

3. **콘텐츠 split 차이 — items 1 차이의 origin**
   - TypesetEngine: `FullParagraph pi=32` (전체) 를 페이지 4 에 배치
   - Paginator: `PartialParagraph pi=32 lines=1..2` (1번째 라인부터) 를 페이지 4 에, 0번째 라인은 페이지 3 에 split
   - **이는 fit/split 로직 차이** — TypesetEngine 이 한 페이지에 더 많이 욱여넣음
   - 본 task 범위 외 (별도 정리 필요), 회귀 영향은 페이지마다 ~1 항목 차이

## Task 1.3 — section 1 의 page_num 시작값

### 진단 결과 (k-water-rfp.hwp section=1)

- v0.7.3: p3 (section=1 첫 페이지) page_num=1, p4=2, p5=3, ... (정상)
- main TypesetEngine: 모두 page_num=1
- main + RHWP_USE_PAGINATOR=1: p3=1, p4=2, p5=3, ... (정상)

→ TypesetEngine 의 NewNumber 적용 로직만 결함 (Task 1.1 의 origin 과 동일).

## 종합 — Stage 2 의 수정 대상 확정

### 본 task 에서 수정할 것
**TypesetEngine 의 `finalize_pages` 의 NewNumber 적용 조건 수정** — 한 줄 (또는 짧은 블록) 변경:
```rust
// 현재
if nn_pi <= fp { page_num = nn_num as u32; }

// 수정 후 (Paginator 시멘틱)
if (nn_pi as i64) > prev_page_last_para
    && first_para.map_or(false, |fp| nn_pi <= fp)
    && page_last_para.map_or(false, |lp| nn_pi <= lp) {
    page_num = nn_num as u32;
}
```

### 본 task 에서 수정 안 할 것 (별도 task 후보)
1. **vpos / hwp_used 누적 표시** — Paginator 도 동일, dump 도구 결함, 별도 처리
2. **콘텐츠 split 차이** — TypesetEngine 의 fit/split 로직 차이, 별도 task

## 다음 단계 (Stage 2)

1. NewNumber 적용 수정안 코드 확정 (Paginator 시멘틱 직접 이식)
2. page_number 사용처 grep 으로 영향 범위 점검
3. 회귀 검증 항목 명세 (kps-ai p1~p11 page_num: 1~8 / k-water-rfp p3~end: 1, 2, 3, ...)

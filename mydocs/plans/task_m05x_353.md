# 쪽번호 처리 수행 계획서 (이슈 미배정 — 사전 계획서)

## 1. 배경

`samples/2022년 국립국어원 업무계획.hwp` 와 한컴 PDF를 비교하던 중,
SVG 출력의 모든 본문 페이지가 동일한 쪽번호 `- 1 -` 으로 렌더링되는 것을 확인.

```
페이지 1 → page_num=1
페이지 2 → page_num=2     ✓ (NewNumber 컨트롤로 1로 리셋되기 직전)
페이지 3 → page_num=3
페이지 4 → page_num=1     ✗ 이후 모든 페이지가 1로 고정
페이지 5..35 → page_num=1
```

(전체 페이지 수도 35 vs PDF 37로 2쪽 부족하나 본 계획서 범위 밖)

## 2. 근본 원인

### 2.1 typeset.rs::finalize_pages 의 NewNumber 적용 로직

`src/renderer/typeset.rs:1639-1657`:

```rust
let mut page_num: u32 = 1;
for page in pages.iter_mut() {
    let first_para = page.column_contents.first()...;

    if let Some(fp) = first_para {
        for &(nn_pi, nn_num) in new_page_numbers {
            if nn_pi <= fp {                  // ← 버그
                page_num = nn_num as u32;     // ← 매 페이지에서 재할당
            }
        }
    }
    ...
    page.page_number = page_num;
    page_num += 1;
}
```

문제: `nn_pi <= fp` 는 "NewNumber 컨트롤이 들어 있는 문단이 현재 페이지의 첫 문단보다 앞선다" 라는 의미.
한 번 NewNumber 가 트리거된 이후의 **모든** 페이지에서 이 조건이 참이 되어 매 페이지마다 page_num 이 1 로 리셋된다.

### 2.2 두 finalize_pages 구현이 공존

| 위치 | 사용 시점 | 상태 |
|------|----------|------|
| `src/renderer/typeset.rs:1624` | 기본 (TypesetEngine) | **버그 있음 (위 2.1)** |
| `src/renderer/pagination/engine.rs:1781` | `RHWP_USE_PAGINATOR=1` 플래그 시 | 다른 구현(이쪽도 보강 필요) |

engine.rs 는 `*para_idx > prev_page_last_para || i == 0` 조건으로 일종의 "새 페이지 진입 시에만" 적용하려 했으나,
`page_last_para = max(파라 인덱스)` 가 표/PartialParagraph 등의 영향으로 단조 증가하지 않을 수 있어 신뢰성 낮음.

### 2.3 표준 동작

쪽번호는 두 종류로 분리해 다루어야 한다.
- **물리 페이지 인덱스 (PgNo)**: 0,1,2,... 단조 증가
- **인쇄 쪽번호 (PrintPgNo)**: 사용자에게 보이는 번호

규칙:
1. NewNumber/StartPage 류 컨트롤은 **소유 문단이 처음 등장하는 페이지에서 한 번만** PrintPgNo 를 갱신
2. 이후 페이지는 PrintPgNo += 1
3. 홀수/짝수 시작 강제(HWPPAGE_ODD/EVEN) 가 있으면 빈 페이지를 한 장 더 진행하고 PrintPgNo 추가 보정
4. PageHide.hide_page_num=true 인 페이지는 **숫자는 계속 흐르되** 표시만 생략 (권장)
5. 구역(Section) 경계: NewNumber 가 없으면 이전 구역 마지막 PrintPgNo 에서 +1 로 이어짐

## 3. 현황 영향 범위

- **페이지 본체에 표시되는 쪽번호**: 잘못됨 (모두 1)
- **PageHide.hide_page_num 동작**: 일부 정상 — 표지에서 숨겨짐 확인
- **PageNumberPos 위치(format/position) 파싱**: 정상 (Errata #12 이미 수정됨)
- **머리말/꼬리말 안의 `\u{0015}` 치환**: 동일한 page_number 값 사용 → 함께 어긋남
- **표 셀/글상자 안의 쪽번호 마커 치환** (`table_layout.rs:1158`, `shape_layout.rs:1304`): 동일한 page_number 인자 → 함께 어긋남

## 4. 개선 방향 (2안 비교)

### 안 A — typeset.rs 의 조건만 최소 수정 (즉시 핫픽스)
- "consumed" set 을 두어 NewNumber 를 1회만 적용
- 트리거 조건을 "현재 페이지에 그 NewNumber 의 소유 문단이 처음 등장" 으로 정정
- 변경 범위: `finalize_pages` 내부 ~30줄
- 장점: 작고 즉시 적용 가능
- 단점: 두 구현(engine.rs / typeset.rs) 중복은 그대로

### 안 B — finalize_pages 공통화 + 안 A 의 정확한 로직 (권장)
- A 의 수정 로직을 별도 함수 `assign_page_numbers(...)` 로 분리하여 typeset.rs / engine.rs 가 공유
- `PageContent` 구조체에 `physical_index: u32` (이미 page_index 존재) + `print_number: u32` 의미를 명확히 분리 (필드명은 page_number 유지하되 "인쇄 번호"로 정의 고정)
- 장점: 단일 진실원, 회귀 위험 감소
- 단점: 약간의 리팩토링

→ **권장: 안 B**. 이번 타스크에서 A 의 정정 로직을 typeset.rs/engine.rs 양쪽이 공유하도록 통합.
HWPPAGE_ODD/EVEN 처리는 후속 이슈로 분리.

## 5. 수행 단계 (안 B)

| 단계 | 내용 | 산출물 |
|------|------|--------|
| 1 | 재현 테스트 작성 — 단일 NewNumber 가 있는 mini HWP 로 page_num 시퀀스 검증 (현재는 1,2,3,1,1,1...) | `tests/page_number_propagation.rs` |
| 2 | 공통 함수 `assign_page_numbers(pages, new_page_numbers, ...)` 신설<br/>- consumed 플래그로 1회 적용<br/>- 트리거: "page 안에 NewNumber 소유 문단이 처음 등장" (FullParagraph / PartialParagraph(start_line==0) / Table / PartialTable(첫 분할) / Shape — 첫 출현 판정) | `src/renderer/page_number.rs` |
| 3 | typeset.rs::finalize_pages 와 engine.rs::finalize_pages 가 공통 함수 호출하도록 치환 | `typeset.rs`, `engine.rs` |
| 4 | 회귀 테스트 — `samples/2022년 국립국어원 업무계획.hwp` 에서 페이지별 page_number 시퀀스가 1,2,3,...,N 으로 단조 증가하는지 확인 (PageHide=true 페이지도 내부 카운터는 흘러야 함) | dump-pages 출력 비교 |
| 5 | 시각 검증 — SVG 푸터의 `- N -` 이 PDF 와 일치하는지 페이지별 비교 (최소 표본: 5, 10, 20, 30) | side-by-side PNG |
| 6 | 단계별 보고서 / 최종 보고서 | `working/`, `report/` |

## 6. 리스크 / 회피

| 리스크 | 회피 |
|--------|------|
| 다른 샘플 (예: 다중 구역, 머리말 안 쪽번호) 회귀 | 단계 1 의 미니 테스트 + 기존 회귀 테스트 통과 필수 |
| RHWP_USE_PAGINATOR=1 경로 누락 | 단계 3 에서 두 경로 모두 변경, 환경변수 둘 다 검증 |
| PageHide.hide_page_num + NewNumber 동시 케이스 | 표지(0.19 PageHide) 직후 페이지(0.20 NewNumber)에서 카운터 흐름 검증 — 표지=숨김, 본문 첫 쪽=1 |
| 표 첫 페이지 PartialTable 이 NewNumber 트리거를 놓침 | 단계 2 에서 "첫 출현" 판정 시 PartialTable 의 첫 분할(continuation 인덱스 0)도 인정 |

## 7. 범위 외 (이번 타스크에서 다루지 않음)

- 전체 페이지 수 차이 (PDF 37 vs SVG 35) — 별도 이슈
- 홀수/짝수 페이지 시작 강제(HWPPAGE_ODD/EVEN) — 별도 이슈
- 머리말/꼬리말 carry 시 쪽번호 위치 상속 — 일부 이미 처리됨, 본 수정과 무관

## 8. 승인 요청

본 계획대로 GitHub 이슈 등록(M???) → `local/task{N}` 브랜치 생성 → 구현 계획서 → 단계 1 부터 진행해도 되는지 확인 부탁드립니다.

# Task #634 Stage 0 — 사전 분석 + 추가 검증

**브랜치**: `local/task634`
**이슈**: https://github.com/edwardkim/rhwp/issues/634
**수행계획서**: `mydocs/plans/task_m100_634.md`
**진행 시점**: 2026-05-06

---

## 1. 가설 H1 추가 검증

### 1.1 HWP+PDF 짝이 있는 샘플 전수 조사

```
2022년 국립국어원 업무계획.hwp | pages=40 pgnp=2 nn=1   ✓ 본질 검증 완료 (Stage 0)
21_언어_기출_편집가능본.hwp     | pages=15 pgnp=0 nn=0   - 쪽번호 컨트롤 없음 (대상 외)
aift.hwp                       | pages=77 pgnp=1 nn=1   ✓ 본질 검증 완료 (선행)
equation-lim.hwp               | pages=1 pgnp=0 nn=0    - 단일 페이지
exam_eng.hwp                   | pages=8  pgnp=0 nn=0   - 쪽번호 컨트롤 없음
exam_math_8.hwp                | pages=1 pgnp=0 nn=0    - 단일 페이지
exam_math.hwp                  | pages=20 pgnp=0 nn=0   - 쪽번호 컨트롤 없음
exam_science.hwp               | pages=4  pgnp=0 nn=1   - PageNumberPos 미등록 (※)
복학원서.hwp                    | pages=1 pgnp=0 nn=0    - 단일 페이지
pua-test.hwp                   | pages=1 pgnp=0 nn=0    - 단일 페이지
text-align-2.hwp               | pages=1 pgnp=0 nn=0    - 단일 페이지
```

(※) exam_science.hwp 는 NewNumber 만 있고 PageNumberPos 가 없음. 한컴 PDF 의 footer 영역
(y=55, 62) 에 1056~1269 개 text op 가 매 페이지 등장 → CTRL_FOOTER (시험지 footer
정보) 또는 마스터쪽 출력. PageNumberPos 가 없으므로 본 task 가설 검증과 무관.

### 1.2 PDF 가 없는 추가 후보 (가설 H1 적용 시 영향)

```
biz_plan.hwp           | pages=6  pgnp=1 nn=1   ⚠ pgnp + nn (PDF 없음, 가설 영향 받음)
endnote-01.hwp         | pages=5  pgnp=1 nn=0   ⚠ pgnp 만 (가설 적용 시 모두 미표시 회귀 가능)
footnote-01.hwp        | pages=6  pgnp=1 nn=0   ⚠ 동일
hwp3-sample.hwp        | pages=16 pgnp=1 nn=0   ⚠ 동일
hwp-multi-001.hwp      | pages=10 pgnp=1 nn=0   ⚠ 동일
issue_265.hwp          | pages=16 pgnp=1 nn=0   ⚠ 동일
KTX.hwp                | pages=27 pgnp=1 nn=0   ⚠ 동일
pic-in-head-02.hwp     | pages=7  pgnp=2 nn=0   ⚠ 동일
table-vpos-01.hwp      | pages=5  pgnp=1 nn=0   ⚠ 동일
```

**중요**: PageNumberPos 만 있고 NewNumber 가 없는 문서가 8건 존재. 가설 H1 그대로
구현 시 이 문서들은 **모든 페이지에 쪽번호 미표시** 로 회귀.

이 8건이 한컴 출력에서 "쪽번호 표시" 인지 "미표시" 인지 모름 (PDF 없음).

### 1.3 가설 정밀화 (H1 → H1')

원래 가설 H1:
> 한컴은 첫 NewNumber Page 발화 전 페이지에는 쪽번호를 미표시한다.

문제점: NewNumber 가 아예 없는 문서에서는 쪽번호가 영원히 표시되지 않게 됨.
하지만 KTX.hwp 같은 일반 문서는 1쪽부터 쪽번호 표시가 정상일 수 있음.

**가설 정밀화 H1'**:
> 한컴은 다음 둘 중 하나를 만족하는 페이지에만 쪽번호를 표시한다:
> 1. **NewNumber Page 컨트롤이 발화한 적 있다 (해당 페이지 또는 이전 페이지)**, 또는
> 2. **(보류 — Stage 0 시점 미확정)**: NewNumber 가 문서 전체에 단 한 번도 없는 경우는
>    PageNumberPos 등록 즉시 표시 시작? (KTX 등 검증 필요)

### 1.4 Stage 0 결론

- 가설 H1' 의 case 1 (NewNumber 발화 후) 은 2건 (aift, 국립국어원) 으로 명확히 확인
- case 2 (NewNumber 미존재 문서) 는 PDF 가 없어 미검증 → **작업지시자 입력 필요**

**작업지시자 입력 사항 A**:
> KTX.hwp / endnote-01.hwp / pic-in-head-02.hwp 등 NewNumber 없는 문서를 한컴오피스에서
> PDF 출력했을 때 쪽번호가 표시되는가 미표시되는가?
> (가능하다면 1건만 PDF 출력하여 footer 텍스트 op 확인)

**Fallback 정책 (작업지시자 입력 전)**:
- Case A: 한컴이 표시함 → 가설 정밀화 H1' 구현 (NewNumber 없을 때는 1쪽부터 표시)
- Case B: 한컴이 미표시 → 가설 H1 그대로 구현 (NewNumber 없으면 영원히 미표시)

A안 (관대) 이 보수적이고 회귀 위험이 작으므로 **A안 default** 로 진행 권장.

---

## 2. PageNumberAssigner 호출 경로 정합성

두 경로 모두 동일 로직:

### 2.1 `src/renderer/typeset.rs:2184` (TypesetEngine)
```rust
let mut assigner = crate::renderer::page_number::PageNumberAssigner::new(new_page_numbers, 1);
for page in pages.iter_mut() {
    let page_num = assigner.assign(page);
    page.page_number = page_num;
    ...
    page.page_number_pos = page_number_pos.clone();
}
```

### 2.2 `src/renderer/pagination/engine.rs:1901` (Paginator)
```rust
let mut assigner = crate::renderer::page_number::PageNumberAssigner::new(new_page_numbers, 1);
for (i, page) in pages.iter_mut().enumerate() {
    ...
    let page_num_u32 = assigner.assign(page);
    page.page_number = page_num_u32;
    ...
    page.page_number_pos = page_number_pos.clone();
}
```

**정합성 확인**: 두 경로 모두 `assigner.assign(page)` 후 `page.page_number_pos` 를
별도로 설정. 이 부분에 `page.show_page_number = ...` 추가 필요. 두 곳 동일 정정.

---

## 3. PageContent 구조

`src/renderer/pagination.rs:33-58` 의 PageContent 에 `show_page_number: bool` 추가
필요. Default `true` (현재 동작 보존).

```rust
pub struct PageContent {
    ...
    pub page_number_pos: Option<crate::model::control::PageNumberPos>,
    pub page_hide: Option<crate::model::control::PageHide>,
    pub show_page_number: bool,  // ← 신규
    ...
}
```

수정 영향: PageContent 를 직접 생성하는 코드 모두 갱신 필요. 검색 결과:
- `src/renderer/pagination/state.rs:233` (default-init)
- `src/renderer/pagination.rs:348` (clone 분기)
- `src/renderer/typeset.rs:304` (default-init)
- `src/renderer/page_number.rs:120` (테스트 mk_page)
- `src/renderer/layout/tests.rs:43, 89, 170, 265, 563, 647` (테스트 mk_page)

`Default` impl 추가 또는 명시적 `show_page_number: true` 추가.

---

## 4. PageNumberPos position=0 vs 새 가드 상호작용

`src/renderer/layout.rs:1099-1102`:
```rust
if let Some(pnp) = &page_content.page_number_pos {
    if pnp.position == 0 {
        return;
    }
    ...
}
```

새 가드는 이보다 **앞** 에 추가:
```rust
fn build_page_number(&self, ...) {
    if let Some(ref ph) = page_content.page_hide {
        if ph.hide_page_num { return; }
    }
    if !page_content.show_page_number {  // ← 신규
        return;
    }
    if let Some(pnp) = &page_content.page_number_pos {
        if pnp.position == 0 { return; }
        ...
    }
}
```

세 가드 모두 **OR** 관계 (어느 하나라도 미표시 조건 충족 시 미표시). 안전.

---

## 5. 구역간 carry 와 NewNumber 상호작용

`src/document_core/queries/rendering.rs:983-1293` 가 구역 간 PageNumberPos / 쪽번호 /
머리말꼬리말 carry 를 처리한다. 본 task 추가 carry 필요:

### 5.1 carry_numbering_started: bool

이전 구역 마지막 페이지의 `show_page_number` 상태를 carry. 단, NewNumber 가 한 번
발화하면 영구 true 이므로 단순히 boolean 으로 충분.

### 5.2 구역 시작 시 처리

```rust
// 새 구역의 PageNumberAssigner 초기화 시 numbering_started carry 반영
let initial_started = carry_numbering_started;  // 이전 구역에서 발화했으면 true
let mut assigner = PageNumberAssigner::new_with_started(
    new_page_numbers, 1, initial_started
);
```

또는 더 간단히:
```rust
// 두 finalize_pages 호출 시 추가 인자로 전달
```

세부 구조는 구현 계획서에서 확정.

---

## 6. HWPX / HWP3 호환

- HWPX (`src/parser/hwpx/section.rs:2012, 2042`): `Control::NewNumber` 동일 IR 생성
- HWP3 (`src/parser/hwp3/mod.rs:325`): 동일

본 fix 가 IR-level (rendering) 변경이므로 **3개 포맷 모두 자동 적용**. 별도 수정 불필요.

---

## 7. Fix 위치 확정 (Stage 2 대상)

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/page_number.rs` | `PageNumberAssigner` 에 `numbering_started: bool` 필드 + `show_for_last_page() -> bool` 액세서. `assign()` 시그니처는 보존 (테스트 호환). 별도 `new_with_started(.., started)` 생성자. |
| `src/renderer/pagination.rs` | `PageContent.show_page_number: bool` 신규 필드 (default `true`) |
| `src/renderer/pagination/state.rs:233` | PageContent 초기화에 `show_page_number: true` 추가 |
| `src/renderer/pagination/engine.rs:1901-1979` | `assign()` 후 `assigner.show_for_last_page()` → `page.show_page_number` 설정 |
| `src/renderer/typeset.rs:2184-2222` | 동일 |
| `src/document_core/queries/rendering.rs:983~1293` | `carry_numbering_started: bool` 추가, 구역간 전달, finalize 호출 시 적용 |
| `src/renderer/layout.rs:1086~1098` `build_page_number` | `if !page_content.show_page_number { return; }` 가드 |
| `src/renderer/layout/tests.rs` | mk_page 헬퍼에 `show_page_number: true` 추가 (6건) |
| `src/renderer/page_number.rs` 기존 5건 테스트 | mk_page 갱신 |

---

## 8. 위험 평가 (수행계획서 §6 보강)

### 8.1 NewNumber 미존재 문서의 처리 (작업지시자 입력 필요)

위 §1.4 참조. KTX/endnote/footnote 등 8건 영향. A안 (관대) 이 안전.

### 8.2 1쪽에 NewNumber 가 있는 문서

```
biz_plan.hwp pi=479 (paragraph N)에 NewNumber → 페이지 3 부터 NewNumber 발화
```

NewNumber 가 1쪽이 아니라도 동일 가설 적용. 이전 페이지는 미표시. 회귀 가능성 있음.

### 8.3 머리말/꼬리말과의 상호작용

`build_page_number` 는 `footer_node` 에 쪽번호 텍스트 노드를 추가. 머리말/꼬리말 자체의
표시는 별도 로직 (`hide_header/hide_footer`). 가드 추가는 footer_node 에 add 하지 않을
뿐, footer 자체는 그대로. ✓ 영향 없음.

### 8.4 rhwp-studio 편집기

편집기에서도 `show_page_number=false` 페이지에는 쪽번호 미표시. 이는 한컴 동작과 일치
하므로 의도된 동작. 사용자가 표지에서도 쪽번호를 보고 싶다면 NewNumber 컨트롤을 1쪽에
추가하거나 별도 옵션 필요 (본 task 범위 외).

---

## 9. Stage 0 결론

1. **가설 H1' (정밀화)** 는 NewNumber 발화 후 페이지 표시는 확실, NewNumber 미존재
   문서의 처리는 작업지시자 입력 후 결정.
2. **수정 범위** 4개 모듈 + 여러 테스트 mk_page 확정 (§7).
3. **회귀 위험**: 8건 (PDF 없는 pgnp-only 문서) 의 한컴 정답 미확인 → A안 (관대) 적용
   권장.

---

## 10. KTX.pdf 보강 검증 (2026-05-06 작업지시자 입력)

작업지시자가 `samples/basic/KTX.pdf` 추가. 분석 결과:

- **PDF 형식**: 1페이지 landscape (841×595pt), 4.5MB 컨텐츠 (벡터 그래픽 위주)
- **HWP 비교**: KTX.hwp 는 27페이지 A4 portrait — 1페이지 landscape 와 매우 다름
- **추정**: 이 PDF 는 KTX.hwp 1쪽 (표지) 만 landscape 모드로 출력한 것 (또는 다른 export 옵션)

**핵심 발견**: KTX.hwp paragraph 0.11 에 **PageHide page_num=true** 컨트롤이 있음:
```
[0] 감추기: header=false, footer=false, master=false, border=false, fill=false, page_num=true
```

이 PageHide 는 paragraph 11 이 처음 등장하는 페이지 (→ 페이지 1) 의 쪽번호를 감춤.
rhwp 의 기존 PageHide 지원과 일치 (`build_page_number` line 1102-1106).

→ **rhwp 페이지 1 SVG 에 쪽번호 미표시는 PageHide 동작 결과** (가설 H1 과 무관).

**KTX.pdf 가 가설 검증에 부적합한 이유**:
1. PageHide page_num=true 가 명시적으로 1쪽 쪽번호를 감춤
2. 한컴 출력 1쪽도 미표시 → 기존 PageHide 동작이 이미 일치
3. NewNumber 미존재 + PageHide 미존재 케이스가 아님

### 10.1 잔여 케이스 분석

PDF 가 없는 8건의 pgnp-only 샘플 중:
- **KTX** : page_num PageHide 있음 → 기존 동작으로 이미 한컴과 일치
- endnote-01 / footnote-01 / hwp-multi-001 / table-vpos-01 : PageHide 있으나 page_num 감추기 아님
- **hwp3-sample / issue_265 / pic-in-head-02** : PageHide 자체가 없음 ★ 가설 H1 적용 시 회귀

### 10.2 결정: **A안 (관대) default** 채택

근거:
1. **검증 케이스 불충분**: hwp3-sample / issue_265 / pic-in-head-02 의 한컴 PDF 가 없어
   가설 H1 (엄격) 적용 시 위 3건의 회귀 여부 미확인
2. **회귀 위험 최소화**: A안 은 NewNumber 가 1회라도 있는 문서에만 게이팅 적용 →
   기존 단순 문서의 동작 보존
3. **검증된 2건 (aift, 국립국어원) 은 둘 다 NewNumber 존재** → A안 으로도 한컴 일치

A안 정의 (가설 H1''):
> 문서에 `NewNumber Page` 컨트롤이 **하나라도 존재하는 경우에만**, 첫 NewNumber 발화 전
> 페이지의 쪽번호를 감춘다. NewNumber 가 전혀 없는 문서는 PageNumberPos 등록 시점부터
> 모든 페이지에 표시 (현재 동작 보존).

### 10.3 구현 영향

- `PageNumberAssigner` 가 `new_page_numbers.is_empty()` 인 경우 `numbering_started = true`
  로 초기화 (즉시 표시 가능 상태)
- 그 외에는 기존 가설 H1 대로 첫 NewNumber 발화 시 `numbering_started = true` 전이

---

## 11. Stage 0 최종 결론

**가설 H1'' (A안) 채택**. NewNumber 존재 여부에 따라 게이팅 분기:
- NewNumber 1개 이상: 첫 발화 페이지부터 표시 (한컴 호환)
- NewNumber 0개: 즉시 표시 (현재 동작 보존, 회귀 0)

수정 위치 §7 의 9개 파일 그대로 (단, PageNumberAssigner 초기화 로직만 약간 변경).

---

## 12. 다음 단계

**Stage 1 진행 전**:
- 구현 계획서 (`task_m100_634_impl.md`) 작성 → 승인 요청

**Stage 1+ 진행 시**:
- TDD RED 통합 테스트 (3건+) → Stage 1
- Fix 적용 → Stage 2
- 광범위 회귀 + 최종 보고서 → Stage 3

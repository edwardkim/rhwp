# Task M100-1113 — Stage 1 진단 보고서

- 이슈: [#1113](https://github.com/edwardkim/rhwp/issues/1113)
- 일시: 2026-05-29
- 단계: Stage 1 (진단) 완료

## 1. 진단 도구

- `examples/dump_odd_header_1113.rs` — Section0 의 CTRL_HEADER `head` subtree (ODD/EVEN) record byte 정밀 dump + 정답지 vs 저장본 record-by-record 비교
- 자료:
  - 정답지: `samples/exam_social.hwp` (536064 bytes)
  - 저장본: `output/poc/issue_1113/exam_social-current.hwp` (427008 bytes, 현재 코드)
  - 양쪽 Section0 record 수 **동일 (809)**

## 2. apply_type 매핑 확정 (이전 세션 오류 정정)

`src/serializer/control.rs:634` + `parser/control.rs:477`:
- HWP5 apply_type **1 = Even (짝수쪽)**
- HWP5 apply_type **2 = Odd (홀수쪽)** ← 본 task 대상

| head | byte[4] | apply_type | 의미 | record 범위 |
|------|---------|-----------|------|-------------|
| #138 | 02 | 2 | **ODD (홀수, 문제)** | 138~159 |
| #160 | 01 | 1 | EVEN (짝수, 정상) | 160~182 |

**이전 세션 round1 실패 원인**: apply_type=1 을 ODD 로 착각 → 짝수쪽 LIST_HEADER tail 정정 → 시각 무효. 본 진단으로 **apply_type=2 = 홀수쪽** 확정.

## 3. 홀수쪽(apply_type=2) 글상자 정답지 vs 저장본 diff

```
CTRL_HEADER#138 (head, apply=2/ODD)
└ ... 표/셀 ...
   └ SHAPE_COMPONENT#152 (글상자 도형)
      └ LIST_HEADER#153
         └ PARA_HEADER#154 → PARA_TEXT#155 (페이지번호 문단)  ← ★ trigger
```

### 3.1 Trigger A (주요) — 페이지번호 앞 placeholder 공백 추가

| record | oracle | gen | 차이 |
|--------|--------|-----|------|
| PARA_HEADER #154 | `09 00...` (char_count=9) | `0a 00...` (char_count=10) | +1 |
| PARA_TEXT #155 | size=18: `[0x12 autonum][aton][0x12 end][0x0d]` | size=20: **`[0x20 공백]`** `[0x12][aton][0x12][0x0d]` | **U+0020 prefix** |

정답지: 페이지번호(autoNum) **단독**.
저장본: 페이지번호 앞에 **공백 U+0020 한 글자** 추가.

→ 한컴 에디터가 좁은 글상자(width=4252) 안에서 "공백 + 페이지번호" 를 줄나눔하고 글상자 높이를 키운다.

### 3.2 Trigger B (관련) — 본문 fixed-width space literal

| record | oracle | gen |
|--------|--------|-----|
| PARA_TEXT #148 (글상자 표 안 "(사회·문화") | `...0x001f...` (HWP5 control char) | `...0x2007...` (U+2007 literal) |

- 정답지: fixed-width space 를 HWP5 control char 0x001f 로 저장
- 저장본: literal U+2007 유지 (`materialize_fixed_width_space_control` 의 "header keeps literal U+2007" 정책)

→ 본 영역은 줄나눔 직접 원인이 아닐 수 있음 (텍스트 폭 영역). Trigger A 우선.

### 3.3 부가 차이 (trigger 후보 아님)

- TABLE #145 byte[3] `04→00`, LIST_HEADER #146 byte[7] `05→00`, PARA_HEADER #147 byte[7] `80→00`, SHAPE_COMPONENT #152 일부 byte — 짝수쪽(apply_type=1)에도 동일 패턴 발생 (짝수쪽 정상이므로 trigger 아님)

## 4. 공백 U+0020 출처 (코드 경로 확정)

### 4.0 본질 — HWPX 원본에는 공백이 없다 (rhwp 가 합성)

HWPX ODD 글상자 raw:
```xml
<hp:run charPrIDRef="63">
  <hp:ctrl><hp:autoNum num="1" numType="PAGE">...</hp:autoNum></hp:ctrl>
  <hp:t/>          ← 빈 텍스트 (내용 없음)
</hp:run>
```

→ **HWPX 원본은 페이지번호 단독, U+0020 공백 없음.** U+0020 은 rhwp 가 HWPX→HWP 변환 중 합성한 것.
정답지(한컴)도 PARA_TEXT 에 visible 공백 없이 autonum 마커(0x0012)로 직접 처리.

즉 "한컴이 공백을 제거" 가 아니라 **"rhwp 가 없던 공백을 합성, 어댑터가 머리말에서 제거하지 못함"** 이 정확한 본질.

### 4.1 HWPX 파서 — placeholder space 합성

`src/parser/hwpx/section.rs:569-573`:
```rust
"\u{0012}" => {
    // [Task #1050] AUTO_NUMBER (0x12) — HWP PARA_TEXT 정합:
    char_offsets.push(utf16_pos);
    visual_text.push(' ');   // ← placeholder space U+0020 합성
    utf16_pos += 8;
}
```
HWPX autoNum 컨트롤 (`\u{0012}` 마커) → PARA_TEXT 조립 시 placeholder space (U+0020) 를 **합성** (HWP5 PARA_TEXT 자리 점유용, Task #1050). HWPX 원본엔 없던 문자.

### 4.2 어댑터 — 공백 제거 (MasterPage 한정, 머리말 누락)

`src/document_core/converters/hwpx_to_hwp.rs:713` `materialize_master_page_autonum_placeholder`:
```rust
if context != ParagraphContext::MasterPage {
    return;   // ← 머리말(HeaderFooter) 은 제외됨!
}
// AutoNumber-only 문단의 placeholder space 제거
para.text.clear();
para.char_count = 9;
```
주석 명시:
> Hancom stores the master-page page number paragraph as **AutoNumber-only: no leading U+0020**.

→ **이 로직이 MasterPage context 에만 적용**되어, 머리말 글상자(HeaderFooter context) 의 AutoNumber-only 문단은 placeholder 공백이 **제거되지 않고 남음** = Trigger A.

### 4.3 context 전파 확인

- `Control::Header` → `adapt_paragraphs_with_context(..., HeaderFooter)` (line 527)
- Header paragraph 의 `Control::Shape` → `adapt_shape_with_context(shape, report, context)` (line 540, context 전파)
- Shape textbox → `adapt_paragraphs_with_context(&mut text_box.paragraphs, report, context)` (line 843, HeaderFooter 유지)

→ 머리말 글상자 paragraph 는 `HeaderFooter` context. MasterPage 한정 제거 로직에서 누락.

## 5. 정정 가설 (Stage 2)

`materialize_master_page_autonum_placeholder` 의 placeholder 공백 제거 로직을 **HeaderFooter context** 의 AutoNumber-only 문단에도 적용.

- 짝수쪽(정상)은 `para.text != " "` (fwSpace+텍스트+autoNum) 조건에서 자동 제외 → 회귀 없음
- 홀수쪽만 페이지번호 단독(text==" ") 이므로 공백 제거 적용

## 6. 위험 평가

| 위험 | 평가 |
|------|------|
| Trigger A 정정만으로 부족 (Trigger B 도 필요?) | Trigger A = 직접 줄나눔 원인 (글자 1개 추가). 우선 단독 정정 → 시각 판정. 부족 시 B 추가 |
| MasterPage 로직 재사용이 머리말 다른 케이스 영향 | `text==" "` + autonum 단독 조건 엄격 → 짝수쪽/일반 머리말 제외 |
| char_count=9 하드코딩 정합 | 정답지 char_count=9 확인 (autonum 8 + 끝마커 1) |

## 7. 다음 단계 (Stage 2)

작업지시자 승인 후:
1. `materialize_master_page_autonum_placeholder` → HeaderFooter context 확장 (또는 신규 함수)
2. `output/poc/issue_1113/exam_social-round1.hwp` 산출 + byte 정합 확인 (PARA_TEXT #155 size 20→18)
3. 작업지시자 한컴 한글 2020/2022 시각 판정

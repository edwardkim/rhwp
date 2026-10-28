# Task m100-3006: HWP3 doc_info.hide_empty_line(빈줄감춤) SectionDef 미배선 수정

## 이슈

- edwardkim/rhwp#3006

## 배경

"accessor 는 정의되어 있지만 실제 변환 함수에서 호출되지 않아 값이 항상 기본값으로 고정되는" 계열의
버그(#2976/#2983 border_connection 패턴)를 HWP3 파서 전반에서 다시 찾는 작업이었다. 이번에는
accessor 형태는 아니지만 동일한 실패 모드였다 — `Hwp3DocInfo`가 파싱한 원시 필드(`hide_empty_line`)가
IR(`SectionDef`) 조립부에서 배선 목록에서 누락되어 있었다.

## 조사 과정 (버려진 후보들)

1. `Hwp3CharShape::is_superscript()`/`is_subscript()` — accessor 는 있으나 `convert_char_shape()`가
   호출하지 않는 것으로 로컬(push-2978) 브랜치에서는 보였다. 그러나 `origin/devel` 최신본을 다시
   확인하니 **이미 수정되어 있었다**(로컬 브랜치가 devel보다 오래되어 생긴 오탐). 이슈 #2991을 생성
   직후 devel 확인으로 발견해 즉시 close.
2. `Hwp3ParaShape::has_border()` — accessor 미호출로 문단 테두리(음영 없을 때)가 유실되는 진짜 버그를
   찾았으나, `gh issue/pr list` 중복 확인 결과 동일 저장소에서 **이미 이슈 #2995 / PR #2997**로 나(또는
   병렬 세션)이 제출한 상태였다. 중복 방지를 위해 폐기.
3. `doc_info.start_page_number`/`footnote_start_number` → `doc.doc_properties` 미배선 — 확인해보니
   과거 PR #2486(CLOSED, 미병합)로 동일 내용이 이미 시도된 이력이 있어 제외.
4. `PageHide` 컨트롤의 `hide_master_page`/`hide_fill` 비트 — 스펙(한글문서파일구조3.0.md:1062)을
   재확인한 결과 HWP3 "홀수쪽시작/감추기" 컨트롤은 애초에 bit 0~3(머리말/꼬리말/쪽번호/테두리)만
   정의하고 bit 4-15는 예약(reserved)이다. 즉 HWP5(`master_page`/`fill` 비트 존재)와 레이아웃이 다를 뿐,
   버그가 아니었다.
5. `doc_info.footnote_line_width` 등 각주 관련 여러 필드 — `hwp3_default_endnote_shape()`가 하드코딩된
   기본값을 쓰고 있어 실제로 미배선 상태이나, `fixup_hwp3_notes()`가 `doc_info`에 대한 접근권이 없어
   plumbing 범위가 "1-15줄 tiny fix"를 넘어서므로 이번 타스크에서는 보류.

## 최종 선택: doc_info.hide_empty_line → SectionDef.hide_empty_line

- 한글문서파일구조 3.0 스펙(`mydocs/tech/한글문서파일구조3.0.md:248`): doc_info offset 122 =
  "빈줄감춤"(0 이외=on).
- 공통 IR `SectionDef.hide_empty_line`은 HWP5(`src/parser/body_text.rs:580`)·HWPX
  (`src/parser/hwpx/section.rs:1290`)에서는 이미 매핑되어 있었으나 HWP3 경로만 누락.
- `gh issue list --search`, `gh pr list --state open --author kevin9327 --search` 로 중복 여부 확인 —
  히트 없음.

## 수정 내용 (Red → Green)

`src/parser/hwp3/mod.rs`:

- `hwp3_hide_empty_line(doc_info: &Hwp3DocInfo) -> bool` 헬퍼 추가 (기존 `hwp3_page_border_fill` 과
  동일한 스타일).
- `Hwp3DocInfo` → `SectionDef` 조립부에 `section_def.hide_empty_line = hwp3_hide_empty_line(&doc_info);`
  한 줄 추가.
- 회귀 테스트 `issue_hwp3_hide_empty_line_wires_doc_info_flag` 추가: `hide_empty_line: 1` →
  `true`, `hide_empty_line: 0` → `false` 단언. 수정 전에는 테스트 대상 함수 자체가 없어 컴파일
  실패(red) → 헬퍼 추가 + 배선 후 green.

실제 fix 본문 diff는 헬퍼 함수(4줄) + 호출부(3줄) 로 7줄 수준이며, 테스트/주석을 포함한 전체는 25줄.

## 검증

```
cargo check --lib   # OK
cargo test --lib issue_hwp3_hide_empty_line_wires_doc_info_flag   # 1 passed
rustfmt --edition 2021 src/parser/hwp3/mod.rs
```

## 남은 과제 (후속 이슈 후보, 이번 범위 밖)

- `doc_info.footnote_line_width`/`footnote_bracket`/`footnote_between_margin`/`footnote_text_margin`
  /`footnote_line_margin` 등 각주 관련 doc_info 필드가 HWP3 파서에서 전혀 소비되지 않음
  (`hwp3_default_endnote_shape()`가 하드코딩된 기본값 사용). `fixup_hwp3_notes()`가 `Hwp3DocInfo`에
  접근할 수 있도록 plumbing 이 필요해 별도 타스크로 분리 권장.

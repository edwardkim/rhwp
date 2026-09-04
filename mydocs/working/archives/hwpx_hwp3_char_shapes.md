---
kind: working
status: active
issue: 5251
---

# HWP3→HWPX 재파싱 char_shapes 경계를 원본 IR 에 맞춘다 (#5251)

작업 브랜치: `fix/5251-hwpx-char-shapes-offset`
대상: `src/parser/hwpx/section.rs`
시험: `tests/cases/issue_5251_hwpx_char_shapes.rs`

## 한 줄

rhwp 가 HWP3 를 HWPX 로 낸 뒤 다시 읽으면, 개체 자리 U+FFFC 는 8유닛으로
세고 pageNum/footer 는 PARA_TEXT 슬롯으로 세지 않아야 원본
`(24,33,38,54)` 경계가 유지된다. 두 보정을 한쪽에만 적용하면 char_count 가
55가 아니다.

## 기록

`#5251`, `samples/issue_265.hwp`. 게이트는 `hwp3-origin` 마커가 있는
자기 산출 HWPX 뿐이다.

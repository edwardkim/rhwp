# 단계별 완료 보고서 — Task #164 Stage 2.1

**이슈**: [#164](https://github.com/edwardkim/rhwp/issues/164)
**단계**: Stage 2.1 — section.xml 텍스트 주입
**브랜치**: `feature/task164-hwpx-serializer`
**완료일**: 2026-04-17

---

## 목표

`Section` IR의 첫 문단 텍스트를 `section0.xml` 템플릿의 `<hp:t/>` 자리에 주입하여, 한글2020에서 오픈 시 해당 텍스트가 표시되도록 한다.

## 구현 내용

### 변경 파일

- `src/serializer/hwpx/utils.rs` — `xml_escape()` 추가 (`&`, `<`, `>`, `"`, `'`)
- `src/serializer/hwpx/section.rs` — 템플릿 기반 텍스트 주입 로직
- `src/serializer/hwpx/mod.rs` — 테스트 2종 추가
- `examples/hwpx_dump_text.rs` — 텍스트 문단 HWPX 생성 예제

### 주입 전략

1. `section.paragraphs.first()`에서 텍스트 추출
2. 제어문자(U+0001..U+001F, 제외: `\n`/`\t`)가 없으면 XML 이스케이프 후 `<hp:t>{escaped}</hp:t>`로 치환
3. 제어문자 있으면 빈 템플릿 유지 (Stage 2.2에서 처리)

## 검증

### 단위 테스트 — 7/7 통과
- `serialize_text_paragraph_roundtrip` — `<hp:t>안녕 Hello 123</hp:t>` 삽입 + 파서 라운드트립
- `xml_escape_applied_to_section_text` — `&`, `<` 이스케이프 확인
- 기존 Stage 1 테스트 5종 모두 유지

### 한글2020 시각 검증
- `output/stage2_text.hwpx` (6970 bytes)
- 바탕화면 복사본 한글2020에서 정상 오픈 + "안녕 Hello 123" 표시 확인 ✅

## 한계

- 단일 문단만 지원 (다문단은 Stage 2.4)
- 제어문자(탭/줄바꿈 등) 미지원 (Stage 2.2)
- `paraPrIDRef`/`styleIDRef` 고정 "0" (Stage 2.4 header.xml 확장 후)

## 다음 단계

**Stage 2.2** — 제어문자(`\t`, `\n`) 처리 및 `<hp:t>` 분할 출력.

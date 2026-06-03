# Task M100 #1237 Stage 1 — PARA_HEADER / LINE_SEG 저장 계약 정리

## 목적

방금 #1183 셀 내부 그림 저장 계약에서 확인된 것처럼, HWP5 저장 경로는 문단 공통 필드의 의미를 정확히 구분해야 한다. 이번 단계에서는 문단 헤더와 `LineSeg` 주변에서 문서/코드 설명이 어긋난 부분을 정리했다.

## 확인한 불일치

| 항목 | 기존 상태 | 정리 내용 |
|---|---|---|
| `LineSeg.tag = 0x00060000` | 일부 코드/문서에서 "HWP 기본 플래그"처럼 표현 | 공식 스펙 bit 17(first segment) + bit 18(last segment)로 정의되는 단일 세그먼트 줄 계약으로 명명 |
| HWPX `<hp:lineseg>` 주석 | 6개 필드를 출력한다고 설명 | 실제 9개 필드(`textpos`, `vertpos`, `vertsize`, `textheight`, `baseline`, `spacing`, `horzpos`, `horzsize`, `flags`) 직렬화로 수정 |
| `raw_header_extra` | count 필드까지 그대로 보존하는 것처럼 오해 가능 | HWP5 저장기는 count 3개를 재생성하고 `raw_header_extra[6..]`만 suffix로 보존한다고 명시 |

## 코드 변경

- `src/model/paragraph.rs`
  - `LineSeg` tag 공식 bit 상수 추가
  - `TAG_SINGLE_SEGMENT_LINE` 추가
  - tag helper 추가: 빈 세그먼트, 첫/마지막 세그먼트, 들여쓰기, 문단 머리 모양
  - `char_count_msb`, `raw_header_extra` 저장 계약 주석 보강
- `src/serializer/body_text.rs`
  - `raw_header_extra[0..6]` count 필드 건너뜀 주석 정정
- `src/serializer/hwpx/section.rs`
  - HWPX lineseg 9필드 직렬화 주석 정정
  - fallback `LINE_FLAGS`를 `LineSeg::TAG_SINGLE_SEGMENT_LINE`으로 연결
- `src/parser/hwpx/section.rs`, `src/parser/hwp3/mod.rs`, `src/renderer/composer/line_breaking.rs`
  - `0x00060000` fallback을 `TAG_SINGLE_SEGMENT_LINE`으로 교체
- 표/개체/클립보드 생성 경로
  - 단일 세그먼트 줄 tag를 상수로 교체

## 문서 변경

- `mydocs/tech/hwp_save_guide.md`
  - PARA_HEADER 저장 계약 추가
  - PARA_LINE_SEG 9필드 저장 계약 추가
  - `LineSeg.tag` 공식 bit 표 추가
  - `0x00060000`의 의미를 단일 세그먼트 줄로 명확화
- `mydocs/tech/document_ir_lineseg_standard.md`
  - `LineSeg.tag` bit 의미와 `TAG_SINGLE_SEGMENT_LINE` 설명 추가

## 정오표 판단

한컴 공식 스펙 문서의 PARA_HEADER / PARA_LINE_SEG 표와 이번 확인 결과는 일치한다. 따라서 `mydocs/tech/hwp_spec_errata.md`에는 새 항목을 추가하지 않았다.

## 검증

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo test -p rhwp task177_lineseg --lib` | 통과 |
| `cargo check -p rhwp` | 통과 |

## 남은 위험

- HWP3 `LineSeg.vertical_pos` 누적 계산은 이번 범위에서 제외했다.
- HWPX에서 `lineSegArray`가 없는 문단을 한컴처럼 완전 재조판하는 문제는 별도 이슈에서 다룬다.
- `src/serializer/hwpx/templates/empty_section0.xml`에는 템플릿 원문으로 `flags="393216"`이 남아 있으나, 현재 serializer는 첫 문단 lineseg를 IR/fallback으로 교체한다.

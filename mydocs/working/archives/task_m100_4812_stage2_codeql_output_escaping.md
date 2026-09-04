---
kind: correction
status: active
issue: 4812
canonical: mydocs/working/task_m100_4812_stage2_codeql_output_escaping.md
last_verified: 2026-08-15
---

# Stage 2: CodeQL 출력 이스케이프 보정

## 계기

PR #4816의 GitHub Advanced Security CodeQL이 조사 스크립트에 두 개의 high 경고를 보고했다.

- Markdown 표 셀은 `|`만 이스케이프하므로, 입력 글꼴명의 기존 역슬래시와 결합하면 표 구분자가
  다시 활성화될 수 있다.
- OnlineWebFonts 검색 결과의 HTML 엔터티를 순차 치환하면 `&amp;lt;`처럼 한 번 인코딩된 입력이
  `<`까지 이중 복원될 수 있다.

## 보정 계약

- Markdown 셀은 역슬래시를 먼저 이스케이프하고, 이어서 표 구분자인 `|`를 이스케이프한다.
- HTML 엔터티는 단일 정규식 매핑으로 한 번만 복원한다. 치환 결과는 같은 호출에서 다시 해석하지 않는다.
- 이 보정은 입력 문서와 외부 검색 응답의 글꼴명을 사람이 읽는 Markdown 증적에 기록하는 경계만
  좁힌다. 공급자 조회 순서, TSV 원문 보존, 글꼴 판정은 변경하지 않는다.

## 검증

- `node --check scripts/survey_korea_downloads_font_jsdelivr.mjs`
- 수정 head의 GitHub CodeQL, CI, Render Diff 완료 뒤 PR #4816의 mergeability를 다시 확인한다.

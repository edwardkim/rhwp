# PR #9141 리뷰 — 누름틀 끝 뒤 컨트롤의 저장 줄 소속

## 최종 판정

**머지 보류 — 원본 세 건의 출력·편집·반복 저장과 기존 필드 경계를 검증 중이다.**

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9141), 작성자 `johndoekim`.
- 원 head `aea7b6a1fadac99257e7b302bf15b5afb3c38225`, base `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`, milestone `v1.0.0`을 API로 확인했다.
- #9058의 FIELD_END 보조 8유닛이 후행 컨트롤 앞에 속하는 경우를 공용 슬롯 계산으로 정리한다.
  관련 번호의 모든 양식을 이 세 원본만으로 완전 해결했다고 추정하지 않는다.

## 검증과 조판 원칙

기존 `mydocs/pr/assets/johndoekim-20261116/issue9058/`의 cursor/text 원본과
`samples/issue9082/textbox_filled_field_tac_tables.hwp`를 검토한다. 같은 입력에 대응하는
기존 한컴 2022 수동 Print PDF 세 건은 생성 출처와 SHA를 확인해 재사용한다.
저장 제품·직접 Print 제품·MCP engine을 혼동하지 않는다. 웹한글기안기 PDF는 공식 기준이 아니다.

필드 안 글 입력과 빈 상태, FIELD_END 앞뒤·후행 컨트롤·글자 없는 문단의 소속을 검증한다.
HWP/HWPX 저장→재열기→재저장 결과와 인접 #9074·#9021·#8199 계약을 확인한다.
소스 UTF-16 위치와 저장 스트림 슬롯을 혼용하지 않고 같은 보조 슬롯 계산을 실제 composer와
fallback 배치가 소비하는지 대조한다. Native/fresh WASM 전쪽 TSV와 직접 판독은 별도로 완료한다.

## Merge 후 contributor PR comment 계획

최종 CI·통합 merge SHA·실제 편집/저장 결과와 세 원본의 같은 페이지 raw·review·overlay를
한국어 존댓말 코멘트에 표시한다. #9058의 원 이슈와 다른 하위 범위를 확인해 완전 해결된
범위만 종료하고, 이 PR을 통합한 사실과 남은 범위가 있으면 구분한다.

# #8096 빈 저장 줄의 Form 기본값 표본

기존 검토에서 공개 엔진 API로 생성한 `form-only-defaults` HWP/HWPX 두 파일을
바이트 그대로 정식 검증 입력으로 등록했다. 새 문서를 추가 수집하거나 손상 표본을
대체한 것이 아니다. 버튼·체크 상자·라디오 단추·콤보 상자·입력 상자만 있는 문단에서
각 개체의 원본 높이와 앞뒤 본문 전진을 대조한다. 기본 배경·문자색 0은 원본의 값이다.
독립 한컴 Print에도 검은 배경이 있으므로 일반 사용자 UI의 회색 배경이나 캡션 표시를
증명하는 입력으로 확대 해석하지 않는다.

[전체 입력/PDF·Print 출처·TSV·대표 패널](../../../mydocs/pr/assets/semanticist21_final_20261026/README.md)의
`form-only-defaults-hwp`·`form-only-defaults-hwpx` 행을 따른다.
Native/fresh WASM은 두 파일 각각 전쪽 raw 99.99194%, 독립 Print와 1쪽 일치다.
기존 `form_only_paragraph_without_lineseg` 회귀의 원본 단위 높이·개체 순서·문단 간격
계약을 검증하며, 이 등록으로 새 px 기대값을 만들지 않았다.

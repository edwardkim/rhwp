# 용지 방향을 정규화한 페이지 예산 대조군

상위 디렉터리의 여섯 입력은 높이만 줄여 저장 폭이 높이보다 크면서 portrait였다.
독립 한컴 2020 변환이 이 모순을 정규화해 PDF의 가로·세로를 교환했다.
그 원본과 첫 변환 PDF는 그대로 보존한다.

이 대조군은 `parse_hwp → PageDef width/height 교환 → landscape=true, attr bit 0=1 →
raw_stream/raw_provenance=None → serialize_document`로 생성했다. section과 SectionDef
control의 PageDef를 함께 변경했다. 본문 영역의 폭/높이가 변경 전과 동일함을 검사했다.
글자·줄 정보·표·그림·여백은 추가로 수정하지 않았다.

이름이 같은 상위 원본의 정상 출력이라고 보고하지 않는다. 대응 한컴 PDF는
`pdf/pr7518_review/valid_orientation/`에 보존하며 최종 실행·직접 시각 판정은 PR 리뷰에 연결한다.

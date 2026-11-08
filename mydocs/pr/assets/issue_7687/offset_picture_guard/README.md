# #7687 미완료 원본의 offset 그림 검사 재활성화 보관

## 선행 처리와 판정

PR #8012의 추가 커밋 `bbd4a09816f733f284a0a11647ab8e56f88d8013`이
`issue_7696_page_anchored_topbottom_band.rs`에 실제 hwpspec 원본 검사를 추가했다.
같은 원본의 선행 이관은 [기존 보관 기록](../reactivated_guard/README.md)에 있다.
현재 열린 #7687의 본문도 같은 원본·공식 PDF와 빈 host 흐름 문제를 명시한다.

최신 기준 `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`의 실제 출력은 72쪽,
후보 `ca26039dad8dcb07b182dd60136ae4a2aa533bd0`은 74쪽이며 독립 PDF는 71쪽이다.
동일 원본·동일 로컬 글꼴 공급으로 다시 확인했다. 목표 문단의 소속 진단이 통과하더라도
원본 전체의 쪽수·전쪽 Native/fresh WASM 90% 조건을 충족하지 않는다. 쪽수 차이로
측정하지 못한 전쪽 raw 값을 만들거나 해결된 문서로 판정하지 않는다.

이는 upstream/devel에 등록된 정상 검사를 제거하는 처리가 아니다. 이미 이관된 원본에
이번 source 추가분이 다시 넣은 새 검사의 수용을 보류한다. 같은 원본의 기존 이슈를
재사용하며 중복 이슈는 만들지 않는다. 이 보관만으로 현재 구현의 회귀가 해소됐다고
주장하지 않는다.

## 원문 보존과 활성 범위

제거 전 모듈 전체와 helper는 `issue_7696_page_anchored_topbottom_band.rs.txt`에
바이트 그대로 보존했다. source commit·Git blob·SHA-256·원본 및 보존 경로와
입력/PDF 해시는 `preservation.json`에 기록했다.

제외한 함수는 `empty_host_line_inside_offset_para_picture_band_is_absorbed` 한 개다.
별도 4쪽 정상 문서를 검증하는 아래 두 함수와 공용 helper·자동 수집 경로는 유지한다.

- `page_anchored_topbottom_band_reflowed_input`
- `page_anchored_topbottom_band_hancom_saved_input`

공통 구현과 원본 HWP·공식 PDF는 변경하거나 삭제하지 않는다. 이미 통과한 진단의 원문과
결과도 보존하며, 보관한 함수를 실행하지 않는 것을 통과 건수에 더하지 않는다.

## 복원 조건

원본 전쪽 쪽수·내용 대응이 독립 PDF와 일치하고 Native/fresh WASM raw 최저값이
각각 90% 이상이며 직접 확대 판독을 마친 뒤 정식 검사로 복원한다. 문단 소유·순서·
누락·중복·개체 밴드와의 관계를 검사한다. 원본 전체의 차이를 목표 문단 한 개의
통과나 정상 합성 문서의 점수로 대신하지 않는다.

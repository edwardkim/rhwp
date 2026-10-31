# PR #8144 한컴 정상 저장·편집 대조군

원 기여자의 비식별 문서 `samples/title-table-after-empty-paragraph.hwpx`와 `samples/paper-square-table-tac-host.hwpx`에서 파생했다. 원본 사례는 그대로 유지한다. 한컴 2024 버전 13.0.0.3901의 정상 저장본과 동일 입력의 Print를 독립 기대 근거로 사용한다.

| 파일 | 생성 방식 | 독립 관측 |
|---|---|---|
| nine-leading-empty.hwp | 원래 선행 빈 문단에 8개를 추가하고 최상위 LineSeg를 제거한 seed를 한컴에서 HWP로 정상 저장 | 9개 빈 줄 뒤 제목·TAC·본문, 1쪽 |
| nine-leading-empty-page-break.hwp | 위 seed의 제목 표 host에 명시적 Page break를 넣고 한컴에서 정상 저장 | 빈 문단이 첫 쪽, 제목·TAC·본문이 둘째 쪽 |
| same-host-full-row.hwp | 원 same-host 문서의 최상위 LineSeg를 제거한 뒤 한컴 정상 저장 | 전폭 TAC가 앵커와 다른 줄을 소유 |
| same-host-with-spare-width.hwp | 같은 seed의 TAC 및 각 행 마지막 셀 너비를 10HU 줄인 뒤 한컴 정상 저장 | 여유 폭 TAC가 앵커 줄을 공유. 셀 안 실제 줄바꿈으로 높이도 바뀌므로 높이 동일성은 가정하지 않음 |
| title-body-edited.hwpx | 원 leading-empty HWPX의 후속 표 host pi4에 `검증 ` 삽입 후 RHWP 저장 | 제목·후속 표·본문과 편집 문자열 보존, 1쪽 |
| title-leading-edited.hwpx | 같은 문서의 선행 빈 문단 pi0에 삽입 후 RHWP 저장 | 제목의 절대 위치와 새 텍스트 줄 보존, 1쪽 |
| same-host-edited.hwpx | 원 same-host pi0에 삽입 후 RHWP 저장 | 삽입된 배분 정렬 텍스트, Square·TAC·후속 본문 보존, 1쪽 |

수동 LineSeg는 기대값으로 쓰지 않았다. 첫 네 HWP의 실제 저장 줄과 표 속성은 한컴이 생성했다. 마지막 세 HWPX는 실제 편집 세션에서 내보낸 바이트이며 Native/WASM에서 동일했다. 이 저장본을 별도로 한컴 Print하여 편집 직후와 다시 열기 출력을 검증했다.

기준 PDF: `pdf/title_page_start/maintainer/`. 전쪽 Native/fresh WASM TSV·대표 PNG·글꼴·source/build·입력/출력 SHA-256과 한컴 작업 ID: [검증 증거](../../../mydocs/pr/assets/pr_8144/README.md), [provenance](../../../mydocs/pr/assets/pr_8144/provenance.json). 테스트는 설치 글꼴을 가정하지 않으며 관계·소유·내용·원본 줄 메트릭을 검사한다.

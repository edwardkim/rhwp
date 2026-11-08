# #7998 표 밖 필수 경계 paint의 소유 대조군

정상 생성·독립 한컴 2020 Print인 `local-regions/local-outer-only.pdf`의 별도 4셀을 유지하고,
명시적 PDF `q 1 0 0 rg x y w h re f Q`로 빨강 띠를 추가한 네 입력이다.
`paint-ownership/origin.json`은 원본과 실제 operator의 좌표·길이·검증 바이트 SHA-256을 보존한다.
`generate_controls.py --output <새 출력 폴더>`로 재생성할 수 있으며 container ID 차이는 구분한다.
추가 PDF operator를 한컴 작성 객체나 수동 LineSeg로 가장하지 않는다. 원래 4셀의 독립 계약은
[지역 축 대조군](local-regions.md)의 정상 생성 API·Print 출처를 따른다.

- `outside-right`, `outside-bottom`: 추론 경계 근처지만 실제 셀과 교차하지 않는 빨강 paint.
  원래 페이지 소유를 보존해야 하며 이를 미소비 경계로 오인해 실제 셀 생성을 취소하지 않는다.
- `crossing-right`: 경계를 가로지르는 paint. 내부 조각과 페이지 잔여를 모두 유일하게 보존한다.
- `unrelated-outside`: 경계 기여자가 아닌 먼 장식. 셀로 옮기거나 외부 경계로 발명하지 않는다.
- 기존 `local-regions/independent-roots.pdf`와 37개 생성 label/주소 JSON도 재사용한다.
  상단 18셀·하단 19셀과 rowspan, 원문 문자·물리 선을 함께 보존한다. 원본을 복제하지 않는다.

## 독립 기대와 실행 증거

모든 입력의 직접 열기·HWP/HWPX 저장 재열기는 Native/fresh Docker WASM 각각 15쪽,
전부 raw 2px 실루엣 100%다. 관련 원본 범위의 실제 그림과 외곽을 확대 판독했다.
외부 빨강 paint의 총면적 기대값은 제품 helper가 아닌 위 PDF `re f`의 실제 w×h다.
HWP 1/100pt 끝점 양자화 때문에 두 끝점의 합 0.02pt에 해당하는 면적 오차만 허용한다.
셀이 가려진 결과를 숨기는 좌표 clamp나 SVG hash/절대 px 핀은 사용하지 않는다.
셀 주소/span, 원문 순서·유일 소유·셀 포함, 외부 페이지 소유와 내부/잔여 분할의
비중복·면적 보존을 문자 삽입·삭제·양 저장 후에도 검사한다.

원본 저장본 6개(HWP/HWPX 각 3개)는 메인테이너가 모두 한컴 정상 열기를 확인했다.
메인테이너의 독립 HWPX Print PDF 3개(Creator Hwp 2020 11.0.0.9166,
Producer Hancom PDF 1.3.0.550)와 편집 저장본을 대조했다.
Native 6쪽 최저 99.66299%, 실제 fresh WASM 저장본 6쪽 최저 98.35379%다.
HWP는 정상 열기·같은 Print 기준 재열기 비교이며 HWP 자체의 별도 Print로 표시하지 않는다.
검증 생성·실행 스크립트, TSV/PNG, source/runtime 신원과 전후 실패는
[stage10](../../mydocs/working/task_m100_7998_stage10.md)에 연결한다.

메인테이너는 2026-10-19 이번 절편을 **시각·편집 구조 통과**로 수용했다.
일반 행열 이동·비표준 경계 스타일 상속·미확정 격자 전체 및 P5 최종 제출 게이트는 별도다.

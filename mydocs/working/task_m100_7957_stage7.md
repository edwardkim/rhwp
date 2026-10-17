# Task #7957 7단계 — PDF 글꼴 이름과 12쪽 위치 보존

사용자는 남은 `content_operators.pdf` 12쪽도 이번에 보정하도록 승인했다.
색상 보정은 직접 통과했으며, 제목·본문의 글꼴 및 간격 차이는 이번 범위에서 계속 검증한다.

## 입력에서 실제 paint까지

원본의 `INPILL+H2hdrM`은 부분집합 표기를 제외하면 설치 TTF의 nameID 6과 같다.
그 파일의 nameID 1/4는 `HYHeadLine-Medium` / `HY헤드라인M`이다. 영문 표기가
영문 전용 글꼴이라는 뜻은 아니다. Gulim→굴림은 기존 규칙이 있지만 H2hdrM은 빠져 있어
WASM trace에서 이름 해석 실패·추정 반각 폭을 관측했다. 원본 폰트 바이너리는 배포하지 않는다.

소비 경로는 PDF BaseFont → `lowering.rs` 부분집합 tag 제거 → IR font face →
`style_resolver.rs::lookup_font_name_decision`의 v2 layout-name projection → metric 조회와
최종 Native/Canvas font family다. 이름 규칙은 기존 v2 append-only change set 절차로
추가하며 다른 네 결정면의 semantic hash를 바꾸지 않는다. 등록된 이름을 먼저 올바르게
연결하고, 실제 없는 글꼴의 대체 선택과 혼동하지 않는다.

12쪽 본문 일부는 BaseFont 자체가 없는 Type3 glyph 프로그램이다. 임의로 Gulim에
연결하지 않는다. `char_x_offsets`의 PDF TJ 위치와 최종 문단의 간격을 별도로 확인한다.
이름 변경만으로 본문 위치까지 해결했다고 판정하지 않는다.

기준 runtime `8684913e0`, 색상 검증 checkpoint `b62c6d9a1`. 진단·수치·전후 출력은
ignored `output/pr-review/issue7957/f7/`에 보존한다. 기존 라이브러리로 원인을 격리한
진단 저장본은 제품 수정본과 구분한다. 기본 Studio 7700 및 그 bundle은 보존한다.

## 원인 분리와 구현

기존 CLI와 별도 진단 저장본으로 이름만 수정한 경우 Native 90.95469%, 같은 원점·폭에서
기존 공통 띄어쓰기 복원도 적용한 경우 96.40680%다. 원본은 PDF TJ 수치 이동으로
공백을 표시하지만 positional lowerer는 문자 문자열만 소비해 이를 잃고 SQUEEZE로
다시 분배했다. 기존 table lowerer의 `char_x_offsets`·`char_widths` 간격 복원을 공통
함수로 추출하여 두 경로가 같은 문자를 측정·배치하게 했다. 기존 0.2em의 단어 경계
추론을 바꾸거나 새 문서별 분기를 넣지는 않았다. 이 추론은 임의의 PDF tracking을
정확한 glyph 배치로 보존하는 계약은 아니며, Type3 원 glyph 재현도 별도 미지원 범위다.

이름은 `H2hdrM`, `HYHeadLine-Medium`을 동일 프로그램의 `HY헤드라인M`에 연결했다.
v2 registry change set의 다른 네 projection semantic hash는 변경 전과 동일하다.
기존 registry 생성 검사에는 이미 적용돼 있던 #4969 supply 2건을 반영하지 않은
831건 핀이 남아 있었다. 현재 833건에 이번 이름 규칙 2건을 더한 835건과 실제 승인된
의미 digest로 명시적으로 갱신했다. lifecycle·projection 검사 40건이 통과한다.

문자 복원은 원문을 삭제·재정렬하지 않으며 원 span의 페이지 원점·폭을 유지한다.
최대 추가 공백 수는 원문 문자 수보다 작아 원래 2MiB 추출 문자열 한도의 2배 이내다.
새 객체·glyph별 textbox는 만들지 않는다. 최종 Native/fresh WASM 전쪽 검증은 아래에
연결하며 위 진단본 점수를 제품 수정본의 결과로 재인용하지 않는다.

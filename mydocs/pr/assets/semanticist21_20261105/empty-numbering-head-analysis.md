# PR #8527 보충 보정 — 빈 번호 모양의 본문 거리

## 수정 전 관측과 독립 근거

제품 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 `levels.hwp` Native raw는
95.17%지만, 개요 9·10수준 본문이 독립 한컴 2020 Print보다 왼쪽에 있다.
동일 쪽의 본문 시작은 Native 233.4·246.7 CSSpx, PDF 텍스트 좌표를 96 DPI로
환산하면 240.030·253.303 CSSpx다. 소수 첫째 자리 출력의 반올림을 고려하면
차이는 글자 크기 10pt의 50%, 약 6.67 CSSpx다.

[원 PR #8527](https://github.com/edwardkim/rhwp/pull/8527)은 8~10수준을 7수준으로
잘못 축약하던 파싱·모델·저장·카운터를 보정했다. 현재 보충 보정은 빈 번호를 다시
그리지 않고 본문과의 원래 거리도 함께 보존하는 범위다.

정식 입력 `tests/fixtures/numbering_levels/levels.hwp`·`levels.hwpx`를 사용한다.
HWPX의 수준 9·10 `paraHead`는 문자열이 비어 있지만 `textOffsetType=PERCENT`,
`textOffset=50`, `autoIndent=1`, `widthAdjust=0`을 보존한다. 저장 LineSeg의
`horzpos`는 각각 9000·10000 HWPUNIT이다. 번호의 표시 유무와 별도의 거리 속성을
구분하는 근거는 [HWP5 §4.2.8 표 38~40](../../../tech/한글문서파일형식_5.0_revision1.3.md#428-문단-번호)과
같은 입력의 독립 Print다. PDF를 다시 출력하거나 차이 영역을 가리지 않는다.

## 생산·소비 경로와 수정 원칙

`numbering_head_text`가 빈 형식에서 `None`을 반환하면 `assign_numbering_markers`는
`Absent`를 저장한다. 이어서 줄 구성의 `list_marker_geometry`와 실제 배치의
`apply_paragraph_numbering` 모두 해당 수준의 `MarkerHead`를 받지 못한다.
그래서 본문과의 거리까지 사라진다. 저장 줄 원점에 문서별 오프셋을 더하는 해결은 사용하지 않는다.

빈 문자열도 해당 수준의 머리 속성과 함께 공통 결과로 보존한다. 줄 구성과 배치는
기존 `list_marker_geometry_for`의 같은 결과를 소비하고, 문자열이 비면 마커 paint만
생략한다. `NumberingMarker::text()`의 공개 의미는 그려질 문자열이므로 빈 머리에서는
기존처럼 `None`을 반환한다. 정의 자체가 없는 번호 문단의 `Absent`와는 구분한다.

기존 대형 `paragraph_layout.rs`의 해당 분기만 좁게 보정한다. 전체 파일 분할이나
문서별 분기를 추가하는 것으로 이 수정 범위를 넓히지 않는다.

## 검증 계획과 완료 조건

- 기존 수준·표시 마커 수·바이트 왕복 계약을 유지한다. 빈 마커 TextRun은 추가하지 않는다.
- 같은 원본의 본문 원점과 저장 문단 여백·원래 거리 사이의 관계를 검사한다.
  특정 설치 글꼴의 글리프 폭에 의존하지 않는다. Native/fresh WASM 양쪽 100.00% 완료 페이지는 직접 판독 후 독립 Print 본문 시작을 ±1 CSSpx로 보완한다.
- 원본 수준 9·10의 빈 머리, 거리 0 대조 및 번호가 있는 기존 수준을 확인한다.
- 보정 후 Native/fresh WASM의 두 형식 전쪽과 같은 영역을 직접 판독한다.
  보호 PR #8389·#8397과 새 base 대비 결과도 유지해야 한다.
- 수정 전 실패·수정 후 통과 및 실제 시각 결과를 확인한 뒤 완료 상태로 갱신한다.

현재는 공통 구현 보정과 같은 원본의 Native/fresh WASM·직접 판독·정식 회귀 검증을 확인한 단계다. 통합 전체 검증을 완료한 것으로 해석하지 않는다.

## 수정 전 검출 결과

원래 R5 라이브러리 SHA `9300b684b8145abf1c2626dd2143a3bd002f0035fbd13b793189f624f7811f2e`에
관계 검사 드라이버를 연결했다. 최소 공개 글꼴 5개 환경에서 HWP·HWPX 둘 다 실패했다.
저장 줄 시작 차이를 뺀 본문 상대 거리 비율은 약 `0.0025`, 원본 머리 속성은 `0.5`다.
[실행 결과 기록](validation-r6/runtime-checks.md)에 두 실패를 보존한다. 실패 기대값을 완화하지 않는다.
R6 제품 `86709ac16711820e61baa3b87e2c4ddaac23573e`의 Native 빌드 뒤 같은 관계 검사는 최소 글꼴에서 2건 모두 통과했다. 기존 집중 회귀 184건도 통과했다. HWP/HWPX 원본은 같은 독립 Print PDF로 Native 전 1쪽을 다시 측정해 각각 100.00000%였으며, 개요 9·10수준 본문 간격과 기존 가시 번호를 review·overlay에서 직접 대조했다. [Native 결과와 실제 증거](validation-r6/empty-numbering/README.md)를 연결한다. fresh WASM도 같은 두 원본에서 100.00000%였고 review·overlay 직접 판독을 완료했다. 같은 입력의 정본 실행과 별도 export+PNG-pair는 실제 raster 해시가 같다. 기존 3건과 신규 5건을 포함한 정식 회귀 8건이 최소 글꼴 환경에서 통과했다. 독립 Print의 1쪽 본문 시작 2개는 양쪽 100.00% 조건에 따라 ±1 CSSpx로 고정하고 관계·내용 보존 검사도 유지했다. 통합 전쪽·보호 PR 대조와 전체 필수 검증은 계속 진행한다.

# 생물독소55쪽 캡션 표의 끝 행 여유 회수

## 독립 근거와 공통 소비 경로

원본 `samples/issue5941/1480000-201900698-native-neartop-reset.hwp` SHA-256은
`18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a`,
독립 Print `pdf/issue5941-1480000-201900698-2020.pdf`는
`fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc`다.
compat2022·Print·96DPI·기존 공급 폴더와 실제 Library/Fonts를 사용했다.

4구역 문단303의 표20은 가시 캡션을 가진 비TAC 문단 기준 TopAndBottom 표다.
원본 앞 두 행은3526/3928HU이며 Name 셀의 두 행 병합은7454HU다.
독립 PDF도 앞 두 행을 보존하고 선언 총높이9766HU에 맞게 끝 행의 빈 여유만 줄인다.
종전에는 호스트 글자가 있으면 모든 행을 비례 축소해 헤더 경계와 병합 글자 위치가 달랐다.

기존 `fit_measured_table_declared_tail_to_declared_height`의 저장 줄·셀 선언·내용 하한
검증을 가시 캡션 호스트에도 적용했다. 저장 되감기·중첩 내용·실제 성장 등 기존 helper의
거부 계약은 유지한다. 미편집 저장 프레임과 편집 경로를 구분한다.
호스트의 글자 유무를 물리 행 프레임 유효성의 대용으로 쓰던 가정을 제거했다.

`원본 cellSz/LineSeg → fit_measured_for_host의 끝 행 계획 → format_table의 측정/예약 →
layout의 같은 fit_measured_for_host → resolve_row_heights_with_common_fit → 실제 TableCell`
경로로 같은 행 높이를 소비한다. 최종 행별 y와 rowspan 높이도 원본·독립 Print와 검사했다.
그림자·표 좌표·내용을 숨기거나 문서명에 따른 예외를 추가하지 않았다.

## 전쪽 시각 비교와 직접 판독

| 범위 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 수정 전55쪽 |95.69921%|95.70411%|
| 수정 후55쪽 |99.98793%|99.99232%|
| 수정 후205쪽 최저 |97.00104%|96.99896%|

각205/205쪽·90% 미달0쪽·쪽수 차이0쪽이다. 비교 기준은 직전C81이며,
양쪽 모두55쪽만0.01%p 이상 바뀌었고0.01%p 이상 하락0쪽이다.
독립 PDF와 Native/WASM review·overlay를 직접 열어 앞 두 행의 괘선·Name 병합 정렬,
끝 행 내용, 캡션·위 표·본문·쪽번호를 확인했다. AA와 글자 주변 미세 차이는 남으며
엄격 RGB 동일이라고 주장하지 않는다.

`captioned_table_keeps_header_rows_and_reclaims_only_the_closing_slack`는
205쪽·표의 단일 소유·셀별 원문 순서/누락/중복·글줄 포함·헤더 높이와 병합 관계·끝 행
여유 회수·캡션 보존을 검사한다. 같은 쪽 양쪽98% 이상과 직접 판독 후 독립 Print의
행 경계899.176/946.165/998.588/1029.275CSSpx를±2CSSpx로 보호한다.
대상은 원본 구역·문단·셀 소속으로 선택한다.

실제 수정 전7건 중6PASS/1FAIL(1.069초), 수정 후7PASS(1.114초)다.
수정 전 실패는 앞 행의 원본 높이 보존 assertion에서 발생했다.
정상 대조군 #5906 끝 행1건·#5751 내용 성장2건·#6271 후속 그림2건도 모두 통과했다.
매 phase에서 최종 포맷 이후 manifest를 준비하고 실제 suite 포함과 실행 건수를 확인했다.

## 출처와 미완료 범위

Native binary SHA-256은 `df2cd3be8bc16e991ef430594b0392ae6a225899db580ff48f527f04f68ff059`, fresh WASM은 `b4ac3aea3010b68c3c61a25d7c9defbd48a9c9801f495f506b059bd581db5f06`다.
부모6ac7a742a2와 후보 production source inventory1150파일이 일치함을 확인했다.
Mac WASM은 locked wrapper web/no-opt 로컬 대체 빌드이며 root pkg와 Studio public 해시가 같다.
Docker 최적화·Studio 실제 소비·전체 회귀 통과로 확대하지 않는다.

상세 producer·전쪽 TSV·로그는 ignored
`output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-552514f4-20261016/biotoxin55-c82/`
아래에 보존한다. Native48조건 영향 비교도 수집 중이다. 직전C78과의 비교에는C80 가는 선
보정 효과가 함께 들어 있으므로 이를C82 단독 변화나 fixed devel 비교로 쓰지 않는다.
sample16 WMF·로드맵153쪽의raw 미달, 가는 선 변경 후 다른 페이지 하락,
실제 Studio/CanvasKit·최종 fresh WASM 전체 범위·최신devel 대비 전쪽·전체 회귀/lint 검증은
PR 준비의 잔여 항목이다. 이 문서205쪽 개선으로 PR 전체 완료를 선언하지 않는다.

| 출력 | review | overlay |
| --- | --- | --- |
| 수정 전 Native |[review](before-native-review.png)|[overlay](before-native-overlay.png)|
| 수정 후 Native |[review](after-native-review.png)|[overlay](after-native-overlay.png)|
| 수정 전 WASM |[review](before-wasm-review.png)|[overlay](before-wasm-overlay.png)|
| 수정 후 WASM |[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|

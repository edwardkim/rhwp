# 공통 수평선의 아래 셀 색상 소유 (2026-10-16)

Issue: #7445. 수정 전 `89835cbfbfe7e739692f3316c659bf34cf9d85a0`, 고정 기준
`cdef0073bb24512daf0622dec5051ac62cb8d57e`에서 보정했다. 새 원격 `0953598d8`을
확인했으며 이 증거를 새 base의 전체 통과로 재인용하지 않는다.

## 독립 근거와 원인

원본 `samples/task2097/75544_pii_bunseok.hwpx`의 규제 차등화 표는 위 셀의 검은 하단과
아래 셀의 흰 상단이 모두 SOLID/0.12mm다. 원본 한컴2020 Print PDF9/28/52쪽은 아래 셀의
흰색을 사용한다. 기존 `merge_border`는 굵기·종류가 같으면 먼저 수집한 검은색을 남겼다.

별도 수동 진단본은 원본 표를 두 쪽에 복제해 두 번째 쪽의 아래 상단만 파란색으로 바꿨다.
한컴 Print도 해당 구간을 파란색으로 그렸다. 흰색 전용 제거가 아닌 동일 선의 색상 소유다.
진단 job `79bfb407-79e2-4229-9ab6-25bd27697af5`, Hancom11.0.0.9136/engine2020,
`pdf_print_method=0`, `hancom2020_pdf_driver_one_up`,2쪽,
PDF SHA256 `a4a85060ee97946df03c5df137b29268a104c3de8bb226ae305a2b0ac7fd80ed`다.
이 수동 진단본은 원본 문서의 시각 수용 기준을 대신하지 않는다.

## 공통 경로

`collect_cell_borders`의 상단 병합은 동일 굵기·종류의 충돌에서 아래 셀 색상을 우선한다.
하단 병합은 이미 수집한 아래 상단을 보존하므로 셀 배열 순서를 뒤집어도 결과가 같다.
None·굵은 선·서로 다른 종류의 기존 우선순위는 유지했다. 수직선의 우선순위는 변경하지 않았다.
일반 표/중첩 표/부분 표/세로쓰기 셀의 수집은 같은 helper를 호출하고,
`render_edge_borders`의 공통 LineNode를 SVG/Canvas2D/Native가 소비한다.

## 전쪽·정식 검사

동일 입력·독립 Print·글꼴 환경에서 Native/fresh WASM 각각 전66쪽을 새로 출력했다.
쪽수66/66, 최저93.77937%,90% 미만0쪽이다. Native devel 대비0.01%p 이상 감소는0쪽이다.
WASM 감소는7쪽에서4쪽(10/29/50/53)으로 줄었으며 남은 감소와 다른 문서 검증은 계속 필요하다.

|28쪽|수정 전|수정 후|
|---|---:|---:|
|Native|99.65899%|99.90314%|
|fresh WASM|99.66192%|99.90397%|

같은 경계의9/28/52쪽 중 Native/WASM 최솟값이 가장 높은28쪽을 px 보호 대상으로 골랐다.
독립 PDF의 흰 선 시작X132.089335/끝X177.025330/Y418.582682CSSpx를 ±2CSSpx로 검사한다.
캐논 캡처는192DPI이며 테스트 좌표는96CSSpx다.9쪽과28쪽에서 원본/역순 셀 배열을 대조하고,
흰 선의 셀 내부 소유·옆 검은 선 보존·표와 셀의 단일 배치·66쪽 보존 관계도 검사한다.

`shared_horizontal_edge_uses_the_lower_cell_color_in_either_cell_order`는 수정 전 실제
1FAIL(검정0 vs 흰색16777215,0.165초) → 수정 후1PASS(0.690초)다.
기존 border 단위 대조군9건도 실제9PASS/0FAIL이다. 실행 정본은 로컬
`c73-formal/border-counterexamples.log`다.
새 검사 추가 전에 위 전66쪽의 Native/fresh WASM90% 선행 조건을 충족했다.
Native/WASM28쪽의 review/overlay를 직접 열어 없애야 할 선 구간·정상 이웃 선·본문을 확인했다.

Native SHA256 `0cb682f0c8ede9e276af23f88a7877ae6f862343415bda78af40ababb5c107d3`, WASM SHA256 `d7664f5b5f8ffeb95ef373ffdf7205d396b3f8848225761ed80b6d7cd6f9f7ce`.
TSV·명령·생산/글꼴/입력/PDF hash·전후 실행 로그는 ignored output의
`regression-policy-audit-20261015/rebase-cdef0073-20261016/c73-*`에 보존한다.
Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
새 전체 회귀·actual 소비·새 base 재검증 완료를 주장하지 않는다. 지침은 변경하지 않았다.

## 비교 이미지

|출력|수정 전|수정 후|
|---|---|---|
|Native review|[review](before-native-review.png)|[review](after-native-review.png)|
|Native overlay|[overlay](before-native-overlay.png)|[overlay](after-native-overlay.png)|
|WASM review|[review](before-wasm-review.png)|[review](after-wasm-review.png)|
|WASM overlay|[overlay](before-wasm-overlay.png)|[overlay](after-wasm-overlay.png)|

# OOXML 값축 글줄 높이와 숫자 표시 간격

## 원인과 독립 근거

이 단계는 `review/semanticist21-20261019`의 전체45개 PR·Draft10개 검토 범위를 유지한다.
원 이슈/해결 경로는 #1882→#1890·#1954, #6624→#6626→#6645이며 이전
[차트 글줄 기록](task_semanticist21_20261020_chart_text_frames.md)의 선행 분석을 재사용한다.
현재 숫자 생략의 `ceil(2font/spacing)`은 외부 PR의 원문이 아니라 이 브랜치의 중간 보정
`fa889d3f48f`에서 추가됐다. 잘못된 현재 출력을 보정하며 기존 기대값을 완화하지 않는다.

작업 디렉터리는 `output/pr-review/semanticist21-20261019/`다. 1003은 같은18pt 차트의
높이만225·230·240으로 바꾼 세 입력이다. 현대 차트와 fallback OLE의 공통 크기를 함께
갱신했고 값·글꼴·범례·차트 XML/CFB 바이트는 보존했다. 독립 진단 입력이며 원본 통과로
보고하지 않는다. 실제 저장 제품에 따라 한컴2020 engine의 PrintMethod0·one-up으로
출력했고 세 요청 모두 성공했다. 입력/PDF SHA와 한컴 버전은 각 complete 영수증에 있다.

1005는 커밋 `11767e35f32b52ab54e899836c5e1dc5ce5be10d`의 Native074 실제 SVG에서
축 끝과 숫자를 읽은 기하 진단이다. 1006은600dpi Print의 숫자 잉크 밴드를 같은0~6 축의
위치와 대조했다. 숫자 자체도 review/overlay에서 직접 확인하며 밴드 위치만으로 완료하지 않는다.

| 차트 높이 | 독립 Print의 숫자(아래→위) | Native074 |
| --- | --- | --- |
| 225 | 0·6 | 0 |
| 230 | 0·4 | 0 |
| 240 | 0·4 | 0·6 |

225의 실제 축 높이는43.04, 230은48.08, 240은58.08이었다. 세 칸의 눈금 간격에
24px 글자의 기존 공통 줄 높이1.3em을 적용하면 각각3·2·2칸의 라벨 간격이 된다.
2em을 반올림하는 가설은230에서3칸을 골라 독립 Print와 다르므로 채택하지 않는다.
단순 좌표 이동이나 제목 줄 높이1.25배로 이 숫자 선택 차이를 숨기지 않는다.

## 적용 계획과 소비 경로

`text_style`의 축 글줄 높이 → `automatic_scale`의 축 간격 예산 → 최종 plot 높이와
`render_value_grid`의 라벨 생략 간격 → 실제 숫자 방출을 연결한다. 측정이 이미 사용하는
1.3em을 공통 helper로 만들고 실제 표시도 소비하게 한다. 원점·축 범위·눈금·글꼴·제목과
별도3D 투영 경로는 변경하지 않는다. 수평 축의 기존 숫자 선택과 정상 대조군도 확인한다.
현재 코드·독립 Print·확대 판독·새 head의 Native/fresh 결과를 연결한 뒤 회귀를 추가한다.

## 별도로 배제한 가설과 진행 중인 검증

기본 외곽선은 명시0.75pt Print 대조로 단위 오류를 입증해 `11767e35f3`에서 보정했다.
Native074는268.40초에 성공했고180개 기록 소스가 그 커밋과 같음을1000에 보존했다.
그 제품의180조건 Native 검증은160조건 통과·20조건 미달로 종료했다. 이전 제품 fresh027의176조건은134조건 통과·42조건 미달로 종료했다.
1009의 같은180조건 Native073/074 대조에서0.1%p 이상 하락은 없었다. 서로 다른 소스의 결과를 이 단계의 최종 검증으로 합치지 않는다.

표식 크기를 pt로 곱하는 가설은 적용하지 않는다. 선언 단위는
[OOXML 사양](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.charts.size?view=openxml-3.0.1)
상 pt이나, 기존759/764의 같은 한컴 Print는2/7/14 변경에도600dpi 차트 전체 픽셀이 같았다.
786의 한컴 재저장 XML에도14가 남았으므로 저장과 실제 소비를 구분한다. 해당 증거를
재확인했고 중복 Print 요청이나 파서/표식 반경 변경을 하지 않았다. 자동 삼각 표식의 실제
모양·그리기 순서 차이는 별도로 남아 있으며 점수 통과로 확대 판독 완료로 바꾸지 않는다.

후속 WASM028 자동 빌드는 숫자 간격 보정을 함께 검증하도록 보류했다. Native074 빌드는
중단하지 않고 성공 제품·소스를 보존한 뒤 대기용 orchestration만 종료했다. 해당 종료143은
제품 빌드 실패가 아니다. 실제 빌드 exit0은 receipt로 확인한다. 전체 필수 gate가 끝나기 전
PR·push·remote review를 수행하지 않는다.

현재 근거는 `chart-axis-label-spacing-controls-1003/manifest.json`,
`mcp-chart-axis-label-spacing-controls-1003/*-complete.json`,
`chart-axis-label-layout-1005/measurements.json`, `chart-axis-label-bands-1006/measurements.json`이다.
로그는 `logs/chart-axis-label-spacing-1003-{generate,print}.log`,
`logs/chart-axis-label-layout-1005.log`, `logs/chart-axis-label-bands-1006.log`,
`logs/chart-axis-spacing-native-before-1007.log`다. 최종 whole-page TSV와 직접 판독을 대신하지 않는다.


## 경계 검증과 공통 글줄 높이 구현

1011의 추가 세 입력은226·227·228 높이이며 다른 값·서식·호스트는 보존했다.
세 공식2020 Print 모두0·6을 표시했다. 1012의 실제 Native074 SVG는 모두0만 표시했다.
1013의 독립 숫자 밴드는 축 높이44.08/45.04/46.08과 연결했다. 230의0·4와 함께 확인하면
1.2em/1.25em 또는2em의 단순 반올림으로는 이 경계를 설명하지 못한다. 측정에 이미 쓰는
1.3em의 글줄 점유는 여섯 입력을 모두 설명한다. 인접 입력을 제외하고 단일 점수에 맞추지 않았다.

`text_style::axis_line_height`를 공통 생산자로 만들었다. `automatic_scale`은 기존과 같은
글줄 높이를 이 helper에서 받고, `render_value_grid`도 숫자 생략 간격을 같은 값으로 정한다.
측정의 간격 예산·nice 단위 선택은 변경하지 않았다. 실제 표시에서만 잘못된 별도2em
점유를 제거했다. 빈 예산과 최소0 라벨의 기존 guard 및 수평 축 분기는 유지한다.

1008의 fmt·release-test 차트 crate 빌드·전체 target Clippy는 통과했다. 기존 검사는
157PASS/8FAIL로 같은 여덟 실패다. 새 회귀·기대값·골든은 추가/갱신하지 않았다.
1010의 모듈73조건은 실제 동결 측정 tuple로 실행했고 미등록 tuple의 추정 폭 fallback은 없다.
그 모듈을 다시 컴파일하지 않고1016에서 추가 경계 세 입력에 재사용했다. 총76개의 모듈
실행이며, 이 사실만으로76조건의 한컴 시각 일치를 주장하지 않는다.

1016의 여섯 숫자 계약은 실제 Native074에서 모두 불일치했고, 새 모듈의 실제 방출에서는
모두 독립 Print와 일치했다. 이는 정식 Rust 회귀나 새 Native/fresh 전쪽 gate가 아니다.
1015의 실제 전쪽 Native074 review·standalone overlay를 함께 열어97.05197%여도 PDF의
4가 빠진 모습을 직접 확인했다. 1014의 잘라낸 모듈 진단과 전쪽 증거를 구분한다.

새 원본·독립 PDF 전체 조건은 기존180개와 새 높이6개를 합친186개다. 소스 단계 커밋 뒤
Native075와 같은 소스의 최적화 fresh028에서 한꺼번에 재검증한다. 미달20조건·자동 삼각
표식 판독·기존 Rust 실패와 전체 통합 검증은 계속 해결하며 이 중간 결과로 PR을 제출하지 않는다.

추가 근거는 `chart-axis-label-boundary-controls-1011/manifest.json`,
`mcp-chart-axis-label-boundary-controls-1011/*-complete.json`,
`chart-axis-label-layout-1012/measurements.json`, `chart-axis-label-bands-1013/measurements.json`,
`chart-axis-label-contract-1016/measurements.json`, `chart-axis-native-before-panels-1015/`,
`chart-axis-line-height-1008-validation.json`, `chart-legend-native073-native074-1009.json`이다.
추가 로그는 `logs/chart-axis-label-boundaries-1011-print.log`,
`logs/chart-axis-label-bands-1013.log`, `logs/chart-axis-label-contract-1016.log`,
`logs/chart-axis-native-before-panels-1015.log`, `logs/chart-axis-line-height-1008-*.log`다.

## 실제 제품 재검증과 새 관계형 회귀

Native075의186조건은166조건 통과·20조건 미달로 끝났다. 최적화 fresh028은739.34초에
빌드 성공했고 Native075와 기록된 제품 소스180개가 같으며 Studio 동기화도 일치했다.
fresh028의186조건 실행은 진행 중이다. source SHA는 두 제품 모두 `a19d7c3d164807d69ef85b40a75908957cb0b2d8`다.

여섯 높이 입력에 한정한 전쪽 검증은 Native/fresh 모두 끝났다. 전체 쪽수는각1=1,
누락0, 최저 raw는 Native97.24897%·fresh96.34703%다. 1024의 같은 영역 review와
독립 overlay를 양쪽에서 여섯 입력 모두 직접 열어 표시 숫자 보존을 확인했다.
이 확대 crop의 진단 점수를 전쪽 TSV 점수로 바꾸지 않았다.

독립 Print의0·6/0·4를 기대값으로 새 파일
`tests/cases/chart_axis_label_line_occupancy.rs`에 관계형 회귀6함수를 추가했다.
기존 검사·이름·기대값·골든을 교체하지 않았다. 실제 문서 출력의 숫자 내용·순서·
비겹침과 항목4개·범례3개·제목의 소유·보존·앞뒤 관계를 검사하며 절대 px는 고정하지 않는다.
입력 HWPX6개·독립 PDF6개와 해시는 `mydocs/pr/assets/chart_axis_label_occupancy_20261021/`에 보존했다.

1026에서 한 번 컴파일한 같은 검사를 Native074/075에 연결했다. 이전 제품은 숫자
assertion에서6FAIL, 이후 제품은6PASS였다. Cargo가 정한 현재 실행 파일의 nextest도
6PASS였다. 빌드·환경 오류를 수정 전 실패로 세지 않았다. 공용 `target/pr-review`,
빌드 잡8개를 사용했고 생성된 suite·manifest는 커밋 대상이 아니다.

## 대조군에서 확인한 추가 결함

1027은 같은 입력·PDF·글꼴 조건180개를 Native074/075에서 대조한 결과이며 최신
`upstream/devel` 대비 검증은 아직 아니다. 0.1%p 이상 하락2건을 확인했다.

| 입력 | Native074 | Native075 | 직접 판독 |
| --- | ---: | ---: | --- |
| `chart-small-budget-700-small-height150-explicit14` | 99.30492% | 98.54921% | PDF0·6을 현재0·4로 바꾼 실제 회귀 |
| `rotated-label-1800-times` | 51.02683% | 50.54393% | PDF는 가로 범주·0·2·4·6, 현재 회전 범주·0·4 |

1028의 실제 전후 출력·review·standalone overlay로 대조했다. 여섯 경계의 통과만으로
1.3em을 모든 배치의 완성 규칙이라고 판단하지 않는다. 범례 예약·실제 축 공간과 글줄
점유를 함께 추적하며 새 분기·배수를 추가하기 전에 반례를 확보한다. 이를 해결하기 위해
1030에서 오른쪽 범례10pt·높이160/165, 하단 범례18pt·높이229의 세 입력을 높이만 바꾸고
공식2020 Print로 출력했다. 실제 제품·Print 결과의 대조를 계속한다. 추가 미달·하락과
최신 devel 대비 검증이 남아 있으므로 PR·push·원격 review를 수행하지 않는다.

별도 표식 진단1021/1023은 원본 HLC/OHLC에서 그리기 순서·크기·흰 외곽선을 나누어
관측했다. 순서 변경의 국소 오차 감소만으로 표식 전체의 정합을 판정하지 않는다. 선언한
`c:size`의 단위나 국소 RMSE를 근거로 제품의 표식 반경·파서·paint 순서를 바꾸지 않았다.

새 근거는 `chart-axis-regression-1026-{results,binaries}.json`,
`chart-axis-native074-native075-1027.{json,tsv}`, `chart-axis-label-declines-1028/`,
`chart-axis-pitch-counterexamples-1030/manifest.json`, `mcp-chart-axis-pitch-counterexamples-1030/`다.
로그는 `logs/chart-axis-regression-1026-{build,before,after,current}.log`,
`logs/chart-axis-label-declines-1028.log`, `logs/chart-axis-pitch-counterexamples-1030-print.log`다.

1032의 세 review·독립 overlay를 열어 높이 160/165의 PDF는 0·4, 높이 229의 PDF는
0·6임을 확인했다. 현재 제품은 세 입력 모두 0·4이므로 229에서도 숫자가 잘못됐다.
전쪽 raw 94.24968%를 숫자 일치로 간주하지 않는다. 1033에서 독립 PDF의 긴 중성색
격자 밴드를 600dpi로 측정해 Native의 축 높이와 분리했다. 이 결과와 작은 높이 150의
기존 Print는 후속 공통 규칙 검토의 반례로 유지한다. 예상과 다른 자료를 제외하지 않는다.

1029 focused Clippy는 exit101이었다. 새 회귀 함수의 실행 실패가 아니라 root lib의
`src/renderer/pua_glyph.rs:193` 수동 find와 `src/renderer/render_tree.rs:400` 큰 enum 오류다.
경고 허용·예외로 우회하지 않고 다음 코드 단계에서 보정한다. 새 테스트와 입력·PDF를
보존한 뒤 같은 공유 target에서 관련 lint와 회귀를 재실행한다. PR 준비 완료가 아니다.

## Native075·fresh028의 186조건 종료 결과

1042에서 양쪽 실행의 종료 상태와 모든 manifest를 확인했다. 두 제품의 source SHA는
`a19d7c3d164807d69ef85b40a75908957cb0b2d8`로 같다. Native와 fresh WASM 각각 186조건·
186쪽을 측정했고 모든 입력의 독립 PDF와 전체 쪽수 1=1, 누락·측정 오류 0이었다.
양쪽 모두 166조건이 90% 이상이고 20조건이 미달이며, 미달 입력의 집합도 같다.
런타임 exit1은 이 20건의 시각 gate 미달을 뜻한다. 빌드 실패로 바꾸어 보고하지 않는다.

같은 180조건의 fresh027→028은 138조건 통과에서 160조건 통과로 늘었다.
기존 176조건과 명시 외곽선 대조 4조건을 합친 수치이며 새 높이 6조건은 이 비교에 넣지 않았다.
외곽선 보정의 개선이 있어도 아래 두 하락을 상쇄하지 않는다.

| 입력 | fresh027 | fresh028 | 변화 |
| --- | ---: | ---: | ---: |
| `chart-small-budget-700-small-height150-explicit14` | 98.94058% | 98.32873% | -0.61185%p |
| `rotated-label-1800-times` | 48.38744% | 47.94300% | -0.44444%p |

두 하락은 Native074/075에서 확인한 입력과 같았다. 작은 차트와 새 높이 229의 실제
숫자 불일치를 공통 조판 규칙으로 보정해야 한다. 설치된 실제 글꼴의 상승·하강 메트릭을
각각 반영하는 가설은 관측 10조건을 설명했으나, 다른 글꼴·경계·DPI의 독립 반례 검증 전에는
구현 계약으로 승격하지 않는다. 단일 배수나 문서별 예외를 추가하지 않았다.

근거는 `chart-axis-products-final-1042.json`, `chart-axis-wasm027-wasm028-1042.tsv`,
`chart-axis-font-metric-hypothesis-1038.json`이며 로그는 `logs/chart-axis-products-final-1042.log`다.
이 결과를 후속 lint 커밋 `fa0bdf1b55`의 최종 시각 검증이나 최신 devel 비교로 재귀속하지 않는다.
최신 devel 대비 검증과 모든 PR의 필수 검증은 계속한다.

## 회전 전환 경계와 자동 범주 간격의 분리

1048·1059는 Times New Roman 18pt 원본에서 개체 폭만 바꾼 독립 대조군이다.
현대 차트와 fallback OLE의 크기를 함께 바꾸고 차트 XML·CFB는 바이트 그대로 보존했다.
한컴 2020 11.0.0.9136의 PrintMethod0·one-up 영수증과 입력/PDF SHA를 보존했다.
폭 412·418·424·426·428·429에서는 범주가 45도로 회전하고, 기존 폭 430에서는 수평이다.
1052·1063의 review와 standalone overlay에서 같은 영역을 직접 판독했다.
제목 잉크의 크기가 같은 것도 확인했으므로 개체 그림을 단순 확대·축소한 대조가 아니다.

Native075는 여섯 새 폭에서 모두 45도로 표시하나 범주 원점과 플롯 높이가 다르다.
새 426·428·429의 전쪽 raw는 각각 74.62210%·74.58211%·74.81201%이며,
각 입력의 한컴/RHWP 전체 쪽수는 1=1이다. 이 입력을 통과 원본이나 새 회귀로 등록하지 않는다.
폭 430만 맞추려고 여백을 고정량 또는 글자 크기의 단일 배수로 줄이면 기존 10pt
20150·20300HU 대조군의 회전이 틀어진다. 실제 글꼴의 advance도 Canvas·Windows GDI와
가깝게 일치하므로 이를 글꼴 누락으로 분류하지 않는다.

1060에서 3D 20pt는 별도의 범주 선택 결함도 확인했다. 독립 Print는 `항목 1·항목 3`을
표시하지만 제품은 `항목 1·2·3·4`를 모두 방출한다. 모든 회전 글줄의 축 정렬 사각형이
서로 분리돼야 한다는 가정은 45도 정상 대조군에도 적용할 수 없다. 같은 각도의 글줄 사이
수직 거리는 범주 중심 간격과 각도에 따라 달라지며 90도에서는 글줄의 가로 점유가 된다.

1064는 같은 3D 20pt의 폭 460·480과 높이 270 대조군이다. 차트 XML/CFB·데이터·서식은
보존하고 한 번에 한 물리 치수만 바꿨다. 세 Print 요청은 모두 성공했다.
1065의 동결 제품 모듈 진단에서 폭 460·480은 45도, 높이 270은 90도이며 실제 중심 간격은
각각 30.31·32.20·31.16px다. 높이 270은 em과 1.3em 사이에 있어 두 점유 가정을 구별한다.
독립 Print의 범주 선택과 전쪽 비교를 확인한 뒤 적용할 점유 규칙을 확정한다.

보정의 생산·소비 경로는 `CategoryLabelLayout`의 방향·측정 결과 → 최종 축 또는
`SceneFrame::projected_size`의 실제 범주 중심 간격 → 계획한 표시 간격 →
`render_category_labels_at`의 범주 텍스트 방출이다. 막대·데이터·범주 소유·눈금 수를
삭제하거나 잘못된 원점을 숨겨 겹침을 없애지 않는다. 2D와 3D의 실제 호출 경로를 대조하고
같은 글줄의 표시 계획을 소비하도록 한다. 독립 반례가 맞지 않으면 구현 가정을 기각한다.

추가 증거는 `chart-label-gap-1050-plan.json`, `chart-label-gap-1061-plan.json`,
`chart-label-gap-panels-1052/`, `chart-label-gap-panels-1063/`,
`chart-label-boundary-panels-1060/`, `chart-category-skip-module-before-1065/`다.
원격 devel은 읽기 전용 조회 1061에서도 `2e3b4f880bd11d35398e6ea0100846611f13a72d`였다.
전체 45개 PR의 검토와 최종 Native/fresh·Rust·lint·devel 무회귀 검증을 계속한다.

1067의 직접 판독에서 세 대조군의 Print 값축은 모두 0~10이었다. 기존 20pt도 0~10인데
제품은 네 입력 모두 0~6이다. `automatic_3d_vertical`이 실제 글줄 예산 2를 최소 3으로
늘려 단위 2·최댓값 6을 고르는 것이 원인이다. 한컴의 단위 5·최댓값 10을 수용하도록
최소 예산을 2로 보정하고 숫자 폭 → 좌측 예약 → 투영 크기까지 같은 결과를 전달한다.
작은 글자·15pt·명시 범위·누적·가로 축의 기존 대조도 함께 확인한다.

높이 270의 Print는 90도로 범주 네 개를 모두 표시한다. 폭 460은 90도·네 개,
폭 480은 45도·네 개다. 1.3em을 범주 생략 간격으로 쓰는 가설은 높이 270에서 틀려
기각한다. 90도 글줄의 em 가로 점유와 실제 투영 중심 간격에 따른 표시 간격을 계획하고
paint는 그 계획을 소비하게 한다. 각도 선택 차이와 실제 위치 차이는 별도로 남겨 해결한다.
전쪽 90% 미달 원본에 새 회귀나 골든을 먼저 추가하지 않는다.

1070의 첫 모듈 대조는 기존 939 조각을 비교 기준으로 사용했으므로 73개 전부 변경으로
표시됐다. 1071에서 실제 Native075와 같은 동결 모듈 1010을 같은 측정값으로 실행해
비교 기준을 바로잡았다. 기존 70개 중 69개는 바이트 동일하고 20pt와 새 크기 3개만 바뀌었다.
이 진단은 전쪽 Native/fresh gate가 아니다.

두 칸 예산의 단위 5를 고른 뒤 데이터 최댓값 5로 한 칸만 남기는 추가 원인도 확인했다.
Print의 0~10과 다르므로 자동 3D 축은 단위를 유지하면서 최소 두 칸을 확보하도록 보정한다.
명시 범위는 이후의 기존 소비 분기에서 유지하고, 최종 투영 높이가 작으면 격자 단위의
부분집합으로 표시 간격을 정하는 기존 경로와 연결한다. 높이 270의 0·5·10과 낮은
프레임의 0·10을 구별해 확인한다. 음수·정상 대조도 실제 출력으로 확인한다.

## 범주 표시 계획과 두 칸 축의 단계 검증

표시 간격은 `CategoryLabelLayout.label_stride`에 보존한다. 2D는 실제 plot 너비,
자동 2.5D는 `SceneFrame::projected_size`의 너비를 소비하며 paint는 이 계획만 읽는다.
공통 장면이 없는 구형 3D fallback에서는 투영 전 너비로 간격을 추측하지 않고 기존 표시를
유지한다. 수평·45도 글줄은 이번 세로 글줄 간격 분기의 비적용 경로다.
최소 두 칸은 자동 3D 값축의 단위 선택 뒤 확보하며 명시 min/max의 기존 소비 순서는 유지한다.
새 테스트·골든·기대값 변경은 추가하지 않았다.

1078에서 공용 `target/pr-review`, 빌드 잡 8개로 단계 검증을 순차 실행했다.

| 검증 | 실제 결과 |
| --- | --- |
| 차트 crate format·build·all-target Clippy `-D warnings` | 모두 통과 |
| 기존 차트 crate 테스트 | 157 통과·8 실패 |
| 변경 전 1008과 실패 항목 대조 | 같은 8개, 새 실패 0개; 기존 실패는 계속 해결 대상 |
| 같은 글자 폭을 소비하는 동결 모듈 전후 대조 | 73조건 중 69조건 바이트 동일 |
| 바뀐 원본 | 기존 3D 20pt와 새 물리 크기 3개만 변경 |
| 새 Print 6개 영수증·파일 SHA | 실제 2020 11.x·PrintMethod0·one-up 및 입력/PDF 해시 확인 |
| 새 코드의 전체 Native/fresh WASM·devel 비교 | 아직 실행하지 않음; 단계 모듈 진단으로 대체하지 않음 |

최종 모듈에서도 원래 낮은 20pt의 값축은 0·10, 범주는 1·3으로 Print와 같아졌다.
높이 270은 범주 1·2·3·4를 보존하지만 중간 숫자 5가 빠진다. 폭 460은 Print의 90도와
달리 45도이고, 폭 480의 45도는 유지된다. 실제 3D 면의 세로 비율 차이도 남아 있다.

1075의 같은 영역 모듈 실루엣은 원래 20pt에서 66.99543%→81.22312%, 높이 270에서
67.51065%→79.95179%, 폭 480에서 67.02939%→71.95939%다. 폭 460은
64.98717%→64.84532%로 하락해 각도·프레임 보정과 재검증이 필요하다.
이 수치는 차트 조각 진단이며 전쪽 Native/fresh 판정값이나 최신 devel 비교가 아니다.
큰 차이가 남아 있으므로 전체 빌드 반복 대신 원인 계층의 독립 대조를 먼저 계속한다.

1074는 두 새 폭에도 기존 원본의 절대 프레임 원점을 가정한 진단 오류였다.
1067의 실제 개체 프레임을 따라 1075에서 바로잡았다. 원본·PDF·제품 좌표를 바꾸지 않았으며
1074의 두 잘못된 수치는 사용하지 않는다. 1076은 600dpi Print의 실제 괘선으로 직접 읽은
앞면 좌표다. 원래 20pt의 높이는 56.96px이나 제품은 약 46.15px여서 이 차이를 보정해야 한다.
이 측정값을 코드의 문서별 상수나 좌표 회귀 핀으로 추가하지 않았다.

기록은 `chart-category-stage-1079.json`, `chart-category-validation-1078.json`,
`chart-module-diagnostic-category-final-1078/manifest.json`,
`chart-category-layout-panels-1075/`, `chart-category-print-geometry-1076/measurements.json`이다.
실행 로그는 `logs/chart-category-final-{build,tests,clippy,module}-1078.log`,
`logs/chart-category-layout-panels-1075.log`, `logs/chart-category-print-geometry-1076.log`다.
원래 전체 45개 PR와 Rust 57건 실패, 최종 전쪽 Native/fresh·최신 devel 0.1%p gate는
완료하지 않았으며 PR 준비·push·원격 review를 진행하지 않는다.

## 2026-10-21 실제 글꼴 표와 독립 GDI 측정의 추가 대조

제품 소스 `43abfbab6748e5345d848de8661098dc5b59b10e`의 Native088로 대표 미달 6조건을
재출력했다. Times 18pt 자동 회전 50.54393%, 5pt 세로 막대 78.47574%, 20pt 폭460/480은
각각 69.56030%·74.04388%, 직각 축 해제 카메라 58.95322%, 실제 corpus035는
87.64310%다. 모두 전쪽 raw이며 이전 모듈 조각 점수와 구분한다. 이 단계에서는 제품 코드와
회귀 기대값을 바꾸지 않았다.

- 독립 Windows GDI 실측1054의 세 글꼴·10크기, 총30조건을 같은 요청 글꼴의 Linux
  등록 face에 있는 VDMX·OS/2 표와 대조했다. 정수 ppem의 해당 VDMX 그룹을 우선하고, 없을 때
  WinAscent·WinDescent를 각각 환산하면 높이뿐 아니라 상승·하강 성분도30건 모두 일치했다.
  Times 18pt의 실측 높이는27px이고 기존1.3em 계산은31.2px다.
- 전진 폭은 같은150조건을 HDMX 또는 글리프별 반올림 HMTX로 대조해146건이 일치했다.
  나머지4건은 HDMX에 해당 ppem이 없는 Times 10.5pt·15pt의 `Label 1`·`officeAV`다.
  따라서 이 fallback으로 모든 GDI 폭을 재현했다고 주장하지 않는다. Times 18pt의
  `S10`은 실제 HDMX/GDI36px이고 현재 본문용 측정 경로는37px다.
- Times 자동 회전 입력의 최신 review와 overlay에서는 Print의 수평 범주와 제품의45도
  범주 차이를 확인했다. 회전으로 바닥 예약이 늘면 plot 높이와 표시 눈금도 함께 바뀐다.
  숨겨진6 눈금을 값축 max4로 오인해 숫자 범위를 바꾸지 않는다.
- 실제 측정 provider가 전달하는 폭·상승·하강과 범례의 예약/표시 간격을 공통 결과로
  소비할 수 있는지 분석한다. 글꼴별 상수나 특정 폭에서만 회전시키는 분기를 추가하지 않는다.
  비직각3D 카메라의 별도 투영 차이도 글꼴 높이 하나로 해결했다고 간주하지 않는다.

근거는 ignored output의 `chart-vertical-metric-table-analysis-1350.json`,
`chart-horizontal-metric-table-analysis-1356.json`, `chart-native088-selected-1354/results.json`,
`chart-native088-panels-1355/provenance.json`이다. 입력·PDF·실제 글꼴 파일 SHA와 실행 로그를
각 원장에서 연결했다. 새 분석은 기존29개 미달·devel 하락의 해소 증거가 아니며 보정을 계속한다.

HDMX가 없는 크기의4개 불일치를 포함해 같은150조건을 실제 FreeType 힌팅으로 대조했다.
일반 힌팅은117건, 모노 힌팅은150건 모두 GDI 전진 폭과 일치했다. 자동 힌팅을 배제한
모노 경로도 같은150건이 일치한다. `chart-hinted-advance-analysis-1370.json`과
`logs/chart-hinted-advance-analysis-1370.log`가 실제 글꼴 파일·실측·실행 모드의 근거다.
이 제한된 실측을 모든 글꼴·문자·크기의 호환 보장으로 확대하지 않는다. Native 전용 측정을
추가해 fresh 경로를 다르게 만들지 않으며, 공통 글꼴 공급과 차트의 측정·배치 계약을 검토한다.

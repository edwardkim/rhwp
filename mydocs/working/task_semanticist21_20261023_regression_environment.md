# 회귀 검사 등록 환경과 저장 프레임 순회 보정

## 범위와 시작 기준

작업 브랜치 `review/semanticist21-20261019`, 시작 커밋 `32f534d4d5`이며
base는 `605ccfd92f2f12f18e14182548d729f9929fda00`다. 기존 45개 PR 범위를
유지하고 새 PR을 체리픽하지 않는다. 행정업무편람의 추가 개선과 다른 PR이 수정 중인
두 사양 문서는 제외한다. TABLE 값 2/CELL은 셀 내부 글줄도 나눌 수 있는 의미로
참고하며, 내부 `RowBreak` 이름을 행 경계 전용 정책으로 해석하지 않는다.

## 등록 환경 검토 방법

1924 전체 실행의 단언 실패 39건은 테스트 함수의 등록 커밋·원 체리픽·현재 소스
조건을 먼저 수집했다. `regression-registration-audit-1948.json`에 등록 이력과 현재
실행 명령, 변경된 Rust 테스트 파일 80개를 기록했다. 이는 목록 수집이며 전체 검사의
등록 적합성 확인 완료를 뜻하지 않는다. `src` 내부 단위 검사와 조건부 Native 검사도
해당 실행 경로에 따라 구분한다.

검사별로 원 해결 PR → 실제 통합·보정 커밋 → 독립 출력·실행 근거를 연결한다.
원본/PDF 해시·Print 출처·compat·DPI·프로필·글꼴 공급·feature와 Native/fresh WASM
전체 쪽수 및 raw 최저값을 대조한다. 절대 좌표 검사는 해당 페이지의 양쪽 98% 이상과
직접 판독 근거도 확인한다. 현재 실패만으로 등록 오류를 선언하거나 기대값을 바꾸지 않는다.

## 현재 확인한 사례

| 검사 | 등록 근거·실행 조건 대조 | 현재 판단 |
| --- | --- | --- |
| 80168의 완료 93쪽 | 함수 등록 `ab965e65e0`, 통합 PR #8148. 원본·한컴 2024 Print 해시 일치. 제출 TSV 원본은 Native/fresh 157쪽, 최저 93.90078/91.66667%, 93쪽 99.99223/99.96152%. Print·compat 2024·96 DPI와 글꼴 경로·producer 해시가 명시돼 있다. | 등록 시각 근거를 확인했다. 현재 컷 회귀는 구현에서 보정하며 독립 기대값을 유지한다. |
| 글자겹침의 non-BMP·같은 앵커 2건 | 등록 `c240d29285`, 원 `a3f80b14ad`, 원 PR #7819. 원 PR은 Native 테스트 통과와 별도로 독립 Print·전체 Native/fresh 시각 근거의 부재를 명시한 Draft다. | 당시 통과 주장만으로 시각 선행 조건 충족을 인정하지 않는다. 대응 입력의 독립 검증과 실제 동작 보정을 계속한다. |
| 같은 글자겹침 2건의 현재 환경 | 전체 실행은 Cargo 기본 feature다. 같은 테스트 소스를 동결 Native125 라이브러리와 `native-skia`로 별도 컴파일해도 두 검사 모두 같은 단언으로 실패했다. | 기본 feature와 Native Skia의 차이만으로 설명할 수 없다. 기존 검사를 숨기거나 feature로 우회하지 않는다. |
| 시장구조조사의 저장 되감김 | 등록 `04404a07be`, 원 `8c9f9e925a`. 당시 기록의 원본·315쪽 Print·본문 되감김 근거와 현재 환경을 대조 중이다. 현재 장시간 경로는 별도 스택으로 확인했다. | 시각·등록 환경 확인을 계속하며 성능 문제를 먼저 보정한다. |

자료는 ignored `output/pr-review/semanticist21-20261019/`의 1931~1948 실행 기록에 있다.
1944 환경 진단은 rustc 인자 구성 오류로 빌드 실패했다. 1945에서 명령을 수정해
실제 컴파일·두 검사를 완료했다. 실패한 진단을 통과 증거로 재분류하지 않는다.

## 성능 보정의 독립 근거와 계획

`native_stored_table_has_intra_paragraph_frames`의 도입은 현재 브랜치의
`2d6f00d4cf`, 선행 행 인증 추가는 `6e0aab2090`이다. 현재 구현은 실제 원본 문단의
재시작 여부를 확인하기 전에 셀별 선행 행 합과 `cell_units`를 계산한다. 1939의
gdb 자식 실행에서 이 두 경로를 확인했고, 1940의 CFB 원문 읽기에서 시장구조조사의
큰 표는 2,195/2,196행·8열임을 확인했다. 전체 테스트의 CPU 사용만으로 원인을
추정한 것이 아니다.

최종 참 판정에는 같은 원본 문단의 인접 저장 글줄이 양수 원점에서 0으로 되돌아가는
경계가 필요하다. 그 조건이 없는 셀은 기존 구현에서도 반드시 false다. 동일한 원본
조건을 먼저 검사해 불필요한 선행 행 순회와 재조판을 건너뛴다. 실제 후보의 기존
유닛·원본 높이·물리 프레임 인증은 유지한다. 문서 이름·행 수 임계값·시간 제한으로
결과를 우회하지 않는다.

보정 후 같은 시장구조조사와 원본 저장 프레임 대조군을 실행하고, 반환 결과·본문
소속과 실행 시간을 확인한다. 큰 표를 처리할 수 있게 된 후보에서 전체 Rust를
재실행한다. 80168의 93쪽 및 부분 배치 미달 보정, lint·fresh WASM과 제출 전
등록 환경 검토도 완료해야 하며 현재는 PR 준비 완료가 아니다.

## 작업지시자의 하락폭 허용 조건 변경

2026-10-23 사용자 답변에 따라 고정한 `upstream/devel` 대비 페이지별 raw 하락폭을
기존 0.1%p 재검토 기준에서 **2%p까지 허용, 초과 시 재검토**로 변경했다. 상대 2%가
아니며 Native/fresh WASM을 각각 같은 조건·페이지로 비교한다. 관련 활성 매뉴얼
7개를 함께 수정했다. 앞으로의 비교는 Decimal로 `기준 − 후보 > 2`를 판정한다.
기존 실행 로그·TSV·결과 JSON의 과거 판정은 덮어쓰지 않는다. 과거 비교 스크립트를
재사용할 때는 이전 0.1%p 조건을 그대로 실행하지 않고 새 실행 명령과 판정을 기록한다.
전쪽 90% 하한, 완료 페이지의 양쪽 98% 보호, 쪽수·내용 보존·직접 확대 판독 및
정확한 회귀 테스트의 기대값은 변경하지 않는다.

## 성능 보정의 실행 결과

Native126은 `32f534d4d5`와 해당 저장 프레임 스캐너 수정만 포함한 고정 소스로
빌드했고 245.21초, exit 0을 확인했다. source hash 확인과 producer 보존은
`char-overlap-native-build-126-result.json`에 있다. 이후 지침 전용 커밋
`15acb82f84`가 추가됐으며 제품 코드는 같다.

1950 실행은 기존 검사 소스를 변경하지 않고 같은 동결 라이브러리의 Native Skia
조건으로 컴파일했다. 시장구조조사 검사 1건은 21.22초에 통과했다. 이전 1924
기본 feature 실행에서는 1,100초 이상 실행 중에 진단을 거쳐 중단했으므로 정확한
전후 배율이나 기본 feature 전체 통과로 보고하지 않는다. 후속으로 기존 저장 원점·
TAC 소유·도형 호스트·셀 글줄 중첩 프레임 검사까지 총 13건이 통과했다. 새 기대값이나
formal 회귀를 추가한 결과가 아니다. 명령·소스 해시·각 검사 시간과 로그는
`current-previous-failures-1950-result.json`에 기록했다.

이 결과는 저장 프레임 순회 보정의 국소 검증이다. 미달인 80168의 93쪽과 중첩 표
부분 배치, 남은 기존 실패의 해결·등록 환경 대조 및 최종 전체 검증을 대체하지 않는다.

## 80168의 첫 조각 이월 판정 보정 계획

1952의 Native126 원본 실행에서 pi=784는 현재 높이 950.8px, 첫 조각 가용 높이
16.7px인데 `min_content=9.1`로 현재 쪽을 유지했다. 실제 첫 CellUnit과 패딩을
소비한 스캔은 30.5px를 요구했고 그 첫 행을 강제로 수용했다. 다음 쪽에는 `[1]`
이후만 배치돼 독립 Print의 전체 표와 후속 제목이 위로 당겨진다. 기존 93쪽 검사의
독립 기대값을 바꿀 문제가 아니다.

생산은 `cell_units`의 원본 유닛, 측정·예산 소비는 `row_cut_content_height`, 실제
부분 배치는 같은 컷의 `table_partial`이다. 현재 사전 이월만 `MeasuredCell`의
빈 호스트 첫 글줄 5.3px와 패딩을 사용해 다른 최소값을 만들었다. 저장 줄 없는
블록 자식의 재조판 프레임이 확정된 행은 첫 원본 유닛을 선택한 컷의 실제 높이를
사전 이월에도 소비하도록 보정한다. 일반 저장 글줄·걸침 행의 기존 계약은 유지한다.
형상·문서 번호·좌표를 기준으로 하지 않으며, 자식 원장을 물리 테두리 높이로
덮어쓰지 않는다. 92·93쪽 및 자식이 실제로 이어지는 3행 입력을 함께 대조한다.

## 누름틀 SVG 실패의 실제 표현 대조

1954에서 동결 Native126의 같은 `samples/field-01.hwp`를 screen layer와 legacy로
직접 출력했다. 양쪽 모두 안내문 25글자가 `fill="#ff0000"`,
`font-style="oblique 18.418deg"`를 갖는다. 원 검사는 SVG 전체 문자열에 `italic`이
있어야 한다고 검사해 실패했다. 안내문이 실제로 직립으로 바뀌었다는 관측과 구분한다.
변경 출처는 현재 브랜치 `bf4579737f`·`0a766f5413`의 독립 Print 합성 기울임 보정이며
`text_italic.rs`의 공통 CSS 표현을 양쪽 SVG가 소비한다. 관련 Print 행렬 근거는 기존
`task_semanticist21_20261020_jamo_tone_italic.md`에 연결돼 있다.

첫 legacy 진단은 `--profile screen`을 함께 지정해 CLI 계약상 exit 2였다. 명시적
profile을 제거한 두 번째 명령으로 실제 legacy 출력을 확보했다. 실패 명령을 성공으로
덮어쓰지 않았다. 원본 전체 시각 근거와 현재 출력 판독을 확인한 뒤 검사 표현을
판정하며, 여기서는 테스트나 렌더링 코드를 변경하지 않았다. XML 속성과 경로는
`field-guide-svg-environment-1954/results.json`에 있다.

## 첫 조각 보정의 Native127 검증 결과

Native127 빌드는 `7c50f3b0ed` 뒤 해당 공통 컷 조회와 사전 이월 소비 수정본으로
242.53초, exit 0이다. 1955에서 기존 80168 완료 쪽 검사 9건과 앞의 저장 프레임
대조군 13건, 총 22건이 기대값 변경 없이 통과했다. 실패하던 93쪽 독립 기준선
검사도 통과했다. 1956의 실제 Native/Print 비교는 92쪽 99.97251%, 93쪽
99.99223%다. 양쪽 review와 standalone overlay를 직접 열어 92쪽에 잘못 남던
표 머리가 없어지고 93쪽의 완전한 표·후속 제목·본문 순서가 보존됨을 확인했다.
일반 모드에는 silhouette.tsv가 없으므로 1956 결과의 빈 pages 목록을 전쪽 TSV로
보고하지 않는다. 수치는 overlay/overlay_metrics.json의 실제 페이지별 raw다.

1960에서 같은 원본·한컴 2024 Print·compat 2024·96 DPI로 전 157쪽 Native
TSV를 별도로 산출해 최저 **93.96812%**, 90% 미만 0쪽을 확인했다. 93쪽은
이 실행에서도 99.99223%다. 명령은 logs/regulatory-native127-full-1960.log,
TSV는 regulatory-native127-full-1960/regulatory-native127-full/silhouette.tsv,
요약은 regulatory-native127-full-1960-result.json에 있다. 현재 fresh WASM과
고정 base 605ccfd92f의 전쪽 직접 비교는 아직 이 결과에 포함되지 않는다.

1958의 소형/대형 자식·영역 제한 4조건도 각각 96.09434/96.45868/99.87220/
99.88779%로 이전 Native125와 같고 전부 1쪽 Print에 대응한다. 1957의 18개
분할 정책 진단도 각 2쪽이며 Native125와 페이지별 결과가 같다. 4조건만 90%
이상이므로 남은 14조건을 완료로 처리하지 않는다. 자식 원장을 물리 부모 높이로
덮어쓰지 않고도 93쪽 회귀를 해소했으며, 분할 정책·부분 배치 보정은 계속한다.

## 18개 정책 조합의 물리 프레임과 소유 범위 분석

1961은 같은 18개 원본·독립 Print의 해시를 재확인하고 PDF 벡터 선·글자 원점과
Native127의 렌더 트리를 각각 기록했다. PDF는 pt, 렌더 트리는 96 DPI px이므로
단위 환산 후 대조한다. 진단 로그와 원장은 ignored output의
`logs/reflow-fragment-owners-1961.log`·`reflow-fragment-owners-1961.json`이다.

- 제한 false·부모 CELL·자식 CELL: Print의 첫 부모는 첫 쪽에 닫히지만 자식은
  2행/1행으로 이어진다. Native는 부모를 두 쪽 모두 이어 그리며 자식 첫 원점에
  부모 텍스트 패딩과 앞줄 점유를 더했다. 첫 자식의 Print 원점은 부모의 물리 상자
  원점과 같고, 현재 원점은 텍스트 원점을 사용한다. 단순한 x 이동만의 결함이 아니다.
- 제한 true·부모 CELL·자식 CELL: 자식 첫 원점은 이미 대응하지만 둘째 조각의
  시작 바깥여백과 첫 조각의 끝 바깥여백이 빠졌다. 사전 예산과 부모 외곽이 함께
  누락한 공간이므로 paint 테두리만 늘려서는 안 된다.
- 제한 false·부모 NONE·자식 CELL: Print의 부모 원자 정책이 제한 없는 자식의
  2행/1행 소유를 없애지 않는다. 현재 Native는 자식 세 행을 첫 쪽에 두었다.
- 제한 false·자식 NONE: Print는 자식 세 행을 첫 쪽에 두고 후속 본문을 둘째 쪽
  상단에서 시작한다. 현재 부모 컷 예산이 자식을 둘째 쪽으로 이월했다. 자식의
  쪽 정책과 부모의 짧은 물리 프레임을 같은 높이로 판정해서는 안 된다.
- 제한 true·부모 TABLE: Print는 부모 전체를 둘째 쪽으로 이월하고 후속 본문을
  첫 쪽에 남긴다. 현재는 부모를 두 쪽에 나누고 후속 본문도 둘째 쪽에 보낸다.
  같은 내용 순서만으로 지연 부동 개체의 물리 소유를 대신할 수 없다.

추가 수정은 부모 프레임 종료·자식의 미소비 원본 범위·부분 조각의 실제 원점/여백을
분리한 공통 계획에서 수행한다. 제한 없는 자식의 정책을 부모에 일괄 덮어쓰거나
부모를 강제로 원자화하는 방식은 위 반례를 깨뜨린다. 원본 유닛 주소는 보존하고,
사전 요구·스캔의 수용 높이·컷·실제 배치가 같은 조각 소유를 소비하도록 추적한다.
다음 보정의 기대값은 이 독립 Print 관측이며, 현재 수치나 잘못된 테두리에서 역산하지 않는다.

누름틀 원본의 1963 Native 대조는 기존 한컴 2020 Print의 전 3쪽과 쪽수가 같고,
raw는 100.00000/99.99969/100.00000%다. 해당 fresh WASM과 screen 안내문 직접
판독은 별도다. 인쇄 프로필의 누름틀 안내문 생략을 화면 안내문의 기울임 증거로
대신하지 않으며, 아직 원 테스트를 변경하지 않았다.

글자겹침의 과거 058 fixture 생성기를 현재 검사와 diff한 결과, 검사 본체·모든 단언은
같지만 생성기의 `char_shape_ids`가 `[0, 0]`에서 `[0, 0, 0]`으로 확장돼 있다.
따라서 그 생성 입력의 Print/TSV만으로 원 검사 입력의 시각 선행 조건을 충족했다고
보고할 수 없다. 형식상 유효성·누락 모양 fallback과 실제 원본 출력은 각각 확인한다.
이 차이를 근거 없이 원본 검사 제거 또는 새 입력으로의 교체 사유로 사용하지 않는다.

## 완료한 재검증과 구분해야 할 실행

1959의 50입력·145쪽 Native 재검증은 전부 완료했고 최저 93.39366%, 미달 0건이다.
각 페이지 raw를 Native125와 다시 대조한 1968에서도 변경 0건이다. 이 결과는
이전 후보 대조이며 최신 devel 직접 비교나 fresh WASM 완료로 바꾸어 보고하지 않는다.
Native127 제품 소스 627경로가 `17d4346fcd`의 Git 바이트와 같은지는 1962에서
확인했다.

1964의 전체 기본 feature 실행은 4,660개까지 로그가 있으나, 턴 중단 뒤 실행기와
nextest PID가 모두 없고 최종 요약·exit code도 없다. 완료로 인정하지 않는다.
`current-full-rust-1964-process-audit-1972.json`에 프로세스 부재를 확인한 근거를
보존하고 같은 소스·공용 target·잡 8개·스레드 12개로 1972를 실행했다. 실행기는
별도 세션으로 보존했고 실제 PID를 확인하며 종료 요약까지 관측한다.

## 같던 여백·위치를 분리한 독립 한컴 Print 대조

1967에서 부모 여백만 변경한 2개, 자식 여백만 변경한 2개를 만들어 총 4개를
한컴 2020 MCP Print로 출력했다. 원본과 구분한 입력·PDF의 해시와 실제 engine,
한컴 빌드, `pdf_print_method=0`·`hancom2020_pdf_driver_one_up`을 확인했다.
부모/자식의 다른 XML 속성 및 나머지 ZIP entry는 원본과 같음을 검증했다.
초기 생성기는 ZIP entry 메타데이터 객체를 쓰기 과정에서 재사용해 두 번째 사본을
완성하지 못했다. 그 불완전 입력은 별도 failed-generation 폴더로 보존했고,
MCP 호출 전 개수 검사에서 멈췄다. entry 메타데이터를 복사하도록 생성기만 고친
두 번째 시도는 4개 ZIP의 CRC와 의미 속성을 확인한 뒤 실제 Print를 완료했다.
원본 파일을 손상이나 Print 실패로 분류하지 않는다.

1969의 Native 대조도 실행했으며 새 대조군 4개는 모두 미달이다. 구체적인 Print
벡터와 글자 원점은 `fragment-margin-owner-print-vectors-1970.json`에 있다.
추가 1971은 부모의 가로·세로 오프셋만 바꾼 제한 false/true 2개다. 두 Print도
같은 인쇄 계약으로 성공했고 벡터는 `fragment-parent-position-print-vectors-1973.json`에 있다.

핵심 판정은 다음과 같다.

1. **제한 false의 자식은 부모 여백과 위치 오프셋을 바꿔도 움직이지 않는다.** 자기
   바깥여백을 바꿀 때만 움직인다. 앞선 1961의 부모/자식 원점 일치는 같은 여백
   수치에서의 관측이었다. 그것만으로 부모 셀 좌표계 소유를 입증할 수 없다.
   현재 `ReflowBlockOrigin.cell_relative`와 `reflow_block_table_origin`의 부모 셀
   원점 재사용 가정은 이 반례에서 성립하지 않는다. 제한 해제 개체의 host 문단/
   단 좌표계와 자기 여백을 보존하는 계획이 필요하다.
2. 제한 true는 부모를 따르고 내용 원점·자기 여백을 소비한다. 부모 세로 오프셋은
   첫 조각에서만 보이며 다음 쪽에서는 소진된다. 가로 오프셋은 이어받기에도 유지된다.
3. 제한 true에서 여백을 늘리면 실제 분할이 기존 2행/1행에서 1행/2행으로 바뀐다.
   그러므로 시작·끝 여백을 paint에서만 보충하거나 원본 컷을 그대로 두는 방식은
   새 예산에서 올바른 내용 소유를 보존하지 못한다. 예산과 실제 배치가 공통 조각
   계획을 소비해야 한다.

현재 전체 Rust 실행이 끝날 때까지 제품·테스트 소스를 고정한다. 위 자료는 후속
공통 계획 보정의 독립 근거이며 미달을 수용한 완료 보고나 신규 formal fixture가 아니다.

## 전체 Rust 실행 범위 확정과 공통 좌표계 보정 시작

1972도 최종적으로 외부 종료 신호 두 번으로 중단됐다. 정상 단언 결과 10,519건을
보존하고, 신호 중단·미실행 1,035건만 같은 빌드에서 1975의 200건 이하 묶음으로
이어 실행했다. 1974 전체 목록과 합집합을 대조해 **11,554건 모두 종료 결과**를
확보했다: 11,498 통과, 56 단언 실패, 미실행 0건, 기본 ignored 45건 별도다.
단일 무중단 실행으로 보고하지 않는다. 1977은 소스 3,324경로의 바이트 불변과
56개 실행 파일의 기존 mtime·현재 SHA를 확인했다. 전체 범위·실패 TSV·로그는
`current-full-rust-1977-audited.json`·`current-full-rust-1977-failures.tsv`에 있다.

1976의 입력군 한정 진단 모델은 Native의 잘못된 좌표를 쓰지 않고 원본 HU·글자 크기·
저장 선행 줄·각 개체의 쪽 정책과 여백으로 프레임·컷을 계산했다. 24조건 모두
페이지별 행 소유가 독립 Print와 같고 양방향 표 선 대응 최대 오차는 0.332022px다.
종료한 부모 프레임을 다음 쪽에 다시 발행한 초기 모델의 오류도 바로잡았다.
이는 제품·formal 회귀의 통과 근거가 아니며 제품에 옮길 계획의 독립 대조다.

후속 보정은 root 호스트 문단의 좌표계를 전체/부분 표 배치의 진입점에서 보존하고,
제한 없는 재조판 자식이 그 좌표계와 자기 여백을 소비하게 한다. 현재 `cell_relative`
가 부모 셀 원점·패딩·오프셋을 물려주는 가정을 제거한다. 제한 있는 자식은 실제
선택된 부분 문단의 흐름 위치를 유지하며 조각마다 자기 시작 여백을 연다. 미소비
자식의 컷·부모 물리 프레임·소스 예약 높이는 계속 분리한다. 이 좌표계 단계만으로
남은 물리 높이·분할 정책 문제를 완료로 판정하지 않는다.

## 호스트 좌표계 단계의 Native129 검증

Native128에서 부분 표의 자식 원점은 개선됐으나, 전체 표 진입점에 이미 위 여백을
포함한 `y_start`를 호스트 문단 원점으로 저장해 위 여백을 중복 소비했다. Native129는
`layout_table_control_block`이 가진 원래 `para_y_for_table`과 단의 문단 시작점을 먼저
보존한다. 재귀 전체/부분 배치는 활성 범위를 덮어쓰지 않으며 제한 없는 자식이
그 원점과 자기 여백을 소비한다. 제한 있는 자식은 선택된 부분 흐름의 원점과
자기 시작 여백을 사용한다. 부모 셀의 clip과 자식의 물리 용지 viewport를 구분한다.

동결 Native129는 컴파일 잡 8개로 248.68초에 빌드 종료 코드 0을 확인했다.
`char-overlap-native-build-129-result.json`이 입력 소스·라이브러리·실행 파일 해시와
명령을 보존한다. 다음 결과는 이 동결 제품에 대한 단계 검증이며 최종 head 검증이나
fresh WASM 검증을 뜻하지 않는다.

| 범위 | 실행 결과·증거 | 판단 |
| --- | --- | --- |
| 기존 저장 프레임·80168 완료 페이지 계약 | `current-previous-failures-1988-result.json`, 22 통과·실패 0·검사 누락 0 | 기존 독립 기대값을 변경하지 않았다. |
| 원래 자식 높이·제한 속성 4조건 | `two-tac-child-height-native129-1991/results.json`, 각각 96.09434·96.45868·99.87220·99.88779%, 한컴 Print와 각각 1쪽 일치 | Native128의 전체 표 원점 후퇴는 복구했다. Native만의 결과다. |
| 부모·자식 분할 정책 18조건 | `reflow-policy-native129-1989/results.json`, 36쪽, 4조건 통과·14조건 미달 | 자식 원점 보정만으로 부모 외곽·예약·정책 소유를 해결하지 못했다. |
| 서로 다른 부모·자식 여백 4조건 | `fragment-margin-owner-native129-1990/results.json`, 모두 2쪽 일치·raw 미달 | 예산·실제 조각의 공통 계획 보정이 필요하다. |
| 부모 위치 변경 2조건 | `fragment-parent-position-native129-1990/results.json`, 모두 raw 미달 | 부모 위치와 제한 없는 자식 위치의 독립성은 높이·컷 정책과 함께 검증해야 한다. |
| 80168 92·93쪽 직접 출력 | `regulatory-page93-native-comparison-1992/results.json`, 실행 종료 코드 0 | 일반 review 모드의 빈 `pages` 배열을 전쪽 TSV 통과 증거로 사용하지 않는다. |

`host-origin-native127-vs129-1995.json`은 직전 후보 Native127과 Native129를
십진수로 페이지별 대조했다. 이는 `upstream/devel` 비교가 아니다. 기존 통과 4조건은
유지됐다. 제한 false·부모 CELL·자식 CELL의 최저값은 34.32542%에서 80.62233%로,
제한 true의 같은 조합은 73.38141%에서 84.22101%로 개선됐지만 여전히 미달이다.
부모 TABLE·제한 true·자식 CELL/TABLE의 2쪽은 각각 28.65269%에서 23.52518%로
5.12751%p 하락했다. 이 후퇴를 2%p 허용폭이나 다른 조건의 개선으로 면제하지 않는다.

`reflow-host-origin-review-1993/`의 1쪽 review와 standalone overlay를 함께 열어
자식 표 원점뿐 아니라 남은 부모 외곽의 파란 선과 이모지 모양 차이를 확인했다.
raw는 약 80.62%이며 시각 완료가 아니다. 구조 차이·글꼴 차이를 숨기지 않는다.

빌드 이후 `layout.rs`의 한 호출을 rustfmt 형식으로 정리했다. 최초 fmt 검사는 그
한 호출에서 실패했고 정리 후 재검사는 종료 코드 0이었다. 동결 빌드 소스 사본과
현재 파일의 rustfmt 정규화 본문이 동일함을 `native129-format-only-1995.json`에
확인했다. 초기 단순 공백 비교는 끝 쉼표 차이를 거부했고, 다음 비교는 도구가 붙인
파일명 헤더 차이를 거부했다. 실제 헤더를 각각 검증한 뒤 본문을 비교했다.
이 형식 동등성은 최종 커밋의 필수 검증을 대신하지 않는다.

## 합친 문단의 TAC 뒤 글자 축 진단

`inline-axis-probe-1987/`은 기존 검사 소스에서 만든 진단용 실행이며 formal 검사를
변경하지 않았다. Plain·Field·Bookmark 각각 편집 직후와 HWP/HWPX 재열기 9상태의
문단 텍스트, 공개 text/logical 변환 결과와 실제 렌더 런을 기록했다. 모델의 텍스트는
`첫문단가나다라`지만 마지막 `라` 런의 `char_start`는 7이고, 공개 변환의 text index 6은
여전히 logical index 6이다. 따라서 단순히 테스트 helper에 공개 변환을 호출하는 것만으로
해결된다고 결론 내릴 수 없다. 원래 원인·축 계약·독립 출력의 대조를 이어간다.
진단 실행 성공을 올바른 화면 배치나 신규 회귀의 시각 선행 조건 충족으로 보고하지 않는다.

## 제한된 자식 조각의 시작·종료 여백 보정 계획

1997은 동결 Native129의 소스 유닛·선택 컷을 6조건에서 실제 실행했다. 자식 3행은
55.533333·51.760000·55.533333px 유닛으로 투영된다. 실제 행 본체는 각각 51.76px이며
위·아래 여백은 전체 자식의 첫·마지막 유닛에만 들어 있다. 그러므로 첫 두 행의 컷과
마지막 한 행의 이어받기에 유닛 합만 사용하면 각 조각의 닫기/열기 여백이 빠진다.
선행 저장 줄의 21.333333px 전진과 자식이 이월된 호스트 빈 줄도 구분해야 한다.

선택된 `nested_row`·재귀 `NestedTableCut`에서 자식의 실제 조각 높이를 질의하고,
동일한 호스트 줄 높이와 자기 양쪽 여백으로 제한된 자식의 내용 프레임을 계산한다.
이 결과를 행의 수용/점유 높이와 셀의 가시 높이가 함께 소비하도록 한다. 유닛 주소와
저장 컷을 바꾸지 않는다. 제한 없는 자식은 부모 외부 프레임과 독립된 내용 스트림을
가지므로 이 포함 관계를 적용하지 않으며, 그 예약·정책 소유 보정은 계속 필요하다.
18정책과 서로 다른 여백/위치 대조군, 기존 완료 페이지 계약을 같은 후보로 확인한다.

## 공통 컷 높이의 Native130 결과와 소비 경로 추가 보정

Native130은 잡 8개·공용 target으로 256.37초에 빌드 종료 코드 0을 확인했다.
첫 실행기는 진행 중이던 fmt를 감지해 컴파일 전에 거절됐다. fmt 호출 형식을 고친
재검사가 끝난 뒤 실제 빌드를 순차 실행했다. 실패한 fmt와 실행기 기록을 보존한다.

- `current-previous-failures-1999-result.json`: 기존 계약 22 통과·실패 0·누락 0.
- `reflow-policy-native130-2000/results.json`: 18조건·36쪽 중 6조건 통과,
  12조건 미달. 제한 true·부모 CELL·자식 CELL/TABLE은 84.22101%에서
  94.63329%로 개선됐다. 정책 전체가 완료됐다는 뜻은 아니다.
- `fragment-margin-owner-native130-2001/results.json`: 제한 true의 자식 여백은
  52.83858% → 93.93876%, 부모 여백은 84.92662% → 94.52942%다.
  제한 false의 두 조건은 81.43136/52.14760%로 미달이다.
- `fragment-parent-position-native130-2002/results.json`: 제한 true는
  52.49219% → 94.71542%, false는 81.26038%로 미달이다.
- `two-tac-child-height-native130-2003/results.json`: 원래 4조건은 모두 90% 이상이며
  Native129의 점수와 쪽수를 유지했다.

`restricted-child-margin-review-2006/`의 2쪽 review와 standalone overlay를 함께
직접 읽었다. 93.94% 통과 이후에도 부모 외곽을 자식 폭에 맞춰 늘린 파란 선과
이모지 모양·색 차이가 보인다. 수치만으로 완료하지 않는다. source/Print의 이모지
차이와 프레임 차이를 분리해 남긴다.

생산·소비 경로는 `reflow_cut_content_height` → `row_cut_height`의 Capacity/Occupied와
`cell_cut_visible_height` → 부분 배치의 행 높이다. 그 뒤의 별도 소비도 대조했다.
`extend_clipped_cell_horizontal_clip_to_nested_table_borders`가 부모 칸 폭을 바꾸고,
`fold_last_row`가 선행 저장 줄간격 8px를 다시 접으며, 정렬은 별도 내용 높이를 선택한다.
현재 후속 후보는 같은 컷의 명시적 프레임을 정렬·셀 clip·꼬리 접기에 전달한다.
부모 괘선 폭과 자식 viewport를 분리하고, 컷을 받은 자식을 부모 clip으로 다시 자르지 않는다.
여러 행에 걸친 칸은 기존 걸침 조각 원장이 소유하므로 단일 행의 인증을 그대로 적용하지 않는다.
Native131의 실제 실행 결과는 별도로 확인하며 Native130을 새 후보 검증으로 재인용하지 않는다.

## TAC 뒤 글자 실패 원 PR과 현재 독립 Print

기존 통합 기록에서 원 PR #7616을 찾고 GitHub 본문·커밋 목록을 다시 확보했다
(`inline-axis-pr7616-2016.json`). 원 등록 46a8bd7b9d가 인용한 테스트 전용 커밋은
7a58f1d7c7이며 원 PR은 줄 경계·누름틀/책갈피 갭과 뒤 글자 소유의 결함을 설명한다.
원 PR은 한컴 출력·fresh WASM 미실행을 명시했고, 이후 통합 보정의 Print 100% 근거는
기존 `semanticist21_20261008_integrated_review.md`에 있다. 원 커밋의 자동 PR 연결
조회는 빈 배열이었으며 이를 원 PR 부재로 판정하지 않고 기존 통합 기록으로 추적했다.

현재 테스트 helper 그대로 만든 HWP/HWPX 6개의 한컴 2020 MCP Print를 완료했다.
HWP에는 템플릿의 2010, HWPX에는 2020 저장 메타데이터가 있으며 실제 생성은
Native129의 공개 API와 기존 helper다. 메타데이터를 한컴 저작·저장 실행 증거로
승격하지 않는다. 각 요청의 실제 engine 2020·PrintMethod 0·1-up 출력 모드·입력/PDF
해시는 `mcp-inline-axis-2005/`에 보존했다. 실패 0건이다.

`inline-axis-native130-2007/results.json`에서 6개 모두 Print와 1쪽 일치·raw 100%다.
렌더 런은 논리 축으로 표 슬롯을 포함하지만 기존 `char_box`는 텍스트 글자 인덱스와
직접 비교한다. `text_to_logical_offset`은 같은 위치의 개체 앞 삽입 경계를 반환하므로
그 함수 호출만 넣는 수정은 부적절하다. 실제 런의 논리 시작을 텍스트 축으로 역변환해
소유 글자를 찾는 방향으로 진단한다. formal 검사 변경은 같은 원본의 fresh WASM과
직접 판독까지 확인한 뒤 수행하며, 기존 내용·줄 순서 관계와 실패 검출은 유지한다.

## Native131의 실제 배치 소비 결과와 남은 차이

공용 `target/pr-review`·잡 8개로 246.04초에 빌드 종료 코드 0을 확인했다.
산출물 SHA-256은 `ce7943ec066da88c1f7ddf159ff793e0709099713b8b9d10234020e54e7eaac0`이며
`char-overlap-native-build-131-result.json`의 소스 628개 해시와 현재 바이트를 대조했다.
`current-previous-failures-2010-result.json`의 기존 계약 22개는 모두 통과했다.

- `reflow-policy-native131-2011/results.json`: 18조건 중 7조건 통과·11조건 미달이다.
  제한 true·부모 CELL·자식 NONE은 40.79432% → 93.37929%로 개선됐다.
  자식 CELL/TABLE은 93.95085%이며, 부모 NONE의 세 조건은 93.39512%다.
  Native130 대비 각각 0.68244%p와 1.71550%p 하락했다. 후자는 내용 경계의
  좌표 이동 없이 기존 clip 밖 잉크가 드러난 차이를 포함하므로 수치만으로 완료하지 않는다.
- 여백 4조건과 위치 2조건은 Native130의 점수와 쪽수를 유지했다
  (`fragment-margin-owner-native131-2012/`, `fragment-parent-position-native131-2013/`).
- 기존 자식 높이 4조건은 모두 90% 이상이다. 제한 false의 큰/작은 글자는
  96.09434/99.87220%, 제한 true는 95.15682/99.03650%다
  (`two-tac-child-height-native131-2014/results.json`). true의 값은 이전보다 낮아
  clip 변경 영역을 직접 판독해야 하며 기존 결과를 그대로 재인용하지 않는다.
- TAC 뒤 글자 6개 저장본은 모두 Native raw 100%·독립 Print 1쪽과 일치했다
  (`inline-axis-native131-2015/results.json`). 원 검사는 변경하지 않았다.
  `inline-axis-helper-diagnostic-2019/`의 별도 진단에서 원 helper 3개는 실패하고
  실제 런의 논리 시작을 텍스트 축으로 변환한 helper 3개는 통과했다. 원본 내용과
  LineSeg·앞뒤 순서 검사는 유지했다. fresh WASM 확인 전 정식 검사 변경은 보류한다.

`restricted-child-margin-review-2018/`의 review와 standalone overlay를 직접 읽었다.
부모 오른쪽 괘선의 폭 차이는 여전히 남아 있다. 앞 절에서 clip 경로를 소비 지점으로
찾았다는 사실이 폭 문제의 해결을 뜻하지 않는다. 선언 폭·행 격자·부분 배치의 최종
폭 선택을 계속 대조한다. 제한 false의 독립 스트림과 부모 TABLE 정책도 미해결이다.
fmt와 diff 공백 검사는 통과했으며, 현재 단계는 로컬 WIP 고정이다. PR 완료 판정이 아니다.

## 함초롬바탕 지정 문자와 실제 흑백 대체 글꼴

사용자가 한컴 GUI의 지정 글꼴을 함초롬바탕으로 확인했다. 원본 HWPX도 같은 글꼴이며,
U+1F9A6 문자는 시스템의 해당 프로그램 cmap에 없다. 독립 Print PDF의 이 문자 네 런은
`SegoeUIEmoji`·색 0의 흑백 출력이다. 지정 글꼴과 실제 fallback을 구분한다
(`emoji-font-source-observations-2020.json`). 기존 RHWP는 로컬 Noto Color Emoji를
사용해 다른 모양과 색을 보였으며, 이를 올바른 출력이나 해결 불가능한 차이로 판정하지 않는다.

Windows 검증 호스트의 원본 `C:\Windows\Fonts\seguiemj.ttf`를 ignored 진단 경로로
읽기 전용 복사했다. 버전 1.29·SHA-256
`7c0244dd8eeb7c6bdecdfc3f9e59833527fc18a66d0295ce47339069692a2b4f`이며,
해당 문자에는 7개 윤곽과 별도 COLRv0 색상 레이어가 모두 존재한다
(`emoji-font-probe-2021/seguiemj.json`). 시스템 등록이나 저장소 글꼴 추가는 하지 않았다.

같은 Native131에 이 프로그램만 추가 공급한 `restricted-child-margin-segoe-2022/`의
새 review·overlay를 함께 판독했다. 2쪽 raw는 93.93876% → 97.41572%로 높아졌고
모양은 가까워졌지만, RHWP의 색상 레이어가 여전히 표시된다. 실제 `overlay_metrics.json` 값을 우선하며
반올림한 PNG 점수와 구분한다. 글꼴 공급과 일반 텍스트의 흑백 윤곽 선택은 별개 원인이다.
해당 출력은 진단 결과이며 수정 완료 증거가 아니다. 일반 텍스트 fallback과 명시적인
컬러 글리프 payload 계약을 구분해 공통 경로의 해결 방법을 검토한다.

## 일반 텍스트의 COLR 기본 윤곽 선택 보정 계획

Native131의 실제 렌더러 snapshot을 쓴 `emoji-font-probe-2021/findings.json`에서
문서 face는 함초롬바탕, 로컬 paint 선택은 Noto Color Emoji, Windows 프로그램 공급 시
선택은 Segoe UI Emoji로 확인했다. 독립 PDF의 부분 글꼴 glyph 3은 Windows 원본
U+1F9A6의 기본 윤곽과 98개 명령의 좌표까지 동일하다
(`emoji-font-probe-2021/pdf-outline-comparison.json`).

`emoji-presentation-controls-2026/plan.json`은 원본 복사와 face/VS16만 바꾼 수동 대조군의
생성 차이를 기록한다. 2020의 명시 face/VS16, 2024의 원본과 명시 face/VS16 Print
3건 모두 같은 흑백 기본 윤곽을 출력했다. 실제 engine·PrintMethod 0·1-up 경로를
`mcp-emoji-presentation-2026/`에서 확인했다. 2024 교차 버전 출력은 동작 대조 자료이며
2020 원본의 정식 기준 PDF를 대체하지 않는다. OpenType COLR 사양도 기본 glyph에
윤곽이 있으면 비컬러 glyph로 그리는 경로를 정의한다
([공식 COLR 사양](https://learn.microsoft.com/en-us/typography/opentype/spec/colr)).

공통 Rust 처리에서 COLR 기본 glyph 모두에 유효한 기본 윤곽이 있는 프로그램만
일반 텍스트용 SFNT 사본으로 만든다. cmap·glyph ID·윤곽·advance·GSUB/GPOS·이름은
유지하고, 컬러 레이어 선택 표와 사본에서 무효가 된 서명만 제외한다. 원본 파일과
명시적인 GlyphRun/GlyphOutline 컬러 자원은 변경하지 않는다. 윤곽 없는 컬러 전용
프로그램은 이 변환을 적용하지 않는다. 특정 문서·문자·face 이름으로 분기하지 않는다.
Native의 nominal·shaping font 생성과 브라우저 일반 FontFace 공급이 같은 함수를
소비하게 하고, strict 자원 replay는 원 프로그램을 계속 사용한다. 공급 글꼴의
원본 해시와 paint 사본 해시를 구분한다. 같은 source/PDF의 Native/fresh WASM과
명시 컬러 자원 대조 검사가 끝나기 전 수정 완료로 판정하지 않는다.

## 일반 텍스트 보정의 Native132 결과

공용 target·잡 8개로 243.53초에 Native132 빌드 종료 코드 0을 확인했다.
`text_font_presentation::ordinary_text_font_program`이 COLR 기본 glyph 전체의 윤곽을
확인하고 SFNT 사본을 만든다. Native의 `fonts::finish`가 이 사본을 캐시해 nominal과
SkShaper 양쪽에 공급한다. 일반 Canvas FontFace와 CanvasKit의 host font 준비는
문서의 같은 Rust API를 소비하며, SVG 임베드도 같은 함수를 사용한다. strict
GlyphRun/GlyphOutline 자원의 원본 프로그램은 이 준비 경로에 넣지 않는다.

Windows 원본 Segoe UI Emoji를 `/usr/local/share/fonts/rhwp-hancom-fallback/seguiemj.ttf`에
등록했다. 원본 SHA-256은 이전 진단과 같으며 실제 `fc-match`가 Noto Color Emoji에서
Segoe UI Emoji로 바뀌었음을 `emoji-system-font-registration-2033.json`에 기록했다.
글꼴 파일은 Git에 추가하지 않았다. 이후 base/후보 비교에는 같은 새 공급 환경을 사용한다.
과거 Noto 환경의 숫자를 새 환경의 무회귀 증거로 재사용하지 않는다.

같은 새 시스템 글꼴·입력·2020 Print·96DPI에서 다음과 같이 직접 대조했다.

| 출력 | 1쪽 raw | 2쪽 raw |
| --- | ---: | ---: |
| Native131, 색 레이어 선택 | 97.91111% | 97.41572% |
| Native132, 기본 윤곽 선택 | 98.82051% | 98.57706% |

증적은 `restricted-child-margin-native131-2034/`와
`restricted-child-margin-native132-2034/`다. 새 2쪽 review·standalone overlay를 함께
확인했으며 해당 문자가 한컴과 같은 흑백 윤곽으로 바뀌었다. 부모 오른쪽 괘선의 폭 차이는
여전히 보이고, 이 페이지를 완료된 98% px 고정 대상으로 승격하지 않는다.

`ordinary-text-font-contract-2035/`는 실제 원본 프로그램에서 12,189개 glyph의 전진폭과
경계 상자, glyph 수·em·cmap·그 외 표의 바이트 보존을 확인했다. 2,786개 컬러 기본 glyph가
일반 사본에서만 비컬러가 되며, 원본에 대한 명시 COLRv0 디코드는 8개 컬러 레이어와 strict
계약을 유지했다. 전체 checksum·재적용의 비적용·잘못된 face/잘린 입력 거절도 통과했다.
비컬러 함초롬바탕과 윤곽 없는 Noto Color Emoji는 사본 변환을 적용하지 않는다.
이는 실행 진단이며 정식 렌더링 회귀나 fresh WASM 완료를 대신하지 않는다.

TypeScript 검사와 기존 글꼴 계약 26개·Sweep 브라우저 계약 2개는 통과했다.
`ordinary-text-font.test.ts`는 원본 바이트 보존·WASM 반환 view와 수명 분리·비적용·무효 face
선택을 검사하며, 기존 계약과 함께 29개 통과했다. 로그는 각각
`ordinary-text-font-tsc-2029.log`, `ordinary-text-font-ts-contracts-2036.log`,
`ordinary-text-font-sweep-browser-2031.log`다. 새 Rust 처리의 직접 실행과 브라우저 연결
계약을 구분하며, 최적화 fresh WASM의 실제 출력 확인 후 최종 판정한다.

앞 단계 Native131과 같은 소스의 WASM045는 864.77초에 빌드·Studio 동기화를 완료했다.
`inline-axis-wasm045-2025/results.json`의 저장본 6개는 모두 raw 100%·전체 쪽수 1 대 1·
누락 0쪽이다. 첫 결과 집계기가 잘못된 manifest 파일명을 읽어 gate를 null로 둔 문제를
바로잡았고, 실제 `silhouette_manifest.json`의 전체 쪽수와 미달/누락 목록을 다시 확인했다.
이 증거는 이전 소스의 TAC 뒤 글자 대조이며 Native132의 fresh 검증으로 재인용하지 않는다.

## WASM046과 Native 50개 입력 재검증

WASM046은 같은 소스에서 849.52초에 빌드 종료 코드 0과 Studio JS/WASM 동기화 해시 일치를
확인했다. 원본 글꼴과 실제 일반 FontFace 사본의 해시는 `textProgramSha256`으로 구분한다.
`restricted-child-margin-wasm046-2043/`의 1/2쪽 raw는 98.81747/98.57541%이며 전체 쪽수는
Print와 2 대 2다. 새 2쪽 review·standalone overlay에서 흑백 윤곽을 확인했다. Native의
98.82051/98.57706%와 소수 차이는 있지만 같은 개선을 확인했으며, 부모 경계 밖 자식 괘선은
양쪽 모두 아직 남아 있다. 이 자료만으로 페이지 완료나 px 회귀 자격을 선언하지 않는다.

`native132-fifty-inputs-2039-audited.json`에서 기존 50개 입력의 같은 범위 145쪽이 모두
90% 이상이며 최저값은 93.39366%다. 입력과 독립 Print의 SHA도 이전 고정본과 일치했다.
원 실행 집계기는 첫 key의 중복 하이픈 정규화를 반영하지 못해 TSV 1개를 찾지 못했다.
원 로그를 보존하고 기존 정규 key의 실제 TSV/manifest로 재출력 없이 감사해 해결했다.
렌더링 실패나 원본 변경으로 분류하지 않는다. 이 50개 범위의 fresh 전체와 새 base 대비
재검증은 별도로 필요하다. 과거 Native127/Noto 환경과의 비교는 devel 무회귀 증거가 아니다.

## 부모 셀 클립의 독립 PDF 근거

`reflow-cell-clip-ownership-2042.json`은 PDF 벡터 선과 실제 클립을 96DPI 좌표로 보존한다.
제한 true에서 자식의 오른쪽 선 자체는 약 696.1px지만 부모 클립 우단은 약 676.3px다.
즉 자식 선언 폭을 줄인 출력이 아니라 부모 셀에서 잘린 출력이다. 제한 false는 자식
자신의 전체 클립을 사용하고 부모 셀 프레임 밖으로 이어진다. 현재 트리에서도 제한 없는
자식은 부모 Cell의 자손이 아닌 Table의 직접 자식으로 배치되므로, 부모 Cell 클립을
다시 켜더라도 이 별도 자식의 클립을 부모 것으로 바꿀 필요는 없다. 다음 보정은 이
소유 관계를 유지하며 닫힌 부모 프레임의 클립과 별도 괘선 확장 게이트를 대조한다.

현재 원격 devel의 읽기 전용 조회는 def947c2149df5bc812d801e6e79be19a403b4d1로,
고정 기준 605ccfd92f와 다르다. 현재 fresh 단계 결과와 생성 JS를 먼저 커밋한 뒤 새 base의
변경·지침을 읽고 동기화한다. 리베이스 자동 적용을 기능 보존 증거로 대신하지 않는다.

## def947c214 동기화와 다음 보정

새 base는 def947c2149df5bc812d801e6e79be19a403b4d1, 리베이스 직후 후보는
c520638aea2f849319c346d4f73b3a27bd0354c8이다. backup 브랜치를 보존한 뒤 197개 커밋을
재적용했다. 진행 중 상태 조회의 인덱스 잠금 경합으로 54번째 적용이 중단됐다. 잠금·활성 Git
프로세스가 자연 종료됐음을 확인하고, staging patch-id가 원본 139811191c와 같은지 검증한
뒤 원 저자·메시지로 해당 커밋을 고정하고 같은 리베이스를 계속했다. 소스 되돌리기·reset·
커밋 누락으로 해결하지 않았다. range-diff의 196개는 같고 2%p 지침 1개의 차이는 새 문서
문맥 앵커 이동뿐이다. 결과 트리의 차이는 새 base의 39개 경로이며 실행 소스 634개는 모두
Native132 snapshot과 같다. 정확한 기록은 `rebase-def947-2045-result.json`과
`rebase-def947-2045-lock-recovery.json`에 있다.

들어온 #7474·#7488·#8171·#8173의 본문·파일·커밋을 `rebase-incoming-pr-*-2047.json`으로
확인했다. 제품 Rust 조판 코드는 바뀌지 않았다. #7488은 명시 opt-in만 cdylib 경로로 바꾸고
기본은 standard release다. #7474는 패키징·브라우저가 같은 standard release 바이트를
소비했는지 확인한다. 새 wrapper/release/workflow Python 계약과 Studio artifact·글꼴 계약,
CI impact 검사를 실행해 성공했다. 실제 최종 standard build의 manifest·브라우저 소비·배포
전후 바이트 일치는 다음 새 후보의 최종 패키지에서 확인해야 하며, 이전 receipt를 새
source SHA의 release manifest로 고쳐 쓰지 않는다. #8171·#8173의 새 절차도 읽었고,
`own-open-pr-preflight-2046.json`에서 본인 열린 PR 0개를 확인했다.

다음 소스 묶음은 독립 PDF 클립 근거에 따라 부모 Cell 클립을 복원한다. 제한 없는 자식은
이미 Cell 밖 Table 자식으로 배치되므로 별도 스트림 소유는 유지한다. 닫힌 reflow 프레임의
괘선 확장 금지는 유지하며 자식 선언 폭·좌표를 줄여 맞추지 않는다. 글꼴 프로그램의 정렬·
중복 검사는 COLR/CPAL/DSIG를 제외하기 전 원본 디렉터리 전체에 적용하도록 보완한다.
두 보정 뒤 같은 입력·Print의 Native와 기존 대조군을 다시 확인한다. 독립 자식 스트림의
부모 물리 프레임 반복과 TABLE 원자 행 이월 문제는 여전히 해결해야 한다.

## Native133의 클립 복원과 디렉터리 검증 결과

새 base 위에서 공용 target·잡 8개로 246.71초에 Native133 빌드 종료 코드 0을 확인했다.
부모 셀 클립은 유지하고 닫힌 reflow 프레임의 괘선 확장 금지도 유지했다. 제한 없는 자식은
기존 Table 직접 자식 소유를 유지하며 셀 경계로 옮기지 않았다. 선언 폭·좌표를 줄이지 않았다.

`restricted-child-margin-native133-2055/`의 전체 2쪽은 Native raw가 각각 100%이며
`native-export.json`의 `pageCount: 2`·`all_pages_exported: true`를 확인했다. 2쪽 review와
standalone overlay를 함께 읽어 부모 경계 밖의 파란 선이 사라진 것을 확인했다. RGB 엄격
일치는 35.99%이므로 이를 전체 픽셀 동일이나 모든 렌더링 정확도 100%로 보고하지 않는다.
fresh WASM은 이 새 후보로 다시 산출해야 한다.

`current-previous-failures-2056-result.json`의 기존 완료 페이지·저장 프레임 계약 22개가
모두 통과했다. `current-previous-failures-2060-result.json`의 strict Native glyph replay
10개도 모두 통과했다. 후자에는 실제 COLRv0 레이어 순서·색 출력, 비트맵·SVG·가변/collection
face와 outline 효과가 포함된다. 일반 텍스트 기본 윤곽 처리를 명시 컬러 자원 재생에 적용하지
않는 계약을 실제 실행으로 확인했다.

`font-directory-duplicate-probe-2058/`은 같은 실제 프로그램의 COLR 디렉터리 항목만 중복한
잘못된 입력을 만들었다. ttf-parser의 face/컬러 glyph 조회가 가능한 이 입력을 이전 Native132
함수는 사본으로 승격했고, Native133은 거절했다. 입력의 glyph·표 payload는 바꾸지 않았다.
이 자료는 진단이며 독립 Print 또는 정식 렌더링 회귀의 증거로 승격하지 않는다.

`reflow-policy-native133-2057/results.json`의 18조건 중 7조건은 모두 전쪽 raw 100%다.
나머지 11조건은 여전히 90% 미만이다. 제한 없는 자식의 독립 스트림과 부모 물리 프레임,
TABLE 원자 행 이월을 해결해야 한다. 지적한 제한 true·자식 여백 사례의 개선으로 이 미달을
상쇄하지 않으며 PR 준비 완료나 전체 회귀 통과를 선언하지 않는다.

다음 fresh 빌드는 새 devel의 #7474 절차대로 `measure_render_diff_wasm.py`의 standard
release 경로와 실제 wasm-opt 관측 manifest를 사용한다. pkg·Studio·보존 package의 바이트
일치를 확인하며, 최종 브라우저 소비까지 같은 package를 사용해야 한다. 새 source SHA를
이전 build receipt에 대입해 release 검증을 우회하지 않는다.

## WASM047의 실제 소비와 흑백 문자 재검증

사용자가 한컴 UI에서 확인한 지정 글꼴을 원본 HWPX와 다시 대조했다. 원본의 이름은
`함초롬바탕`이다. 이 환경의 같은 글꼴 프로그램에는 U+1F9A6 glyph가 없으며, 독립 Print
PDF에 포함된 `SegoeUIEmoji`의 해당 윤곽은 Windows 원본 글꼴의 U+1F9A6 윤곽과 같다.
지정 글꼴과 실제 출력에 사용한 대체 글꼴을 구분한다. 지정명을 바꾸지 않고 일반 텍스트의
대체 글꼴을 흑백 기본 윤곽으로 준비하는 공통 처리를 Native와 fresh WASM에서 확인했다.

WASM047은 source SHA `bbd1bfab9af431b44e5ed85d1ff302438cf7a506`의 깨끗한 tracked
소스에서 standard release·잡 8개로 840.78초에 종료 코드 0을 확인했다. 실제 wasm-opt 실행,
`pkg`·Studio·보존 package의 JS/WASM 해시 일치를 확인했다. 같은 앱 초기화에서 받은 WASM
응답을 CDP로 관측한 해시는 `147d3556d226a353184389787a3179a5cb4ad9525ac30c031849f59763f80bba`다.
`verify_release_wasm.py --require-consumption`도 통과했다. 재요청한 파일이나 추정 해시로
브라우저 소비를 대신하지 않았다. 근거는 `release-standard-wasm047-2063/build.json`,
그 아래 `consumption/`, `logs/wasm047-release-consumption-2070.log`다.

`ordinary-text-browser-consumption-2069-result.json`에서 실제 CanvasKit 앱과 WASM bridge를
통한 일반 글꼴 준비를 확인했다. 입력 프로그램 해시는 `7c0244dd…a2b4f`, 일반 텍스트용 사본은
`4206bc9c…961d8`이며 원본 바이트는 보존된다. 이 관측은 명시 컬러 자원 재생의 계약과 구분한다.

같은 원본과 2020 Print의 전체 2쪽을 Native133과 WASM047에서 다시 비교했다. 양쪽 모두
raw 실루엣 100/100%, 쪽수 2 대 2, 누락 0쪽이다. TSV는
`restricted-child-margin-native133-scores-2067/silhouette.tsv`와
`restricted-child-margin-wasm047-scores-2071/silhouette.tsv`에 있다. fresh의 review·standalone
overlay 두 쪽을 직접 읽어 흑백 문자와 부모 셀의 클립을 확인했다. 2쪽 엄격 내용 픽셀은
38.29%이므로 raw 100%를 전체 색·픽셀 동일로 보고하지 않는다. fresh 원본 이미지·전체 쪽수
메타데이터는 `restricted-child-margin-wasm047-2066/`에 보존했다.

기존 TAC 뒤 글자 helper의 저장본 6개도 Native133에서 전체 1쪽 대 1쪽, 각 raw 100%를
확인했다(`inline-axis-native133-2068/results.json`). 이 여섯 입력의 현재 fresh 검증과 정식
helper 보정은 이어서 진행해야 한다. 아직 남은 11개 재조판 정책 미달과 전체 Rust 실패의
해소를 이 결과로 대신하지 않으며, PR 준비 전체 완료로 판정하지 않는다.

## TAC 뒤 글자 회귀 helper의 위치 축 보정

원 해결 PR #7616의 공개 API 재현·원 테스트와 #7882 통합 근거를 다시 대조했다. 기존
`char_box` helper는 `RenderRun.char_start`의 논리 위치를 본문 글자 번호와 바로 비교했다.
표·누름틀·책갈피 슬롯 뒤에서는 두 축이 달라지므로 문서의 `logical_to_text_offset`으로
변환한 뒤 해당 글자를 찾도록 보정했다. 원래 문단 내용·저장 LineSeg 시작점·앞 글자 같은 줄·
표와 뒤 글자의 순서 관계는 그대로 유지했다. 절대 좌표 핀이나 새 수치 기대값을 넣지 않았다.

같은 원본 HWP/HWPX 저장본 6개와 독립 2020 Print는 Native133·WASM047에서 각각 전쪽
raw 100%, 전체 쪽수 1 대 1, 누락 0쪽이다. 증거는 `inline-axis-native133-2068/results.json`과
`inline-axis-wasm047-2073/results.json`이다. `inline-axis-direct-review-2076/`에서 동일 원본
raster의 6개 확대 비교와 대표 review·standalone overlay를 생성해 내용과 표 앞뒤 순서를
직접 읽었다. 표시용 crop은 별도 manifest에 기록했고 원본 raster와 TSV는 유지했다.

`inline-axis-formal-2077/`은 이전 정식 파일의 사본과 보정한 정식 파일을 같은 Native133
library에 연결해 실행했다. 수정 전 3 FAIL, 수정 후 3 PASS이며 실패를 숨기는 필터·expectation
완화는 없다. 로그는 `logs/inline-axis-formal-2077-{before,after}.log`다. 정식 파일과 기존
자동 suite 경로를 유지했으며, helper 보정은 제품 library의 코드 변경이 아니다.

## 셀 단위 나눔의 호스트 밴드와 선행 채움

#7288의 원 해결 #7357·#7378과 통합 #7383, 이후 #8129의 본문·변경 범위를 읽었다.
#8129는 아직 열린 PR이므로 적용하지 않았다. 현재 구현의 첫 단위 이월과 행 스캔이 모두
같은 부모 호스트 밴드를 조회하도록 보정했다. `ReflowBlockCellFlow.host_bottom`은 부동
자식의 프레임을 제외한 부모 글줄 점유다. 전체 `bottom`·자식 원점·CellUnit·예약 높이는
유지한다. 부모의 CellBreak 정책과 검증 가능한 재조판 프레임에 적용하며, 제한 없는 자식과
걸침 셀은 기존 경로의 원장을 유지한다. 임의 수치 허용치나 문서 식별 조건은 추가하지 않았다.

독립 확인을 위해 바깥여백 0/두 배, 짧은/긴 본문, 더 짧은 본문 대조군을 원본과 구분해
생성하고 한컴 2020 Print로 출력했다. 5개 모두 부모 셀의 자식 3행을 다음 쪽으로 통째로
옮긴다. `atomic-fresh-band-inputs-2075.json`, `atomic-owner-band-inputs-2078.json`에
변경점·입력 SHA를 보존했다. 여백 0 입력은 첫 worker가 terminal 실패한 뒤 같은 바이트로
한 번 재요청해 정상 Print를 확보했다. 기존 실패와 두 job의 근거를 모두 보존했다.

Native134는 분할 대신 통째 이월했지만 뒤 본문까지 밀려 3쪽이 됐다. 저장 줄의 존재와
visible-host만 요구하던 선행 채움이 NO_LS 빈 호스트의 점유를 빠뜨린 원인을 추적했다.
기존 글줄 출력과 선행 채움의 포맷·본문 전진을 `deferred_float_flow.rs`로 옮겨 공통으로
사용하고, 실제 이월이 확정된 빈 재조판 호스트도 자신의 글줄과 뒤 문단을 현재 쪽에 남긴다.
명시적인 쪽·단 경계나 다음 개체는 앞쪽에 채우지 않는다. 별도 좌표·높이를 덮어쓰지 않는다.

Native135는 공용 target·잡 8개로 240.58초에 빌드 종료 코드 0을 확인했다.
`reflow-policy-native135-2087/results.json`의 기존 18조건 중 10조건이 전쪽 raw 100%이며,
이전 미달이던 제한 true·부모 TABLE의 3조건은 최저 7.57797/25.28369/7.57797%에서
각각 100%·쪽수 2 대 2로 개선됐다. 추가 경계 5개도 전쪽 raw 100%·2쪽 일치다.
제한 false의 8조건은 여전히 미달이며 이 결과로 상쇄하지 않는다.

명시적 쪽 나눔과 뒤 문단 2개를 추가한 두 반례도 별도 Print로 확인했다.
`reflow-policy-prefill-native135-2091/results.json`에서 각각 3쪽/2쪽이 Print와 같고
전쪽 raw 100%다. 명시적 나눔의 뒤 문단은 3쪽에, 두 뒤 문단은 순서를 보존해 1쪽에
한 번씩 놓인다. `reflow-cellbreak-native135-review-2092/`의 1·2쪽 review와 standalone
overlay에서 뒤 본문·자식 3행·부모 외곽을 직접 읽었다. 엄격 픽셀 동일을 주장하지 않는다.

기존 완료 페이지·저장 프레임 계약 22개도 같은 Native135에서 모두 통과했다
(`current-previous-failures-2093-result.json`). 이전 전체 실패 중 외부 테스트 47개를 정식
소스에서 다시 컴파일해 실행한 결과는 3 PASS / 44 FAIL / 누락 0개다. 통과한 3개는 위
TAC 조회 helper 검사다. 목록은 `current-external-rust-failures-2090.tsv`이고, 내부 검사
9개는 이번 실행에 포함되지 않았다. 이 실행을 전체 Rust 통과로 보고하지 않는다.

Native135의 실제 빌드 소스는 `producers/native135-source-overlay/manifest.json`에
기준 커밋과 dirty Rust 파일 7개의 내용·해시로 보존했다. 빌드 뒤 호출 인자 줄바꿈만 fmt에
맞게 정리했으며, 다음 제품 빌드와 최종 head의 필수 검증은 새 소스에서 수행한다. 이번
프레임 보정의 fresh WASM 검증과 정식 회귀 추가는 아직 완료하지 않았다. 제한 없는 자식의
독립 스트림·부모 물리 프레임을 함께 보정한 뒤 최종 후보에서 검증을 묶어 진행한다.

## HWPX 표 삽입 자기검증의 저장 줄 캐시 계약

기존 실패 `issue_5819_insert_table_options`는 원 PR #7068과 그 검토 기록을 먼저
확인했다. 같은 입력 `samples/issue5162_field_wraps_table.hwpx`에서 실제 표 삽입과
저장은 성공하지만 `--verify`가 임시 재조판 캐시 12개 차이를 원본 손실로 판정했다.
원인과 값은 `table-insert-roundtrip-2096/`과 `logs/table-insert-cache-provenance-2098.log`에
보존했다. 선행 #5847·#5850·통합 #5853의 계약은 합성 줄을 저장하지 않고, 정상 원본의
쪽 상대 세로 위치를 복원하는 것이다. 이 저장 계약과 메모리 조판 캐시를 구분한다.

`source_vertical_positions`를 저장기와 HWPX 자기검증이 공유한다. 자기검증은 두 IR의
사본에서 저장 대상 줄 캐시를 비교하며 원본 문서와 저장 바이트를 수정하지 않는다.
기존 엄격 `diff_documents`와 HWP5 검증 경로는 유지한다. 자식 표·캡션·도형·머리말·
각주 등도 같은 저장 계약으로 순회한다. 문자 모양이나 정상 원본 줄의 속성 차이는
계속 검출한다. 렌더링·조판 계산이나 시각 기준값을 바꾼 수정이 아니다.

Native136은 HEAD `3871c5b5468d903875f6ecf8a839e8774831c09d`와 해당 dirty 소스의
해시를 기록하고 공용 target·잡 8개로 238.62초, 종료 코드 0을 확인했다.
`char-overlap-native-build-136-result.json`이 실제 소스·제품·라이브러리 해시를 고정한다.
새 구조 검사 7개는 기존 엄격 비교를 사용한 선행 실행에서 5 PASS / 2 FAIL,
보정된 API에서 7 PASS다. 원본 속성 손실 반례는 유지한다. 기존 #5819 CLI/MCP 검사도
원본 함수 그대로 5 PASS이며, 이전 실패 2개가 해소됐다. 다른 기존 실패를 통과로
계산하지 않는다. 명령·로그·결과는 `table-export-cache-after-2104/`에 보존했다.
같은 표 삽입의 저장 HWPX는 수정 전후 SHA-256이
`2e12401c6a942d21b1749f831272acce600d9501f8bd6e1454c0351a06c473cb`로 동일하고,
ZIP 내부 11개 항목도 모두 동일하다. 새 `--verify`는 종료 코드 0·차이 0개다.

신규 검사 수집 중 발견한 suite target 예산 문제는 예산을 늘리지 않고 처리했다.
크레이트 루트 참조가 없는 #4969 request contract를 기존 정책의 모듈 통합 방식으로
수집한다. 원본 standalone 실행과 중첩 모듈 실행 모두 39 PASS이며 정규화한 검사 이름
목록도 같다. 다른 shaping 예외는 유지한다. canonical generator로 Cargo target을
동기화한 뒤 1748 sources / 7601 static test attrs / 28 suites + 19 exceptions =
47/48 targets를 확인했다. 로그는 `logs/table-export-cache-suite-check-2103.log`다.
전체 Rust 재실행과 제한 없는 자식 흐름의 보정·최종 fresh 검증은 계속 필요하다.

## 제한 없는 원자 자식의 부모 프레임과 이어받기 진단

`unrestricted-policy-2106/implementation-plan.md`에 독립 Print·선행 #7288 근거와
공통 소비 경로를 정리한 뒤 수정했다. 쪽 제한을 끈 부모 안의 자식 표가 모두 NONE
원자 개체이고, 측정기가 인증한 부모 프레임이 현재 쪽에 들어가면 부모의 CELL/TABLE
설정만으로 자식을 다음 쪽으로 분리하지 않는다. `row_frame_heights`의 부모 프레임으로
통째 수용을 판정하되, 자식의 선언 정책·원점·내용 유닛과 기존 전체 흐름 예약은 보존한다.
제한된 자식·분할 자식·걸침 칸은 이 새 수용 조건의 대상이 아니다. 부모의 프레임이
들어가지 않는 경우도 기존 경로를 유지한다.

Native137은 공용 target·잡 8개로 243.04초, 종료 코드 0을 확인했다. 실제 HEAD는
`51e31ba54036151bf535b2f977a89d58d956b677`이며 dirty 소스 2개를 포함한 해시는
`char-overlap-native-build-137-result.json`에 있다. 기존 23개 정책·여백 대조군은
17개가 전쪽 raw 100%로 통과했고 6개는 여전히 미달이다. 부모 CELL/TABLE·자식 NONE
두 조건의 최저값은 각각 2.16283%에서 100%로 개선됐다. 쪽수도 Print의 2쪽과 같다.
`reflow-policy-native137-2108/results.json`이 전체 결과이며, 별도의 명시적 쪽 나눔·
뒤 문단 두 개 반례도 각각 3쪽/2쪽·전쪽 raw 100%다
(`reflow-policy-prefill-native137-2109/results.json`). 개선한 원자 자식 사례의 1·2쪽
review와 standalone overlay를 `unrestricted-atomic-native137-review-2112/`에 생성했다.
1쪽에서 자식 3행의 보존과 짧은 부모 외곽을 직접 확인했다. 엄격 픽셀은 35.73%로,
raw 100%를 색·두께·픽셀 전체 일치로 보고하지 않는다.

같은 동결 제품에서 완료 페이지·저장 프레임 22개는 모두 PASS다
(`current-previous-failures-2110-result.json`). 이전 전체 실행에서 실패한 외부 47개를
현재 정식 소스에서 재컴파일·재실행한 결과는 5 PASS / 42 FAIL / 누락 0개다
(`current-previous-failures-2111-result.json`). CLI 실패 2개가 통과로 바뀌었고,
이 결과는 전체 Rust·fresh WASM 통과를 뜻하지 않는다.

### 후속 본문 원점의 직접 대조와 잘못된 진단의 정정

남은 분할 자식의 범위를 분석하려고 본문 가용 공간만 늘린 대조군 2개를 추가했다.
변경은 `hp:pagePr/hp:margin`의 bottom 69000 → 62000뿐이며 내용·표·글꼴은 그대로다.
`unrestricted-fullfit-inputs-2113.json`에 원본·수동 대조군 SHA와 변경점을 보존했다.
MCP 2020의 독립 Print 2개는 모두 성공했고 `pdf_print_method=0`을 확인했다.
`mcp-unrestricted-fullfit-2113/`에 job·실제 출력 모드·PDF SHA를 기록했다.
Native137 결과는 두 파일 모두 1쪽 일치·raw 97.07029%이나 직접 원점 대조는 실패다.
후속 본문의 Native 기준선은 258.53333px, Print 기준선은 225.67338px로 약 32.86px
차이다. 점수 통과로 배치 결함을 면제하지 않는다.

작은 용지의 분할 사례도 후속 본문 기준선은 Native 91.50667px, Print 83.90830px로
약 7.60px 차이다. 앞선 ignored 진단에서 PDF `blocks`의 y=19.82px를 이 본문의
원점으로 해석한 가설은 잘못됐다. 그 블록에는 자식 표 글자가 함께 묶여 있었다.
실제 본문 `span`의 bbox 윗변은 72.89793px이며 기준선은 별도로 위와 같다.
`unrestricted-policy-2106/frame-stream-observations.json`을 정정했다. 진단 Rust의
실제 run baseline·반올림 전 bbox는 `logs/unrestricted-line-origins-2115.log`에 있다.
다음 보정은 부모의 물리 프레임 소유, 공동 앵커 자식들의 최대 점유 끝, 분할 뒤 여백과
후속 본문 예약을 같은 원장으로 연결한다. 소스 유닛 삭제나 부모 괘선 숨김으로 대신하지
않는다. 이 단계의 fresh WASM·새 정식 렌더링 회귀와 최종 전체 검증은 아직 필요하다.

## 공동 앵커의 전량 점유와 부모 NONE 분할 소유

`external-flow-plan-2117.md`의 근거에 따라 부모의 순차 글줄과 외부 자식의 점유를
합산하던 가정을 제거했다. `reflow_block_cell_flow`가 같은 자식 본체 높이·원점으로
`external_bottom`을 발행한다. `reflow_unrestricted_table_flow`는 단일 물리 행의
비걸침 재조판 프레임에서 부모 프레임과 공동 앵커 자식들의 최대 끝을 구분한다.
부모 오프셋·바깥여백을 Format의 내부 높이 축으로 변환하며, 소스 행 높이와 유닛은
변경하지 않는다. 부모 프레임 산출은 측정기와 같은 helper를 사용한다.

Format의 최종 effective height와 total height가 이 결과를 소비한다. 자식 분할 소유도
같은 결과에 담아 부모 NONE 강제 통째, prepare의 원자 규칙, scan의 분할 진입이 함께
구분한다. 부모 `page_break` 모델 값은 그대로다. 여러 물리 행·걸침·제한 자식이 섞인
경로는 기존 원장을 유지한다. 코드 위치는 `reflow_external_flow.rs`, `typeset/table.rs`,
`block/unrestricted_float.rs`, `block/prepare.rs`, `scan/row_entry.rs`다.

Native138은 HEAD `99c0d7f10d7a2ab3c385c83192f38bc862c4ed26`와 dirty 소스에서
236.52초·종료 코드 0을 확인했다. source 해시는 `char-overlap-native-build-138-result.json`,
실제 Rust 소스 10개의 사본·해시는 `producers/native138-source-overlay/manifest.json`에
보존했다. 새 전체 수용 대조군 두 개는 97.07029%에서 모두 100%·1쪽 일치로 개선됐다
(`reflow-policy-fullfit-native138-2121/results.json`). 후속 본문의 실제 run 기준선은
258.53333px에서 225.89333px로 이동했다. Print의 225.67338px 대비 차이는 약 0.22px다.
`logs/unrestricted-line-origins-2125.log`와 Print의 개별 span을 대조했다.
`unrestricted-fullfit-native138-review-2124/`의 review·standalone overlay에서 자식 3행,
짧은 부모 외곽과 바로 뒤 본문을 직접 확인했다. 엄격 픽셀 39.05%이며 raw 100%를
전체 픽셀·색·괘선 두께의 동일로 보고하지 않는다.

23개 정책·여백 대조군은 17개 전쪽 100%, 6개 미달이다. 부모 NONE의 두 미달은
자식 통째 출력에서 2+1행 분할로 바뀌어 최저 4.48044% → 83.24590%다. 6개 모두
83.24590%로 모였으며 부모 프레임 반복과 이어받기 여백을 계속 보정해야 한다.
전체 결과는 `reflow-policy-native138-2119/results.json`이다. 명시적 쪽 나눔·후속
본문 두 개 반례도 각각 3쪽/2쪽·전쪽 100%다. 기존 완료 페이지·저장 프레임 22개는
PASS, 이전 실패 외부 47개는 5 PASS / 42 FAIL / 누락 0개로 이전과 같다
(`current-previous-failures-2122-result.json`, `current-previous-failures-2123-result.json`).
fresh WASM, 새 정식 렌더링 회귀 및 최종 전체 검증은 완료하지 않았다.
## 독립 자식 조각과 부모 물리 프레임의 소유 분리

전량 점유 보정 뒤에도 분할 자식이 부모 프레임과 바깥 아래 여백을 다음 쪽에 반복했다.
`reflow_external_cut.rs`에서 원본 source cut이 선택한 자식 내용의 높이와 부모 프레임의
소유를 함께 발행한다. `row_cut_height`와 물리 예산 스캐너는 같은 결과를 소비하며,
부모 패딩을 예산에서 다시 차감하지 않는다. 부분 배치도 같은 소유를 소비해 첫 조각에만
실제 부모 Table/Cell 프레임을 그리고, 이어받기는 원래 자식 소유 정보를 유지한다.
유닛 삭제·선언 폭 축소·임의 clamp로 대신하지 않았다.

첫 부모/호스트 앵커에서 소비한 자식의 세로 오프셋은 이어받기에서 반복하지 않는다.
MCP 2020 Print의 양의 오프셋 대조군에서 첫 쪽 자식은 300HU 내려가지만 다음 쪽은
오프셋 없는 사례와 같은 원점이다. 부모 프레임만 첫 쪽에 남고 자식 내용은 전량 이월되는
대조군도 다음 쪽에서 같은 원점을 갖는다. 원본·변경점은
`external-coordinate-inputs-2128.json`과 `external-child-opening-inputs-2137.json`,
실제 Print 영수증과 PDF 해시는 대응 `mcp-*` 경로에 보존했다.

셀 간격이 있는 표는 표 외곽과 칸 외곽의 스타일 소유를 분리했다. 공통
`render_cell_box_borders`를 전체/부분 표의 바깥 프레임에서도 사용한다. 셀 테두리만
NONE인 경우 표 외곽은 남고, 표 테두리만 NONE인 경우 칸 외곽은 남는 독립 Print 2개로
확인했다. 간격 0의 기존 edge-grid 정책은 유지한다. #6311의 원 해결 PR #6420과
통합 PR #6523을 추적했으며, 광범위한 fallback 완화로 기존 검사가 깨졌던 전력을 확인했다.
근거는 `issue6311-outer-border-provenance-2135.json`과
`pr6420-outer-border-provenance-2135.json`이다.

Native140은 잡 8개·232.98초·종료 코드 0이다. 처음 빌드의 타입 경로 누락 오류와 로그를
보존하고 보정 뒤 재빌드했다. 35개·66쪽의 Native TSV는 최저 95.93231%, 쪽수 일치이며
34개 문서는 raw 100%다. 정책·full-fit·명시 경계·좌표·스타일 소유·첫 자식 수용 대조군은
`reflow-*-native140-2141`부터 `2146`의 `results.json`에 있다. 실행 스크립트의 경로 치환
오류로 중단한 마지막 그룹은 기존 로그를 보존하고 올바른 경로로 재실행했다.
또한 마지막 두 입력의 PDF 연결 목록 오류를 MCP 영수증의 실제 입력 경로·SHA로 정정했으며,
잘못된 목록은 `external-child-opening-matrix-2139-before-source-pairing-fix.json`에 남겼다.

`unrestricted-split-native140-review-2154`의 2쪽 review·standalone overlay를 직접 확인하고
사용자에게 함께 표시했다. 흑백 문자·부모 프레임 비반복·후속 본문 원점은 일치한다.
raw 100%는 엄격 RGB 일치가 아니다. Print의 얇은 괘선은 0.36pt(96dpi에서 0.48px)이며
같은 좌표에 셀·표 선을 중복 출력한다. Native tree의 반올림 두께는 0.5px다.
근거는 `native140-border-and-body-observations-2155.json`이다.

마지막 첫 자식 전량 이월 사례는 점수 통과와 별개로 미완료다. Print는 후속 본문을 첫 쪽에
남기지만 Native140은 다음 쪽으로 보냈다. Native141은 같은 컷 결과에 자식 내용 소유를
추가하고 기존 후속 선배치 공통 경로를 재사용해 쪽 소속을 보정했다. 다만 실제 배치는
본문을 약 24px 위에 두어 93.32630%다. `external-opening-flow-native141-2169`에서
최종 paint가 `ComputedEmptyHost`의 글줄 전진만 다시 소비하고 수용한 부모 프레임의
흐름 끝을 전달받지 못함을 확인했다. 이 소비 경로를 다음 단계에서 보정한다.

Native141은 233.41초·종료 코드 0이며 정상 대조군 22개, 기존 표 테두리·cellzone 5개는
모두 PASS다. 과거 외부 실패 47개는 5 PASS / 42 FAIL / 누락 0개다
(`current-previous-failures-2164/2165/2166-result.json`). Native141의 35개·66쪽 재실행은 쪽수 일치, 34개 문서 raw 100%, 최저 93.32630%다.
`logs/native141-matrix-batch-2163.log`에서 모든 그룹의 종료 코드 0을 확인했다. 이 단계의 fresh WASM, 새 정식 렌더링
회귀와 최종 전체 검증은 완료되지 않았으며 PR 준비 완료로 판정하지 않는다.

### 수용한 부모 흐름 끝의 최종 배치 전달

Native141의 첫 쪽은 `ComputedEmptyHost { line_advance: 21.33333 }`가 수용한 부모
프레임의 흐름 끝을 대체했다. `FLOW_ORIGIN/FLOW_END`에서 부모 조각 뒤 y=162.66667px를
후속 본문 원점으로 쓰는 것을 확인했다. `layout.rs`의 최종 항목 전진은 기존
`stored_paragraph_flow_ends`를 소비하므로, 후속 선배치가 가능한 계산 호스트의 마지막
개체에 대해 수용한 `host_end`를 이 경로에 전달했다. 문서별 좌표 보정이나 paint 위치
clamp를 추가하지 않았다. 선배치의 적용 가능 여부는 공통 helper가 반환하고, 다음 내용
예약 뒤의 끝을 부모 프레임 끝으로 잘못 기록하지 않도록 선배치 전 값을 사용한다.

Native142는 잡 8개·240.06초·종료 코드 0이다. 같은 35개·66쪽은 **전쪽 raw 100%**,
전체 쪽수 일치, 누락 0쪽이다. `reflow-*-native142-2170`부터 `2175`의 TSV·results와
`logs/native142-matrix-batch-2176.log`에 보존했다. 첫 자식 전량 이월 사례의 후속 본문은
첫 쪽 y=187.3px로 돌아왔고, 두 번째 쪽에는 자식 3행이 원래 순서로 이어진다.
`external-opening-native142-review-2182`의 두 쪽 review·standalone overlay를 직접
확인하고 사용자에게 함께 표시했다. 흑백 문자와 내용 소유도 확인했으며 얇은 선의
raster 농도 차이를 RGB 일치로 보고하지 않는다.

같은 library에서 정상 대조군 22개와 표 테두리·cellzone 5개는 모두 PASS다. 외부 기존
실패 47개는 여전히 5 PASS / 42 FAIL / 누락 0개다. 원 실패 API setup으로 현재 저장본
20개를 내보내는 진단 1개도 PASS이며 정식 회귀 변경의 시각 선행 조건을 대신하지 않는다.
`current-previous-failures-2177/2178/2179/2185-result.json`에 결과를 보존했다.

#5790 원 해결 PR #5814와 archive 기록을 다시 읽었다. 과거 PDF의 Print 출처 확인을
대신 추정하지 않고 같은 원본을 MCP 2020·PrintMethod 0으로 새로 인쇄했다. 실제 제품은
11.0.0.9136, 모드는 `hancom2020_pdf_driver_one_up`이다. Native142의 전 13쪽은 쪽수 일치,
최저 raw 96.10356%이며 `charoverlap-original-native142-2188`에 보존했다. 이 문서의
fresh WASM·변경 영역 직접 판독과 실패한 SVG 검사 의미의 검토는 이어서 수행한다.

fresh 빌드 전 원격 devel은 `def947c2149df5bc812d801e6e79be19a403b4d1`로 같아 추가
리베이스가 없으며 본인 열린 PR은 0개다(`pre-wasm-*-2187`). Native 완료를 fresh WASM
완료로 바꾸어 기록하지 않으며, 최종 전체 검증·새 회귀 등록·PR 제출은 아직 필요하다.

### fresh WASM048과 실제 Studio 일반 글리프 소비

`8c32167d3d87f41beb5c6b045615f4f4e7df112d`에서 표준 release·잡 8개로 빌드한
WASM048은 849.05초·종료 코드 0이다. `char-overlap-fresh-wasm-build-048-result.json`과
`release-standard-wasm048-2184/build.json`에 보존했다. 실제 앱 초기화 응답도 WASM
SHA-256 `ce0e060772086f2e6bc907b70224c083cb3c48e728b00cda468ad0762c00cc22`와
일치하며 `logs/wasm048-release-consumption-2196.log`에서 세 배포 위치를 확인했다.

- 표·이월 대조군 35개·66쪽: 전쪽 raw 100%, 쪽수 일치.
  `reflow-wasm048-full-matrix-2203-result.json`과 그룹별 원본 TSV에 기록했다.
- 원 caret API 저장본 20개·20쪽: 전쪽 raw 100%, 쪽수 일치.
  `caret-original-current-api-wasm048-2206-result.json`에 기록했다.
- #5790 실제 원본 13쪽: 최저 raw 96.10629%, 쪽수 일치.
  `charoverlap-original-wasm048-2207-result.json`에 기록했다.

이 Sweep은 Rust WebCanvas를 호출한다. 이 결과나 일반 글꼴 변환 API의 성공만으로
Studio TypeScript CanvasKit의 글리프 소비까지 검증했다고 판단하지 않았다. 사용자가
함초롬바탕으로 확인한 U+1F9A6은 해당 글꼴 프로그램에 없으며, 한컴 Print PDF는
Segoe UI Emoji의 검은 윤곽을 사용한다. 원본 글자모양 이름은 그대로 유지한다.

실제 파일 선택 → loadFile → loadBytes → renderer session 경로에서 기존 코드는
HCR Batang만 읽고 Segoe 프로그램은 읽지 않았다. 단독 그림문자도 복합 문자로 분류해
`🦦나나끝` 전체를 그리지 않는 별도 소비 결함을 재현했다
(`ordinary-text-browser-consumption-2213-result.json`). 다음 공통 경로를 보정했다.

1. 일반 텍스트 후보를 문서 선언·본문 대체·그림문자·generic 순서로 구성하고,
   host 준비와 CanvasKit의 실제 글리프 선택·관측이 같은 후보를 소비한다.
   기존 #8109의 주 글꼴 준비 요건과 문서 두 번째 후보 선택 계약은 유지한다.
2. Unicode Extended_Pictographic의 단독 문자는 명목 글리프로 그린다. 기존 그림자
   원형 기호도 유지하며 VS·ZWJ·결합 부호·피부색 수식자·국기 조합은 보류한다.
3. Canvas2D의 host 별칭을 주 글꼴뿐 아니라 준비된 글리프 대체 후보에도 전달한다.
   원래 OS의 컬러 프로그램으로 다시 넘어가지 않도록 하며 원본 bytes는 변경하지 않는다.

실제 시스템의 HCR Batang·Segoe UI Emoji를 기존 HostFontProvider API로 공급했다.
headless Local Font Access가 0개를 반환한 환경과 구분했으며, 실제 공급 프로그램의
경로·SHA·face 이름은 `ordinary-text-real-host-fonts-2208.json`에 고정했다. 글꼴 파일은
커밋하지 않는다. `ordinary-text-browser-consumption-2231/2228-result.json`은 두 프로그램의
실제 읽기, 앞뒤 한글, 원래 producer positions, backend와 frontend 파일 해시를 기록한다.

동일한 2쪽 한컴 Print PDF·96dpi·실제 앱 캡처의 raw 값은 CanvasKit 98.68405% /
98.48776%, Canvas2D 96.87361% / 97.23718%다. `ordinary-text-studio-canvaskit-2231`과
`ordinary-text-studio-canvas2d-2228`의 `tsv/silhouette.tsv`·review·standalone overlay를
생성하고 첫 쪽과 CanvasKit 둘째 쪽을 직접 대조했다. 흑백 윤곽과 뒤 한글은 복원됐다.
앱의 표 경계·편집 가이드 차이는 남아 있어 이 점수로 문서 전체 개선 완료나 px 핀의
추가 요건을 충족했다고 선언하지 않는다. Rust WebCanvas의 raw 100%와도 구분한다.

검증은 `rhwp-studio`에서 수행했다. 관련 단위 검사 37건 PASS, 전체 `npm test`는
2,384 PASS / 2 SKIP / 0 FAIL, `npx tsc --noEmit`과 `npm run build`도 종료 코드 0이다.
로그는 `logs/ordinary-text-focused-ts-2226-attempt02.log`,
`logs/ordinary-text-studio-full-unit-2229.log`, `logs/ordinary-text-tsc-2227.log`,
`logs/ordinary-text-studio-build-2230.log`에 있다. 정식 #7688 E2E에 서로 다른 실제 cmap의
혼합 런을 독립 face별 그리기와 대조하는 검사를 추가했다. 이 합성 자원 소비 계약을
한컴 배치의 시각 정답지로 승격하지 않는다. 해당 검사와 기존 #8109의 두 번째 후보·
주 글꼴 우선·늦은 갱신·실패 시 Canvas2D 전환은 모두 PASS다
(`logs/ordinary-text-font-preservation-2227.log`). 같은 정식 브라우저 검사를 수정 전
`8c32167d`의 소스에서 실행해 혼합 런의 독립 face 소비 오류로 FAIL함을 확인했다.
이 단계에서 소유한 다섯 코드 파일만 전후 bytes·SHA로 고정했고 `finally`에서 현재
수정본을 정확히 복원했다. 같은 검사의 수정 후 PASS와 함께
`ordinary-text-font-before-after-2233-result.json`에 기록했다. 최종 실물 앱 캡처와
글꼴 관측도 재실행해 요청 `함초롬바탕` → 실제 `Segoe UI Emoji`·local face·실패 0을
확인했다(`ordinary-text-browser-consumption-2231-result.json`,
`logs/ordinary-text-real-host-paint-trace-2231-attempt02.log`). 최종 관측 분류 수정의
TypeScript 재검사는 `logs/ordinary-text-final-tsc-2232.log`에서 종료 코드 0이다.

#4138의 두 현재 API 저장 HWP는 SHA-256이 같아 같은 입력을 MCP 2024에서 Print했다.
제품 13.0.0.3901·PrintMethod 0·`frame_print_to_pdf_ex_one_up`이며 PDF는 196쪽이다.
`input_preprocess=scripts_disabled`를 숨기지 않는다. 원본의 DefaultJScript는 실행
코드가 없는 빈 저장 스트림임을 `issue4138-empty-script-proof-2212.json`에 입증했다.
현재 Native도 196쪽이지만 전쪽 대조에서는 195쪽 미달·최저 raw 17.92141%다.
`issue4138-native142-2217/results.json`에 보존했다. 쪽수 일치만으로 기존 197쪽 검사를
196쪽으로 갱신하지 않으며, 이 문서와 나머지 실패·최종 전체 검증은 계속 해결해야 한다.


## 2026-10-24 새 기준 6097253468 리베이스와 기능 보존 검증

최신 지시는 로컬 보정·검증과 PR 준비까지다. PR 생성·push·merge는 대기한다.
리베이스 전 head는 `ba9646bbdc25effe4e4d1e9f909f15c5d22938f4`, 이전 base는
`3536e72819e165dbe959ee42cfd4e4cf8db33ce7`이다. 새 base
`60972534680a0c5e51e54ec189b92e252e81a867` 위의 리베이스 head는
`ed427895aa80aff7045242a85cd1bbe8afd328ca`다. 245개 커밋을 적용했으며
백업 브랜치 `backup/semanticist21-pre-rebase-20261024-3043`을 보존했다.
작업 중이던 수식 글꼴 명령 변경 12개 파일도 SHA를 기록한 stash
`d9bb9772b0561cb22cc2d4866b5be878ecb3c36f`에 보존했다. 아직 이를 새 head의 검증 완료 코드로
표시하지 않는다. 현재 리베이스 보정이 안정된 뒤 복원한다.

`local_validation.md`의 「리베이스로 반영한 PR의 기능 보존 검증」을 적용한다.
자동 적용·컴파일·range-diff만으로 기능 보존을 판정하지 않는다. 새 base와 후보를 같은
입력·독립 Print·글꼴·backend·DPI에서 실행한다. 현재 수식 보정 대상의 검증도 유지한다.

| 반영 PR | 기능과 교차 경로 | 현재 실제 결과 | 남은 필수 확인 |
| --- | --- | --- | --- |
| #8178 | 선택 실행 보고서의 상태·증적 무결성·공유 정보 제외·실제 HTML 소비 | 새 base와 후보 각각 Node 계약 40 PASS, 실제 Chromium 156 검사 1 PASS | 일반 확장 제품 검증은 최종 후보의 기존 게이트에서 유지 |
| #8191 | 표 분할·줄/원점/예약 소유·글꼴·그림·공개 출력 | 독립 입력/PDF 원장 100항목 모두 로컬 존재 확인; 새 Native 빌드 진행 | 동결 49조건·1,772쪽의 새 base/후보 Native/fresh WASM, 중요한 경계의 확대 판독·기존 관계 회귀 |
| 현재 보정 | 구형 CHAR·그룹·첨자 소유·원본 HWP/HWPX·현대 수식 대조군 | 이전 Native190 결과는 이전 source의 이력 | 새 source의 전쪽 TSV·동일 영역 review/overlay·미세 간격 보정 |

리베이스 충돌은 두 경로의 계약을 대조해 해소했다. 표 괘선의 `BorderEdge` 소유,
필드 종료 슬롯과 개체 슬롯의 분리, 모델 문자/표시 번호의 분리, 원 글꼴과 실제 paint의
폭 소유, 그림의 회전 전 상자·외곽선, 저장 줄의 물리 공간과 내용 컷을 보존한다.
첫 실제 Native 빌드(3046)는 중복 정의·테두리 타입 연결·공개 helper 경로·쪽 번호
수집의 가변 참조 오류 11건으로 실패했다. 이를 통과로 기록하지 않는다. 같은 내용의
후속 자식 모듈 정의를 사용하고, 쪽 번호는 새 devel의 공통 source 순서 수집 결과를
소비하도록 보정했다. 3052에서 다시 빌드한다.

첫 fmt 검사에서 삭제된 회귀 5개를 가리킨 것은 이전 generated suite의 파생 상태였다.
정본의 `rust-test-suite-manifest.mjs --prepare`로 다시 생성하고 fmt를 적용했다.
파생 harness/manifest는 제출 변경에 포함하지 않는다. 새로운 테스트나 기대값·baseline은
이번 리베이스 보정에서 완화하지 않았다. 전체 기능 보존과 시각 검증은 아직 완료 판정 전이다.

증적은 ignored `output/pr-review/semanticist21-20261019/`의 `rebase-6097253468-3043-*`,
`rebase-pr8178-*-3045/3047-*`, `rebase-pr8191-input-inventory-3048.json`,
`logs/rebase-native191-build-3046.log` 및 `logs/rebase-native192-build-3052.log`에 보존한다.

3052에서는 같은 수집 함수의 쿼리 소비자 2곳에서 이전 인자를 제거하지 않은 오류가 추가로
확인됐다. 호출 지점을 모두 검색해 연결했고, 3053의 Native193 빌드는 254.83초에
성공했다. source SHA-256 목록의 전후 동일성을 확인하고 실행 파일·rlib를 별도로
고정했다. 3054에서 49조건·1,772쪽의 후보 Native 전쪽 검증을 시작했다.
새 base도 3051의 SHA 고정 archive에서 동일 feature·profile·공용 target·빌드 잡 8로
3056에서 빌드한다. 현재 기능 보존 판정은 검증 진행 중이며 준비 완료가 아니다.
원본/PDF 100항목의 SHA-256은 기존 입력 원장과 모두 같고, 리베이스 전후 245개
커밋 제목의 순서도 모두 같다(3055). 후자는 실제 기능 검증을 대신하지 않는다.

3054/3059의 실제 출력 대조에서 표 시작 위치의 후퇴를 검출했다. 같은 원본의 20쪽에서
새 base의 표 상단은 830.1px, 후보는 826.1px이며 앞 문단과 표 host의 흐름 시작
824.2px는 같다. 표 높이도 같다. 최종 표 배치까지 추적해 host의 문단 위 간격이
생산·측정·배치 중 어디서 소실되는지 확인한다. 문서별 좌표 보정은 추가하지 않는다.

3061에서는 독립 Print로 검증했던 구형 문자 평면 9쪽과 그룹 대조군 22쪽이 각각
최저 raw 70.71289%, 62.24617%로 후퇴했다. 새 HFT 논리 프레임의 조기 반환이
기존 CHAR 좌표 평면 투영을 건너뛴다. 기존 독립 Print는 문자 평면의 가로 위치뿐
아니라 글꼴 글줄 프레임도 입증한다. 따라서 지원하는 선형 문자·그룹에는 기존
글꼴 프레임과 CHAR 평면을 함께 적용하고, 지원하지 않는 분수·복합식은 새 HFT
프레임을 유지한다. 평면의 수용 여부를 반환해 경계를 명시한다. 테스트 기대값이나
허용치를 완화하지 않고 두 대조군 및 새 base의 분수·한글 경로를 다시 실행한다.

3068에서 새 base/후보의 49조건·1,772쪽 Native 실행 종료와 독립 쪽수 일치를
확인했다. 새 base는 미달이 없고 후보의 분할 저장본 두 조건은 176·177쪽이 미달이다.
205쪽 문서의 20쪽과 분할 저장본의 7·176·177쪽은 2%p 초과 하락이다. 3069의
새 review·overlay를 열어 177쪽의 앞 행 누락과 뒤 본문 이동을 직접 확인했다.

분할 저장본의 원인을 기존 단계 기록 `task_semanticist21_20261024_opening_row_frame.md`,
원 보정 `0a6021cbb9`와 `ff8f719f17`, 새 base의 원점/행 프레임 소비와 대조했다.
같은 자식 표의 개체 선언 높이는 원본 115쪽의 첫 물리 조각을 나타내지만, 부모 폭을
바꿔 재저장한 197쪽의 현재 행 높이를 인증하지 않는다. 새 base의 공통
`RenderNormalizationOverlay::nested_table_has_reflowed_host`가 이 출처를 이미 소유한다.
새 첫 조각 helper와 완료 행 회수도 이 출처를 소비하게 연결한다. 기존 셀·글줄은
보존하며, 오래된 개체 선언을 현재 프레임으로 다시 승격하지 않는다.

또한 일반 중첩 글줄의 분할 허용을 모든 원본 저장 줄로 확대한 결과, 재조판하지 않은
행의 첫 줄이 앞쪽 조각으로 이동했다. 현재 재구성 글줄의 컷과 원본 저장 행의 경계를
구분한다. 재조판 슬롯·합성 줄은 일반 글줄 컷을 사용하고, 원본의 명시적 내부 재시작은
기존 별도 경로를 유지한다. 원본 115쪽, live/저장본, 197쪽 대조군 모두 재검증한다.

Native195는 `f7c8f7001f`에서 빌드했다. 3076/3091의 49조건·1,772쪽은 모두 90% 이상,
쪽수 차이와 새 base 대비 2%p 초과 하락이 없다. 3079의 문자 평면 9쪽·그룹 22쪽은
최저 99.98654%·99.98025%, 현대 대조군 3쪽은 99.56555% 이상, 새 독립 Print를 쓰는
원본 HWP/HWPX 각 115쪽은 최저 97.47718%다. 새 base fresh WASM052도 실제
wasm-opt를 거쳐 전 49조건·1,772쪽이 90% 이상이다. 후보의 fresh 검증 완료는 아니다.

3078의 기존 정식 검사 250개는 240 PASS·9 FAIL·1 ignore다. 같은 두 실패 모듈을
새 base에서 실행한 3083은 27 PASS·0 FAIL이므로 기존 실패로 면제하지 않는다.
3081의 첫 기준 실행기는 환경 변수·직접 의존성 인자가 빠져 컴파일에 실패했고, 이를
수정한 실제 결과가 3083이다. 3084의 55/56/61/200자 편집 후보 저장본은 재열기에서
115→116쪽으로 늘어난다. 3086의 같은 56자 저장본 두 개를 한컴 2024
13.0.0.3901·PrintMethod 0으로 새 출력하면 모두 115쪽이다. 3087의 원시 레코드는
DocInfo와 레코드 순서가 같고 LineSeg만 2,056곳 다르다.

3088/3089에서 첫 38개 유닛의 소유·높이는 같지만 첫 쪽 컷은 base 37, 후보 38이다.
수용 용량에서 아직 다음 줄 전진까지 필요한 줄을 마지막 잉크만으로 인증한 분기가
다음 글줄을 앞 쪽에 흡수한다. 새 base의 저장 슬롯·문단 끝 인증을 복원하며, 실제
점유와 수용 용량을 구분한다. 또한 NO_LS 빈 호스트라는 사실만으로 행 내부의 이어받기
조각에 바깥여백을 반복한 경로를 확인했다. 새 행으로 시작하는 조각과 행 내부 컷을
구분하고, 원본 저장·계산 호스트가 별도로 인증한 여백 계약은 유지한다. #7518의
독립 Print와 원 해결 기록, 최초 `e511ba4d43`의 적용 경계를 함께 대조한다.

3090의 #7518 실제 원본·최소 입력 4개는 양쪽 고정 Native로 다시 출력했다.
부분 배치로 통합된 자식 표의 셀·행 높이는 같지만, 직계 격자 Line의 반 획이
최종 Table bbox에 다시 합산돼 자식 표만 87.04→87.306…px로 커졌다. 측정한
정렬 내용 높이와 마지막 bbox가 달라진 것이다. 표 직계 격자선의 그리기 범위를
흐름 높이에 합산하지 않고, 실제 셀 내용의 확장과 괘선 paint는 유지한다. 같은
완결 표의 통배치·부분 배치가 같은 논리 프레임을 소비하는지 재검증한다.

Native196(`58eef2f1ed`)의 3094는 245 PASS·4 FAIL·1 ignore다. #7114의 다섯
저장·재열기 계약과 #7518의 완결 내용 정렬은 복구됐다. 남은 #7518 세 검사 외에
표 끝과 마지막 칸의 간격 검사가 추가로 실패했다. 직계 격자선의 중심이 소유한
끝 간격까지 제외한 것이므로, 중심 좌표는 보존하고 반 획만 논리 높이에서 분리한다.

3099의 실제 소비 경로에서 원래 host의 수용·paint 시작은 양쪽 모두 56.693…px,
확정 float 계획은 없지만 후보의 표 원점만 1.88px 아래이고 흐름 끝은 3.76px 더 길다.
원인은 `reflow_recursive_overlay_frame`이 자식의 재귀 구조만 보고 행 내부 이어받기에도
바깥여백을 여는 것이다. 새 행의 상단과 이미 시작한 행의 컷을 공통 함수에서 구분해
수용 예산과 실제 배치가 같은 판정을 소비하게 한다. source ID나 좌표로 분기하지 않는다.

3097은 네 #7518 입력을 기존 독립 PDF와 전쪽 비교한 진단이다. 후보의 미달뿐 아니라
현재 새 base에서도 원본 그림 띠·nested-split은 각각 최저 89.14610%·86.16709%다.
이 결과를 숨기거나 테스트 통과를 시각 통과로 대신하지 않는다. 입력/PDF 출처·실제
출력 환경과 기존 원 해결 증거를 다시 대조하며 현재 구현 보정과 검증을 계속한다.

Native197(`0e11783563`)에서 #7114 다섯 검사와 #7518 원본의 표 원점·완결 정렬을
재확인했다. 남은 nested-split의 실패는 이어받은 자식이 위 여백을 다시 받아
소유 셀 내용 원점보다 약 3.77px 아래에 놓이는 것이다. 실제 자식 시작·끝 컷은
이미 유닛에서 확정됐지만, 부분 배치 helper와 부분 내용 높이가 위·아래 여백을
모든 조각에 다시 더했다. 같은 `owns_anchor`/시작·끝 소유를 원점과 예약 높이에
연결한다. 제한을 켠 자식과 외부 호스트를 참조하는 자식 모두 첫 원점만 위 여백을
소비하며 끝 조각만 아래 여백을 소비하도록 보정하고, 같은 정식 검사·전쪽 대조를
다시 수행한다. 원본 저장본·내용·기대값은 바꾸지 않는다.

3103의 기존 세 입력은 Windows 한컴 2020 11.0.0.9136·PrintMethod 0으로 다시
출력했다. 원본·valid_orientation/nested-split·valid_generated/terminal-follower이며
입력은 모두 프로그램 생성 HWP로 저장 제품 메타데이터가 없다. 원 PR의 2020 인쇄
계약을 재현한 것으로 기록하며 한컴 저장본으로 부르지 않는다. terminal-tail의 새
출력은 worker 종료 코드 3762504530으로 실패했다. 기존 PDF를 실패 때문에 폐기하거나
성공으로 바꾸어 기록하지 않는다. 성공한 새 Print와 비교한 3107에서도 base/후보의
미달이 재현돼, 이전 PDF의 Creator 표기만으로 생긴 차이는 아님을 확인했다.

3104의 전체 결과는 244 PASS·5 FAIL·1 ignore다. 우선 실행한 기존 실패뿐 아니라
대조군까지 끝난 결과를 구분한다. 3110에서 auto-height의 현재 첫 유닛 10은
`nested_cut=([0],[1],false)`인 실제 자식 내부 컷이고, 일반 원본의 같은 유닛은
`nested_cut=None`인 완결 자식 행이다. parent가 continuation이라는 이유만으로
둘을 같게 처리하지 않는다. 공통 원점/예산 판정은 현재 행과 첫 미소비 유닛의 실제
재귀 컷 소유를 사용하고, 아직 소비하지 않은 미래 자식은 앞 일반 글줄의 프레임을
바꾸지 않게 한다. 블록 인덱스를 유닛 인덱스로 잘못 해석하지 않도록 namespace도
확인한다. 이 수정과 자식 첫/끝 여백 보정을 같은 다음 빌드에서 검증한다.


### 리베이스 기능 보존 재검증과 첫 유닛 소유 연결 — 2026-10-24

고정 새 base는 `60972534680a0c5e51e54ec189b92e252e81a867`, 직전 후보는
`d8fe5e5058b3f0a3092807db6f342453b53926c9`다. 3112의 기존 정식 검사 전체 결과는
246 PASS·3 FAIL·1 ignore다. 3106의 Native197 대조는 49조건·1,772쪽 모두 완료했지만
현재 Native198의 최종 증거로 표시하지 않는다. 후보 fresh WASM과 최종 전체 검증도 남아 있다.

남은 #7518 세 검사는 정확한 기존 계약을 유지하고 구현을 보정한다. 실제 부분 배치
호출자가 `owns_anchor`를 부모 프레임 소유로 대체해 첫 자식 유닛을 이미 소비한 조각에도
바깥 위 여백을 반복했다. `CellUnit.para_idx`의 소유 범위와 선택한 시작·끝 컷에서 같은
소유 판정을 만들고, 부분 내용 측정과 최종 자식 원점이 이를 함께 소비하게 연결한다.
정상 전체 배치와 외부 호스트의 별도 부모 소유 계약은 보존한다.

테두리의 반 획은 실제 부분 표의 paint 상자에 포함되는 기존 계약이다. 3090에서 발견한
완결 자식 표의 전체/부분 경로 차이를 고치면서 이 계약까지 바꿨으므로, 논리 격자 중심의
통일은 처음부터 끝까지 소유한 완결 중첩 표로 한정한다. 분할 조각과 루트의 실제 잉크
경계는 원래대로 보존하고, 내용 확장도 함께 센다. 원본·독립 Print·기존 테스트 기대값을
바꾸지 않는다. 원 PR·실제 통합·독립 출력은 위 #7518 선행 분석과 3103/3114에 연결한다.
이 보정의 기능·시각 결과는 다음 동결 빌드에서 새로 확인하며 PR 생성은 계속 대기한다.


### 추가 devel 동기화와 혼합 호스트의 실제 fallback — 2026-10-24

원격 확인에서 #8195가 추가돼 새 base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`로
252개 커밋을 리베이스했다. 후보는 `3a0338ec6527d7ff6fcf20ae49aaa89b25edeb4d`다.
3124에 이전/새 base·후보와 커밋 제목 순서·Rust 입력의 동일성을 기록했다.
이번 추가 PR은 Studio 오류·복구 통지이며 Rust 소스 차이는 없다. 동결 producer의 실제
빌드 SHA는 덮어쓰지 않고 source 동등성 근거를 별도로 연결한다.

#8195의 세 Node 모듈은 새 base 3121과 후보 3127에서 각각 52 PASS·0 FAIL이다.
3125는 새 base Studio와 같은 Rust 소스의 fresh WASM052를 사용한 Chromium 실제 E2E
성공이다. 3123은 archive에서 `@wasm` 별칭이 참조하는 pkg를 준비하지 못한 실행기 오류이며
로그를 보존했다. 경로를 바로잡은 3125와 구분한다. 후보의 실제 fresh E2E는 아직 남아 있다.

Native199에서 테두리 계약은 회복했지만 nested-split과 terminal-follower는 여전히 실패했다.
전체 셀 흐름이 없는 혼합 호스트는 공통 원점 helper 뒤의 `reflow_nested_table_origin`으로
배치돼, 새 첫 컷 소유를 전달하지 않고 `true`로 덮어쓰고 있었다. 기존 문단 진입의 소유
판정을 공통 컷 helper로 연결하고, 이미 구한 값을 전체 흐름·fallback 양쪽에 전달한다.
각 자식마다 셀 유닛을 다시 조회하는 중복 계산도 제거한다.

3126의 실제 수용 원장은 follower 첫 쪽에서 base의 [1,13] 대신 후보 [1,12]를 선택했다.
첫·끝 유닛의 before/after가 이미 예약한 바깥여백을 `mixed_nested_flow_extra`가 중간 컷에
다시 더해, 필요한 높이가 206.626…px로 늘어나 다음 행을 밀어냈다. 유닛이 edges를 소유하면
그 예약만 소비하고, 별도 경로는 공통 시작·끝 소유로 여백을 한 번 센다. 최종 실제 원점과
수용 요구가 같은 컷을 사용하도록 함께 보정한다. 기존 검사와 독립 PDF를 바꾸지 않는다.


### 행 경계와 셀 내부 viewport의 별도 물리 계약 — 2026-10-24

Native200(`f1fe3ed301`)에서 nested-split은 전6쪽 최저 19.69562→96.66160%,
terminal-follower는 전3쪽 최저 65.35158→90.22315%로 개선했다. 기존 두 실패도 PASS다.
그러나 자동 행·1×1 재귀 대조군은 p4에 UNIT 018을 앞당겨 실패했다. 전체 3129는
246 PASS·3 FAIL·1 ignore이며 최종 PASS로 보고하지 않는다.

원 독립 PDF에 연결한 `assert_recursive_reflow_child`는 셀 내부 재귀 viewport에서
바깥 위 여백을 매 쪽 반복하고 문단 오프셋만 첫 조각에 적용함을 검사한다. 행 사이 컷은
이 물리 viewport가 아니므로 first/last 유닛만 여백을 소유한다. 실제 NestedTableCut의
셀 내부 시작·끝 컷을 공통 판정으로 만들고, 요구 높이·edges 예약·전체 흐름 원점·fallback이
같은 결과를 소비하게 한다. 문단 앵커의 첫 소유와 물리 바깥 프레임의 열림을 분리한다.
블록 슬롯을 셀 유닛으로 해석하지 않는다. 정확한 기존 검사와 독립 출력은 보존한다.

3133의 실제 프레임 대조에서 마지막 두 입력의 하락은 page 2에서 소비한 물리 높이가
base보다 100HU 작아, page 3의 빈 꼬리가 그만큼 늘어난 차이다. 3134의 실제 원본
속성은 그림 띠·tail·follower 본문의 보이는 모든 문단이 NO_LS이며, 분할197쪽의 본문은
1,330개가 저장 좌표를 갖는다는 것을 확인한다. 원본 저장 쪽 프레임의 부재를 재구성한
저장 사다리의 존재로 대체한 `rowbreak_fragment_bounds`의 가정이 잘못됐다.
내용 없는 저장 앵커 줄은 본문 좌표계의 증거가 아니다. 실제 저장 본문 좌표가 있는
재구성 경로는 현재 예산을 유지하고 NO_LS 일반 격자는 기존 content/blank 하단을
구분한다. 첫·끝 빈 띠의 높이와 뒤 문단을 다시 대조한다. 확대 판독에서 원본 2쪽의
89.14%와 행·본문 위치 차이도 확인했으므로 별도 미달로 보존하며 보정을 계속한다.


### 빈 호스트 글줄의 첫 조각 소유와 후속 문단 — 2026-10-24

Native201의 실제 빌드는 3138이며 269.86초·종료 코드 0이다. 첫 3135는 실행기 사본의
feature 이름을 잘못 바꿔 Cargo가 컴파일 전에 거절한 오류다. 실패 로그와 수정 실행을
분리해 보존했다. Studio 전체 Node 3139는 2,461 PASS·0 FAIL·2 기존 skip이며 두 건은
pkg-node 미준비로 실행하지 못한 실제 WASM 왕복 검사다. 후보 WASM 준비 뒤 별도 수행한다.

3137에서 9개 원본/PDF 쌍을 양쪽 고정 Native로 전쪽 대조했다. 자동 행·1×1 재귀 입력의
후보 최저는 98.86792%, 재생성 주변 프레임은 98.18859% 이상이고, nested-split은
96.66160%다. terminal-tail의 빈 꼬리도 98.46727%로 회복했다. 원본 그림 띠는 여전히
89.14359%로 미달이며 base도 89.14610%다. 준비 완료로 판정하지 않고 원인 보정을 계속한다.

3133/3141에서 follower p3의 Table/Cell bbox는 새 base와 동일해졌지만, 뒤 본문은
81.1→86.6px로 내려갔다. 같은 FLOW_END의 확정 전진 24.413…px 뒤에
`finish_paragraph_float_flow → ParagraphFloatPlacement::paragraph_end`가 첫 호스트의
계산 글줄 29.88…px를 다시 적용했다. 생산은 `computed_host_frame::resolve`, 소비는
`prepare_table_fragment_budget`의 다른 frame 복제 → `commit_fragment`의 최종 상자
등록 → 문단 종료이며, clone이 첫 글줄 소유를 그대로 반복한 것이 원인이다.
이어받은 ComputedEmptyHost는 원래 글줄을 다시 소비하지 않고 EmptyHost의 확정
물리 끝을 사용한다. 아직 첫 조각을 소비하지 않고 이월한 경우에는 계산 글줄을 유지한다.
정확한 기존 검사·독립 PDF·실제 후속 본문 위치를 다시 확인한다.


### 리베이스 뒤 현재 판정과 직접 판독 잔차 — 2026-10-24

현재 code source는 `35911dd623664de14dcb3f6c1de843a4bdce8216`, base는
`9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`다. Native202 빌드 3142는
270.19초·exit 0·source_unchanged=true다. 기존 50개 모듈의 3143은 전체
249 PASS·0 FAIL·1 ignore다. 테스트·기대값·baseline은 이번 보정에서 변경하지 않았다.

3144의 전쪽 Native 대조는 같은 9개 입력과 독립 PDF 각 52쪽이다. 후보 최저는
nested-split 96.66160%, terminal-follower 95.73533%, terminal-tail 98.46727%,
두 orientation 재귀 대조군 98.86792%, 두 generated 재귀 대조군 98.18859%다.
follower p3의 raw는 100%이며 표·후속 본문의 같은 영역을 3148 review/overlay로 남겼다.
원본 그림 띠는 p2 89.14359%이며 base 89.14610%와 별도로 최소 gate 미달이다.
이 수치들을 fresh WASM 또는 최종 시각 수용으로 표시하지 않는다.

Native201의 #8191 전49조건·1,772쪽은 모두 90% 이상이고, 3149의 Decimal 대조에서
2%p 초과 하락은 0쪽이다. 현재 Native202의 같은 전쪽 검증은 3145에서 진행 중이다.
Native201 결과를 Native202의 완료 증거로 대체하지 않는다. #8178의 Node/실제 browser,
#8195의 양쪽 Node와 새 base 실제 browser 증거는 앞 기록과 연결하며 후보 fresh browser,
pkg-node 두 실제 왕복 검사, 최종 전체 Rust/lint는 아직 완료 전이다. PR 생성은 사용자
지시에 따라 계속 대기한다.

3148의 최신 nested-split p5 review와 standalone overlay를 실제로 열었다. raw 96.66%여도
내부 표의 위 선·가운데 정렬 글자가 PDF보다 위에 있는 차이가 보인다. 현재 정식 grid
검사는 첫 조각 뒤 outer top을 0으로 기대하지만, 이 직접 PDF 판독을 그 기대값의 근거로
대신할 수 없다. 셀 내부 컷 대조군의 반복 물리 프레임 계약과 행 사이 컷의 실제 소유를
생산 유닛·측정 높이·최종 원점에서 다시 연결한다. 이미 예약된 여백을 paint가 잃었는지,
측정까지 달라진 것인지 먼저 구분하며 임의 좌표 보정이나 기대값 완화로 닫지 않는다.
현재 249 PASS를 이 시각 잔차의 해결로 보고하지 않는다. 관련 Native/fresh WASM 전쪽과
독립 기대값 조건을 충족한 뒤에만 필요한 회귀 변경을 한다.

리베이스 전에 별도 보존한 수식 명령 작업 `d9bb9772b0561cb22cc2d4866b5be878ecb3c36f`는
아직 적용하지 않았으며 보존돼 있다. 3146의 read-only patch preflight는
legacy_coordinate_plane.rs의 현재 bool 반환·HFT 우선순위 보정과 자동 적용이 충돌함을
확인했다. 현재 source에 부분 적용하거나 그 파일을 되돌리지 않았다. 이 작업도 최종 PR
준비의 완료된 구현으로 표시하지 않는다.

### 제출 전 남은 보정 범위 확정 — 2026-10-24

작업지시자의 최신 지시는 원본 그림 띠 p2 89.14359%와 중첩 표 위치 차이의 보정 후
PR 준비다. Visual Sweep 및 지침 변경 커밋 ea212595c7은 cc5e4c34b0에서 전부
되돌렸으며 해당 파일을 추가 수정하지 않는다. 새로운 개선 항목으로 범위를 넓히지 않는다.

3162/3168은 원본의 같은 독립 한컴 Print와 Native202를 다시 추적했다. 원본 row5의
첫 조각은 일반 글줄 8개를 소비한다. 현재 row 단위 분류가 뒤 문단의 중첩 표까지
포함해 일반 글줄이 아니라고 판정하면서 선언 최소의 빈 물리 공간을 이 조각에서
예약하지 않는다. row_step의 분류를 실제 물리 예산으로 선택한 컷에 연결한다.
예약·PageItem의 부분 행 상자·이어받는 잔여는 기존 공통 전달 경로를 사용한다.
독립 PDF의 p1 row5 수평 경계는 950.8000–1061.3987px, p2는
58.4960–243.4133px다. 이 관측값은 구현 상수로 넣지 않는다.

3150/3151의 nested-split p5는 원본 첫 소유 유닛의 내부 셀 컷이 행 범위로 접히며
바깥 위 여백 소유가 사라진다. 원본 첫·끝 유닛과 투영 컷을 함께 조회하는 공통 helper를
부분 내용 측정·mixed 물리 예약·실제 자식 원점이 소비하도록 연결한다. 첫 문단 오프셋과
이어받기 물리 여백은 별도 소유로 유지한다. source patch와 각 파일 해시를 보존한
Native203의 실제 빌드는 3165다. 3163은 실행기 사본의 경로 치환 오류로 Cargo 시작 전에
실패했고, 로그를 보존한 뒤 경로만 고친 3165를 실행했다. 아직 보정 성공으로 판정하지 않는다.

보정 전 고정 source 35911dd623의 fresh WASM053은 3154에서 정상 종료했다.
실제 wasm-bindgen 및 wasm-opt `-O` 실행을 확인했고 표준 release·source_unchanged가
모두 참이며 총 1244.66초다. 이 package를 Native203의 fresh 증거로 표시하지 않는다.
3161의 한컴 재저장 주변 대조군 3개는 실제 Print가 완료됐으나 Native에도 별도 차이가
있어 정상 수용 근거로 사용하지 않는다. 해당 자료를 원본 NO_LS 입력의 출력으로 대체하지 않는다.

### 원본 90% 미달과 이어받기 원점의 재검증 — 2026-10-24

Native203 3165는 320.02초·exit 0이다. 일반 글줄 소유를 조회하려고 재사용한
`row_cut_uses_outer_line_frame`은 개체 viewport도 포함하므로 이 분류에는 맞지 않았다.
3166에서 원본은 89.66280%로 남았고 orientation grid는 19.69562%로 후퇴했다.
기존 3167도 246 PASS·3 FAIL·1 ignore다. 해당 실패를 숨기지 않고, 실제 선택한
유닛에 일반 글줄만 있는지 별도로 확인하는 query로 수정했다. 독립 PDF의 원본
p2 시작 58.496px는 본문 상단과 overlay 바깥 위 여백의 합에 대응한다. 일반 글줄과
재귀 viewport가 같은 문단 추종 물리 프레임을 열도록 예약·paint의 공통 소유를 연결했다.

이 수정의 Native204 3171은 267.17초·exit 0·source_unchanged=true다. 빌드 시 source는
cc5e4c34b0 위의 dirty patch이며 `source_matches_commit=false`를 명시했다. 실제 patch는
`native204-source-3171.diff`, 파일 해시와 producer는 해당 실행 receipt에 있다.
3172에서 원본 2쪽은 99.78422/99.99896%, nested-split 전6쪽은 최저 98.35546%,
이어받기 p5는 99.99880%다. 나머지 자동 행·1×1·generated 대조군은 전쪽 98% 이상이다.
fresh WASM의 이 source 재검증은 아직 남아 있다.

3175의 review·standalone overlay를 직접 열었다. p5 위 여백은 회복했고, p4의 마지막
빈 행 하단에는 PDF와 차이가 남아 있어 점수만으로 완료 판정하지 않는다. terminal-tail의
세 번째 빈 조각도 68.30196%로 후퇴했다. 원본 Print는 그 빈 물리 조각의 상단에도
58.496px의 같은 outer frame을 갖지만 후보는 내용 유닛이 없다는 이유로 56.693px에
그렸다. 모든 내용 컷을 소비한 NO_LS 행의 남은 물리 띠도 같은 프레임을 열도록 보정했다.
이 마지막 보정은 Native204의 검증 범위에 포함되지 않으며 다음 producer로 다시 확인한다.

3173의 기존 50개 모듈은 248 PASS·1 FAIL·1 ignore다. 남은 기존 grid 검사는 후속
자식의 outer top이 0이어야 한다는 기대값이며 현재 독립 Print와 맞지 않는다. 테스트·기대값은
아직 변경하지 않았고, 새 source의 양쪽 시각 선행 조건을 충족한 뒤 필요한 관계 검사를
새 이름으로 마련한다. 원본·중첩 표·후속 본문과 정상 대조군의 검증을 함께 유지한다.

3103의 원본·nested-split·follower Print PDF를
`pdf/pr7518_review/print-20261024/`에 해시 그대로 보존했다. 원본 입력 SHA, engine 2020
11.0.0.9136, Print method 0, 전체 쪽수와 job ID를 provenance.json에 기록했다.
이전 PDF와 실패한 terminal-tail MCP job은 보존하며 기준 파일을 조용히 덮어쓰지 않는다.

### Native205 확인과 중첩 표 마지막 빈 행의 추적 — 2026-10-24

3176은 d007b6d1f3의 깨끗한 source에서 246.46초, exit 0으로 완료됐다.
3178은 후보 9개 조건 전52쪽이 모두 90% 이상이며 원본 2쪽은
99.78422/99.99896%, nested-split p4/p5는 98.35546/99.99880%다.
terminal-tail의 빈 세 번째 조각도 복구했다. 3179는 기존 집중 회귀
248 PASS·1 FAIL·1 ignore이며, 남은 실패의 과거 outer top=0 기대값은
독립 Print와 맞지 않아 아직 변경하지 않았다. 이 결과는 전체 Rust나 fresh
WASM의 완료를 뜻하지 않는다.

3180은 같은 producer로 실제 컷과 바깥 프레임 예산을 추적했다. 중첩 표 p4의
원자 행은 루트 프레임 여백을 열지 않고 386.613px의 예산을 사용한다.
첫 행 앞 공간을 포함한 자식 3행의 예약은 379.813px이며 부모 패딩을 더한
383.573px를 그대로 그린다. 독립 Print는 같은 3행을 보존하면서 마지막 행의
빈 최소 공간만 줄이고 바깥 프레임을 닫는다. 다음 단계에서는 실제 내용 컷을
유지하면서 마지막 행의 내용 최소·물리 예산·바깥 띠를 하나의 결과로 만들고,
예약과 자식의 마지막 행 배치에 같은 결과를 전달하는 가설을 확인한다.
선만 옮기거나 내용 밖을 clip하는 보정, 문서 ID·독립 관측 좌표 상수는 사용하지 않는다.

### 원자 중첩 행의 마지막 빈 최소와 바깥 프레임 보정 — 2026-10-24

3182는 d007b6d1f3 위의 공개 patch를 Native206으로 빌드했으며 250.79초,
exit 0·source_unchanged=true·source_matches_commit=false다. 3183은 기존
9조건 전52쪽 모두 90% 이상이다. 원본 p1/p2는 99.78422/99.99896%,
nested-split p4는 98.35546%에서 100%, p5는 99.99880%를 유지했다.
자동 행·혼합 셀과 generated 3종도 기존 쪽수와 90% 이상을 유지한다.
3185의 동일 원본·Print review와 standalone overlay를 직접 판독했다.
p4 마지막 행 하단의 큰 차이가 사라졌고 앞 두 행·텍스트 순서도 보존된다.
3186의 실제 owner 상자는 y=58.6, h=379.1이며 자식은 y=80.3, h=352.0이다.
p5 자식 원점은 y=64.2로 유지한다. PDF 관측값을 코드 상수로 넣지 않았다.

새 reflow_nested_band 공통 결과는 source 컷·마지막 행의 내용 최소와 선언 최소·
물리 예산으로 부모 상자와 자식 마지막 행 높이를 함께 만든다. 실제 물리 컷
생산자는 수용한 마지막 행을 같은 결과로 반환하고, row_step은 physical_height를
예약·이어받기 상자에 전달한다. table_partial은 같은 컷과 수용 상자로 조회한
last_row_height를 자식의 기존 end_row_height_override 소비 지점에 전달하며,
그 뒤 마지막 행 접기가 수용 상자를 다시 줄이지 않게 한다. 후행 flow 띠와
viewport의 닫는 띠를 선언된 빈 행 높이와 구분한다. 글자를 자르거나 선만 옮기지 않는다.

3182 이후 실제 적용 계약을 명시했다. 부모 칸 예산을 공유하는 NO_LS·비TAC·
TopAndBottom·RowBreak 자식만 해당하며 독립 무제한 float, 저장 줄, 셀 내부 컷,
걸침·블록 슬롯과 복수 활성 소유 칸은 기존 원장을 유지한다. 이 계약 정리는
Native206 자체에 포함되지 않았으므로 다음 깨끗한 commit producer와 fresh WASM에서
다시 대조한다. 시각 지침·Visual Sweep·기존 테스트 기대값은 변경하지 않았다.

### 종료 경계의 양쪽 시각 검증과 새 회귀 — 2026-10-24

4adb7faf48의 깨끗한 source를 Native207(3187)과 표준 release fresh WASM054(3191)로
검증했다. Native는 255.50초, fresh WASM은 1084.98초에 exit 0으로 종료했다.
실제 wasm-bindgen 0.2.127과 wasm-opt `-O` 실행 및 source 불변을 확인했다.
3188/3193의 9조건 전52쪽은 양쪽 모두 90% 이상이고 독립 PDF와 쪽수가 같다.
원본 p2는 Native 99.99896%, nested-split p4는 양쪽 100%, p5는
Native 99.99880% / fresh WASM 100%다. 3196/3199의 review·standalone overlay를
직접 판독해 마지막 빈 행과 이어받기 원점의 차이가 해소됐음을 확인했다.
제출 자료는 `mydocs/pr/assets/issue_7500_reflow_closing_band_20261024/`에
원문 TSV 18개, 대표 review·overlay 12개와 입력·PDF SHA를 보존했다.

이 선행 조건을 충족한 뒤 기존 outer top=0·고정 행 높이 검사를 새 이름의
소유·순서·보존·부모 포함 검사로 교체했다. 완료된 p4/p5의 독립 Print 96 DPI
경계는 별도 새 이름 검사에서 ±2px로 보호한다. 기존 나머지 검사의 동작은 유지하고
공용 render-tree helper를 분리해 case 998줄, helper 387줄로 정리했다.
3203의 실제 집중 실행은 250 PASS / 0 FAIL / 1 ignore다. 3204는 같은 두 새 검사를
수정 전 Native205와 수정 후 Native207에서 실행했다. 수정 전에는 자식 닫는 띠의
부모 포함 위반과 p4 부모 종료 440.3px / 독립 437.5507px 차이로 각각 FAIL,
수정 후에는 모두 PASS였다. 입력 누락이나 실행 환경 오류로 실패한 것이 아니다.

3192의 광범위 Native 검증은 49조건 1772쪽 전체가 90% 이상이며 독립 PDF 쪽수와
일치한다. 최저는 original-sample11-user-print p50의 92.42855%다. 3197은 고정한
base 대비 2%p 초과 하락 0쪽과 0.1%p 초과 하락 44쪽을 구분해 기록한다.
광범위 fresh WASM(3194)과 전체 Rust·최종 lint는 아직 완료로 판정하지 않는다.

Studio 3195의 누락 font·sample과 3198의 npm/hwpctrl-ocx 누락은 검증용 source
사본의 준비 오류였다. root의 사용자 변경은 되돌리지 않고 정확한 source blob과
공용 입력·font 경로를 보완했다. 3198 재실행은 Node 2463 PASS / 0 FAIL / 0 SKIP,
3201의 TypeScript·production build와 실제 Chromium 명령 복구 검사는 exit 0이다.
브라우저 로그의 주입 오류는 #8104 테스트가 의도한 실패이며 결과 JSON에서 복구와
기본 오류 처리 억제를 실제로 확인했다. 626d21cd33의 요청된 Vite 경고 설정은
3205/3206에서 다시 검증했다. production build의 세 대상 경고는 없어졌고
Node 2463개와 실제 브라우저 검사가 통과했다. 런타임 번들 해시도 기존과 같다.

3207은 Cargo·rustc·wasm-pack·wasm-opt가 모두 종료됐고 lsof 열린 참조가 없음을
확인한 뒤 24시간 이상 지난 debug/incremental의 정확한 하위 캐시 114개만 정리했다.
약 4.36GiB를 확보했다. 공유 target, deps, fingerprint, 실행 파일은 보존했다.
삭제 목록과 전후 여유 공간은 ignored output의 cleanup receipt에 남겼다.

3209의 전체 검증은 suite 준비 단계에서 중단됐으며 Cargo는 실행되지 않았다.
helper 분리로 생긴 path 속성을 수집기가 독립 target 예외로 분류해 49/48개가 됐다.
동일한 기존 helper 모듈의 허용 방식에 따라 해당 case만 moduleIntegrationOverrides에
등록한다. 예산은 늘리지 않고 기존 28개 suite 안에 통합한다. 3210 재준비는
1805 sources / 7839 static test attrs / 28 suites + 20 exceptions = 48/48개로
통과했다. 전체 검증은 이 정책 보정까지 커밋한 별도 Rust 검증 worktree에서 실행한다.

### 광범위 양쪽 전쪽 검증 완료와 전체 Rust 컴파일 보정 — 2026-10-24

3194의 fresh WASM 49조건 1772쪽은 실제 종료·exit 0·overall_pass=true다.
최저는 original-sample11-user-print p50의 92.47460%이며 모든 쪽이 90% 이상이고
독립 PDF와 쪽수가 같다. 같은 base의 TSV와 십진수로 비교한 최대 하락은
Native 0.46069%p / fresh WASM 0.49098%p다. 현행 2%p 초과 하락은 양쪽 0쪽,
별도 진단용 0.1%p 초과는 Native 44쪽 / fresh WASM 29쪽이다. 양쪽 base/후보의
TSV 원본 196개와 입력·PDF·TSV SHA, 전쪽 비교표를
`mydocs/pr/assets/semanticist21_final_20261024/`에 보존했다. 지침이나 판정 도구를
변경하거나 이 수치를 반올림해 하락 0쪽으로 바꾸지 않았다.

3211/3212는 별도 worktree의 초기 준비와 Cargo의 상위 workspace 자동 탐색 때문에
본 검사 전에 실패했다. worktree를 기본 저장소 바깥
`/home/tsjang/rhwp-rust-review-semanticist21-20261024`로 옮겼으며 고정 공유
`/home/tsjang/rhwp/target/pr-review`는 그대로다. 전체 tracked 입력을 checkout했고
source diff가 없음을 확인했다. 3214의 suite 준비와 전체 포맷 검사는 통과했다.

3214의 전체 Rust 컴파일은 cfg(test) helper에서 인자 불일치 7건을 검출했다.
원인은 리베이스로 반영한 #8191(6097253468)의 dash plan helper와 현재 브랜치의
9d13ca7a9e의 trailing_note_slots 계약이 내부 unit-test wrapper에서 연결되지 않은
것이다. 고정 base에는 note 슬롯 인자가 없고 현재 운영 함수에는 있으므로 base 자체의
오류로 보고하지 않는다. wrapper에 같은 인자를 전달하고 주석 없는 대조군은 0으로
명시한다. assertion·기대값·운영 함수는 바꾸지 않는다. 큰 기존 조판 파일의 테스트
helper 인자 3줄만 보정하며 이 오류 때문에 전체 조판 파일을 재구성하지 않는다.
수정 후 전체 Rust와 lint는 다시 실제 실행해 확인한다.

3217은 완료된 과거 Native193/195/196/197/200/201/202 전쪽 실행의 열린 파일
참조가 없음을 확인했다. 그 실행들의 PNG·render-tree·SVG 임시 캐시 37212개를
정리해 약 4.23GiB를 확보했다. 현재 base/후보, 원문 TSV·실행 로그·review·overlay는
보존했다. cache 확장자 검사에서 중단된 3216은 삭제 전에 실패했고, 수정된 3217의
실제 삭제 목록과 확보 공간은 output receipt에 있다.

### 전체 컴파일의 이름 충돌과 package 소비 확인 — 2026-10-24

3218은 인자 보정 후 실제 전체 컴파일을 571.48초 실행했으나 OLE 회귀 모듈의
E0618 1건으로 종료됐다. #8191의 새 SVG 문자열 변수 visible_text가 현재 브랜치의
5544aba532의 표시 문자열 helper를 가렸다. 지역 변수만 svg_visible_text로 바꾸고
검사 대상·기대값은 유지한다. 큰 기존 파일의 이 이름 충돌만 고치며 전체 재구성으로
범위를 넓히지 않는다. 다음 준비 컴파일에는 --keep-going을 사용해 나머지 target의
컴파일 오류도 한 번에 수집한 뒤 정식 전체 nextest를 실행한다.

3218 실행기의 처음 head 필드는 checkout 종료 전에 읽은 c0722f7921이었다.
실제 컴파일 전 worktree의 b0a05a86fe와 source diff 없음·바뀐 unit source 해시를
독립적으로 확인했으며 3219 audit에 남겼다. 실행 로그를 이전 head의 검사로
표시하지 않고 이 source audit과 함께 사용한다. 현재 오류는 테스트의 컴파일
실패이며 전체 Rust PASS가 아니다.

3220에서 승인된 원 PR 45개 head가 모두 그대로임을 원격으로 재확인했다.
원 PR 테스트 파일은 전부 현재 경로에 있고 실제 suite 연결을 개별 로컬 리뷰 초안에
추가했다. 원격 리뷰·추가 체리픽·PR 생성은 수행하지 않았다.
3221의 @rhwp/editor unit, frontend WASM binding·embed 계약, TypeScript 타입,
npm pack --dry-run과 실제 Chromium embed 검사는 모두 exit 0이다. 동결 4adb7faf48의
검증 사본을 사용했으며 이후 source 변경은 cfg(test) helper 또는 테스트 파일뿐이다.

3222에서 기본 pkg와 public의 생성 WASM도 fresh054와 해시를 맞췄다.
public/rhwp.js는 이미 fresh054와 같아 tracked 변경이 없었고, 이전 생성 파일은
output에 백업했다. source 입력과 실행 로그는 보존했다. 3223 unit-tier 사전 검사도
exit 0이며 최종 전체 lint·검증의 완료를 대신하지 않는다.

### 전체 11773건의 실제 실패 수집과 보충 측정 공통 원인 — 2026-10-24

3224는 실행기에 넣은 cargo test --keep-going이 지원되지 않아 Cargo 컴파일 전에
종료됐다. cargo build --tests --keep-going으로 정정한 3226의 준비 컴파일은
212.59초·exit 0이다. 같은 후보 422f718486의 정식 전체 nextest는 966.16초 동안
11773개를 실제 실행해 11694 PASS / 79 FAIL / 45 SKIP, exit 100으로 종료됐다.
최종 summary의 실패 목록이 반복되므로 중복을 제거한 3233의 원장은 정확히79건이다.
lint·Native Skia 후속 gate는 실패 후 자동 실행하지 않았고 PR 준비 완료로 표시하지 않는다.

3233은 Visual Sweep과 같은 시스템·사용자·assets 폰트 경로를 명시해 작은 실패25개를
기존 실행 파일로 재실행했다. 25개 모두 같은 실패였으므로 글꼴 경로를 임의로 바꾸어
해결됐다고 주장하지 않는다. 원본·Print·원 해결 PR과 현재 base/후보를 함께 대조한다.
실패는 43개 Rust 파일에 걸쳐 있으며 보충 측정16건, 차트12건, 표시/원문 비교와
실제 배치·corpus 증가를 구분해 일괄 분석한다. 한 기대값을 고칠 때마다 전체 빌드를
반복하지 않고 같은 원인의 변경을 모아 재검증한다.

보충 측정의 첫 공통 원인을 실제 소비 경로로 확인했다. 7924a87937은 선언 체인의
다른 face에서 빌린 폭을 declaredGlyphFallbackMetric으로 구분하고 backend 측정을
허용한다. 리베이스로 반영한 #8191의 accepts_prepared_glyph_advance는 기존
heuristicHalfwidth·metricNarrowPunctuationOverlay만 수용해 이 경로를 누락했다.
현재 collection과 실제 전진폭 소비가 모두 이 helper를 사용한다. 세 미확인 출처를
하나의 조건에 통합하고 사용되지 않는 중복 requires_backend_measurement는 제거한다.
실제 원 face의 DB 글리프 hit·명시적인 HWP 폭 규칙은 계속 제외한다. 테스트 assertion과
공급자의 독립 입력값은 바꾸지 않는다. 이 운영 코드 변경은 아직 실제 재검증 전이며
이전 4adb7faf48의 TSV를 새 source의 시각 완료 증거로 대신하지 않는다.

3230에서 issue7216/tall_table_before.hwpx의 실제 한컴2020 Print가 성공했다.
#5715의 축약 HWP는 DocInfo 구역 수2 / 실제 BodyText 스트림1개가 달라 MCP가
preflight에서 거절했다. 원본을 보존한 채 3234에서 DocInfo 평문 offset4의
구역 수만1로 정정한 별도 진단 대조군을 만들었다. 그 밖의 모든 CFB 스트림은
바이트가 같고 원본 SHA도 보존했다. 3235에서 이 대조군의 실제 한컴2020 Print가
성공했다. 대조군을 원본의 통과로 바꾸어 보고하지 않는다. 3236에서 원본 과학 시험지,
각주1937, tall-table7216 및 이 별도 대조군을 같은 base/후보의 Native/fresh WASM으로
비교하고 있다. 해당 결과를 확인하기 전에는 회귀 기대값이나 baseline을 갱신하지 않는다.

3227은 현재 base·보정 전후 producer를 보존하고, 열린 파일 참조가 없으며 24시간
이상 지난 이전 char-overlap producer 복사본155개만 정리했다. 약11.47GiB를
확보했고 공유 target은 건드리지 않았다. 삭제 목록과 전후 공간은 output에 남겼다.

# 표 바깥여백 소유의 독립 대조

이 문서는 중간 분석 기록이다. 최종 보호 검증이나 PR 준비 완료 보고가 아니다.
원본·기준 PDF·판정 점수·문턱은 변경하지 않았다.

## 선행 원 해결과 현재 경로

원 해결 #2052는 `issue2007_nested_cell_pagination_42065.hwp`의 1×1 중첩 셀을
물리 쪽에 분할했다. #8397은 원본 글꼴 프로그램 선택을 포함한 전쪽 기능 보존을 검증했다.
현재의 미달은 쪽수나 분할 경계를 변경하는 방식으로 해결하지 않는다.

실제 표 원점과 글리프 원점의 읽기 전용 대조에서 1쪽의 표 괘선과 본문은 같은 방향으로,
10쪽의 내부 표와 본문은 같은 수평 이동량으로 어긋났다. 글꼴 파일 차이만으로는
표 괘선의 이동을 설명할 수 없어, 표 바깥여백 한 필드만 바꾼 입력을 독립 Print로 대조했다.

## 독립 출력

[독립 영수증](independent-margin-receipt.json)은 원본·PDF SHA, engine, job, 쪽수와
벡터 위치를 기록한다. 한컴 2020 Print이며 웹한글기안기의 PDF가 아니다.
출력 PDF 3개는 `pdf/issue7732-outer-margin/`에 공식 보관한다. 진단 입력은
회귀 검사의 동일 원본 바이트 변경으로 재현하며 원본 파일을 덮어쓰지 않는다.
원본, 위 바깥여백 140HU→0, 오른쪽 141HU→0의 세 출력은 모두 17쪽이다.
같은 실행 환경에서 위여백 제거는 1쪽 표 윗변을 -1.439pt, 오른쪽 여백 제거는
10쪽 괘선을 +1.439pt 이동했다. 글자처럼 취급된 15쪽 표의 오른쪽 여백 변경은
관측한 벡터·본문 원점을 바꾸지 않았다. 이 대조 PDF로 기존 기준 PDF를 대체하지 않는다.

## 공통 소비 지점과 후보

- 수평: `compute_table_x_position`의 중첩 표 경로는 왼쪽 바깥여백만 차감했다.
  문단·단 기준 오른쪽 정렬 자리차지 표의 오른쪽 바깥여백을 같은 기준 폭에서 차감한다.
  TAC 표의 글줄 펜과 별도 overlay 정렬은 기존 계약을 보존한다.
- 수직: 원본 문단은 책갈피와 번호 재시작 제어를 표와 함께 저장한다. 두 제어는
  글줄의 잉크·공간을 점유하지 않지만 기존 빈 앵커 판정은 이를 가시 개체로 취급했다.
  HWP5 사양 4.3.10.6의 새 번호 지정과 4.3.10.11의 책갈피는 각각 카운터 설정과 이름 정보다.
  표시할 번호를 가진 `AutoNumber`나 본문 필드를 이 비점유 분기에 추가하지 않는다.
  `control_leaves_table_anchor_empty`에서 두 설정의 비점유 계약을 공유한다.
  이미 인증된 저장 원점을 기존 `ParagraphFloatPlacement`가 받아 `table_top`·
  `occupied_bottom`을 수용·전진·최종 paint에 함께 전달한다.
  앞줄 형상·문서별 수치로 원점을 추정하려던 후보는 실제 소비를 바꾸지 못해 제거했다.
  원문 오프셋이나 임의 페이지별 px 이동량을 수정하지 않는다.

새 `issue_7732_table_outer_margin_ownership`의 첫 두 관계 검사는 보정 전 모두 FAIL이다.
설치 글꼴을 가정하지 않고 입력의 여백만 변경하여 원점 이동량·폭·교차 축의 보존을 검사한다.
TAC 부정 대조도 포함한다. 공통 보정 후 최소 글꼴 검사 3건은 모두 PASS했다
(빌드 3분, 실제 검사 0.235초). 최초 후보의 누락된 함수 참조와 경로 미소비는
로컬에서 수정했고, 실제 소비 지점까지 검출한 실패 로그는 output에 보존한다.
Native 전17쪽·fresh WASM과 전체 보호 검증은 이 단계의 최종 소스에서
별도로 확인하며, 결과가 나오기 전에 통과로 표시하지 않는다.


## Native 전17쪽 재검증

`2749e491e843ec851de12467751ec200ab551014`의 실제 Native 바이너리로 같은 원본·기준을
재출력했다. [전쪽 TSV](native-after.tsv)·[생산/검증 기록](native-after-result.json)에서
17쪽 일치, 보호값 하락0쪽, 최저99.86%를 확인했다. 1·11·14·15쪽은100.00%, 10쪽99.86%다.
1·10·11·14·15쪽의 review·overlay를 모두 직접 판독했다.
[1쪽 review](native/p001/review.png)·[overlay](native/p001/overlay.png),
[10쪽 review](native/p010/review.png)·[overlay](native/p010/overlay.png).
전체 보호 검증·최종 Rust 게이트를 이 Native 결과로 대체하지 않는다.


## fresh WASM 전17쪽 재검증

같은 소스의 새 WASM으로 [전쪽 TSV](wasm-after.tsv)·[생산/검증 기록](wasm-after-result.json)을
확보했다. Native와 같이 17쪽 일치, 보호값 하락 0쪽, 최저 99.86%다.
1·10·11·14·15쪽의 review·overlay를 직접 판독했으며 원문 소속·순서의 새 누락·중복은 관측되지 않았다.
[1쪽 review](wasm/p001/review.png)·[overlay](wasm/p001/overlay.png),
[10쪽 review](wasm/p010/review.png)·[overlay](wasm/p010/overlay.png),
[14쪽 review](wasm/p014/review.png)·[overlay](wasm/p014/overlay.png).
Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며
Docker 최적화 빌드로 보고하지 않는다. 실제 Studio 소비 경로의 검증은 별도 기록한다.


## Gulim 공급의 판정 범위

[프로그램과 글리프의 구분 기록](gulim-supply-qualification.json)에 따라 Native에는 Windows 원본 5.03을 명시 공급했다.
WASM은 실제 선택 기록에서 기존 공급 TTF 2.21(`6ff0f495…`)을 확인했다. 두 경로가 같은 버전을
선택했다고 기록하지 않으며 2.21 또한 사용된 366개 글리프의 윤곽·전진폭이 모두 일치했다.
수용 기록의 TTC 전체 파일 SHA와 같다고 주장하지 않는다. 독립 Print가 사용한 366개 글리프는
Windows 5.03 원본의 윤곽·전진폭과 모두 일치하고 누락이 없으며 PDF revision도 5.03이다.
PDF 내장 FontFile2는 읽기 전용 대조에만 사용했고 새 폰트 생성·렌더링 입력으로 사용하지 않았다.
사용자 지시에 따라 원 TTC 위치를 다시 질문하지 않고 이 실제 공급 신원과 사용 글리프 대조를
분리해 기록했다. 현재 소스의 Native/fresh WASM 전17쪽 보호 하락 0쪽과 직접 판독을 함께 확인했다.
비공개 글꼴은 Git에 포함하지 않는다.

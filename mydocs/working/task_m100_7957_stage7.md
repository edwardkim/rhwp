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

## 제품 Native 검증

runtime `d6af03754`의 실제 Native Skia CLI SHA-256은
`20c30eb2b075023a3b838de640a17532c99677a52165787fdf6434f0f9fa41b6`다.
6개 원본 313쪽을 모두 새로 출력하여 전체 쪽수가 독립 원본 PDF와 일치했다.
12쪽은 88.96829→96.40680%이며 전체 Native TSV에 90% 미만·누락이 없다.
같은 원본/쪽을 색상 checkpoint와 비교한 점수 하락도 없다. F7 전쪽 TSV·원 PNG와
`native-score-summary.json`에 기록했다. 생성 저장본에 이름만 바꾼 앞선 진단과 구분한다.

본문의 간격 복원은 12쪽 외에도 operators 5–11쪽에서 개선을 보였다. 원점·표제 테두리·
줄 소속을 유지하면서 원본 PDF처럼 단어 사이가 구분되는지 직접 대조했다.
최저 90% 통과가 Type3 원 glyph의 완전 재현을 뜻하지 않는다. 그 glyph의 획과 원의
크기 차이는 여전히 fallback의 제한이며, 이름이 있는 H2hdrM의 문제와 구분한다.

기존 유한 규칙은 유지했고 새 이름 동일성 규칙은 다음 두 개다.

- `H2hdrM` → `HY헤드라인M`
- `HYHeadLine-Medium` → `HY헤드라인M`

글꼴을 공급하지 않은 Linux Studio에서 원본 글꼴이 저절로 설치되는 것은 아니다.
Windows의 실제 프로그램을 공급한 Visual Sweep과 일반 fallback만 가능한 환경을
구분한다. 검증용 Studio 탭에는 같은 원본 프로그램을 로컬로만 공급하고 그 hash를
기록한다. 파일을 저장소에 추가하거나 스토어 패키지에 포함하지 않는다.

### 동일 원인의 regular Malgun 별칭

같은 확대 비교에서 `MalgunGothic` regular의 Native 획이 지나치게 가는 것도 확인했다.
설치 파일 name table의 동일성으로 `MalgunGothic` → `맑은 고딕`을 별도 #7959 change
set에 추가했다. `malgunbd.ttf`의 `MalgunGothicBold`는 다른 굵기 프로그램이라는
반례를 기록하며 regular 이름으로 합치지 않았다. PDF가 추출한 별도 bold 속성은 유지한다.
다른 네 projection의 의미는 그대로다. 앞선 H2 두 이름 보정용 WASM 빌드는 최적화
단계에서 중단하여 이 추가 보정과 합친 source로 한 번 검증한다. 그 미완료 산출물을
fresh WASM 검증 결과로 사용하지 않는다.

font-rule 전체 수는 기존 833 + 새 이름 동일성 3 = active 836이다. 관련 Rust/Node
검사는 원래 830개 sealed 의미와 공개 change set만으로 기대 의미를 재구성하여
추가 규칙·기존 규칙·retired 이력을 모두 대조한다. 생성 harness의 소스 weight drift도
재생성·check로 해소했다. 이전 집중 실행은 0 tests로 종료되어 검증 통과로 세지 않는다.

일반 `MalgunGothicBold` 전체 face를 regular family로 합치는 것은 font name만을 가진
다른 입력에서 굵기를 잃을 수 있으므로 이번 이름 규칙에 넣지 않았다. 지금 문제였던
regular PS 이름의 잘못된 대체와 glyph 굵기 검증을 우선 완료한다.

### 직접 판정과 추적 연결 보완

사용자는 `975655b49`의 Docker WASM(`60a17a1b…`)을 실제 Studio에서 확인하여
12쪽의 **시각 판정 통과**를 승인했다. Native 96.40680%, fresh WASM 96.80514%다.
`E:\tmp\issue7958-font-review-20261017\index.html`에 원본·전후 PNG를 복사했다.
이 source의 전쪽 Native/fresh WASM은 모두 313쪽이며 각 문서에 90% 미만이 없다.

별도 public font trace 회귀에서는 새 규칙의 임의 서술형 ruleId가 기존 trace의 canonical
candidate identity와 연결되지 않아 1 FAIL을 검출했다. 아직 공개하지 않은 신규 change
set의 세 식별자를 기존 canonical 생성식으로 바로잡았다. 기존 규칙·선택 tuple·글꼴·좌표는
바꾸지 않고, 연결 검사를 완화하지 않았다. 모든 active layout-name 174개의 ID가 실제
trace 생성식과 연결되는 Node 검사를 추가하여 관련 검사 49 PASS다. 새 source의 public
Rust trace 및 최종 빌드 검증을 다시 수행하며 위 판정·PNG를 다른 source로 바꿔 기록하지 않는다.

### 간격 회귀의 공개 최소 재현

합성 단일 TJ 양성 대조군은 수정 전에도 통과하므로 결함 검출 증거로 채택하지 않았다.
실제 결함 경계인 서로 다른 text object의 위치 기반 간격을 가진 최소 Type3 PDF를
별도로 만들었다. 첫 줄은 물리 간격, 둘째는 연속 문자, 셋째는 음수 TJ kerning이다.
font bytes·원 문서를 포함하지 않는 생성본이며 독립 Poppler / Native / fresh WASM
전쪽 100%와 같은 배율 판독 뒤 정식 검사를 추가했다. 기본 fallback과 같은 HCR Batang
외부 프로그램을 명시적으로 공급한 합성 계약이며 일반 Type3 원 glyph 일치 증거로 확대하지 않는다.

같은 Rust 검사 소스를 보존한 수정 전 라이브러리에 연결해 실행하면 첫 줄의
`가나다라` / `가나 다라` 차이로 FAIL, `4d3ab26fe` 라이브러리에서는 1 PASS다.
IR만 보지 않고 최종 TextLine의 내용·위아래 순서·상대 점유 폭 및 HWP/HWPX 저장 후
재열기까지 검사했다. 과거 px 또는 hash 핀은 추가하지 않았다. 공개 입력과 생성 절차는
`pdf/issue7958/`, 원본 로그·TSV·PNG는 `f7/final/`, 검출 실행은 `f8/word-tests-*`에 있다.

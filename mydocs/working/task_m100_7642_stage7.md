# 이슈 #7642 — HWP5 저장 줄의 원문 간격과 좌표 권위

이전 단계: [쪽 테두리](task_m100_7642_stage6.md). 시작 head: `2153f7ef5`.
전체 실패 원본·시험지·수식·#1139 전체 범위를 유지한다.

## 추가된 검증 범위

사용자가 지정한 수식 입력 37개와 첨부한 회귀 목록 174개(42개 소스)를 검토 범위에
추가했다. 목록의 테스트는 원본 입력 23개를 참조한다. 수식 입력 목록과 테스트 목록의
범위는 서로 같지 않으므로 두 목록을 각각 보존한다. ignored
`output/pr-review/issue7642/equation-regression-scope-20261008/`에 입력 SHA와
원문 TSV, 검사 후보 목록을 기록했다. 숫자가 있는 모든 검사를 삭제하는 대신 실물의
고정 좌표·영역 선택을 문단·줄·개체 소유와 순서·포함·겹침 관계로 변경한다.
내용 수량·포맷·원본 속성의 독립적인 계약은 보존한다. 변경 전에는 해당 원본의 관련
전체 페이지 Native/fresh WASM 최저 90%와 확대 판독을 먼저 충족해야 한다.

사용자가 제공한 HML Print 기준은 실제 경로
`tests/fixtures/hml/exambank_math_equations_min-2024.pdf`이며 Hwp 2024
13.0.0.3901, 1쪽이다. 최초 Native 대조는 6.22797%로 미달했다. 직접 PNG와
PDF bbox를 확인하니 기준 PDF의 일반 본문 높이는 약 0.12pt이고 글자가 같은 x에
축소·중첩되며 네 수식만 정상 크기로 보인다. 이 HML은 기존 #4161·#4141이
RATIO/RELSIZE 자식의 생략을 검사하려고 만든 합성 입력이다. 사용자가 정상 수동
Print 출력임을 확인했으므로 이 PDF를 해당 원본의 유효한 기준으로 수용한다.
본문 축소·중첩은 기준을 거부할 사유가 아니라 RHWP의 원본 해석을 대조할 관측값이다.
원본·제공 PDF·실패 PNG와 `equation-37-reference-diagnostic.json`을 보존한다.
기존 스키마 기본값 계약과 한컴의 실제 입력 해석을 구분하고, 가시 본문을 임의로
없애거나 정상 생성 대조군의 통과를 이 원본의 일치로 보고하지 않는다.

### 언어 기출 첫 호스트 계획의 실패와 재보정

최초 `stored_float_host` 시도는 문단 앞 간격을 포맷 단계에서 먼저 제거하고 표 경로가
이미 수용한 뒤 문단의 소비 높이를 첫 호스트의 끝으로 되감았다. 원본 첫 쪽은
63.91561%에서 61.26497%로 악화했고 앞 간격 대조군도 미달이었다. 이 시도의
바이너리·소스 해시와 실패 출력은 `language-shared-host-provenance.json`,
`language-shared-host-first-native/`, `language-first-margin-after/`에 보존했다.
테스트나 기대값 변경으로 이 실패를 숨기지 않았다.

재보정에서는 표 배치가 사용하는 원래 포맷 간격을 유지하고, 실제로 전체 수용된 첫
PartialParagraph에 저장 줄 원점·끝과 앞 간격 0인 계획만 게시한다. 부분 문단 측정은
이미 앞 간격을 제외한 줄 전진량을 사용하므로 뒤 문단의 소비 높이를 다시 쓰지 않는다.
배치가 그 계획의 원점과 끝을 사용하며 첫 단의 저장 vpos 앵커도 같은 원점에 맞춘다.
`language-owned-prefix-provenance.json`의 Native 빌드는 exit 0(5분 3초)이다.
원본 첫 쪽은 **97.30%**, 앞 간격 raw 0·570·1140·2280 대조군은 모두
**97.26151%**로 통과했다. 원본 전체 15쪽 Native 최저값은 **94.90393%**이며
누락 쪽은 없다. fresh WASM은 추가 실행 중이다.
첫 쪽 review PNG를 직접 열어 제목 표·첫 단 본문·다음 문단·오른쪽 단의 순서를
대조했다. 같은 첫 호스트 영역의 288dpi 대조에서도 줄과 표 경계는 맞지만 글꼴 획과
글자 모양 차이가 남아 있다. 점수 통과를 정밀 시각 일치 완료로 확대하지 않는다.
fresh WASM과 필요한 추가 판독을 확인하기 전에는 회귀 변경 완료로 판정하지 않는다.

### OLE WMF 세로 표제의 출력 비율

`SO-SUEOP.hwp`의 OLE 미리보기는 MapMode 8의 비등방 WMF이며 source window는
4560×10260, 목적 개체의 원본 extent는 14816×49532 HU다. WMF image embedding의
`xMidYMid meet`가 두 축을 같은 배율로 묶어 세로 표제를 압축했다. WMF 두 경로의
목적 extent를 그대로 소비하도록 `preserveAspectRatio=none`으로 수정했다. EMF와
일반 bitmap의 적용 계약은 바꾸지 않았다. 읽기 전용 진단의 첫 쪽은 36.58541%에서
84.73761%로 개선됐지만 아직 미달이다. 이를 생산 출력의 통과 증거로 쓰지 않으며
새 생산 Native 전쪽 출력과 글꼴·본문 차이를 계속 검증한다.

### 수학 시험지의 고정 표와 본문 TAC 줄 소유

최신 Native의 `exam_math.hwp` 전체 20쪽 최저는 76.39626%,
`exam_math_8.hwp` 1쪽은 81.84285%, `exam_math_no.hwp` 20쪽 최저는
76.16969%다. 이 입력들의 기존 회귀는 아직 변경하지 않는다. 12쪽 review를
직접 판독해 왼쪽 본문 전체가 위로 이동하고 오른쪽 단은 같은 위치에 남는 것을
확인했다. ignored `math-page12-tree/`, `math-page12-pagination.json`과
`source-all-sections-inspect/exam_math.json`으로 실제 소유 경로를 대조했다.

구역 1의 p22는 컨트롤 0에 Page/Bottom 고정 표, 컨트롤 1에 본문 TAC 표를
갖지만 저장 글줄은 하나다. 그 줄의 높이 3623HU는 TAC 높이 2348HU와
아래 바깥여백 1275HU의 합이다. 다음 p23의 첫 원점 4371HU는 이 높이와
저장 줄간격 748HU의 합이다. 앞 문단 뒤 간격과 다음 문단 앞 간격은 모두 0이다.
실제 Native p23 원점은 단 시작에서 3623HU만 전진해 후행 간격을 버렸다.

생산 경로는 `flow_table::place → stored_tac::prepare_coanchored_first_line →
commit_stored_tac_control → inline_placements → Table paint의 advance_end`다.
일반 paint는 앞 고정 표 때문에 원본 줄 사영을 거부하고 `line_segs[control_index]`
폴백에서도 줄을 찾지 못했다. 고정 표는 자체 용지·쪽 원점을 소유하므로 본문 TAC
줄의 앞 흐름 개체로 쓰지 않는다. 원본 TAC 줄의 실제 소속과 완전한 바깥 상자를
확인하고, 다음 저장 원점이 연결되는 경우에 같은 계획으로 후행 간격 전량을
수용·배치한다. fit에 실패하거나 연결 증거가 없으면 이 새 고정 표 단축을 쓰지 않는다.
새 생산 출력과 정상 저장 대조군을 확인하기 전에는 개선 수치를 확정하지 않는다.

`fixed-decoration-tac-provenance.json`의 생산 Native 빌드는 exit 0(4분 48초)이다.
실제 p23 첫 본문 원점은 단 시작에서 3623HU가 아니라 4371HU만큼 전진한다.
12쪽은 **76.39626% → 95.23736%**로 개선됐다. 첫 쪽은 **83.57551%**로
미달이어서 해당 원본의 전체 통과나 회귀 변경 조건 충족으로 판정하지 않는다.
처음 Clippy 실행은 로그에 종료 문구가 있지만 tool exit 143이었으므로 성공 증거로
쓰지 않고 같은 명령을 재실행해 exit 0을 확인했다.

첫 쪽 제목의 글꼴 차이를 분리하려고 굴림 대체 글꼴의 임베드 사본만 Noto Sans KR
Regular로 바꾼 읽기 전용 진단을 수행했다. 결과는 84.11648%로 여전히 미달이어서
그 글꼴 변경을 생산 코드에 채택하지 않았다. 원본 속성·원점·Glyph 이동량을 바꾸거나
점수에 맞춰 글꼴 이름을 강제하지 않는다. 진단 원본·대체 글꼴·명령은
`math-gothic-regular-diagnostic/`에 보존했다.

## 독립 입력과 관측

`hwp3-sample16-hwp5-2018.hwp`는 실제 HWP5 CFB 저장본이다. 독립 한컴 Print와
한컴 2020 정상 재저장 진단에서 원래 문단 속성과 저장 줄을 대조했다. p69의 원문
spacing_before=1704HU, 첫 vpos=852HU다. p70은 raw vpos=4904HU다. 정상 재저장도
같은 값을 유지한다. Print 3쪽 첫 제목 baseline은 82.32pt다.

현재 파서는 오래된 작성일·문단/스타일 개수 비율로 HWP3 변환 계보를 추정하고,
spacing_before를 852로 줄인 뒤 vpos에서 그 값을 누적 차감한다. 따라서 p69의
vpos는 0, p70은 3200이 된다. Native 첫 baseline은 73.72pt, 3쪽 일치율은 46.26170%다.
실제 저장본의 값을 HWP3 비교 입력에 맞추어 바꾼 이전 가정이 독립 Print와 맞지 않는다.

읽기 전용 원문 필드를 복원한 모델 재생의 3쪽 결과는 다음과 같다. 원본 파일을
변경하지 않았고 실험 결과를 생산 코드·원본 전체 통과로 승격하지 않는다.

| 실험 | 3쪽 일치율 |
| --- | --- |
| 원문 vpos만 복원 | 67.25535% |
| 원문 문단 간격만 복원 | 50.75703% |
| 두 원문 값 복원, 기존 계보 좌표 경로 유지 | 46.68349% |
| 두 원문 값 복원, 일반 HWP5 캐시 경로 | 92.80141% |

마지막 재생의 첫 baseline 82.24pt는 Print 82.32pt와 같은 배치다. 두 원문 값만
복원하고 기존 좌표 보정을 남기면 문단 간격을 다시 계상해 일부 내용이 넘친다.
같은 값 생산 이후 실제 예약·배치의 별도 분기를 함께 고쳐야 한다.

## 생산에서 최종 소비까지

- 생산: `parser/mod.rs`의 ParaShape 절반 보정과 `normalize_variant_paragraph_vpos`.
  strict/lenient CFB 두 진입 경로의 같은 원문 값을 보존한다.
- 해소: `style_resolver.rs`의 HWP5 ParaShape /2 계약. 원문 1704는 유효 852HU다.
  추가 parser /2는 이 계약의 중복이다.
- 정책: `Document::layout_profile`이 계보와 컨테이너를 함께 해석한다. 실제 HWP5
  문서의 저장 기하를 오래된 작성일이나 계보만으로 HWP3 좌표 경로에 넘기지 않는다.
- 소비: section 진입·vpos·꼬리 예산, paragraph format/overflow, paragraph/table paint가
  같은 정책과 원문 줄을 소비한다. helper 이후 앵커 선택·배율·정규화도 실제 코드로 대조한다.
- 글꼴: HFT ASCII 호환과 실제 컨테이너 좌표 권위를 구분한다. 저장 줄의 독립 폭 증언을
  사용하는 기존 글꼴 대조군을 확인하고 폰트 이름만으로 배치를 맞추지 않는다.

## 검증과 기대값 변경 순서

1. 생산 Native로 원본 2018/2024 전쪽, HWP3 native, HWPX·편집 대조군을 재출력한다.
   실제 호출 경계와 쪽·문단·개체 소속, 누락·중복을 함께 확인한다.
2. 영향 전체 페이지의 최저 90%와 직접 확대 판독을 확인하고 fresh WASM도 같은 범위를
   대조한다. 월별·시험지·수식의 다른 실패 원인을 이 통과로 면제하지 않는다.
3. 관련 원본 미달 상태에서 기존 px/hash/baseline을 바꾸지 않는다. 독립 기대값으로
   필요한 검사만 문단·줄·개체 소유·순서·포함·겹침 관계로 바꾼다. 올바른 검사는 유지한다.
4. source가 바뀌면 영향 시각 증적·회귀·lint를 다시 수행한다. 전체 회귀·PR 준비·CI·merge
   완료로 보고할 수 있는 최종 범위와 아직 미달인 범위를 구분한다.

## 증적

ignored `output/pr-review/issue7642/failed-inputs-20261008/`의
`raw-spacing-inspect.log`, `raw-spacing-replay/scores.json`과 재생 SVG/PNG,
`hwp3-resave-spacing-result.json`을 따른다. 정상 재저장 job은
`df26dfe9-4019-49d2-bbf1-2405bf3682b6`, engine 2020 / 11.0.0.9136이다.
이는 간격·캐시 출처 진단용 HWP이며 원본 Print PDF를 대체하지 않는다.
TSV·실행 로그·중간 JSON·다음 실행 계획은 output에만 보존한다.

### 첫 생산 출력과 다음 소비 경계

`raw-hwp5-provenance.json`의 Native로 두 저장본을 각각 64쪽 비교했다.
2018 저장본은 3쪽 92.80141%로 개선됐지만 10쪽이 90% 미만이다. 1쪽 제목의
글꼴·폭과 12쪽 문단 원점 차이를 직접 PNG로 확인했다. 원문 보존만으로 전체
완료라고 판정하지 않으며, 기존 검사와 baseline은 아직 변경하지 않는다.

12쪽 p265의 첫 저장 vpos=568HU는 앞 간격 1136/2와 같다. 다음 p266은 앞
간격 0이고 vpos=2520HU다. `2520 - 568 = 1300 + 652`로 첫 줄의 실제 전진량과
일치한다. 그러나 `stored_first_margin_is_page_relative`는 다음 앞 간격도 양수여야
한다는 조건 때문에 같은 쪽 원점의 이 증언을 버렸다. 첫 줄은 자체 앞 간격으로
제자리에 놓이고 뒤 문단은 그 간격만큼 위로 이동했다. 다음 간격이 0이어도 저장
전진량과 맞으면 원점 증언을 수용한다. 이 공통 helper는 측정의
`vpos_snap_current_height`와 배치의 `build_single_column`에서 같이 소비한다.
명시적 쪽 나눔·합성 줄·도형 원점 제외 조건은 독립 계약에 따라 유지한다.

`zero-next-margin-provenance.json`의 재출력에서는 2018 저장본 미달이 10쪽에서
3쪽(1·35·42쪽)으로 줄었다. 최저는 82.95736%, 12쪽은 98.870%다.
2024 저장본은 11쪽에서 4쪽(1·18·35·42쪽)으로 줄었고 최저는 73.80016%다.
두 원본 모두 전체 64쪽을 비교했으며 쪽수 누락은 없다. 이는 Native 중간 결과로,
fresh WASM·전체 회귀·완료 판정의 대용이 아니다.

### 가운데 정렬의 말미 공백과 자간

1쪽의 실제 휴먼명조 TTF를 임베딩해도 85.68781%였으므로 글꼴 공급만으로
면제하지 않았다. 원본 글상자를 유지하고 첫 제목 자간만 -17·0·15%로 달리한
진단 입력 세 건을 한컴 2020에서 정상 저장한 뒤 Print했다. 각 1쪽이며 모든
Save/Print job이 성공했다. 첫 출력은 미달이어서 정식 회귀·fixture로 등록하지 않았다.
생성 코드·MCP 결과·PDF·Native PNG는 output의 `title-spacing-counter/`와
`title-spacing-native-before/`에 보존했다.

독립 Print의 첫 제목 x는 -17%에서 123.72pt, 0%에서 119.04pt,
15%에서 182.64pt다. 기존 Native는 각각 120.05·113.08·173.98pt다.
기준선은 Native 238.16pt, Print 238.20pt로 같다. `title-spacing-origin-comparison.json`
참조. 줄 끝 분리 공백의 폭과 마지막 가시 글자의 부호 있는 자간이 정렬 폭에
포함된 것이 가로 차이와 대응한다. 문단별 좌표를 더하지 않고 Center 줄의 말미
공백을 제외하며, 마지막 가시 글자 뒤 자간을 공통 측정 함수로 정렬 폭에서
제외한다. 글자 전진량·원문·줄 소속은 보존한다. 컨트롤 표제·겹침·표시 치환처럼
원문 좌표와 다른 경로는 이 증거로 확장하지 않는다. 새 생산 출력으로 대조군과
원본 전체를 다시 검증한 뒤에만 회귀 변경 여부를 판정한다.

42쪽 p681은 가시 글자가 없는 문단이지만 실제 공백 4개 뒤에 TAC 표가 있다.
`char_offsets=[0,1,2,3]`, `control_text_positions=[4]`다. 공백 전진량과 표의
문단 내 위치를 함께 보존해야 한다. 표 폭만 가운데 정렬하던 일반 Table 항목의
폴백에서 공백 전진량과 표 바깥 상자를 함께 정렬한다. 원본에는 확정 flow plan이
없었으며, 계획이 있는 경로는 그 계획의 좌표를 최종 원점으로 계속 소비한다.
`tac-row-table42/` 재출력의 42쪽은 98.64000%다. PNG를 직접 표시해 표의 좌우
괘선과 내용 순서를 대조했다. fresh WASM과 영향 대조군은 아직 미검증이다.

### 원형 그라데이션의 공통 물리 기하

제목의 실제 가로 원점을 보정한 뒤에도 세 대조군은 85.05·85.23·88.24%였다.
SVG의 `objectBoundingBox` 원이 직사각형의 가로·세로 비율을 따라 타원으로
늘어났고, Canvas는 이미 물리 좌표의 원을 썼다. 독립 Print는 원형이다.
`gradient_geometry::radial_circle`이 두 출력에 같은 중심과 반지름을 공급한다.
도형 bbox에서 계산하며 특정 페이지 좌표나 문서 이름으로 분기하지 않는다.
사각형·원뿔형 그라데이션의 별도 구현 완료를 주장하지 않는다.

`circle-provenance.json`의 생산 Native 빌드는 exit 0이며, 정상 Save/Print한
자간 -17·0·15% 세 대조군의 일치율은 각각 **99.93203·99.93239·99.94077%**다.
첫 대조군의 새 `review.png`를 표시해 제목 줄과 배경을 직접 판독했다.
같은 바이너리로 2018 원본 64쪽을 빠짐없이 비교한 결과 35쪽만 89.56113%다.
전체 통과로 판정하지 않고 기존 테스트·baseline을 계속 유지한다.

### 35쪽의 연속 TAC 표와 저장 진입 밴드

p602의 저장 줄은 vpos 8032HU, LH 2528HU이고 표 바깥 상자도 2528HU다.
p603의 원점 11128HU와의 차이는 `2528 + 568`이다. 568HU는 다음 문단의
앞 간격 `1136 / 2`다. 배치는 직전 TAC 뒤 앞 간격을 생략해 p603 괘선을
그 간격만큼 위로 놓았다. 측정에서도 p601의 앞 간격을 흐름 예약에서 제외한
커서를 실제 글줄 끝과 그대로 비교해 완전한 저장 줄 계획을 거절했다.

측정의 `stored_tac::prepare`는 앞 문단에서 제외한 간격의 실제 소비 기록을
조회하고, 앞 글줄 끝·후행 간격·다음 표 앞 간격이 원문 사다리와 맞는지 확인한다.
유효한 밴드는 표 원점·점유 끝을 한 번 확정해 `inline_placements`로 배치에
전달한다. 배치의 `layout_table_control_block`은 그 원점과 `advance_end`를
그대로 사용하고 기존 폴백에서 간격을 다시 추정하지 않는다. 합성·편집·높이가
성장한 표·각주·옆 어울림 등 기존 제외 계약은 유지한다.
`inherited-band-provenance.json`의 생산 Native 빌드는 exit 0이다. 원본 35쪽은
**89.56113 → 98.49032%**이며 새 review PNG를 표시해 본문 상자와 뒤 그림을
직접 대조했다. 같은 원본의 전체 64쪽과 fresh WASM은 재검증 중이다.

이후 같은 Native로 2018 원본 전체 64쪽을 재비교했다. 누락 쪽 없이
**최저 96.53982%**, 90% 미만 0쪽이다. 실루엣 전용 결과의 `not_evaluated`는
최종 승인 판정이 아니며, 변경 경계의 직접 판독과 fresh WASM은 별도로 확인한다.

원본 p600..605로 진단 입력 세 건을 만들었다. 첫 문단 앞 간격만 0·1704·3408HU로
달리하고 저장 줄을 지운 뒤 한컴 2020에서 정상 Save/Print했다. 같은 표를 공유하는
DocInfo 속성은 변경하지 않았으며 생성 절차는 `stored-tac-entry-counter.rs`에 있다.
세 독립 1쪽 대조군의 Native는 각각 **68.61680 → 99.32092**, **69.56904 → 99.33646**,
**67.08828 → 99.34009%**다. source 변경 전 바이너리에서 의도한 간격 결함으로
미달이었고 새 생산 출력에서 통과했다. 같은 source의 fresh WASM에서도
세 대조군은 각각 **99.32092·99.33646·99.34009%**로 통과했다.
web wrapper build exit 0, package BG SHA-256은
`c842e4d454fb440e6ad0b8479e78240b2e40df9d2237430237a5dbe20545cd20`다.
Studio JS/WASM과 루트 pkg의 SHA 일치를 확인했다. 정식 회귀 검사는 아직 남아 있다.

### 그림 차이의 엔진 교차 진단

2018/2024 원본의 18쪽 그림은 같은 WMF 바이트다. SHA-256은
`1927d7c64d345e4467f5936b3e9a28cfe1b395bf5a2301f5fef6fb1ff829cb5d`다.
두 원본을 각각 한컴 2020과 2024에서 교차 Print했다. 두 원본 모두 2020은
작은 이미지 123개와 벡터 draw 820개, 2024는 큰 JPEG와 1bit 마스크 두 개 및
draw 85개를 출력했다. 입력 변경만으로 생긴 차이로 단정하지 않는다.
WMF의 ROP·브러시·마스크 의미와 출력 경로를 추적 중이며 아직 원인을 해결한
것으로 판정하지 않는다. 교차 PDF는 진단용이며 저장 버전에 맞는 원본 기준
PDF를 교체하거나 90% 미달을 면제하지 않는다.

이후 진단에서는 현재 WMF 변환기가 연속 `PATINVERT → DPa → PATINVERT`의
1bit 흑백 브러시를 버려 바탕을 단색으로 칠하는 것을 확인했다. 원래 마스크의
검정 부분에만 브러시 색을 남겨야 한다. 실제 레코드의 8×8 비트와 팔레트를 읽어
같은 그림 SVG에 역흑백 마스크를 적용한 진단 PNG를 산출했다. 표준 `pdftoppm`
기준 PNG와 대조한 18쪽은 2020 **99.00788%**, 2024 **98.80888%**다.
별도 `pymupdf` 캡처의 99.03973·98.82697%와 섞지 않는다. 생산 코드 반영과
원본 전쪽·기존 #6469 대조군·fresh WASM 검증은 아직 남아 있다. 자동 점수 통과와
배경 색의 직접 판독을 구분하며 진단 결과만으로 기존 회귀 검사를 바꾸지 않는다.

비트 연산의 독립 정의는 [Microsoft MS-WMF TernaryRasterOperation](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/1605dd68-a635-4639-ab81-99ff3e3fc5a3)를 따른다.
같은 XOR 상자 안에 포함된 흑백 DPa는 마스크 밖에서 두 XOR이 상쇄되고
검정 부분에서만 브러시 색이 남는다. 포함 관계는 비트맵 크기나 특정 체커보드에
의존하지 않는다. 현재 생산 보정은 단색 첫 칠을 지우고 실제 중간 브러시를
역흑백 SVG 마스크로 연결한다. 기존 벡터 마스크도 같은 반전·색칠 경로를 쓴다.
서로 다른 마지막 상자·브러시·클립, 사이의 다른 그리기, 색 팔레트 등 원래 음성
대조군은 이 계약으로 수용하지 않는다. 기존 한 단위 halftone overscan 근사는
별도 남은 범위로 구분하며 일반 포함 관계의 근거로 승격하지 않는다.

기존 #6469 원본 슬라이스 `samples/issue6469/wmf_fill_shapes.hwpx`도 입력 그대로
한컴 2020에서 Print했다(job `8b4dbb87-df8f-4037-8706-7c910f29d8aa`).
첫 전체 Native 비교는 Chromium Navigation 30초 timeout으로 종료했으므로
시각 미달이나 통과로 세지 않는다. 동일 바이너리·원본·PDF에 기존 raster timeout
설정 `RHWP_VISUAL_RASTER_TIMEOUT_MS=120000`만 지정하여 별도 경로에서 재시도한다.

현재 WMF 마스크 보정 Native 바이너리의 SHA-256은
`21c1b70f9bfc6f836390a41d9983fd83d2c8e2a34a1c33601f25891e609ba0e7`다.
2018·2024 원본을 각각 전체 64쪽 비교한 결과 두 문서 모두 최저 **96.53982%**이며
미달·누락 쪽은 없다. 결과는 `stage7-final-full-native/results.json`에 있다.
이 파일명의 `final`은 당시 source를 동결한 이름일 뿐, 승인·검증 완료 판정이 아니다.
해당 source의 fresh WASM은 빌드했으나 전체 128쪽 비교는 아직 실행하지 않았다.

기존 #6469 원본 15쪽을 같은 Print PDF와 비교하면 2쪽이 변경 전 **86.10788%**에서
변경 후 **82.67783%**로 악화된다. 다른 쪽은 90% 이상이지만 이 대조군은 미충족이다.
기존 테스트·기대값은 수정하지 않았다. 새 마스크의 통과한 원본과 실패한 대조군을
합쳐 완료로 보고하지 않는다.

2쪽의 WMF 그림만 떼어 표시한 PNG와 전체 페이지의 같은 그림 영역은 픽셀 단위로
동일하다. 앞선 육안 진단에서 독립 그림에서는 아이콘이 보존된다고 생각한 것은
잘못이었다. 페이지 클립·글꼴 스타일 제거, SVG root 크기 명시, 반전 PNG와 투명
패턴으로 대체한 진단도 누락된 선을 복원하지 못했다. 이 진단 변경은 생산에 넣지 않는다.

생산 WMF 재생기는 음수 `RestoreDC`에서 저장 스택을 읽지 않고 현재 상태를 복사했다.
양수 번호도 0 기반으로 처리했고 복원한 상태 이후의 저장 상태를 폐기하지 않았다.
선택 객체와 SVG clip ID도 DC와 별도로 보관하여 복원되지 않았다.
[Microsoft SaveDC](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-savedc)와
[RestoreDC](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-restoredc)의
계약에 따라 같은 저장 시점의 모드·펜·브러시·클립을 함께 복원하고 저장 스택을 소비한다.
생성·삭제된 객체 표는 현재 재생 스트림의 자원으로 유지한다. 이 원인 수정의 실제
출력 효과와 반례는 아직 검증 중이며, 아이콘 누락의 해결이라고 단정하지 않는다.

실제 `RestoreDC` 보정 Native(SHA-256
`7e9acc63378005aeaca75234e3e06f83e1026044358f35f24791ca4ffd636897`)의
#6469 2쪽은 **82.67783%**로 이전과 같다. 저장 상태 결함은 존재하지만 이 페이지의
누락된 선을 해결한 것으로 쓰지 않는다.

### 메타파일 자르기의 생산·소비 경로

#6469 원본 image3의 `imgClip=(0,0,14682,13182)`는 전체 범위 15000을 줄인 선택이다.
`imgDim`은 없으며, 정상 한컴 HWP 저장 후에도 같은 선택과 imgDim 없음이 보존된다.
2018 원본의 WMF는 명시 `imgDim=(15000,15000)`을 보존한다. 기준 PDF에 맞춰 원본
좌표를 바꾸지 않았다. 읽기 전용 SVG 진단에서 이 자르기를 적용하면 #6469 2쪽은
**98.32372%**다. 원본 image3의 자르기만 전체 선택과 좌상단·우하단 inset 선택으로
바꾼 HWPX 대조군 두 개를 정상 한컴 Save → Print2020했다. 이 대조군은 원본 통과와
구분하며 `vector-crop-counters`에 입력·Print 결과를 보존한다.

첫 생산 수정(`rhwp-vector-crop`, SHA-256
`e857c77ef5ccb70dd36af49c8f8dbc79814d5d068815972da11edc20befe59cb`)은
legacy `SvgRenderer::render_image_node`만 원본 WMF 형식으로 판단했다. 기본 layer
출력은 이미 `resolve_image_payload`가 SVG로 바꾼 바이트를 받아 그 조건에 들어가지
않았으며 2쪽은 **82.67807%**에 남았다. 같은 helper의 존재를 소비 완료로 세지 않는다.

현재 보정 경로는 `Picture::crop_reference_size → ImageNode → PaintOp::image`에서
원본 WMF의 명시 imgDim 또는 기본 메타파일 범위를 확정한다.
`image_node_with_resolved_payload`도 바이트 교체 전에 이를 보존한다. SVG·Canvas는
변환된 SVG의 viewport 크기와 보존된 동일 자르기 범위를 사용하고, layer JSON·DOM도
정규화된 PaintOp의 범위를 받는다. WMF SVG root의 width/height는 viewport와 같다.
새 생산 바이너리 검증과 실제 Canvas/DOM 소비 결과는 아직 확인 중이다.

PaintOp까지 좌표 범위를 전달한 Native(SHA-256
`3ba14958daef7940a8502c591f04ef7f4982cb272007346d3c96833e2f0644e2`)의
원본 #6469 15쪽은 최저 **93.18449%**, 미달·누락 0쪽이다. 2쪽은 **98.32372%**다.
정상 Save/Print 대조군 두 선택의 2쪽도 **98.36974·98.39247%**다. 기존 단순 선택은
동일하고 inset 선택은 **82.07956 → 98.39247%**로 개선된다. 대표 review PNG를 실제
표시해 확인했으나 아이콘 내부 선이 아직 빠져 있으므로 시각 완료로 판정하지 않는다.
동일 source의 fresh WASM 빌드와 원본 2018·2024 전체 Native 재비교는 진행 중이다.

이후 같은 Native로 원본 2018·2024 각각 64쪽을 완료해 두 문서 모두 최저
**96.53982%**, 누락·미달 0쪽을 재확인했다. fresh WASM wrapper도 exit0이며
JS `7d4a47d0714a9cc9ba6a731d7d2920dfea493a8fdcd316fd7d8f37deaf605130`,
WASM `056381bcea0ae1c4c5b6f8e5ddac0b89ed66eb06f2510b83f6431ed94a21a7e3`다.
루트 pkg와 Studio public이 두 파일 모두 일치한다. 이는 EMF+ 상세 그림 지원을
추가하기 **전**의 source 증거이며 새 source에 그대로 적용하지 않는다.

같은 source의 #6469 fresh WASM 15쪽도 최저 **93.18449%**, 미달·누락 0쪽이다.
주택임대차 원본 3쪽은 같은 Native로 재실행해 1쪽 **77.62412%**를 확인했으며,
기존 signed empty host 예약 차이는 아직 미충족이다. 다른 문서의 통과로 면제하지 않는다.

### 내장 EMF와 남은 아이콘

#6469 WMF image3·image4에는 표준 `WMFC` 청크가 각각 38개·31개 있으며 합치면
306980·250392바이트의 EMF가 복원된다. 이는 2018·2024 원본의 비표준 `TNPP`
escape와 다른 경로다. 표준 근거는
[MS-WMF enhanced metafile record](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/cfc88064-d86d-4b52-9374-3ce27d456179)다.
내장 EMF 바이트로 바꾼 진단 입력은 현 EMF renderer에서 2쪽 **66.78494%**였고,
패스 안의 PolyBezier16·Polyline16을 동등한 MoveTo/To 레코드로 바꾼 진단은
**70.65977%**였다. 둘 모두 생산에 적용하지 않았다. 자르기 개선만으로 누락된
아이콘·회색 배경의 확대 판독 결함을 면제하지 않는다.

추가 원문 판독에서 EMF+의 `EmfPlusImage` 객체가 다시 **EMF+ only 메타파일**을
포함하는 것을 확인했다. 상위 WMF·GDI fallback에는 이 내부 그림의 상세 선이 없고
EMF+ `DrawImagePoints`로 이어야 한다. image3은 내장 그림 4개, image4는 3개다.
원문 EMF+ Path/FillPath/DrawPath와 내장 그림을 판독한 읽기 전용 SVG 진단에서는
네트워크 지구본의 내부 곡선, 센싱의 여러 호, 아래 흰 아이콘들이 보존된다. 해당
진단은 clip·행렬 합성·일부 객체 필드를 아직 검증하지 않아 생산 출력·통과 증거가
아니다. 생산 반영 시 표준 지원 범위를 확인하고 미지원 데이터에서는 기존 전체
경로로 돌아가야 하며, 부분 출력의 성공을 전체 그림 지원으로 승격하지 않는다.

작은 HWP3 변환 HWP5 대조군 `samples/hwp3-sample-hwp5.hwp`의 16쪽 Native는
최저 **46.80351%**이고 3·8쪽이 미달이다. 해당 두 쪽은 stage6 이전 동결 Native에서도
같은 **63.52031·46.80351%**다. 원본 HWP3 `samples/hwp3-sample.hwp`도 16쪽 중
3·8쪽이 **56.33181·54.22358%**다. 따라서 새 profile 변경의 악화로 단정하지 않으며
기존 미충족을 해결하지 않고 #2403의 기존 provenance 기대값을 바꾸지 않는다.

이전 단계 failed-18의 Chromium 기동 오류를 해당 문서만 재시도해 21쪽 최저
32.84897%를 확인했다. 기동 timeout은 환경 실패, 정상 재실행의 미달은 실제 시각
미충족으로 구분한다.

### EMF+ 생산 출력의 클리핑 경계

`src/emf/emfplus/`는 표준 EMF+ 객체·변환·클리핑·저장/복원을 재생한다. 지원하지
않는 객체·실제 GDI 그리기·불완전한 스트림이면 기존 WMF 전체 경로로 돌아간다.
중첩 그림은 동일한 좌표 결과로 `DrawImagePoints`의 원문 SrcRect와 목적 삼각형에
배치한다. TileFlipXY는 원문 이미지 속성에서만 수용한다. 깊이·점·레코드·산출물
예산을 공유하며 임의의 그림 ID·페이지 좌표로 분기하지 않는다.

첫 생산 Native 빌드는 성공했으나 실제 2쪽은 **41.33083%**였다. SVG `clipPath`
안에 넣은 그룹이 브라우저의 클리핑 도형으로 수용되지 않아 모든 그림을 잘랐다.
이를 도형 직접 변환으로 보정하고 저장 상태를 ID 대소가 아닌 저장 순서로 복원했다.
[MS-EMFPLUS Restore](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-emfplus/524b7c3c-9b25-4a33-8598-b5cea7173d0d)의 스택 계약을 따른다.
실패 review도 output에 보존하며 빌드 성공을 출력 통과로 세지 않는다.

보정 Native 빌드는 exit 0, SHA-256
`4585744a66b06d2d2a0788bf45b1240b77018a5f29606421c78ce7438cfa0250`다. 실제 원본
2쪽은 **98.56717%**로, 새 review를 표시해 지구본 내부 곡선·센싱의 여러 호·작은
아이콘을 확인했다. `emfplus-clip-provenance.json`·`wmf-emfplus-clip-native/`를 따른다.
같은 배율의 더 높은 해상도에서 가는 선·아이콘과 회색 배경을 추가 판독 중이다.
15쪽 전체·자르기 대조군·fresh WASM은 아직 이 결과로 대체하지 않는다. 예약
PathFlags의 값만으로 fill mode를 추정하지 않으며 미검증 계약을 완료로 기록하지 않는다.

### 음수 줄간격을 가진 빈 부동 호스트

주택임대차 1쪽의 p2는 빈 InFrontOfText 도형 host다. 저장 줄 높이 2600HU와
줄간격 -1560HU의 전진량은 1040HU이고, 다음 p3의 저장 원점 차이와 정확히 같다.
기존 format의 overlay 전용 fallback은 이를 400HU 줄로 바꾸어 fit와 실제 흐름의
장부가 달랐다. `stored_host` 계획을 비-TAC Square·글앞·글뒤 호스트에 적용하되,
편집되지 않은 실제 HWP5 저장 줄과 다음 정상 원점의 독립 등식으로 수용한다.
음수 줄간격 자체를 거절하지 않고 양수 전진량을 검증한다. 도형의 가시 영역·줄
점유 높이·다음 원점 전진량을 구분하며, 미지원·편집·합성·쪽 나눔은 기존 경로를 유지한다.
이 계획은 fit의 줄 높이와 흐름 전진량을 각각 전달하고 수용된 FullParagraph 항목에
게시한다. 배치는 같은 계획의 줄과 end를 소비해 뒤 표의 원점을 재추정하지 않는다.

원본 첫 네 문단에서 host의 문단 줄간격만 40·100·160%로 달리한 입력을 만들고,
저장 캐시를 지운 뒤 한컴 2020 정상 Save/Print를 요청했다. 생성·job 결과는 ignored
`housing-host-counters/`에 보존한다. 아직 Save/Print·생산 출력 비교가 끝나지 않아
정식 fixture·회귀 테스트로 등록하지 않는다.

중첩 아이콘의 고해상도 판독에서 오른쪽 끝이 잘리고 반사된 모서리가 생기는 것을
확인했다. 독립 Windows GDI+ `Image.GetBounds(UnitPixel)`를 원문 EMF+ only 7개에
실행했다. EMR_HEADER의 잉크 Bounds(47×47)를 전체 이미지 창으로 보는 가정이
잘못됐다. 실제 경계는 Frame의 .01mm 좌표를 기록 장치의 pixel/mm로 환산한 원점과
inclusive 크기다. 예를 들어 첫 그림의 원점은 (-0.510056853, -0.510810733),
크기는 (48.9817734, 48.97972)다. 이는 상위 원문 SrcRect와 독립적으로 일치한다.
[Image.GetBounds](https://learn.microsoft.com/en-us/windows/win32/api/gdiplusheaders/nf-gdiplusheaders-image-getbounds)도
메타파일 원점이 0이 아닐 수 있음을 설명한다. source 화소는 메타파일 단위 계약이며
회귀 선택·실물 배치를 고정하는 화면 px 상수가 아니다.

`metafile_source_bounds`가 원문 Frame/기록 장치로 원점·크기를 계산하고 child SVG의
viewBox와 TileFlipXY가 같은 경계를 소비한다. 잉크 경계·수동 crop·패스의 최대값으로
대신하지 않는다. 새 Native 빌드는 exit 0이며 SHA-256은
`1bea989219e914e9c77c43778148b25373b2582631bbd747575c405f6698db6c`다.
아직 해당 출력의 확대 판독과 fresh WASM이 끝나지 않았으므로 완료 판정은 보류한다.
Windows 실행은 SSH → PowerShell → System.Drawing/GDI+이며, 원문·bounds JSON·PNG·
실행/정리 로그를 output에 보존했다. 작업 전용 임시 디렉터리의 파일을 확인하고
모든 GDI+ 객체와 실행이 끝난 뒤 원격에서 삭제했다. 기본 Windows 저장소는 변경하지 않았다.

주택 임대차 signed host 보정 Native의 원본 전체 3쪽은 최저 **94.21534%**,
누락·미달 0쪽이다. 새 review를 실제 표시해 주 표의 원점 회복을 확인했다.
계약 내용 제목과 작은 간격 등 직접 판독 차이는 계속 확인한다. 대조군의 첫
동기 MCP 호출은 client 제한 시간에 걸렸고 결과를 확보하지 못했다. 절차를 다시
읽고 정본의 `start → status → download`로 세 대조군의 6개 Save/Print job을
모두 성공시켰다. `housing-host-counters/mcp-async-results.json`이 실행 증거이며
이전 동기 호출의 실패를 성공으로 기록하지 않는다. before/after 및 fresh WASM
비교가 끝나기 전 정식 테스트·baseline은 유지한다.

프레임 보정 뒤 고해상도 Native에서 아이콘 오른쪽의 잘림·반사 모서리가 사라짐을
직접 확인했다. 원본 2쪽은 **98.56775%**, full/inset 자르기 대조군도 Native
통과다. 같은 source의 fresh WASM wrapper는 exit 0(16분 51초), BG SHA-256
`d45e37cc7ce98a653ea3cca8b0f8a073ac5cc4a2e63d99e9fdbcd4067c324cc6`이고
JS는 `7d4a47d0714a9cc9ba6a731d7d2920dfea493a8fdcd316fd7d8f37deaf605130`이다.
루트 pkg와 Studio public의 두 파일이 각각 일치한다. 실제 WASM의 원본 2쪽도
**98.56775%**다. 전쪽·대조군·정식 회귀·lint는 별도로 남아 있다.

종료된 이전 `circle-full-native`·`inherited-band-full-native`·`inherited-band-full-wasm`
실행의 재생 가능한 중간 SVG 348개(11.19GiB)를 정리했다. 제거 전 상대 경로·쪽·크기·
SHA-256을 `derived-svg-cleanup-20261008.json`에 기록했다. 원본 HWP/HWPX·PDF,
모든 PNG·TSV·JSON·로그, binary/WASM과 경계 1·3·12·18·35·42쪽 SVG는 보존했다.
공유 target이나 다른 작업 파일은 정리 대상이 아니다.

### 다른 실패 입력의 현재 진단

`21_언어_기출_편집가능본.hwp`의 Native 15쪽 중 1쪽만 **63.91561%**로 미달이다.
해당 쪽의 새 review를 표시했다. 첫 문단의 raw 앞 간격은 1140/2=570HU이고,
전폭 Paper 기준 표의 하단+바깥 아래 여백은 첫 저장 글줄의 원점과 일치한다.
Native 첫 단 본문은 Print보다 같은 570HU만큼 아래로 이동한다. 둘째 단은 별개
소유자이므로 같이 이동시키는 근거가 없다. 실제 첫 단 범위와 원문 앞 간격만으로
중복 가정을 확인한 읽기 전용 SVG 진단은 **96.38936%**다. 생산 구현·통과 증거는
아니며 먼저 실제 예약→첫 항목 원점→배치 경로와 앞 간격 대조군을 확인한다.

실제 HWP3 입력은 `samples/SO-SUEOP.hwp`다. Native 원본 46쪽은 최저
**36.58541%**이며 여러 쪽이 미달이다. 같은 원문을 한컴 2020에서 정상 Save 후
Print한 별도 진단 대조군도 선택 1·2·14쪽 중 **43.20906·89.69715%** 미달이 남았다.
따라서 Save만으로 해소했다고 판정하지 않는다. 첫 줄·문단 속성은 두 입력에서
대부분 같고 일부 후속 vpos가 재계산된다. 1쪽 세로 표제의 가로·세로 비율을 새
review에서 직접 확인했다. 새 진단 입력은 원본과 Print PDF를 대체하지 않는다.
원본 `samples/3-09월_교육_통합_2023.hwp`의 20쪽 대조의 미달은 후반 쪽에도 남아 있으며
전체 최저 **30.95918%**다. 다른 문서의 통과로 면제하거나 기존 px/hash 기대값을
바꾸지 않는다.

### 공간 확보와 현재 로컬 검증

workspace 빌드 첫 실행은 `No space left on device`와 linker Bus error로 실패했다.
공유 `target/pr-review`는 보존하고, 종료된 중간 검증 SVG 187개 10.79GiB와
이전 `output/pr-review/pr7600`의 실행 중이 아닌 ELF·중복 SVG 1442개 34.19GiB를
정리했다. 원본 HWP/HWPX·PDF·PNG·TSV·JSON·로그·글꼴과 현재 실행 폴더는 보존했다.
정리 전 경로·크기·SHA-256은 ignored output의 `derived-svg-cleanup-third-20261008.json`과
`output-cleanup-receipt-20261008.jsonl`에 있다. 해당 파일은 Git에 포함하지 않는다.

정리 뒤 workspace 빌드는 exit 0(1분 47초), workspace all-targets Clippy는
exit 0(2분 43초)다. Native Clippy·WASM Clippy·fmt도 각각 exit 0이다.
base `60efc4b7b380b2a183c20c3ee0ab24a60bc2d832` 기준 suite manifest는
1491 sources / 48 integration targets로 통과했고, unit tier 정책도 4199 tests로 통과했다.
새 upstream base의 정책 비교·전체 회귀·미달 원본 개선·관계 검사 변경은 아직 남아 있다.

현재 Frame 보정 코드의 #6469 Native **15쪽 전쪽 최저 93.18449%**를 확인했다.
앞서 완료한 2018·2024 Native 128쪽도 두 원본 각각 **96.53982%** 이상이다.
fresh WASM 전쪽 비교는 별도로 진행 중이며 이전 소스의 결과로 대신하지 않는다.

언어 기출 앞 간격 대조군은 raw 0/570/1140/2280HU 각각을 정상 Save 후 Print했다.
네 저장본 모두 첫 vpos=9014HU, 뒤 문단 vpos=11400HU이고 Print 첫 소개 글자의
경계도 네 경우 완전히 같다. 따라서 전폭 Paper 표의 배제 하단에서 시작하는 첫 글줄은
자기 문단 앞 간격을 다시 더하지 않는다는 독립 증거다. 보정 전 Native는 각각
**97.26151 / 77.36926 / 56.39057 / 53.49682%**다. 아직 생산 구현의 통과가 아니다.

### 전폭 절대 표 뒤 첫 부분 문단의 공통 원점 계획

한컴 정상 Save/Print 네 대조군의 동일 첫 원점은 앞 간격을 배제 예약 뒤에 다시
가산하지 않는다는 근거다. `paragraph_flow::place`의 첫 host 포맷 → 저장 줄과
`absolute_table_vertical_geometry`가 증언하는 물리 원점/끝 → 실제 전체 부분 문단
수용 시 `TypesetState` 확정 → 같은 `InlineFlowPlan`의 부분 문단 paint 소비를 연결한다.
측정에서 기존처럼 부분 글줄의 앞 간격을 제외하면서 paint에서 다시 가산하는 불일치를
해소한다. 쪽/문단 ID나 화면 px 좌표로 대상을 고르지 않는다. 실제 HWP5 저장본의
첫 host, 절대 전폭 TopAndBottom 표, 저장 첫 원점=배제 하단, 다음 저장 줄 관계가
증언하는 경우에 적용하고 편집/합성 줄/그 외 컨트롤/부분 수용은 기존 경로로 남긴다.
코드·전체 Native/fresh WASM·정상 앞 간격 대조군의 실행 결과는 보정 뒤 연결한다.

### OLE 메타파일 목적 extent의 독립 축

`SO-SUEOP.hwp` 1쪽 OLE은 원문 extent=14816×49532HU, 표준 WMF의
MM_ANISOTROPIC(8)·window=4560×10260이다. 두 좌표계는 같은 종횡비가 아니며
OLE의 목적 사각형은 가로·세로를 독립 환산한다. 현재 ShapeLayout의 WMF preview
및 WMF Contents 임베딩은 `xMidYMid meet`로 두 축을 묶어 표제를 세로로 줄인다.
같은 원문 목적 사각형을 유지하고 독립 축만 적용한 읽기 전용 SVG 진단은
**36.58541% → 84.73761%**다. 표제 윤곽은 직접 대조에서 맞지만 본문 글꼴/간격
차이가 남아 있으므로 90% 이상·완료로 판정하지 않는다.

원문 WMF → 동일 converter → 목적 OLE bbox → RawSvg payload → SVG/Canvas/DOM
소비로 이어지는 WMF 두 경로의 aspect 처리를 목적 extent에 맞춘다. EMF 및 네이티브
비트맵은 별도 경로다. 원본과 정상 Save 진단의 전쪽 Native/fresh WASM을 재검증하고,
90% 미만인 동안 기존 SO 회귀와 기대값은 유지한다.

### 수학 12쪽 머리말의 직접 판독과 원점 재검토

사용자가 `fixed-decoration-tac-math-review/math-fixed-decoration/review/review_012.png`의
머리말 차이를 지적했다. 해당 쪽 실루엣 값은 95.23736%지만 숫자 `4`가 용지 왼쪽에
붙고 제목의 굵기·장평도 다르다. 점수 통과와 직접 판독 판정은 분리하며 이 영역은
**미충족**이다. 90% 통과를 완료 또는 회귀 기대값 변경의 근거로 사용하지 않는다.

쪽번호는 일반 PageNumberPos가 아니라 `Header → Rectangle → TextBox → TextRun`에
소속된다. 원본 머리말의 쪽번호 도형은 Paper-Left, 가로 오프셋 -14HU이며 Native
글자 원점은 -0.2px다. 독립 한컴 Print PDF의 같은 숫자 원점은 52.32pt다.
원본 본문/머리말 틀 좌단과 오프셋을 더한 위치에 해당한다. 이는 고정 화면 위치가
아니라 머리말 틀의 좌단과 개체 오프셋 관계를 확인하는 진단 관측값이다.

생산 경로는 `build_header/build_footer → layout_header_footer_paragraphs →
layout_shape → ObjectPlacementFrame::position → Rectangle/TextBox/TextRun`이다.
그림은 #6608에서 이미 머리말·꼬리말 틀을 Paper/Page 참조 영역으로 소비하지만,
도형은 물리 용지와 본문 영역을 따로 전달했다. 직접 도형 경로와 표/그림 혼합 문단의
도형 경로 모두 같은 머리말·꼬리말 틀을 전달하도록 보정 중이다. 바탕쪽과 본문은
별도 호출 경로다. 좌표 clamp·숫자/문서 ID 분기·고정 px 보정은 추가하지 않는다.

독립 범위 확인을 위해 원본 도형을 사용한 대칭/비대칭 여백 대조군을 만들었다.
Paper의 Left/Center/Right, Page/Para 기준과 세로 Paper 기준을 같은 문서에서
정상 한컴 Save 후 Print한다. 생성 코드·입력·MCP 상태·실행 로그는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/header-frame-counters/`에 보존한다.
대조군과 수정 후 전쪽 Native/fresh WASM은 실행 결과가 확보된 뒤 연결하며,
기존 `header_paper_relative_shape_uses_page_origin`의 잠정 기대값은 그 전에 바꾸지 않는다.

288dpi의 같은 머리말 영역 확대 판독에서 세로 구분선도 제목을 가로지르는 것을
확인했다. 저장 도형은 Para-Top, 세로 오프셋 5669HU, 높이 92012HU이며,
참조 틀 원점+오프셋으로 시작해야 한다. 그러나 `layout_shape`의
`flow_with_text` 단 높이 clamp가 머리말 밴드 높이보다 긴 선을 밴드 위쪽으로
되돌렸다. 머리말/꼬리말 노드의 직접 도형은 본문 흐름에 참여하지 않으므로
이 단 내부 제한을 적용하지 않고 저장 오프셋을 보존하도록 보정 중이다.
그룹·바탕쪽·본문의 노드 소유는 별도 경로다. 수정 전 확대는
`math-header-before-zoom/native-before-vs-pdf-288.png`다.

HY굴림 공급만 바꾼 읽기 전용 진단은 수학 1쪽 84.01577%, 12쪽 95.35270%다.
글자 원점·장평·스타일은 그대로 두었고 생산 글꼴 정책은 변경하지 않았다.
제목은 이 진단에서도 한컴의 HFT 출력보다 가늘다. 단순 글꼴 공급 변경을
완료 또는 일반적인 해결책으로 판정하지 않는다. 진단 코드·글꼴 SHA·점수는
`math-hy-gulim-diagnostic/diagnostic.json`과 같은 배율 확대에 보존한다.

원점 보정 Native 빌드는 exit 0(5분 37초)이며 같은 12쪽은 95.90482%다.
이어 머리말 밴드 clamp를 제외한 빌드도 exit 0(6분 22초)이고 12쪽은
**96.29921%**다. 쪽번호가 머리말 좌단으로 이동하고 세로 구분선이 가로 구분선
아래에서 시작하는 것을 직접 확인했다. 최신 대표 이미지는
`header-frame-math-review/math-header-frame/review/review_012.png`다.
두 빌드의 전체 source hash와 binary hash는 각각 ignored provenance JSON에 있다.
이는 선택 12쪽 Native 증거이며 전체 페이지·fresh WASM 통과로 확대하지 않는다.

### HFT 장평 변환의 세로 크기 계약

독립 Print의 12쪽 HFT `/T7`는 원본 제목 글자 모양 2830HU에 대해
`Tf 236 × 0.120 = 28.32pt`, `Tz 94`를 출력한다. 원본 크기 28.30pt와 인쇄
격자의 반올림 차이만 있으며 세로를 √0.94로 줄이지 않는다. 원본의 괄호 뒤
글자는 같은 Tf와 `Tz 90`을 사용한다. FontMatrix는 0.001이고 Type3 charproc는
`20 w B`로 합성 굵기 0.02em를 적용한다. 따라서 굵기 계수는 임의로 늘리지 않는다.

기존 #5821의 TrueType 출력 기반 √r 규칙을 HFT 대체 face에도 적용해 제목 글자가
작아졌다. 글꼴 출처(`TextStyle.hft_hangul_face`)에 따라 선언 세로 크기를 보존하는
공통 `ratio_draw_params`를 만들고 SVG·CanvasTextFont·Skia·glyph lowering이 이를
소비하도록 보정 중이다. 본문 advance·줄 구성·원점·자간은 변경하지 않으며,
TrueType 경로는 기존 독립 규칙을 유지한다. 읽기 전용 제목 단독 진단은 1쪽
83.40755%, 12쪽 95.41733%로 글꼴 윤곽 차이를 해결하지 못했다. 이 보정의 근거는
점수 상승이 아닌 원본 크기와 독립 Print의 변환 계약이며 최종 구현은 전쪽·대조군에서
다시 검증한다. 제목의 윤곽/굵기 차이는 여전히 미충족이다.

HFT 크기 보정 첫 Native 빌드는 컴파일 오류 출력 없이 exit 143으로 종료됐다.
원인은 확인되지 않았으며 실행 중 Cargo/Rust가 없는 것을 확인한 뒤 동일 명령을
재시도한다. `stage7-hft-height-build.log`는 실패 증거로 보존하고 통과로 계산하지 않는다.
완료된 월별 세 원본의 중복 중간 SVG 57개(4.51GiB)만 추가 정리했다. 각 원본의
첫 쪽·마지막 쪽·최저값 쪽 SVG와 전쪽 PNG/PDF/TSV/JSON/log, 입력과 frozen binary는
보존했다. 정리 목록·SHA는 ignored `derived-svg-cleanup-monthly-completed-20261008.json`이다.

HFT 크기 보정 Native 재시도는 exit 0(5분 57초), fmt check는 exit 0이다.
frozen binary SHA-256은 `35c85e58ef04daa30fd02cf1d007d3b0570dcac10a748b021303e22215352dbf`이며
전체 source SHA는 `hft-height-provenance.json`에 고정했다. 수학 세 원본의 전쪽
Native를 새로 실행하며 현재 Native Clippy와 최소 머리말 대조군 Save/Print도 진행 중이다.
첫 대칭 대조군의 Save는 1,800초 종료로 실패했으며 원래 입력과 상태를 보존했다.
새 대조군은 Paper Left/Center/Right 글상자 하나와 비대칭 본문 여백을 가진 HWPX를
생성한 뒤 한컴 정상 Save/Print한다. 이 대조군의 통과를 기존 실패 입력의 통과로
바꾸어 기록하지 않는다.

첫 HFT 크기 구현의 실제 SVG가 이전과 동일한 것을 확인했다. 출처로 재사용한
`hft_hangul_face`는 `resolved_to_text_style`에서 문서별 ASCII 반각 수용 조건과
AND된 측정 플래그였다. 제목의 실제 HFT 치환 정보와 다른 상태이므로 이를 그리기
출처로 사용한 가정을 제거했다. 별도 `hft_font_face`에 현재 언어 슬롯의 원본 HFT
치환 정보를 직접 전달하고 공통 그리기 변환이 이를 소비하도록 보정한다.
ASCII 폭 수용·측정은 기존 조건을 유지한다. 앞선 동일 출력과 Clippy 통과를
이 수정의 개선 증거로 재사용하지 않는다.

### HML 기준 PDF 분류 정정

사용자가 `exambank_math_equations_min-2024.pdf`를 정상적으로 수동 Print한 기준임을
재확인했다. 본문이 작게 관측된다는 이유로 기준을 부적합하게 분류했던 판단을
철회한다. 원본 HML SHA와 제공 PDF SHA를 유지하고 수동 Print 기준으로 수용한다.
약 0.12pt 본문·겹침의 관측은 한컴 기준 출력의 상태이며, RHWP와 다른 이유를
원본의 RATIO/RELSIZE 생략·언어 슬롯·상대 크기 해석 경로에서 진단한다.
이전 기준 거부 기록은 오류 이력으로 보존하되 이 정정이 현재 판정이다.
기존 6.22797%는 출력 미달이며 기준 부족으로 면제하지 않는다. 관련 기존 테스트
기대값은 독립 계약과 출력 보정·90% 선행 조건을 확인하기 전 변경하지 않는다.

### 공식 HML/HWP/HWPX 샘플 확보

사용자가 제공한 `win10-ted:C:\temp`의 네 파일을 바이트 그대로
`samples/hml/exambank_math_equations_min.hml`, `.hwp`, `.hwpx`, `-2024.pdf`로 복사했다.
HML과 PDF는 기존 fixture와 SHA-256이 동일하다. 각 출처·생성 방식·저장 제품·해시는
`samples/hml/README.md`에 기록했다. 원격 사용자 파일은 변경/삭제하지 않았다.

한컴 저장본의 실제 metadata는 2020 / 11.0.0.9136이다. 따라서 이 두 저장본의 정식
대조 PDF는 engine 2020 Print로 별도 산출하고 원본 HML의 사용자 2024 Print와 구분한다.
저장본은 장평·상대크기 0, 문단 줄간격 0%, 수식은 non-TAC 및 7200×7200HU로
해석했다. RHWP HML adapter는 누락된 백분율과 문단간격을 100/160%로 채우고
수식을 강제로 TAC + intrinsic 크기로 만든다. 이는 독립적인 한컴 저장본과 실제
호출 경로에서 확인한 차이다. 스펙의 속성 기본값과 부모/자식 엘리먼트 생략 상태를
구분하고 기존 정확한 스키마 검사·출력 계약을 약화하지 않는 보정을 진단한다.
새 회귀 검사나 기대값 변경은 출력 보정과 Native/fresh WASM 90% 확인 이후다.

공식 샘플 네 파일과 출처 README는 `a1279f26b61573aa36ae571effe26150df695773`으로
커밋했다. 저장본별 2020 MCP Print도 모두 성공했다. HWP job은
`19c14841-7d71-42d9-9872-904b0efb69f5`, HWPX job은
`eab3de53-7848-4795-8e99-fddb7f881214`이며 실제 제품은 11.0.0.9136,
PrintMethod 0 / hancom2020_pdf_driver_one_up, 각 1쪽이다. PDF 경로·SHA와 입력 SHA는
`hml-official-saved-print/references.json` 및 공식 샘플 README에 연결했다.
원본 HML의 사용자 2024 PDF는 그대로 보존한다.

### HFT 출처 전달 보정의 fresh WASM 완료

`stage7-hft-font-origin-wasm-build.log`의 wrapper가 exit 0(15분 40초)으로 완료했다.
`wasm-hft-font-origin`에 package를 고정했고 BG SHA-256은
`e0fee2332f807e2df9053317ab10d4ad7ef7147b6b444397e025019b3ca68464`이다.
pkg/public의 JS와 BG는 각각 바이트 동일하다. Native/fresh WASM의 비대칭 여백
Paper Left/Center/Right 한컴 정상 저장·Print 대조군은 각각
96.42644%, 96.42644%, 96.30292%다. 실제 원본 수학의 90% 미만 쪽이나 제목의
윤곽 차이를 이 대조군 통과로 면제하지 않는다. 테스트 기대값은 아직 변경하지 않았다.

저장본별 2020 Print PDF와 README 연결은 `41ee5338b`로 커밋했다.
버전을 맞춘 Native 직접 대조에서 HWP 6.34209%, HWPX 16.96783%로 모두 미충족이다.
`hml-official-saved-native-review/results.json`에 각 명령과 결과가 있으며 HWP의
대표 `review_001.png`도 직접 판독했다. 본문 크기·소속과 수식 배치 차이가 실제로
보이므로 90% 미달을 기준 PDF 문제나 글꼴 예외로 분류하지 않는다. 공식 샘플 채택은
진단 원본 보존이며 신규 렌더링 회귀 테스트나 baseline 통과의 근거로 사용하지 않는다.

## 중간 커밋의 상태와 다음 분석

사용자 요청에 따라 이 단계의 생산 보정·독립 PDF·진행 기록을 중간 커밋으로 고정한다.
최신 Native/fresh WASM provenance에 기록된 전체 source와 현재 source의 SHA가 모두
동일함을 확인했다. 최신 Native/WASM Clippy는 각각 exit 0이며 이번 fmt check도
exit 0이다. 전체 회귀·workspace/all-targets·최종 base 정책 게이트는 아직 완료되지 않았다.
이 커밋을 PR 준비 완료나 시각 검증 전체 통과로 판정하지 않는다.

월별 원본의 현재 before 진단은 Native 1~8번 최저 29.82037~38.77224%로 모두 미달이다.
그 뒤 입력은 동일 frozen before binary로 계속 실행 중이다. 원본 수학의 미달 쪽,
HML 및 한컴 저장본의 차이, 나머지 실패 원본을 보정하고 검증할 범위는 그대로 유지한다.

HML의 상대 크기·장평 0/1/100을 분리한 한컴 2020 Print 대조에서 상대 크기 0과 1의
그리기 크기는 모두 약 0.12pt였고 상대 크기 100은 약 9.95pt였다. 장평이 0인 경우에도
단어 내부 글리프는 자연 전진으로 그리며 단어 원점은 같은 x로 되돌아간다.
원본 저장 LineSeg가 있는 입력이므로 이 대조만으로 재조판의 전진량을 확정하지 않는다.
공식 HWPML 표 36/38은 RATIO/RELSIZE 속성의 기본값 100을 명시하지만 자식 요소
전체 생략 시 한컴의 호환 해석은 이 스펙 표만으로 입증되지 않는다. 관련 원문과
Print 실행 증거는 ignored hml-zero-format-counters/에 보존했다.

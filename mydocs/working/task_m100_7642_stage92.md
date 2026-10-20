# Stage 92 — HFT 대체 글꼴과 실제 glyph 공급의 분리 진단

Issue: #7642. 변경 전 code source는 `c6ccb1cb64452d444c154a69545bbc90804e395c`다.
Stage 91에서 실패 원본 7개와 대조군 5개는 전쪽 직접 Skia 90% 이상이다.
`samples/exam_math.hwp`, `samples/exam_math_no.hwp`의 글꼴 차이는 남아 있다.
55개 원본/853쪽과 기존 회귀 91파일, #1139 전체 관계형 전환의 범위를 유지한다.

## 독립 관측과 가정

- 같은 원본의 독립 한컴 2020 Print를 유지한다. 원본 구역1 큰 제목의 face는
  `신명 신그래픽`, bold=true, 4100HU, ratio92다. 작은 머리글은 `신명 디나루`,
  bold=false, 2100HU, ratio85다. PDF의 대응 T7/T6는 Type3이며 family 이름을
  복구할 수 없으므로 T7을 임의 TTF와 같은 face라고 주장하지 않는다.
- canonical Native SVG 17쪽은 78.96865%다. 실제 내장 face는 제목의
  Gulim Regular, 작은 머리글의 Noto Sans KR ExtraLight다. 제목에 적용된
  합성 stroke는 0.02em이며 원본의 bold와 같은 결과인지 독립 출력으로 확인해야 한다.
- Linux에는 Gulim과 HYGulim이 설치되어 있다. `굴림`이라는 한국어 family는
  `fc-match`에서 Noto로 떨어진다. SVG의 local 별칭은 Gulim을 알고 있으나
  Skia 후보에는 공통 설치 별칭과 family chain 처리가 누락됐는지 실제 경로를 확인한다.
  이 누락이 현재 두 문서의 모든 차이를 설명한다고 단정하지 않는다.

## 진단 계약

현재 SVG의 원점·장평·본문·수식·줄 구성을 보존한 glyph 공급/합성 굵기 대조군을
ignored output에 만든다. 일반/합성 bold, 실제 설치 face를 각각 분리해 독립 Print와
대조한다. 수정 SVG는 RHWP의 수정 후 실제 출력이나 공식 통과 증거로 사용하지 않는다.
조판 기대값·회귀 테스트·원본·기준 PDF는 바꾸지 않는다.

원본 속성 해소 → 공통 스타일의 측정 메트릭과 paint face → SVG/Skia/Canvas의
실제 공급과 굵기 적용을 추적한다. 설치 별칭의 의미와 실제 글리프 공급 문제,
사양이 필요한 HFT 대체 정책을 구분한다. 독립 근거가 없는 수치 보정이나
특정 문서 ID/제목 문자열 분기는 추가하지 않는다.

이 계획 시점에는 진단과 구현·검증 결과 모두 미검증이다. 적절한 일반 보정이
확인되면 한 배치로 적용하고 선행 타입 검사/Native 빌드, 원본·대조군 직접 비교,
Native/fresh WASM 전쪽 TSV와 확대 판독을 이어간다. 정식 회귀는 해당 선행 조건
통과 뒤 소유·내용 순서·보존·포함 관계로 바꾼다.

## 진단 결과와 실제 누락 속성

`stage92-font-diagnostics/receipt.json`의 6개 대조군은 전체 쪽 78.411~79.754%다.
폰트 교체·합성 굵기 분리만으로 현재 차이를 설명하지 못했다.
다음 소스 추적에서 `CharShape.char_offsets`가 파싱·저장되지만 `ResolvedCharStyle`과
`TextStyle`에는 전달되지 않는 것을 확인했다. 텍스트 stream의 UTF-16 위치를
의미하는 `Paragraph.char_offsets`와는 다른 속성이다.

독립 Print T7의 실제 glyph 기준선은 큰 제목 185.44, 괄호 183.04다.
현재 SVG 기준선에 원본의 10%×4100HU, 7%×3300HU 위치를 적용하면 각각
185.48과 183.04로 대응한다. 이는 원본 비율의 진단 관측이며 고정 px 값으로
회귀를 만들지 않는다. 그 대조군의 전체 17쪽은 85.34440%로 개선됐지만 미달이다.
`stage92-source-offset-diagnostic/receipt.json`은 수정한 glyph와 원본 퍼센트를 기록한다.
대조군을 RHWP 실제 수정 후 출력이나 공식 통과로 보고하지 않는다.

## 구현과 소비 경로

언어별 글자 위치 퍼센트를 `ResolvedCharStyle.font_positions`에 보존하고
언어 슬롯을 선택할 때 `TextStyle.font_position`으로 전달한다. 기본 0은 직렬화에서
생략한다. 원본 글자 위치는 글자 크기에 비례하는 기준선 이동으로 소비한다.

`resolve_single_char_style → resolved_to_text_style → TextStyle.script_draw_metrics →
SVG/HTML/Skia/CanvasTextFont/paint JSON의 실제 glyph 기준선`을 대조했다.
단순 glyph replay에는 위치 변환 계약이 없으므로 그 조건을 명시하고 원본 위치를
소비하는 positioned text 경로를 사용한다. 줄 원점·저장 줄간격·가로 advance는
글리프 위치와 구분한다. font position 0, 양수/음수, 다른 언어 슬롯과 기존
첨자·그림자·회전의 소비도 후속 검증 대상이다. 다른 저장 메트릭이나 자간 정책은
독립 근거를 더 확보한 뒤 판단한다.

## 독립 합성 입력 산출 상태

원본 HFT/CharShape에서 재사용한 22개 글자 위치·자간·굵기 대조 행을 만들었다.
첫 MCP 2020 저장 job `ae9b6820-ad35-44b4-9878-409cdf47ad0b`는 opening_document에서
worker exit3221225477로 terminal failed다. 초기 합성 입력의 미사용 charPr가
기존 template의 다른 언어 font id를 참조하던 잘못을 확인하고 모두 유효한 참조로
고쳤다. 원본 v1과 실패 job 기록을 보존했다. 별도 v2 job
`a9251eac-c0e0-48b2-a37a-171d47455206`도 terminal failed다. 이 입력은 정상 한컴
저장본·Print로 산출되지 않았으며 독립 시각 기준이나 새 정식 fixture가 아니다.
기존 수학 시험지의 정상 Print 증거는 계속 유지한다.

이번 구현은 실제 누락 속성의 복원이며 현재 결과를 90% 통과로 선언하지 않는다.
정식 회귀·기대값은 아직 변경하지 않았다. 선행 검사·Native 빌드와 같은 원본 전체
직접 비교, Native/fresh WASM·전체 회귀의 필수 검증은 이어서 실행한다.


## 실제 코드 검증과 새 원인 증거

구현 source는 `dfb7748b51ad6fb0daa7057e26d37930be9e05e3`다.
`stage92-font-position-type-checks.json`에서 fmt, Native Clippy, WASM lib Clippy,
Skia lib check의 exit code가 모두 0이다. 고정 `target/pr-review`의 순차 Native
release-test/native-skia 빌드는 exit 0, 422.31초다.
`font-position-native-provenance.json`은 빌드 전후 source 해시와 고정 실행 파일을 기록한다.
workspace/all-target Clippy, 전체 nextest, fresh WASM과 55원본/853쪽의 최종 검증은
아직 완료되지 않았다. 일부 선행 검사 통과로 전체 통과를 보고하지 않는다.

같은 원본 `samples/exam_math.hwp`와 독립 Print PDF의 canonical Native SVG 17쪽을
재출력했다. `stage92-math-font-native-svg/math-font/silhouette.tsv`는
**85.39350%**다. 이전 source c6의 78.96865%보다 개선됐지만 90% 미만이며,
`re_review_required`다. 새 실제 출력으로 만든
`stage92-math-native-review/review/review_017.png`를 직접 열어 사용자에게 표시했다.
제목의 큰 기준선 차이는 줄었으나 제목의 글자 사이 간격과 작은 머리글의 폭·굵기 차이가
남아 있다. 이 PNG는 앞선 수정 SVG 대조군과 구분한다. 기존 회귀와 기대값은 변경하지 않았다.
191쪽 원본 9개와 대조군 51쪽의 직접 Skia 비교는 같은 고정 binary로 실행 중이다.
실행 중인 receipt를 전체 완료 증거로 쓰지 않는다.

합성 입력 v3에서는 정상 template의 기존 font 참조를 유지하고 원본의 두 HFT face만
추가했다. bold 노드도 charPr의 정상 자식 순서로 배치했다. HWPX 직접 open job
`1b886c80-2dae-4570-931e-ea2ef736b966`도 terminal failed다. 같은 내용을 RHWP가
HWP로 저장한 뒤 한컴에 전달해 open 경로를 분리했다. RHWP 변환의 `--verify`는
exit 0이며, 한컴 정상 저장 job `f09a59cc-403e-41a8-905f-2d942973fe8b`와 Print job
`4fa30e0a-5eb9-4350-8626-44b953bab6d9`는 모두 succeeded다.
실제 engine 2020, Hancom `11.0.0.9136`, Print method 0,
`hancom2020_pdf_driver_one_up`, PDF 2쪽을 확인했다.

이어 같은 두 face의 정체 ASCII 94자씩을 별도 한컴 저장·Print로 산출했다.
저장 job `3bbb775d-da72-4447-8a95-5c1874e67f05`, Print job
`1e01a81c-9469-457c-a32d-bde6e39e4045`는 succeeded이며 같은 engine/Print 계약과
전체 1쪽이다. 정상 저장 HWP와 PDF 4개는
`pdf/issue7642-hft-char-style-diagnostic/`에 원본 byte를 보존했다.
아직 정식 회귀 fixture가 아니며 Git에 추가하지 않았다. 입력·산출 SHA, 22개 행의
속성 보존 및 188개 ASCII 메트릭은 ignored output의
`stage92-char-style-probe/independent-font-program-evidence.json`과
`preserved-independent-outputs.json`에 기록했다.

한컴 정상 저장본을 RHWP가 HWPX로 진단 추출한 `export-hwpx --verify`는 exit 3이다.
23개 문단의 저장 LineSeg가 추출 HWPX에서 생략된 차이다. 이 실패를 round-trip 성공으로
보고하지 않는다. 별도로 추출 charPr를 원본 합성 행과 대조하여 22개 행 모두의
height·bold·언어별 ratio/spacing/offset이 보존된 사실을 확인했다. 이 추출본은
정상 한컴 HWP 원본이나 독립 Print를 대신하는 기준 파일이 아니다.

독립 Type3 /Widths에서 디나루 숫자는 660/1000em, 신그래픽 숫자는620/1000em이며
두 face의 괄호는500/1000em이다. 작은 머리글 숫자를 설치 대체 글꼴의 폭으로 재면
행 전체 폭과 가운데 정렬 원점이 달라진다. 22행 대조는 굵기만 바꿔도 전진폭이
증가함을 보여 준다. 정체·굵게 Type3의 intrinsic /Widths는 동일하고 실제 TJ 간격이
다르다. 기존 신그래픽의 자간 무시 정책은 이 굵기 전진과 음수 자간이 상쇄되는 사례를
자간 전체의 비적용으로 해석했을 가능성이 있다. 이는 다음 일반 보정의 원인 근거이며,
특정 쪽의 px 차이를 보정 값으로 사용하는 근거가 아니다. 정확한 굵기 간격과 공백·
슬롯·마지막 글자 소비를 독립 출력으로 확인하고 메트릭/자간 소비 경로를 함께 보정한다.


## 원본 전체 쪽 직접 Skia 완료

`stage92-original-skia/receipt.json`은 동일 source dfb의 원본 9개/191쪽을 모두
재출력·측정한 terminal complete 기록이다. 기준 PDF와 전체 쪽수는 모두 같다.
7개 원본은 전쪽 90% 이상이고 수학 두 원본은 미달이 남는다. 직접 Skia 결과를
canonical Native SVG/fresh WASM의 전체 제출 TSV 완료로 바꾸어 보고하지 않는다.

| 원본 식별자 | 전체 쪽 | 최저값 | 90% 미만 쪽 |
| --- | ---: | ---: | --- |
| all-24 | 21 | 96.42088% | 없음 |
| all-20 | 22 | 95.61623% | 없음 |
| all-25 | 21 | 96.42080% | 없음 |
| all-21 | 22 | 95.61623% | 없음 |
| all-28 | 21 | 96.42088% | 없음 |
| all-29 | 21 | 96.42080% | 없음 |
| all-39 | 20 | 86.08289% | 2, 9, 17 |
| all-41 | 20 | 85.26152% | 1, 9, 13, 16, 17, 18 |
| all-10 | 23 | 90.53366% | 없음 |

`samples/exam_math.hwp`(all-39)의 최저값은 이전 c6의78.69607%에서86.08289%로,
`samples/exam_math_no.hwp`(all-41)는77.76098%에서85.26152%로 개선됐다.
미달 쪽은 각각2·9·17쪽과1·9·13·16·17·18쪽이다. 90% 미달 상태의 기대값 변경이나
PR 생성은 계속 보류한다. 대조군 5개/51쪽의 새 source 재검증은 별도로 진행 중이다.


22행 합성 대조군도 같은 source dfb의 canonical Native SVG로 독립 Print와 비교했다.
Native와 PDF의 전체 쪽수는2/2이며 페이지별 TSV는58.73752%와49.88545%다.
원본 시험지 통과 근거로 쓰지 않으며 이 대조군에도 정식 회귀를 추가하지 않는다.
`actual-native-print-baseline-comparison.json`은0·10·21% 위치, 정체·굵게, 서로 다른
자간/크기의22개 행을 Native와 Print에서 소유 순서대로 대조한다. 실제 기준선의
최대 차이는0.88524px이다. 이 값은 진단 관측이며 회귀의 px 허용치로 사용하지 않는다.
Print 드라이버의 좌표 스케일·격자 잔차와 가로 폭 차이를 구분한다. 음수 위치,
언어별 다른 상대 크기, 첨자와의 조합은 아직 이 독립 대조군에서 검증하지 않았다.
RHWP의 compat는2022로 실행했다. MCP engine2020과 RHWP compat 값은 다른 계약이다.
초기 잘못된 compat2020 호출은 CLI 사용법 오류(exit2)였고 렌더를 실행하지 않았다.
정상 재실행은 미달 TSV를 보존하고 gate 때문에exit1로 끝났다.


## 대조군 51쪽 완료

같은 source dfb의 `stage92-cell-shape-controls-skia/receipt.json`도 terminal complete다.
5개 원본의 전체51쪽을 직접 재출력했고 독립 PDF와 전체 쪽수가 모두 같다.

| 대조군 | 전체 쪽 | 최저값 | 90% 미만 쪽 |
| --- | ---: | ---: | --- |
| samples/21_언어_기출_편집가능본.hwp | 15 | 93.82475% | 없음 |
| samples/exam_eng.hwp | 8 | 95.34003% | 없음 |
| samples/exam_kor.hwp | 20 | 90.65208% | 없음 |
| samples/exam_science.hwp | 4 | 93.82529% | 없음 |
| samples/exam_social.hwp | 4 | 97.34725% | 없음 |

현재 단계의 원본191쪽+대조군51쪽 직접 Skia 비교는 모두 종료됐다. 미달 수학 원본을
포함한55원본/853쪽의 canonical Native/fresh WASM 최종 제출 검증과 확대 판독,
관련 회귀 전체의 관계형 전환·전체 nextest·workspace/all-target lint는 미완료다.
추가 코드 보정 뒤 이전 증적을 새 source의 통과로 재사용하지 않는다.

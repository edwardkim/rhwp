# 자소의 실제 폭·성조 정렬·합성 기울임의 공통 보정

## 독립 근거와 기존 구현 대조

- 옛한글 원 해결 이슈 #2099 / PR #2128의 표준 자소·원문 주소 보존은 유지한다.
  현대 NFD는 Unicode Hangul의 L/V/T 합성으로 같은 음절을 식별한다. 가시 glyph가 있는
  실제 source face에서만 paint용 음절을 선택하고 원문·편집 좌표는 변경하지 않는다.
- `oldhangul-modern-font-advances-218.json`과 `hangul-jamo-metric-source-224.json`은
  HCR source 4개 프로그램의 cmap/hmtx를 확인했다. 중성·종성의 실제 0폭을 미수집으로
  취급한 1em 폴백과 현대 NFD의 폭·glyph 선택을 함께 보정한다. Print의 낱자 원점도
  같은 0폭을 입증한다(`oldhangul-modern-print-trace-213.json`).
- 굵은 성조의 빈 문맥 자소는 일반체와 전진이 다르다. `emphasis-canvas-tone-bold-probe-226.json`
  의 실제 Canvas 측정에서 문자열 끝점 정렬은 양쪽 glyph 원점을 맞춘다. 반 em 같은
  고정 보정량 대신 Canvas의 right 정렬과 SVG의 end anchor를 같은 역할로 사용한다.
- `emphasis-italic-print-matrices-208.json`의 한컴 Print 합성 기울기는 0.333이다.
  기존 Native 0.25·CanvasKit 0.2·브라우저 기본 합성을 같은 본문 계약에 맞춘다.
  실제 italic face는 추가 shear를 적용하지 않는다. 별도 Arial/Times italic 입력의
  Print가 실제 italic face와 한글 fallback의 합성을 구분하는 대조군이다.

## 실제 소비와 반례

측정의 문자 폭·trace와 배치 advance는 같은 자소 메트릭을 소비한다. Native/SVG/Canvas2D/
CanvasKit의 paint용 문자열은 같은 유일한 PUA 또는 현대 합성 음절 후보와 실제 가시 glyph를
대조한다. 원 paint op의 표시 문자열·주소·전진은 유지하며 재생 뷰에서만 glyph를 바꾼다.

공통 성조 geometry는 기준점과 문자열 정렬 역할을 제공하고 실제 glyph는 각 backend의
같은 글꼴·굵기에서 그린다. 본문 synthetic italic은 실제 italic 프로그램과 구분한다.
SVG/PDF 호환 경로와 브라우저의 CSS 표현도 확인하며 수식 전용 italic 근사는 이 본문
기울기의 근거로 대신하지 않는다.

기존 66조건, 현대 자소 2조건, 실제 italic 대조군과 #7694의 새 저장 경로를 batch로
검증한다. 새 #7694 HWP 두 파일의 Print 원문은 적용한 source head와 SHA가 일치함을
`pr7694-update-applied-227.json`에서 확인했다. 전쪽 수치와 직접 확대 판독, 수정 전후
검출을 확인한 뒤 정식 회귀를 추가·수정한다. 현재 미세 차이는 수치 통과로 면제하지 않는다.

## 소스 자소와 실제 실행

강조점은 실제 0 전진과 독립 결합 문자도 표시한다. 원문을 스타일별로 나누기 전의
자소 소속을 복원하되 인라인 개체·주석은 별도 소유자로 취급한다. 일반 줄 경로는
실제 논리 슬롯과 선두 주석 prefix 생략을 투영하고, 전용 인라인 표 경로는 이미
보유한 원문 scalar 위치를 사용한다. 좌표 일치나 run 첫 글자의 Unicode 범주만으로
소속을 추정하지 않는다. 생성 표시값은 원문 대응 확인 없이 합치지 않는다.

- 동결 Native041의 #8064 보존: 179조건·2,082쪽 전부 90% 이상, 최저 90.16447%,
  기준 대비 0.1%p 이상 하락 0건. 전쪽 TSV 해시 재확인은
  `output/pr-review/semanticist21-20261019/pr8064-native041-final-audit-239.json`에 있다.
  현재 보정 소스나 fresh WASM의 최종 검증으로 승격하지 않는다.
- TypeScript는 Studio 작업 디렉터리의 `npx --no-install tsc --noEmit`으로 통과했다.
  루트 npx 호출은 프로젝트 TypeScript를 선택하지 못했으며 정식 타입 검사 통과로
  기록하지 않는다. 실제 성공 로그는 `logs/jamo-tone-italic-typescript-236-attempt02.log`다.
- Native042는 공유 `target/pr-review`, 컴파일 잡 8개로 전체 보정 소스를 함께 빌드했다.
  이어서 독립 Print의 현대 자모·성조·italic·0폭·소스 경계 대조를 수행하며,
  fresh WASM과 직접 판독·기준 대조 전에는 완료 판정이나 회귀 기대값 변경을 하지 않는다.

## Native042 / fresh WASM017 실행과 다음 보정

- 제품 소스는 `d2256b24e013dc6d123e13c05bae116809b67b13`에 고정했다.
  Native042 소스 125경로의 commit 바이트 동일성은
  `native042-committed-source-equivalence-242.json`으로 확인했다.
  Native 빌드 228.61초, 최적화 fresh WASM 714.64초 exit 0이며 Studio 동기화 해시도 같다.
- Native 74조건 중 70조건 수치 통과. 현대 자모 Batang/Dotum, italic/bold/bold-italic
  강조점은 100%다. 추가한 5개 분리 입력은 스타일·각주 3조건 통과, 인라인 표 2조건 미달이다.
  #7694 새 author HWP 두 파일도 별도 비교했으며 양쪽 모두 미달이다.
  점수 통과와 직접 판독·기준 대비 하락 판정은 분리한다.
- fresh WASM017은 같은 제품 소스로 위 입력을 합친 81조건을 비교 중이다.
  `wasm017-combined-252-jamo-tone-italic-fixed-wasm/progress.json`과 실행 로그를
  확인하고, 이 기록의 중간 진행값을 최종 완료로 사용하지 않는다.

### 실패 원인의 독립 근거

1. 단독 종성·결합 문자: 원본 SFNT와 Print 내장 글리프의 실제 윤곽·0 전진은 같다
   (`zero-jamo-pdf-font-audit-246.json`). Native SVG의 본문 clip x=99.4667이 실제
   잘린 왼쪽 픽셀과 일치한다(`zero-jamo-clip-diagnostic-247/`). 전진 0을 잉크 없음으로
   해석한 clip 범위를 공통 소비 지점에서 보정 중이다. `text_ink`의 보수적 em 범위는
   실제 잉크 측정값이나 배치 좌표가 아니며 원문·전진·흐름을 변경하지 않는다.
2. 빈 인라인 표: 공용 생성 API가 커서의 36pt 글자모양은 상속하지만 셀 저장 LineSeg와
   행 최소 높이는 10pt로 만든다. 원 해결 #2095 / `9494ba8c61c`의 글자모양 상속과
   현재 `create_table_native` / `create_table_ex_native`의 생산·소비를 대조했다.
   잘못된 저장 줄을 정상 한컴 저장 정보의 수용 조건 완화로 해결하지 않는다.
3. #5874 writer 원본: 원문에는 단 정의가 없다. 한컴 진단용 HWP 저장에서 유효 PageDef는
   기본 여백이며, 원문에 colPr만 추가하면 Print가 선언 여백을 적용한다
   (`italic-column-origin-control-253.json`). 두 Print의 첫 원점은 각각
   (85.0339,107.6424)pt와 (56.6093,65.0889)pt다. 원본·대조군은 구분하며 원본의
   미달을 대조군으로 대체하지 않는다. manifest 변경 두 대조군은 info에서 입력 파싱
   실패해 Print 요청하지 않았고 정상 대조군으로 세지 않는다.
4. 실제 Times italic: Native 94.84380%, fresh WASM도 미달이다. 실제 공급 프로그램·
   PDF의 한글 fallback·브라우저 face 선택을 추가 대조한다. Arial의 100%로 대신하지 않는다.
5. #7694 새 chart-before/after는 각각 Native 4.81258% / 89.01177%다.
   공개 저장 경로와 원본 OLE/OOXML 앵커·제목·축을 추가 분석한다.

추가 보정은 아직 빌드·시각 재검증 전이다. 새 회귀·fixture/golden은 해당 원본의
Native/fresh WASM 90% 이상·직접 판독을 완료한 뒤에만 추가하며 미달 기대값을 갱신하지 않는다.

### 실제 Canvas 기울임 계약 재확인

`canvas-italic-font-contract-256.json`에서 CSS 문법 수용 여부와 실제 잉크 메트릭을
분리했다. HCR 48px H의 우측 잉크는 normal 34px, 기본 italic 42px,
`oblique 18.418deg`도 42px이며 Canvas getter는 지정 각도를 버린다.
현재 CSS 각도만으로 공통 shear가 적용됐다고 주장하지 않는다. 실제 italic face의
glyph coverage를 확인해 그 경로는 유지하고 upright source에만 공통 합성을 적용하는
추가 보정이 필요하다. Times 대조군 미달도 함께 확인한다.

fresh WASM017의 81조건 실행은 종료됐다: 수치 통과 73조건, 미달 8조건, exit 1.
이전 패키지의 ExtB 상한 예외로 중단됐던 preserve-042/054/099는 실제 새 WASM에서
모두 렌더 완료했고 각각 92.56774% / 92.56774% / 96.49315%다.
이는 전체 #8064 fresh WASM 179조건의 최종 보존 판정이나 현재 미커밋 clip 보정의
검증을 대신하지 않는다. 전체 실패 목록은 `wasm017-combined-252`의 progress/results를 따른다.

## 공통 소비 경로 보정 묶음과 Native043

0 전진 글자의 잉크 범위를 본문 clip과 paint bounds가 함께 소비하도록 보정했다.
실제 source cmap와 face 스타일을 선택적으로 등록하여 정체 프로그램만 공통 -0.333
잉크 변환을 적용하고, 실제 italic/oblique는 보존한다. Canvas의 CSS 각도 지원 판정을
실제 합성의 증거로 사용하지 않는다. 미수집·자원 상한은 기존 브라우저 선택을 보존한다.

빈 셀의 상속 글자모양과 줄 메트릭은 일반 재조판 생산자를 사용한다. 읽은 원본의 빈
한 줄 em 캐시는 합성 입력뿐 아니라 독립 한컴 저장의 3600/3060/2160HU와 대조했다
(`table-effective-line-metrics-262.json`). 원본을 바꾸지 않고 측정과 배치의 공통
FrameGeometry를 갱신한다. 비어 있지 않은 압축 줄·다중 안내 줄·0높이는 제외한다.

단 정의가 없는 writer HWPX의 유효 기본 용지는 원본 Print와 colPr만 추가한 반례의
실제 원점 차이에 따라 parser에서 결정한다. 단 정의가 있는 정상 문서는 보존한다.
위 보정을 한 제품으로 동결해 Native043으로 빌드한다. 아직 성공·시각 통과를 주장하지
않으며, 원본과 독립 Print의 재검증을 마친 뒤 회귀의 추가·변경을 결정한다.

### Native043 관측과 잔여 범위

최초 빌드의 Arc 타입 오류를 수정했고 두 번째 빌드가 215.43초·exit 0으로 끝났다.
`logs/char-overlap-native-build-043-attempt02.log`와 제품 소스 지문을 보존했다.
`ink-face-critical-265-ink-face-fixed-native`는 15조건 중 13조건이 90% 이상이다.
0폭 자소, 현대 자모 양 계열, bold-italic, 실제 Arial/Times italic, writer 원본 두
engine과 colPr 반례는 100%다. 소스 경계 혼합 입력은 99.18718%로 개선됐다.
빈 인라인 표 두 원본은 본체 크기만 맞고 부모 줄 기준선이 낡아 73.79481%와
62.64501%다. 이 원본의 최종 통과나 새 회귀 추가를 주장하지 않는다.

Times 한글의 Print 내장 글리프와 실제 HBATANG.TTF의 윤곽·전진·em이 같다
(`physical-italic-hangul-source-audit-264.json`). 별도 공급 폴더에서 사용하지 않는
Malgun을 제외한 62,697,380바이트를 공급해 Native의 64MiB 한도를 지켰다. 과거
글꼴 조건을 덮어쓰지 않았으며 같은 공급으로 frozen Native042도 100%임을 확인했다.

#7694는 별도 진단용 한컴 저장본을 얻었다(`mcp-chart-effective-save-269`). 원본
Print를 기준으로 유지한다. chart-before의 실제 저장 줄은 18750/15938HU로 바뀌며
원본의 1000/850HU 캐시가 개체 높이를 담지 못함을 확인했다. 정규화 저장본도 Native
89%대로 차트 내부 라벨·간격이 남아 있어, 호스트 줄과 차트 내부를 모두 보정해야 한다.
현재 묶음을 로컬 중간 커밋으로 고정한 뒤 후속 공통 프레임 보정을 계속한다.
fresh WASM018·전체 Rust·전체 시각 검증·최종 확대 판독은 아직 완료하지 않았다.

## 단일 인라인 상자의 저장 호스트 프레임 보정

`composer/inline_box_rows.rs`는 실제 인라인 상자를 담지 못하는 단일 저장 줄을
기존 렌더 정규화의 inline_frames 경로에 투영한다. 한 개체와 중립 표지만 허용하고
여러 줄·캡션·고정 줄간격·다른 세로 정렬은 기존 소유자에게 둔다. 표는 활성 글꼴보다
작은 빈 셀 캐시가 있을 때 기존 HeightMeasurer의 실제 본체 높이를 소비하며, 표의
바깥 여백을 같은 줄 상자에 포함한다. 개체 높이와 줄 기준선은 일반 재조판의 같은
apply_inline_control_line_height에서 발행한다. 단순 paint 좌표 clamp를 추가하지 않는다.
기존 정규화가 후속 원점과 공통 composed 결과를 함께 갱신하며 편집 원문은 보존한다.

Native043에서 실패한 표 두 조건과 #7694의 원본을 Native044로 직접 비교한다.
정식 회귀 추가·기대값 변경은 Native/fresh WASM 조건 충족 후에만 수행한다.

### Native044 관측과 직접 판독

Native044는 218.82초·exit 0이다. `ink-box-critical-277-ink-box-fixed-native`의
17조건 중 15조건이 수치 통과했고, 표 선두/중간 두 원본은 모두 100%로 개선됐다.
새 review·standalone overlay는 `ink-box-critical-panels-279`에 보존했다. 직접
판독에서 글자·결합 부호의 소속과 표 본체·호스트 기준선이 원본 Print와 맞는 것을
확인했다. 100%는 2px raw 실루엣이며 RGB/획 AA 완전 일치라고 보고하지 않는다.

#7694 chart-before는 이전 4.81258%에서 89.38709%로 개선됐다. 위쪽 잘림은
해소됐지만 차트 내부 축·라벨 간격이 남았다. chart-after도 89.00396%로 미달이다.
원본 OOXML·Contents·OlePres000을 `pr7694-chart-source-268`에 보존했다.
전체 전역 fontdb와 명시 HCR 두 파일의 선택을 독립 resvg 진단으로 대조한 결과
같은 HCR Dotum 프로그램을 골랐고 렌더 PNG도 바이트상 같았다. 이 입력의 내부
차이를 글꼴 누락으로 처리하거나 무관한 글꼴 조달 변경으로 해결하지 않는다.
`logs/native-svg-font-probe-278-attempt02.log`를 참고한다.

다음 단계는 같은 Native044 제품 소스의 최적화 fresh WASM018이다. Native 결과만으로
새 회귀·최종 통합 판정을 만들지 않는다. 남은 차트 내부와 다른 필수 항목의 보정도
계속하며, 원격 push/PR은 준비 완료 후 작업지시자의 승인 전에 수행하지 않는다.

## Native044 / fresh WASM018 전쪽 재검증과 CSS face 조회 보정

같은 제품 소스의 Native044와 최적화 fresh WASM018을 사용했다. WASM018은
710.57초·exit 0이며 Studio JS/WASM 동기화 해시가 일치한다. 기존 74조건에 자소·표·
단 정의·업데이트된 차트 경계를 더한 82조건의 전쪽 검증에서 Native는 80조건,
fresh WASM은 81조건이 수치 통과했다. Native의 #7694 chart-before/after는 각각
89.38709%/89.00396%, WASM chart-before는 87.98250%다. 두 빈 표 원본은 양쪽
100%다. 이 결과는 최종 upstream/devel 전체 보존 검증이나 전체 Rust 완료가 아니다.

`ink-box-combined-280-ink-box-fixed-native`, `ink-box-combined-289-ink-box-fixed-wasm`에
TSV와 입력·Print·제품 지문을 보존했다. 중간 Native042→044의 같은 입력·PDF·글꼴
80조건에서는 0.1%p 이상 하락이 없었다. 이는 고정 devel 대비 179조건 대조를 대체하지 않는다.

### 실제 Canvas의 기울임 소비 확인

`canvas-italic-actual-paint-295.json`에서 실제 원문 `Ha 나`는 등록 완료 뒤에도
CSS italic + CTM c=0으로 그려지고, 장식 점만 공통 c=-0.333을 소비했다. 등록 실패나
본문 호출 경로의 누락이 아니었다. TextFontCoverage가 CSS 체인을 내부 비인용 목록으로
분리해 따옴표를 family 이름에 포함한 것이 원인이다. 기존 svgtypes 파서로 이름을
해석하도록 고쳤다. 플랫폼 generic의 실제 face는 등록 정보만으로 확정하지 않는다.

수정 전·후 같은 HCR 프로그램의 독립 진단에서 큰따옴표/작은따옴표 family는
`None → 정체 face`로 바뀌고, 비인용 family는 동일하며 generic은 양쪽 None이다
(`logs/quoted-font-selection-297.log`). 진단은 정식 회귀나 Print 일치 판정을 대신하지
않는다. Native045/fresh WASM019로 실제 출력과 실제 italic 프로그램을 다시 대조한다.

### 차트의 별도 반례와 남은 배치 차이

원본 `samples/143E433F503322BD33.hwp`의 구형 차트는 최신 제품에서도 Native
99.27590%/WASM99.29729%를 유지했다. 새 패널 `legacy-user-current-panels-294`에서
제목·막대·값축·범례를 직접 확인했다. 본문 일부의 줄 간격 잔차는 별도로 남아 있다.

#7694는 편집 가능한 OOXML 차트다. 공개 CSV 편집 경로로 첫 값만 4.3→8.6으로
바꾼 원본을 한컴에서 Print했으며, Native84.37007%/WASM87.99311%로 내부 배치
미달을 확인했다. 낡은 OlePres000 미리보기로 대체하지 않는다. Contents의 물리 영역은
0으로 비어 있고 title도 없어 현재 OOXML의 실제 배치 근거로 수용할 수 없다
(`logs/chart-legacy-presentation-trace-286-attempt03.log`).

네 글꼴 속성 대조군의 한컴 Print와 추가 세 대조군을 확보했다
(`mcp-chart-font-controls-290`, `mcp-chart-font-controls-296`). 기본 10pt에서 제목
14pt 명시 Print는 원본과 픽셀 동일하지만, 기본9pt/12pt에서 제목을 각각12.6pt/16.8pt로
명시한 Print는 생략본과 같지 않았다. 따라서 단순 1.4배 상속 가정은 채택하지 않는다.
원 해결 이슈 #6624와 PR #6626의 실측·상수 도입 근거를 확인했으며, 현재 Print의
값축·제목·범례를 독립 근거로 추가 분석한다. 차트 상수를 점수에 맞춰 조정하지 않는다.

원격 원문은 `open-pr-heads-298.json`에서 45PR/10Draft이며 이전267 대비 head 변경이
없다. upstream/devel도 `5e2b9bfbf08573a31fa1e11a2445ff63fc0755c0`으로 같다.
새 PR·push·comment는 준비 완료 후 작업지시자의 승인 전에 실행하지 않는다.

Native045는 218.57초·exit 0이며 fmt 검사도 통과했다. 이 단계의 코드 변경은 CSS
family 조회 1곳이다. Native 82조건 재검증과 같은 소스의 최적화 fresh WASM019를
이어 수행한다. 45개 원 PR 모두 reviewer 요청이 지정되어 있음도303에서 재확인했다.

### Native045 / fresh WASM019 결과

제품 코드는 `38ef03ee969b482776b91db6abeeaa9d6d9ea90c`에 고정했다. 최적화 fresh
WASM019는 708.97초·exit 0, Studio 동기화 해시 일치다. WASM SHA-256은
`dd84451796df922a1b98579c7b0450c6effa8b661f3617702ccdad8f6f127b49`다.
실제 Canvas 기록309에서 본문 H/a/나도 normal 프로그램과 CTM c=-0.333을 소비했다.

82조건·242쪽의 Native045 결과는 Native044와 완전히 같다(80조건 통과·차트2조건
미달). fresh WASM019는 81조건 통과·기존 chart-before87.98250% 미달이다.
WASM018 대비 아래4조건만 개선됐고 다른 모든 쪽은 같다. 양쪽 모두 중간 제품 대비
0.1%p 이상 하락이 없다(`native044-native045-intermediate-comparison-304.json`,
`wasm018-wasm019-intermediate-comparison-319.json`). 고정 devel 전체 검증과 구분한다.

| 조건 | WASM018 | WASM019 |
| --- | ---: | ---: |
| italic | 99.04077% | 100% |
| bold-italic | 99.19506% | 100% |
| 실제 Arial italic + 한글 fallback | 98.97001% | 100% |
| 실제 Times italic + 한글 fallback | 99.35042% | 100% |

실제 Canvas의 italic 전후 표준 패널314를 직접 판독했다. 여섯 강조 기호와 본문 H/a/나의
기울기·위치가 Print와 맞는다. 이100%는2px raw이며 작은 가장자리 AA 잔차가 남는다.
다른 세 개선 조건의 패널은320에 별도로 산출하고 확대 판독한다.

#8064 전체179조건·2,082쪽의 Native045/fresh WASM019 보존 검증은 각각307/310에서
진행한다. 같은 원본·Print·DPI·글꼴을 고정 devel5e2b와 대조한다. 일부 진행값을 전체
통과로 쓰지 않으며, 두 실행의 최종 exit code·전쪽 비교·직접 판독을 확인해야 한다.

### 공개 API 회귀와 source 단위 테스트 정책

정책313에서 `pua_glyph.rs`에 추가한 cfg(test) 모듈1개 때문에 base 대비4102>4101로
실패했다. 테스트를 `tests/cases/text_font_coverage_registration.rs`의 공개 API 계약2건으로
옮겼다. 등록한 정체 face의 CSS 선택·잉크 변환 및 원문 HWP/HWPX 불변, 선택적32MiB
상한의 비중단·이전 스타일 추론 무효화를 확인한다. 공개 Noto 프로그램을 명시적인
FontFace 별칭으로 등록하는 계약 검사이며 그 별칭 문서의 한컴 형상 일치를 주장하지 않는다.

초기 검사에서 미등록 family가 시스템의 알려진 fallback으로 가는 상태를, 등록된 프로그램을
해석할 수 없어 추론을 중단하는 상태와 같다고 가정했다. 두 상태의 독립 계약에 맞게
검사를 정리했고, 초기316은 같은 공개 API와 원본 프로그램으로 동결 Native044에서
의도한 face 선택 실패2건, Native045에서2건 통과다. 양쪽 모두 -D warnings로 컴파일했다.
추론을 하지 않는 상태에서 요청한 family 사다리와 선언 기울임을 유지하고, 뒤의 정상
프로그램 등록이 그 미검증 상태를 양수로 덮지 않는지 검사한다.

정책315는4101tests/296modules로 통과했다. source 삭제는 파일 끝의 cfg(test) 모듈뿐이며
제품 prefix 바이트 동일성은317에 입증했다. 자동 수집은 regression_suite_019에 배정됐고
manifest의 고정 base 검사도317에서 통과했다. 정식 Cargo 경로의 집중 실행318과 전체
필수 Rust/lint 검증을 마쳐야 최종 제출 검증 완료로 판단한다.

### 중첩 표·미주 실패의 다음 공통 프레임 보정 근거

변경하지 않은 원본 #7833 입력은 Native04572.41579%/WASM01972.30346%다.
중첩 표를 담는 저장 줄만 선언 본체6000HU+원본 바깥여백566HU와 일반 기준선으로
바꾼 진단본, 해당 줄만 생략해 일반 재조판한 진단본은 양쪽 모두100%다(305/312).
원본과 변경한 진단 입력은 SHA·범위를 구분하고 원본 보정 완료로 보고하지 않는다.
원본 Native의 표준 review·overlay306도 판독했다. 앞/뒤 글자가 Print의 아래 기준선이
아닌 위쪽에 배치되는 문제가 확인됐다.

공통 소비 경로는 `recompose_cell_lines_with_nested_tables` → `FrameGeometry` →
HeightMeasurer와 table_layout의 `paragraph_with_frame_geometry`다. 현재 저장 줄이 있는
호스트는 이 프레임 재조판에서 제외된다. 다음 보정은 본체 크기 전달과 줄 흐름의 소유를
구분해야 한다. 기존 `inline_table_heights`는 둘을 함께 의미하므로 저장 줄의 높이 보정만으로
새 줄 나눔의 소유를 선언하지 않는다. private helper 이후의 실제 배치까지 함께 확인한다.

기존 실패 검사는 생성 참조 번호를 셀의 직접 TextRun으로 가정한다. 현재 실제 출력은
FnMarker와 원문 TextRun을 구분한다. 원본 형상 보정과 양쪽90% 이상을 먼저 완료한 뒤
번호·원문·소스 주소의 독립 소유를 검증하는 회귀로 보완한다. 미달 상태에서 기대값을
낮추거나 입력을 정상 진단본으로 교체하지 않는다. 차트 내부 자동 배치도 계속 미해결이다.

정식 Cargo318에서 helper가 quick-xml0.42의 문자열 QName을 바이트로 취급해 컴파일이
실패했다. 개별 컴파일이 캐시의0.41을 선택했던 점을 확인했고, helper를 실제0.42 API로
보정했다.321에서 dependency .d의0.42 버전까지 고정해 -D warnings로 다시 컴파일했다.
source SHA는 `0c5967a076a3b5c4cd6401ddba46cb8aced3fdb031dec0e20a8a76c043fd3627`다.

기존 private 검사의 가시 PUA 재사용 금지도 공개 API로 유지했다. 공개 Noto 프로그램의
H 윤곽을 가리키는 합성 format4 cmap를 메모리에만 만들고, 등록→실제 source PUA 방출→
대용량 보조 정보 거절→후속 정상 등록에서도 과거 PUA 미재사용과 원문 HWP 불변을
검사한다. 파일·glyph 윤곽·사설 프로그램을 추가하지 않는다. 이 검사는 물리 배치의
정답지가 아니라 공급 프로그램과 원문 주소의 계약 검사다. 최종321의 동결 Native044는
1PASS/2FAIL, Native045는3PASS다. PUA 자원 상한은 기존 정상 대조군이고 두 face 선택
실패만 CSS 인용 결함의 검출 증거로 사용한다. 정식 Cargo는322에서 다시 실행한다.

패널320의 bold-italic과 실제 Arial/Times italic도 같은 배율로 직접 판독했다.
Latin의 실제 italic 프로그램과 한글 fallback의 합성 기울임이 함께 맞으며, 장식 기호의
소속·간격과 본문 기준선이 보존된다. 가장자리 AA 잔차는 RGB 완전 일치로 보고하지 않는다.

자동 배정이 source weight 변경 뒤019→003으로 바뀌었다.322는 이전 suite를 고정해
0건 실행·exit4였으므로 통과로 세지 않는다. case 실행 wrapper로 실제 자동 배정을
선택한324는 정식 nextest3건 모두 통과했다(컴파일12.00초·실행0.408초). 정책 Node323도
35건 통과했고, manifest322는1652sources/7152static test attrs로 고정 base 검사를 통과했다.
최종 제출 전 전체 nextest·필수 lint·전체 시각 대조와 남은 원본 보정을 계속한다.

## 저장 중첩 표 호스트의 공통 프레임 보정

#7833 원문과 원 해결 #4961→#5122의 주소 계약을 대조했다. 높이만 수정한 진단 입력과
일반 재조판 입력은 모두 Native/fresh WASM100%지만, 원문은72%대로 남아 있다.
수정 근거는 단일 저장 줄이 인라인 본체·원본 바깥여백을 담지 못하는 데 있다.
원문 문자·제어문자 순서·줄 경계·HWPX 주소 축은 그대로 유지해야 한다.

`inline_box_rows::normalized_host_with_table_height`로 본문과 셀의 규칙을 공유한다.
셀은 기존 nested_height 측정 callback을 사용해 같은 깊이·폭·호환 정책을 유지한다.
고정 높이 표는 선언 크기를 보존하고, 보호되지 않은 상자가 저장 줄 밖에 있거나
빈 em 캐시가 부실할 때 실제 본체 높이를 같은 측정기에서 받는다.

`nested_table_frames`는 기존 가로 프레임 처리를 그대로 거친 뒤 높이 결과를 발행한다.
`FrameGeometry.stored_partition`은 물리 높이와 재줄바꿈 소유를 구분한다. 높이만
갱신한 저장 줄은 source line membership·HWPX char axis를 유지한다. 기존 재조판
프레임의 소유·주소 갱신과 빈 em 프레임의 기존 동작은 보존한다.

생산→소비는 공통 normalized_host → 셀의 recompose_cell_lines_with_nested_tables →
FrameGeometry → HeightMeasurer와 table_layout의 paragraph_with_frame_geometry다.
원본을 바꾸지 않고 해당 줄의 기준선·점유를 실제 배치까지 전달한다. HWP3 원점 보존,
NO_LS의 기존 재조판, 여러 줄·여러 비중립 개체·캡션·고정 줄간격은 기존 경로를 따른다.

새 제품 Native046/fresh WASM020에서 원문과 정상 대조군을 검증한다. 진행 중인
Native045/WASM019 전체179조건 검증은38ef03ee96 제품의 보존 증거이며 새 코드 결과로
보고하지 않는다. 원문 양쪽90%·직접 판독 전에는 기존 미주 회귀의 기대값을 바꾸지 않는다.

Native046 빌드는 성공했다(268.08초, 공유 target/pr-review·잡8).330에서 바꾸지 않은
HWP/HWPX 원문은 각각100%이고, 저장 높이만 바꾸거나 생략한 두 진단본도100%다.
원문 HWP는329의2020 PrintMethod0 출력 SHA
`dd662cb83ed3dcfbede58117e723dd8841dbc0ba1a0f60bbcc4b45513a941cba`와
대조했다. 진단본의 통과와 원문 검증을 구분한다. 새 fresh WASM020·직접 판독·정상 대조군은
계속 수행하며, 이 Native 결과만으로 미주 회귀 기대값은 아직 바꾸지 않았다.

## 원본 중첩 표의 양쪽 선행 검증과 새 회귀

Native046/fresh WASM020은 변경하지 않은 HWP/HWPX 원문 각각1쪽 모두100%다.
331/346 review·overlay에서 셀·기준선·번호·후속 원문을 직접 판독했다. 테두리 색·AA의
RGB 잔차는 실루엣100%와 구분한다. 원본·독립 Print·전쪽 TSV·대표 PNG를
`mydocs/pr/assets/issue_7833_nested_source`에 같은 바이트로 보존했다.

새 `nested_inline_table_source_ownership`의348 독립 실행은 Native045에서 기준선1FAIL·
주소 정상 대조군1PASS, Native046에서2PASS다. 줄 상자 원점과 실제 글자 기준선은
구분해 bbox.y+baseline으로 검사한다. 기존 검사의 직접 셀 TextRun 가정만 제거하고
실제 FootnoteMarker·생성 미주 접두사·원문 주소와 trace 전후 모델/저장 불변을 보호했다.
349 정식 Cargo는 harness 미갱신으로0건·exit4여서 통과가 아니다.353 prepare/check는
1653sources/7153static attrs로 성공했고354에서 정식 자동 수집을 재확인한다.

기존82조건·242쪽은 Native045→046, WASM019→020의 페이지별 점수가 모두 같다.
남은 #7694 차트 미달은 유지되며 전체 통합 완료로 보고하지 않는다.

## #8064 전체 보존에서 검출한 sample16 회귀

Native045의179조건 중177조건은 통과했으나 sample16 HWP5/HWPX가64→66쪽으로
후퇴했다. 같은 입력·Print에서 base는64쪽이며 Native043도64쪽,044/045/046은66쪽이다.
340/341/342에서 첫 차이를8쪽 문단196의 자리차지 사각형으로 추적했다. 실제 Common
프레임 높이는1300HU, 내부 그리기 좌표는6832HU다. 새 inline_box_rows가
flow_height_hu의 상한을 저장 줄 높이로 잘못 대입해 뒤 표197이 다음 쪽으로 밀렸다.

원 해결 #4333→#4607과 실제 공통화 e7bc06c6e988를 읽었다. 기존 max 흐름 상한을
전역으로 바꾸지 않는다. 새 저장 호스트 정규화의 입력을 실제 배치의 Common 프레임으로
맞추고 sample16 양쪽64쪽·전쪽90%와 기존 중첩 표·표 선두/중간·차트 호스트를 함께
재검증한다. 이관이나 기대 쪽수 변경으로 회귀를 면제하지 않는다.

## 편집 가능한 차트 제목의 독립 대조

338/339에서 제목14pt를 명시하면 전역 글꼴9/10/12pt의 제목 픽셀이 완전히 같았다.
기본 제목은 전역12pt에서 명시18pt와 같고, 단순1.4배 가정은 성립하지 않는다.
351/352의 같은 바이트 원본→9pt→원본 재인쇄3건은 기존 Print와 전쪽 픽셀이 모두
같았다. 실행 순서나 MCP 세션의 불안정으로 이 차이를 설명하지 않는다. 기본 크기·
자동 플롯의 실제 규칙을 더 확인하며 관측 좌표를 제품 상수로 맞추지 않는다.

354의 정식 자동 수집은 실제2건·2PASS다(컴파일13.98초, 실행0.041초).
이 단계의 원본·새 관계 회귀·증거를 일반 커밋으로 고정하고 sample16 보정을 계속한다.

## 실제 도형 프레임을 저장 호스트 줄에 전달

새 정규화의 Shape 분기만 실제 paint에 전달되는 Common 높이를 읽게 한다.
내부 도형 좌표가 더 큰 경우의 흐름 상한은 기존 함수와 소비 경로에 그대로 둔다.
클리핑·숨김·문서 ID 분기·좌표 clamp는 추가하지 않는다. 원본의 Common 높이와
여백이 저장 줄에 들어맞으면 줄과 후속 저장 원점을 투영하지 않는다. Native047에서
기존 중첩 표4조건, 표 선두/중간·글꼴·차트82조건, sample16 HWP5/HWPX64쪽을
같이 대조한다. 보정 후의 페이지 수·전쪽 Visual Sweep·fresh WASM은 실행 전이다.

Native047 빌드는242.51초·exit0이다.357의 실측 pagination dump는 두 원본 모두
64쪽으로 복원됐고, 중첩 표 원문·두 진단본4조건도Native100%를 유지했다.
현재88조건·374쪽의 Native TSV와 고정 base 대조를 진행한다. sample16의 전쪽
90%와 직접 판독, 같은 제품 소스의 fresh WASM021은 아직 완료 전이다.

## 도형 호스트 보정의 양쪽 완료 페이지와 새 회귀

Native047/fresh WASM021의 sample16 HWP5/HWPX는 모두64쪽이다. 최저 raw는 각각
Native97.73925/96.24353%, WASM97.69328/96.14581%이며 고정 devel 대비0.1%p
이상 하락은0쪽이다.359/371에서 같은 입력·PDF·글꼴·세대·프로필·스크립트 계약도
대조했다. 영향8쪽은 네 경로 모두100%이며360/371의 review·overlay에서 제목 뒤
표의 시작·셀·종료를 직접 확인했다. 기존 최저 쪽의 잔차는 해결 주장과 구분한다.

373은 새 이름4개 검사로 Native046에서4FAIL,047에서4PASS다. 원본과 Print를
그대로 쓰고 전쪽 수와 내용 소속·순서·누락/중복을 함께 보호한다.100%인8쪽만
독립 PDF 원시 텍스트 원점·수평 괘선에서 얻은 기준선354.239990234375px, 표 상단
366px, 하단994.5919596354166px를±2px로 고정한다. 이 좌표는 출력에서 역산하지 않았다.
새 evidence는 `stored_shape_host_frame_20261020`에 보존하며 원본/PDF는 이미 커밋된
동일 파일을 재사용한다. 정식 Cargo 자동 수집과 필수 전체 검증은 이어 수행한다.

365의 첫 SSH 결과는 Windows 오류 인코딩 때문에 읽지 못했다. UTF-8 출력을 지정해
366에서 등록 글꼴 경로·SHA를 확인했고,367은 HANDotum을 ignored output에만 복사했다.
시스템 등록본을 바꾸지 않았다.368은 현재 공급본2.100과 Windows2.002의 차트 한글·숫자
윤곽·전진 폭과 hhea/OS2 세로 메트릭이 같음을 확인했다. 실제 MCP의 선택 글꼴을
이 Windows 조회만으로 단정하지 않으며 큰 배치 차이를 이 버전 차이로 설명하지 않는다.

369의 첫 생성은 modern chart가 switch/case 안에 있는 점을 놓쳐0입력이었다. 이를
통과로 세지 않고 primary chart와 fallback OLE의 Common 크기를 함께 수정한4진단본을
다시 Print했다. 원본의 no-LS 호스트·원래 내부 좌표는 유지했다.374에서 절반215×125와
높이만두배430×500 모두 생략 제목이 명시14pt와 같았다. 따라서 차트 크기에 따른
제목 자동 축소 가정은 배제한다.375는 전역9/11pt의 생략값과 명시12/16pt를 분리해
원래 기본 크기의 실제 규칙을 추가 확인한다. 진단본 통과를 원본 통과로 대신하지 않는다.

## 실제 도형 프레임의 정식 회귀 검증 완료

표 내용의 추가·누락·중복을 전체 문자열로 확인하도록 보완한 최종 검사도 Native046에서
4FAIL, Native047에서 4PASS였다. source의 정규화한 셀 내용과 실제 표 내용 567자가
완전히 같은지 독립 대조했다. 382에서 suite prepare·check 후 실제 case resolver가 고른
regression_suite_025를 실행했고, 새 4건과 중첩 표 정상 대조군 2건 모두 통과했다.
컴파일 12.74초, 실행 0.644초, 공유 target/pr-review, 빌드 잡 8개·테스트 스레드 12개다.
전체 Rust·lint·Studio 및 통합 검토 완료와 구분한다. 실행 로그·중간 JSON은 output에만
보존하고 새 회귀·최종 TSV·대표 PNG와 이 단계 기록을 일반 커밋으로 고정한다.

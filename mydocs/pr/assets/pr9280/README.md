# PR #9280 검증 자료

## 동일한 본문과 다른 파일 버전

한컴 2020에서 HWP 5.0.1.5와 5.0.1.6 원본을 직접 인쇄하고, 한컴 2020과 2024에서 각각 HWPX로 저장했다. 15개 원 기호의 본문과 굵은 글자의 UTF-16 시작 위치를 비교했다. 인접한 여덟 문자를 추가한 대조군에도 같은 절차를 적용했다. [원본 표본](../../../../samples/issue5140-version-boundary/README.md) · [구형 인접 기호](neighbors-v5.hwp) · [현대 인접 기호](neighbors-v6.hwp) · [독립 PDF](../../../../pdf/issue5140-version-boundary/)

초기 기여본의 저장 결과와 메인테이너 보완 전의 추출값은 접수 당시 자료에 남겼다. [15종의 저장·추출 비교](symbols.tsv) · [인접 코드 비교](neighbors.tsv) · [한컴 작업 기록](hancom-jobs.tsv)

## 같은 코드로 저장한 괘선과 현대 문자

기존 #5800 표본의 FileHeader는 5.1.0.1이다. 본문에 있는 U+A832 84개를 한컴은 현대 문자로 보존한다. 한컴 2020 Print의 포함 글꼴을 열어 보니 해당 84자리에서는 굴림의 누락 글리프 윤곽이 인쇄됐다. 같은 HWP의 파일 버전 바이트만 5.0.1.5로 바꾸어 다시 인쇄하면 같은 자리에서 이중 가로 괘선이 나타난다. 두 문서 모두 한 쪽이며, HWPX 재열기에서도 본문 문자와 출력의 버전 차이를 확인할 수 있다. [현대 HWP](../../../../samples/issue5800-hancom-symbol.hwp) · [현대 Print](../../../../pdf/issue5140-version-boundary/issue5800-hancom-symbol-2020.pdf) · [구형 HWP](../../../../samples/issue5140-version-boundary/issue5800-v5.0.1.5.hwp) · [구형 Print](../../../../pdf/issue5140-version-boundary/issue5800-v5.0.1.5-2020.pdf)

## 글꼴 폭의 근거

Windows 한컴에서 사용한 함초롬바탕을 포함해 함초롬바탕·돋움 글꼴 프로그램 6개에서 cmap과 hmtx를 읽었다. U+A000–U+A4CF와 U+A800–U+A87F의 글리프 유무와 전진폭이 여섯 프로그램에서 각각 일치했다. U+A806은 글리프가 있고 전진폭이 0이다. U+A832는 이들 글꼴에 글리프가 없다. 표의 구간 해시는 코드 순서대로 전진폭을 16비트 빅엔디언으로 직렬화해 계산했으며 미수록 문자는 65535로 기록했다. [프로그램 해시·구간별 측정](font-metrics.tsv) · [원 사례 검증에 사용한 글꼴 선택](windows-font-environment.json)

## 단독 결합 부호와 명시적인 점선 원

HWP 5.0.1.6 공개 표본의 본문을 편집해 30개 조합을 만들었다. 첫 문서는 라틴 글자·실로티 나그리 글자·반복 부호·직접 입력한 원을, 둘째 문서는 공백·모음 부호·글머리 기호를, 셋째 문서는 구두점과 폭 없는 연결 문자를 다룬다. 세 HWP를 Windows 한컴 2020에서 직접 인쇄해 본문과 PDF 문자 그리기 순서, 후속 C의 가로 위치를 쪽별로 대조했다. 한컴 PDF는 폭 없는 연결 문자 두 종을 문자표에 공백으로 적는다. HWP와 HWPX의 본문 검사에서는 원래 코드 포인트를 그대로 비교한다. [검증용 HWP](../../../../samples/issue5140-version-boundary/) · [한컴 Print PDF](../../../../pdf/issue5140-version-boundary/) · [문단별 원문·표시·좌표](syloti-hancom.tsv)

라틴 B 다음에 붙인 U+A806에는 점선 원이 나타난다. 같은 실로티 나그리 기본 글자 뒤에서는 부호가 정상 결합한다. 원을 직접 입력하면 원 하나의 폭이 조판에 반영되며, 단독 부호를 표시하려고 생성한 원은 뒤 라틴 글자의 조판 위치를 밀지 않는다. HWPX 저장 후에도 이 구분을 확인했다. [Microsoft의 결합 대상이 없는 부호 처리 규칙](https://learn.microsoft.com/en-us/typography/script-development/use#defective-clusters)

## 짧은 줄과 긴 줄에서 U+A832 뒤의 위치

Windows 한컴 2020에서 함초롬바탕과 함초롬돋움의 보통·굵은 글씨로 같은 문자를 인쇄했다. U+A830–U+A839에 해당하는 40곳 모두 굴림의 글리프 0을 그렸다. 굴림 파일에서 이 글리프는 윤곽이 있으며 전진폭은 1em이다. 앞뒤에 A와 B를 넣은 짧은 줄에서는 기호와 B의 원점이 약 한 글자 너비만큼 떨어진다. 각 글꼴의 실제 글리프 5개도 같은 인쇄물에서 대조했다. [60곳의 인쇄 글꼴·글리프·좌표](indic-hancom.tsv) · [검증에 사용한 프로그램별 문자표와 폭](indic-font-metrics.tsv)

10pt 기호를 너비 425.2pt인 문단에 10개부터 400개까지 넣었다. 한컴은 기호 84개와 85개를 한 줄에 넣고, 86개를 85개와 1개로 나눈다. 이때 한 줄에 들어간 기호의 표시 간격은 개수에 따라 좁아진다. 각 문단의 줄 수, 줄별 글자 수, 첫 글자와 마지막 글자의 원점은 독립 PDF에서 직접 읽었다. 원래 문서와 HWPX 재열기 모두 이 30개 문단을 대조한다. [HWP 원본](indic/) · [저장한 HWPX](../../../../tests/fixtures/issue5140-version-boundary/indic/) · [한컴 인쇄본 11개](../../../../pdf/issue5140-version-boundary/indic/) · [독립 인쇄의 55줄](indic-line-breaks.tsv)

짧은 줄에서는 기호와 B가 각각 보인다. 84개가 든 실제 문서에서는 기호가 한 줄에 들어가고 도장란 문단이 바로 뒤에 온다. WASM의 문단 원문과 렌더 트리에서도 원본·저장본의 줄별 글자 수를 직접 확인했다. [짧은 줄의 수정 전후](indic-short-comparison.png) · [84개 연속 기호와 다음 문단](indic-long-comparison.png) · [글꼴과 줄별 비교](indic-controls.png) · [WASM의 60개 문단](indic-wasm-line-breaks.tsv)

## 저장과 화면 검증

30개 결합 조합의 HWP와 HWPX를 대조했다. 한컴과 표시 순서가 일치한 경우는 수정 전 24개, 수정 후 60개다. 편집·실행 취소·재저장 후 본문 인덱스와 후속 문자를 확인했다. 결합 부호 없이 줄 바꿈 없는 공백 U+00A0만 둔 대조군에서 후속 글자는 한컴보다 2.677px 오른쪽에 놓인다. 이 위치는 수정 전후 같은 값이다. [단독·정상 결합 확대](syloti-after.png) · [60개 표시 순서](syloti-display.tsv) · [13개 편집 상태](syloti-edit.json) · [공백 대조](space-control.tsv)

최종 HWPX 9개를 Windows 한컴 2020에서 다시 인쇄했다. 각 파일의 1쪽은 원본 인쇄와 96 dpi RGB 차이가 0픽셀이다. 한컴에서 저장한 문서와의 문자·굵은 서식 시작 위치는 32개 비교로 확인했다. [재인쇄 결과와 SHA-256](hancom-reprint.tsv) · [재인쇄 PDF](../../../../pdf/issue5140-version-boundary/reopened/final/) · [문단·UTF-16 서식 위치](save-xml.tsv)

최종 WASM을 Studio에 연결하고 Chromium에서 Canvas2D와 CanvasKit, 화면·인쇄 모드, 기본 글꼴 환경과 Windows 검증 글꼴 공급을 조합해 80번 렌더링했다. CanvasKit 40건 모두 해당 backend에서 그리기를 마쳤다. 단독 부호가 포함된 행 120곳의 픽셀을 확인했고 구형 원본·저장본의 화면·인쇄 출력 네 쌍은 수정 전과 동일했다. [두 backend의 실제 캡처 기록](studio-renders.tsv) · [CanvasKit의 수정 전후 확대](studio-syloti-ink.png) · [행별 픽셀 대조](studio-ink.tsv) · [구형 대조군](studio-legacy-control.tsv)

Native와 fresh WASM에서 원본·저장본·대조군 37쪽씩의 쪽별 TSV를 보존했다. 독립 Print와의 최저 실루엣 일치율은 Native 99.17%, WASM 98.97%다. [74개 측정값](indic-visual.tsv) · [쪽별 원본 TSV](tsv/) · [대표 비교·오버레이](visual/)

## 최종 회귀와 기존 문서 보호

Rust 전체 회귀 13,971개, 집중 회귀 36개와 Native 공식 검사 4,072개가 통과했다. 회귀에는 공개 글꼴 다섯 파일만 노출했고, 한컴 인쇄와의 시각 대조에는 문서별 공급 글꼴을 사용했다. 검토 디렉터리 아래 `target/pr-review/pr9280-source`에서 제품을 분리 빌드해 Native 실행 파일과 최적화된 WASM의 SHA-256을 기록했다. Studio 2,832개, Rust 포맷·Clippy·workspace 빌드와 새 표본의 보안 검사도 통과했다. [검사별 결과와 바이너리 SHA-256](local-validation.json) · [58개 입력의 검사 결과](sample-inspection.tsv)

보호 문서 23종은 Native와 fresh WASM 각각 2,023쪽의 공식값이 모두 같았다. 입력·독립 Print·글꼴·TSV 해시는 원본 TSV와 함께 보존했다. [쪽별 보호값 비교](protection/pages.tsv) · [측정 조건](protection/conditions.json) · [Native TSV](protection/native/) · [fresh WASM TSV](protection/wasm/)

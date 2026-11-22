# PR #9280 검증 자료

## 동일한 본문과 다른 파일 버전

한컴 2020에서 HWP 5.0.1.5와 5.0.1.6 원본을 직접 인쇄하고, 한컴 2020과 2024에서 각각 HWPX로 저장했다. 15개 원 기호의 본문과 굵은 글자의 UTF-16 시작 위치를 비교했다. 인접한 여덟 문자를 추가한 대조군에도 같은 절차를 적용했다. [원본 표본](../../../../samples/issue5140-version-boundary/README.md) · [구형 인접 기호](neighbors-v5.hwp) · [현대 인접 기호](neighbors-v6.hwp) · [독립 PDF](../../../../pdf/issue5140-version-boundary/)

초기 기여본의 저장 결과와 메인테이너 보완 전의 추출값은 접수 당시 자료에 남겼다. [15종의 저장·추출 비교](symbols.tsv) · [인접 코드 비교](neighbors.tsv) · [한컴 작업 기록](hancom-jobs.tsv)

## 같은 코드로 저장한 괘선과 현대 문자

기존 #5800 표본의 FileHeader는 5.1.0.1이다. 본문에 있는 U+A832 84개를 한컴은 현대 문자로 보존한다. 한컴 2020 Print의 포함 글꼴을 열어 보니 해당 84자리에서는 굴림의 누락 글리프 윤곽이 인쇄됐다. 같은 HWP의 파일 버전 바이트만 5.0.1.5로 바꾸어 다시 인쇄하면 같은 자리에서 이중 가로 괘선이 나타난다. 두 문서 모두 한 쪽이며, HWPX 재열기에서도 본문 문자와 출력의 버전 차이를 확인할 수 있다. [현대 HWP](../../../../samples/issue5800-hancom-symbol.hwp) · [현대 Print](../../../../pdf/issue5140-version-boundary/issue5800-hancom-symbol-2020.pdf) · [구형 HWP](../../../../samples/issue5140-version-boundary/issue5800-v5.0.1.5.hwp) · [구형 Print](../../../../pdf/issue5140-version-boundary/issue5800-v5.0.1.5-2020.pdf)

## 글꼴 폭의 근거

Windows 한컴에서 사용한 함초롬바탕을 포함해 함초롬바탕·돋움 글꼴 프로그램 6개에서 cmap과 hmtx를 읽었다. U+A000–U+A4CF와 U+A800–U+A87F의 글리프 유무와 전진폭이 여섯 프로그램에서 각각 일치했다. U+A806은 글리프가 있고 전진폭이 0이다. U+A832는 이들 글꼴에 글리프가 없다. 표의 구간 해시는 코드 순서대로 전진폭을 16비트 빅엔디언으로 직렬화해 계산했으며 미수록 문자는 65535로 기록했다. [프로그램 해시·구간별 측정](font-metrics.tsv) · [원 사례 검증에 사용한 글꼴 선택](windows-font-environment.json)

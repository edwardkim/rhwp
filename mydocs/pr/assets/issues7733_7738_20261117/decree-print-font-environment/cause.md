# 시행령 Print의 실제 영문 프로그램과 비교 환경

기준은 변경하지 않은 `78494-virtual-convergence-industry-decree-2020.pdf`다.
30쪽의 PDF 리소스는 PalatinoLinotype-Roman·Bold이고, 가운뎃점의 실제 glyph id는16·20이다.
PDF 내장 subset에는 cmap이 없으므로 Unicode cmap을 임의 추정하지 않고 MuPDF texttrace의
Unicode·glyph id 연결로 글리프를 확인했다. 두 내장 프로그램의 가운뎃점과 설치된
`pala.ttf`·`palab.ttf`는 윤곽이 완전히 같고, hmtx도512/2048em으로 같다.
[프로그램·윤곽·전진 근거](pdf-program-proof.json)에 전체 파일 SHA와 해당 glyph 비교를 기록했다.
비공개 글꼴·추출 글꼴은 Git에 포함하지 않는다.

명시 환경 없이 원본 이름 HCI Poppy의 공급 프로그램을 사용하면 같은 문장 `시설·`의
마지막 토큰이2.61px 넓어져 줄에서 밀린다. 독립 PDF는 Palatino의0.25em 점을 사용한다.
PDF 프로그램으로 재조판한 근거가 있으므로 `HCI Poppy`만 Palatino Linotype의 자연 전진을
선택한다. 다른 face는 원본 메트릭을 유지하며 문서·PDF·DPI·판정 문턱은 바꾸지 않는다.
명시 환경은 base/후보와 Native/fresh WASM에 똑같이 적용한다. 이는 원본 글꼴의
자동 대체 개선이 아니라 독립 Print가 실제 사용한 비교 환경을 재현하는 설정이다.

같은 설정의 base/후보 Native 전74쪽 정밀 수치는 모두 같았다. 기본 환경의30쪽64.73203%는
명시 환경에서100%이며, review·overlay를 직접 판독해 줄 소속·뒤 문단 위치가 맞음을 확인했다.
전74쪽 중59쪽이100%이나13·19·36쪽은98% 미만이므로 해결 완료로 처리하지 않는다.
fresh WASM과 남은 실제 조판 결함의 보정은 진행 중이다.

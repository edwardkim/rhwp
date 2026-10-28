# 계산 빈 호스트 글줄과 자리차지 표 프레임

제품 `bc649064faaa2193380c1ab5b2567b4470f2472e`, 고정 devel `5e2b9bfbf08573a31fa1e11a2445ff63fc0755c0`.
공개 표 생성 API의 호스트 글자 크기250/1000/4000HU와 offset0/1700HU를 분리한12입력,
다음 문단 앞 간격3000HU의 정상2행 표2입력이다. HWP/HWPX 입력·PDF 바이트는 변경하지 않았다.
글자 크기는 원본 글자모양을 복제해 raw cache를 비우고 `set_document`로 구성했다.
작은 HWP에 남은 계산 LineSeg의 옛 높이를 독립 저장 조판으로 취급하지 않는 반례다.
문서 ID나 글자 크기별 예외를 구현하지 않는다.

한컴2020(11.0.0.9136) MCP의 PrintMethod0·1-up PDF를 독립 기준으로 사용했다.
HWP의 `scripts_disabled`와 HWPX의 `none` 입력 전처리를 `provenance.json`에 구분했다.
원본·PDF SHA256, 엔진·출력 경로·전쪽 수·전처리와 Native/fresh raw·TSV SHA가 같은 파일에 있다.
로그·MCP raw 응답·중간 JSON·재실행 계획은 ignored output에 보존한다.

14입력의 전쪽1쪽은 Native/fresh WASM 모두 raw100%다. 같은 입력·Print·글꼴로 고정 devel과
각각 대조했고0.1%p 이상 하락은 없다. 실제 TSV56개를 native/wasm 및 base/native·wasm에
바이트 그대로 보존했다. 작업 중 경로가 TSV에 남아 있어도 입력·PDF의 SHA로 위 복사본과 대응한다.
새 회귀는 문단·표 소유, 앞뒤 내용 순서·보존, 줄이 표 위 공간에 들어가는 경우와 표를 피하는 경우,
큰 빈 줄의 점유와 앞 간격 중복 방지를 보호한다. 완료100%쪽은 `expected.json`의 독립 Print
격자/잉크 bbox에서 읽은 좌표를96DPI 종이 원점±2px로 추가 보호한다. 잉크와 font box를 같은
값이라고 가정하지 않는다. PNG에는 review와 standalone overlay를 함께 보존했다.

실루엣100%는 RGB 완전 일치가 아니다. 얇은 괘선의 raster 농도 차이는 남고, 실제 원본의
0.36pt(96DPI0.48px) 선 폭과 같은 외곽/줄 위치를 대조한다. 선을 임의1px로 두껍게 하지 않았다.

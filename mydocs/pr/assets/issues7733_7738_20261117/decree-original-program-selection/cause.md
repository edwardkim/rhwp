# 부분 Print 환경과 원본 TrueType 선택의 구분

시행령 18쪽의 남은 6픽셀은 같은 한양중고딕의 하이픈 두 곳과 K-META 부근에 있다. 원본 PDF의 H2gtrM TrueType 하이픈은 853/1024em이며 설치된 H2GTRM.TTF의 hmtx와 윤곽 명령이 같다. 다른 글꼴로 맞출 대상이 아니다.

그런데 기존 HCI Poppy→Palatino Print 환경 JSON은 그 한 항목만 선언했고 다른 원본 프로그램의 자동 선택을 포함하지 않았다. Native 실제 출력은 한양중고딕의 같은 이름 CFF `HY Junggothic`을 선택했다. 원본 메트릭은 TTF HYGothic-Medium의853/1024em인 채 표시 프로그램만 달라 하이픈 잉크가 짧아졌다. 자동 환경에서는 같은 HY중고딕의 TTF HYGothic-Medium이 선택됨을 실제 관측했다.

기존 독립 Print로 확인한 HCI Poppy의 Palatino 대응과 원본 메트릭 보존은 유지한다. 다른 face는 실제 자동 공급 환경의 TTF/TTC/OTF 선택을 그대로 가져와 완전한 새 JSON으로 기록한다. Native와 WASM의 자원 신원 표현을 구분하며 각 경로의 base/head에는 같은 환경을 쓴다. 이전 부분 JSON의 결과는 당시 자료로 보존하고 새 환경 결과로 바꾸어 쓰지 않는다.

현재는 선택 원인의 진단이며 새 환경의 집중·전쪽 비교와 보호 검증 완료를 뜻하지 않는다.

## 실제 집중 결과

새 환경에서13·18쪽은 Native와fresh WASM 모두100.00000%·미대응0이다. Native18쪽의 같은 쪽 review·overlay를 직접 판독했다. 이 조건의 de467 이전 소스와120483b82f 소스 값도 각각 같아, 프로그램 선택 보완과 코드 효과를 구분한다.61쪽은 Native99.55732% / WASM99.55724%·각480픽셀로 남는다. 이 부분은 상단 문장의 ASCII 작은따옴표 줄 소속이 독립 Print와 달라 다음 단계에서 원인을 확인한다. 새 환경 전쪽 검증은 아직 완료하지 않았다.

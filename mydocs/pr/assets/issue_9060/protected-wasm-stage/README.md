# sample10 fresh WASM 전쪽 보호 검증

> 이 문서는 당시 단계의 분석 기록이다. 최종 결과와 대표 증적은 [검증 기록](../verification-record.md)과 [최종 대상 증적](../final/main/README.md)을 따른다. 원시 로그·중간 분석 JSON·중간 PNG는 제출에서 제외했다.

최적화한 실제 fresh WASM 제품의 전체 763쪽을 기존 독립 Print PDF와 비교했다.
전체 쪽수가 일치하며, 정본 보호 TSV의 공식 두 자리 값보다 하락한 쪽은 0쪽이다.
최저 99.56%로 기존 보호 하한을 유지했다. Native 결과나 통합 PR 전체의 완료를 뜻하지 않는다.

- 코드: `30ee27003f`의 Canvas Print 그림 경계 보정. 이후 `54c164fd2b`는 검증 기록만 정정했다.
- WASM SHA-256: `14f10c0ca41cb0b6cf36d71cae565106df519501e72610d21696f7834b049dc3`.
- JS SHA-256: `bc741316480f6aa314c007421cf4c4980692ad239983b4e288a486858da0c280`.
- 입력: `samples/hwp3-sample10.hwp`, SHA-256 `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48`.
- 기존 기준: `pdf/issue8371/hwp3-sample10-2020.pdf`, SHA-256 `310e3251e910ac8c7c6f34d0e50d85c4a0ecbbadd65e69fe4f5071370931a918`.
- Print profile, compat 2022, 96 DPI, 색상 경계 보정 전 2px raw 실루엣이다.
- 일반 설치 글꼴 전체와 기존 수용 기록의 CFF 프로그램을 공급했다. 최소 글꼴 회귀 실행 환경과 구분한다.
- 연결 그림 `oracle.gif`·`rdb02.gif`·`s1.jpg`도 실제 바이트와 SHA를 확인해 명시 공급했다. 마스킹하지 않았다.
- 원본 PDF를 다시 출력하거나 판정 조건을 변경하지 않았다. 글꼴 파일을 포함하지 않는다.

[전쪽 TSV의 검증 요약](../verification-record.md) · [명령·입력·연결 그림·생산물·완료 기록](../verification-record.md).
12쪽의 합성 기울임 영역은 [review](../verification-record.md)와
[같은 쪽 overlay](../verification-record.md)로 확인한다. 해당 쪽의 공식 표시값은 100.00%다.
이는 2px 실루엣 값이며 엄격한 RGB 동일성을 뜻하지 않는다.

검증 당시 진행 중인 Native 전용 보정은 이 WASM package에 포함되지 않는다.
보호 기준의 상승 반영은 전체 보호 범위의 최종 검증과 하락·누락 0쪽 확인 후 별도로 수행한다.

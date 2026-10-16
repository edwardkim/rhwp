# Visual Sweep WASM의 실제 글꼴 환경 초기화

## 원인과 수정

Native CLI와 Studio가 사용하던 공통 글꼴 선택을 WASM PNG 실행기는 명시 프로필이 없을 때 호출하지 않았다. 따라서 설치 face를 나중에 Canvas에 등록해도 첫 `pageCount`와 줄 구성은 다른 메트릭 환경에서 계산됐다.

수정은 실제 FontTools 공급 바이트의 SHA-256·FontFace 로드 성공을 확인한 후보만 Rust 공통 API에 전달하고, 반환된 환경을 첫 쪽수 계산 전에 적용한다. 원문 face·텍스트는 편집하지 않는다. 내장 face는 설치 대체로 선언하지 않으며 기존 내장 자원 공급 경로를 유지한다. 명시 프로필이 우선하고 작업자 간 환경이 다르면 실패한다. 없는 API·공급 실패·hash 변경·로드 거절을 portable 환경의 성공으로 바꾸지 않는다. 마스크·DPI·2px 관용·90% 문턱은 변경하지 않았다.

## 실제 검증

- 입력: `samples/issue6803/1376496-neighborhood-facility-land-table.hwp`와 대응 독립 한컴 Print PDF, 전5쪽,192DPI,compat2022,캡처8.
- Rust/WASM 생산 정본: ignored output의 `stored-justify-tail-final-candidate-v25a/producer.json`과 `wasm-producer.json`. 소스 hash 고정 빌드이며 Rust 변경은 이 실행기 보정에 없다. Mac fresh WASM은 web/no-opt 로컬 대체 빌드다.
- head 자동 선택: Native 최저97.60018%,fresh WASM 최저97.31422%,각5/5쪽. 자동·명시 환경의 PNG SHA는 각 backend 전5쪽 일치했다.
- 고정 devel `463c48cc228170b576c832591b76826116e86a49`에는 자동 선택 API가 없다. 양쪽에 동일한 실제 공급 프로그램 환경을 명시 적용하여 비교했다. base 최저 Native94.75972%,WASM94.56858%;후보와의 Decimal 비교에서0.01%p 이상 하락0쪽. 이는 명시 환경 비교이며 base 자동 선택 통과를 주장하지 않는다.
- 원래 초기화 계약 검사: 수정 전3건 중 의도한 첫 조판 검사1FAIL·나머지2PASS. 수정 후 공급·내장·명시·과거 API·hash/로드 실패·서버 계약 Node12PASS,기존 Visual Sweep Python142PASS.
- `review.png`와 같은 좌표의 `review-detail.png`를 직접 판독했다. 본문 줄 구성·표 이어받기·좌측 병합 셀의 내용 소속과 외곽이 대응하며 글자 래스터 차이는 남아 있다. 픽셀 완전 일치를 주장하지 않는다.

전쪽 TSV·실제 공급 face/hash·생산자와 결과（로컬 중간 자료 `evidence.json`）에 연결했다. 이 결과는5쪽 실행기 보정 검증이며 원43조건·추가 대상·전체 회귀·실제 소비 경로36건의 최신 완료를 대신하지 않는다. 이전 `canonical-font-environment-candidate-v27`은 driver가 C25를 참조한 진단이며 출처를 정정했고 최종 결과로 사용하지 않았다.

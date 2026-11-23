# 소스32의 교차 검증 게이트

생산 소스는 `2f46647df16da00365b22a1364cdc0b28b8344bc`다. 상위 생산물 영수증의 최적화 Native·fresh WASM을 사용했다. 최신 devel 동기화 이후의 결과와 구분한다.

- Canvas 글꼴 browser26개, Studio 글꼴 별칭·Canvas 비교·실제 CanvasKit 준비 검사: 모두 exit0. `browser/validation.json`에 실제 명령·시간·JS/WASM SHA가 있다. CanvasKit 준비의 대표8개 입력·실제 backend 정보는 원본 JSON에 보존한다. 대표 검사를 전 문서 UI 검증으로 확대하지 않는다.
- 기여자 공통4개 문서의 Native/fresh WASM8조건: 모두100.00%. 두 경로의 review PNG를 직접 확인했다. 라틴 줄바꿈, angle 줄, 그림 crop과 색, 그룹 외곽과 그라데이션이 보존된다. `contributor/`에 raw TSV·manifest·review·overlay가 있다.
- HWPX 편람 Native383쪽: 최저99.97%, 공식 표시100.00%382쪽, 정본 하락0쪽. 원본 전쪽 TSV와 공식 gate를 `handbook-native/`에 보존한다. 23문서 전체 보호 검증의 통과를 대신하지 않는다.
- Native Skia공식3종과 suite·unit 정책은 각 하위 원본 로그를 따른다. 전체 nextest·최신 devel 보호 검증은 별도 종료 결과가 필요하다.

Studio의 추적 JS는 잠금 wrapper가 실제 생성한 바인딩으로 동기화했다. 같은 기준 devel의 재빌드 JS와도 SHA가 같아 새로운 Rust API 구현이 추가된 것은 아니다. WASM의 신규 바이트를 Git에 넣지 않았다.

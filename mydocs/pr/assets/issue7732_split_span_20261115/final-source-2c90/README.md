# #7732 후속 보정의 최종 제품 소스 검증

제품 소스 `2c90fa288bb4d48764eb82dbb025df4a12637ea9`, 고정 base `e7ee272cfa33a960b0d0d924983d29e6509903e0`의 기록이다. PR·원격 CI·병합 완료 선언이 아니다.

- [생산자](producer.json): 새 Native와 fresh WASM의 실제 SHA. WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
- [보호 문서 결과](protected-validation.json): Native/fresh WASM 각각 18종·1,771쪽, 총 36개 데이터셋·3,542행. 기존 보호 하한 대비 하락 0행, 상승 6행이다.
- [전쪽 base/head 비교](claim-whole-comparison.json): 각주 50쪽과 새 devel의 #9060 문서 14쪽. backend별 원 TSV와 대표 raw/PDF PNG·review·overlay는 `claims/`에 보관한다.
- [중첩 표 11쪽의 공통 paint 보정](../soft-wrap-decoration/README.md): 줄 끝 별도 공백 run의 밑줄만 제거하며 본문·render tree는 유지한다.
- [Print 브라우저 실행 조건 대조](../raster-environment/README.md): 통상 공급 글꼴을 사용한 소프트웨어 Print 재검증과 실제 GPU Studio 소비 검증을 구분한다.

보호 전쪽 결과는 한 고정 실행 조건의 각 문서 전쪽 TSV이며, 페이지별 실행 최댓값을 조합하지 않는다. 최종 수용된 상승분은 정본 보호 하한에 반영한다. 원본·PDF·글꼴 바이트와 기준 점수는 보정을 위해 변경하지 않았다. 비공개 글꼴 프로그램은 Git에 포함하지 않는다.

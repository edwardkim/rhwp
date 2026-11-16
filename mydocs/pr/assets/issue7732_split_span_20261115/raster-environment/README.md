# macOS의 Print 래스터 실행 조건 대조

제품 소스는 `2c90fa288bb4d48764eb82dbb025df4a12637ea9`다. 입력·독립 한컴 Print PDF·글꼴 공급·render tree·JS/WASM을 고정하고 Chrome의 가속 조건을 대조했다. 이 기록은 제품 코드 보정이나 판정 기준 변경을 뜻하지 않는다.

`jongbu_law.hwpx` WASM 1쪽은 Mac Chrome의 GPU/Graphite 경로에서 99.96967%(공식 99.97%), `--disable-gpu`에서 99.98059%(공식 99.98%)다. 미일치 20픽셀과 13픽셀뿐 아니라 이진화된 잉크 합집합도 각각 65,939와 66,972픽셀로 달랐다. PDF PNG는 동일했다. 정상 GPU 결과는 하한 99.98%를 만족하지 못한 기록으로 보존한다.

- [GPU 지표](mac-gpu-metrics.json) · [소프트웨어 진단 지표](mac-software-diagnostic-metrics.json)
- [고정 base의 소프트웨어 지표](fixed-base-software-metrics.json): 같은 조건에서 base와 후보의 해당 PNG는 같다.
- [Mac Chrome/CDP 관측](mac-browser-environments.json): GPU 실행과 소프트웨어 실행의 `gpu.featureStatus`를 구분한다.
- [참조 호스트의 현재 브라우저 관측](reference-browser-current.json): 브라우저 정보만 읽었으며 문서 검증은 Mac에서 수행했다. 이 현재 관측을 과거 TSV 생성 당시의 브라우저 신원으로 소급하지 않는다.
- [최종 전쪽 조건](environment.json) · [최종 전쪽 결과](../final-source-2c90/protected-validation.json)

진단 한 쪽을 통과 선언에 사용하지 않고 같은 소프트웨어 Chrome 조건으로 Native/fresh WASM 각각 18종·1,771쪽, 총 3,542행을 다시 검증했다. 기존 정본 대비 하락·누락은 0행이다. 여러 실행의 쪽별 최고값을 합치지 않았고, 각 데이터셋은 한 최종 실행의 전쪽 TSV다. 판정식·2px 관용·90%·공식 두 자리 보호 하한은 변경하지 않았다. 실제 GPU Studio Canvas2D/CanvasKit 소비 검증은 별도로 수행한다.

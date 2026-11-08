# 중첩 셀 공백 보정 후 Native 전쪽 검증

제품 code head `0322cfe65b99c24685e5a30f67ae49c7e7f11272`, 검증 base
`06181802e574dbd91e62375536751dea8c270fcd`의 실제 결과다.

- [Native 전쪽 TSV](native-whole93.tsv): 93조건, 2,994쪽, 최저92.24%, 90% 미만0쪽.
- [정본 보호값 대조](native-whole93-audit.json): 위 전쪽에 포함된1,770쪽과 별도 각도 대조군1쪽을
  합쳐 Native1,771쪽 모두 확인했다. 공식 두 자리 표시값 하락·누락은0쪽이다.
  93조건의 측정 범위를 별도1쪽으로 늘려 표기하지 않는다.
- 일반 실행과 별개로 `--handbook-gate` 옵션만 지정하여 Native383쪽을 실제 재출력했다.
  [gate 결과](required-handbook-native-gate.json)는 passed이며,
  [편람 전쪽 TSV](required-handbook-native.tsv)의 최저는130쪽99.96%다.
  이 실행은 일반 Mac 글꼴 환경이며 최소 글꼴 격리 결과로 확대하지 않는다.
- 130쪽 [review](handbook-native-p130-review.png)와 [overlay](handbook-native-p130-overlay.png)를
  직접 확인했다. [Native raw](handbook-native-p130-raw.png)와
  [독립 PDF raw](handbook-native-p130-pdf.png)도 함께 보존한다. 본문·그림·쪽 번호의 소속과 위치를 보존했고 알려진 WMF 테두리 차이는 남는다.

[실제 실행·입력·PDF·producer·TSV hash](producer.json)를 보존한다.
별도 실행 중인 fresh WASM 전쪽 검증이나 보호값 하락 해결, PR 전체 준비 완료를 의미하지 않는다.

## 코드·Studio 추가 검증

같은 제품 코드의 [Rust 필수 게이트](rust-validation.json)는 Native 기본 Clippy,
workspace build, workspace all-target Clippy, doctest, 고정 base 대비 suite manifest와
source unit tiers 검사를 순차로 모두 통과했다. WASM Clippy와 Native Skia 필수 3종은
[최소 글꼴 환경 증적](../macos-minimal-full-validation/README.md) 및
[중첩 셀 보정 검증](../nested-declared-word-space-budget/validation.json)에 연결한다.

[Studio 검증](studio-validation.json)은 TypeScript 검사 통과와 **2,784 PASS / 0 FAIL /
2 skip**다. Node 테스트가 제외된 실제 화면·CanvasKit 소비 경로를 통과한 것으로 확대하지 않는다.

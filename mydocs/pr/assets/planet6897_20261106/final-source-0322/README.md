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

## 실제 브라우저 소비 경로

[실제 Studio 초기화·backend 증적](backend-consumer-validation.json)과
[CanvasKit 화면](canvaskit-group-drawing-screen.png)을 보존한다. `group-drawing-02.hwp`는
실제 software CanvasKit으로 출력됐으며 숨겨진 Canvas2D 덮어쓰기·미등록 글꼴 fallback은 없었다.
regular/bold의 두 공급 순서, 실제 supplied serif, 늦은 Local Font Access 갱신 후의 원문 byte 보존을
확인했다. 문서 열기 안내에서는 테스트 전용 프로필에서 실제 대체 글꼴 선택 버튼을 눌렀다.
자원 공급 실패를 주입한 마지막 대조군은 의도한 Canvas2D fallback이며 CanvasKit 통과로 세지 않는다.

실제 `instantiateStreaming`에 전달한 WASM 응답 hash는 frozen pkg·루트 pkg·Studio/public과
일치했다. 초기 JS 응답은 Vite 변환 후 hash로 기록했다. 원 JS와는 WASM 자원 URL 재작성 한 곳과
마지막 빈 줄만 다름을 별도로 확인했으며, 원 JS hash와 변환 후 hash를 같은 값으로 표시하지 않는다.
실제 WASM·FontFace·Canvas 관계형 검사24건과 글꼴 규칙16건도 모두 통과했다.
이 증거는 모든 실물 문서의 Studio 전쪽 검증이나 독립 PDF 일치율을 의미하지 않는다.

## fresh WASM 전쪽 실행과 남은 보호값 하락

[WASM 전쪽 TSV](wasm-whole93.tsv)와 [보호값 대조](wasm-whole93-audit.json)는
같은 Rust/WASM 제품 코드의93조건·2,994쪽을 포함한다. 처음91조건·2,986쪽을 완료했고,
한겨레결체 이름 저장소 때문에 중단된2조건·8쪽만 [공급 보정 후 전쪽 재출력](../font-post-unused-storage/README.md)으로
보완했다. 영향받은 다른 실제 공급 대조군도 전3쪽 재출력해 PNG 동일성을 확인했다.
Native/fresh WASM 최저는 각각92.24% / 92.29%, 90% 미만은 두 경로 모두0쪽이다.

**정본 보호 TSV의 Native1,771쪽 하락은0쪽이나 WASM1,771쪽 중10쪽의 공식 두 자리 값은
하락한 상태다. 따라서 PR 준비 완료·제출·승인·통합으로 판정하지 않는다.**
WASM 하락은 sample10의133·251·509·658·659·660·708·757쪽과
sample16 HWP2024·HWPX의35쪽이다. 기준값·글꼴 선택·PDF·DPI·비교 문턱을 바꾸어 통과시키지 않았다.

이 문서의 제품 code head는 Rust/WASM0322이며, 이후 Python 글꼴 공급 보정은
`d1698e9f32`다. 그 출처와
프로그램·테이블 보존 근거는 위 공급 보정 기록에 따로 연결한다. 과거 전체 빌드를 새 Python
검사 실행으로 확대하지 않는다.

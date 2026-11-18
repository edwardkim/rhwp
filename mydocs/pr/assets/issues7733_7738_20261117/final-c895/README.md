# 최종 후보 c895 전쪽 시각 증적

제품 소스 기준: `c89572f72a8cdbd7c4f14262dae44eee9938fa2a`.

원본 5종의 Native 252쪽과 fresh WASM 252쪽을 각각 새로 산출했다. 정상 원본 글꼴 환경, PDFium 96DPI, Print, 2px raw 기준이며 504쪽 모두 98% 이상이다. 최소 글꼴 환경은 회귀·lint/build 검사에 별도로 사용한다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 |
|---|---:|---:|---:|
| chemical | 103 | 99.38114% | 99.43167% |
| spec13 | 71 | 99.68651% | 99.66582% |
| outline-demo | 3 | 100.00000% | 100.00000% |
| decree | 74 | 99.16669% | 99.12883% |
| outline-minimal | 1 | 100.00000% | 100.00000% |

Native는 parent `fbbfb55c`와 기록된 dirty 소스로 빌드한 실행이다. 실행 HEAD를 다시 붙이지 않고, `native-committed-source-equivalence.json`에서 최종 커밋의 동일 제품 소스 바이트와 Cargo.lock을 확인했다. fresh WASM은 최종 커밋에서 루트 `pkg`를 `--no-opt`로 빌드한 Mac 로컬 대체 결과이며 Docker 최적화 빌드가 아니다.

시행령 70쪽은 Native 100.00000%, fresh WASM 99.99795%다. WASM의 미대응 1픽셀은 그림 밖 중간 열의 문장부호 부근에 있어 WMF·그림 차이로 분류하지 않는다. 바탕 결측점 보정 뒤에도 화학표시 기준의 일부 본문에 미세 글리프·간격 차이가 남는다. 점수·PDF·문턱을 바꾸어 100%로 만들지 않았다.

`cases/`의 각 페이지에는 같은 producer의 raw·PDF·review·overlay PNG 네 가지를 함께 보관한다. 직접 판독 여부와 메모는 `direct-reading.json`에 개별 기록한다. 모든 자료를 판독 완료했다고 주장하지 않는다.

최소 글꼴 전체 회귀는 13,757 PASS·0 FAIL·45 skip(692.176초)이며 fmt·manifest·Native/WASM/workspace Clippy·workspace build와 Native Skia 공식 3종(4,072 PASS)도 통과했다. 세부 원문은 `rust-gates/`에 보관한다.

편람 HWPX Native 383쪽 필수 gate는 보호 표시값 하락 0으로 통과했다. raw 미대응은 43쪽 1픽셀, 130쪽 23픽셀, 141쪽 1픽셀이며 공식 표시값은 130쪽 99.97%, 나머지 382쪽 100%다. `required-handbook/`에 전쪽 TSV와 gate를 보관한다.

정본 보호 18종 전체와 실제 Studio 소비 경로의 최종 수용은 아직 미완료다. 부분 결과를 PR 준비 완료로 대체하지 않는다.

기준 PDF는 변경·재산출하지 않았다. 특히 공식 개정 1.3 사양 PDF는 원문 그대로 사용한다.

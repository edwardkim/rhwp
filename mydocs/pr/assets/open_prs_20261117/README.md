# 열린 PR 3건 — 데스크톱 Print 기준 통합 검증

원 PR [#9144](https://github.com/edwardkim/rhwp/pull/9144)·[#9149](https://github.com/edwardkim/rhwp/pull/9149)·[#9152](https://github.com/edwardkim/rhwp/pull/9152)의 고정 head를
`upstream/devel` `1e2488ebd432deb7c326be532f8a91945f260231` 위에 체리픽했다.
최종 제품 source는 `2c5e3fe38726d23bec1c970ba3f92a49e15cae34`, 회귀 source는
`c2ec0621772bf802bc573dab7bdb1bf82a2e21e3`다. 이후 커밋은 증적·문서만 포함한다.
원 PR별 주장과 판정은 [9144 리뷰](../../archives/pr_9144_review.md),
[9149 리뷰](../../archives/pr_9149_review.md), [9152 리뷰](../../archives/pr_9152_review.md)에 기록했다.

## 독립 PDF와 시각 결과

**웹한글기안기 PDF는 공식 비교·기대값·보호 하한에 사용하지 않았다.** 이번 원 PR에 포함된
해당 PDF 2종 모두 같은 원문을 MCP로 재산출했다. 실제 제품은 데스크톱 한컴 2024
`13.0.0.3901`, 출력은 **Hancom PDF Print**다. Microsoft Print to PDF 또는 PDF 저장/내보내기가
아니다. [입력·PDF SHA-256·MCP 영수증](reference-pdfs.json)을 보존했다.

96DPI·compat2024·print·canonical PDFium·색상 경계 보정 전 2px raw로 각 원문 전체 1쪽을 대조했다.

| 원문 | base Native | base fresh WASM | 최종 Native | 최종 fresh WASM | 실제 Studio Canvas2D / CanvasKit compat / default | Native SVG / WASM SVG |
| --- | --- | --- | --- | --- | --- | --- |
| `samples/issue8582/hwp_band.hwpx` | 36.37% | 36.31% | 100.00% | 100.00% | 100.00% / 100.00% / 100.00% | 100.00% / 100.00% |
| `samples/tab_overflow/tab_stop_beyond_line.hwpx` | 55.75% | 55.42% | 100.00% | 100.00% | 100.00% / 100.00% / 100.00% | 100.00% / 100.00% |

[Native 원본 TSV·PNG](claim-evidence-native.json), [fresh WASM 원본 TSV·PNG](claim-evidence-wasm.json),
[실제 소비 경로 10건과 판독](consumer-validation.json)을 따른다. 쪽수 불일치·내용 누락·중복은 없다.
직선 띠의 앵커/후속 문단과 탭 앞뒤 글의 줄 소속을 같은 쪽의 review·overlay로 확인했다.
작은 안티앨리어싱·획 두께 차이는 남으므로 2px raw 100%를 RGB 바이트 동일성으로 표현하지 않는다.

실제 Studio는 파일 입력과 정상 `initializeDocument`를 사용했다. 원본 설치 TTF를 공식
`rhwpStudio.fonts.setProvider` API로 공급하고 선택적 브라우저 감지 안내는 실제 닫기 버튼으로 닫았다.
headless의 브라우저 로컬 글꼴 권한은 denied였으므로 로컬 감지 통과로 주장하지 않는다.
CanvasKit은 명시적 software 경로로 실행했으며 실제 backend·원본 typeface 1개·runtime readiness,
대체/미등록 글꼴·숨은 Canvas2D overlay·render error 0을 확인했다. guard를 수정하거나 우회하지 않았다.
[초기화 JS/글꼴 공급](consumers/initialization-and-local-fonts.json),
[초기화 WASM 실제 응답 6건](consumers/initialization-wasm),
[빌드 산출물 해시](consumers/actual-build-manifest.json)를 구분해 기록했다.

## 보정과 회귀 검증

- #9144는 시각 변경이 아닌 진단 nullable 계약이다. [실제 표 삽입·저장·재열기와 저장 줄 대조](diagnostic9144-actual-cli.json)를 확인했다. 관련 없는 시각 그림을 이 PR의 개선 증거로 사용하지 않는다.
- #9149의 원 기여 HWP5 변환 검사는 통과했지만 원본 HWPX는 58.23%였다. [합성 사다리의 흐름 되감기 원인](band-cause.md)을 공통 provenance 규칙으로 보정했다. 문서명·쪽별 예외는 없다.
- 새 HWPX 좌표 회귀는 [보정 전 실제 FAIL → 보정 후 실제 PASS](band-red-green-validation.json)로 결함 검출을 확인했다. 원본 HWPX의 100%를 HWP5 변환본의 별도 픽셀 점수로 재사용하지 않는다.
- #9152는 원 기여가 줄 소속을 복원했다. 웹한글기안기 피치 대신 독립 Print 좌표와 정상 문단 피치·워드 호환 대조를 함께 검사한다.
- [최종 집중 7건](focused-regression-validation.json), [최소 글꼴 환경 전체 회귀·lint/build·Native Skia](rust-validation.json)를 완료했다. 전체 nextest는 **13,708 PASS / 0 FAIL / 45 skip**, Native Skia는 **본체 lib 3,852 PASS·12 ignored, workspace 멤버 182 PASS / 별도 2·36 PASS**다.

공유 target에서 보정 전 source archive를 빌드한 직후의 첫 after 호출은 보정 전 좌표를 재사용했다.
이를 수용 결과에서 제외하고 파일 내용은 그대로 둔 채 최신 제품 source를 확실히 재컴파일해 2건 PASS를
확인했다. 이전 13,708건 전체 PASS는 이 archive 검사 전에 완료한 최신 제품·테스트 결과다.

## 보호 문서와 실행 환경

- [정본 Native](protected-native-validation.json): 18종·1,771쪽, 공식 두 자리 표시값 하락·상승 0쪽.
- [정본 fresh WASM](protected-wasm-validation.json): 18종·1,771쪽, 하락·상승 0쪽. 기준을 낮추거나 상향할 변경은 없다.
- [명시적 `--handbook-gate`](handbook-validation.json): Native 383쪽, 최저 99.97%, 하락 없음.
- [기여자 공통 보호 검사](contributor-validation.json): 공개 입력 4종 × Native/fresh WASM 8건 PASS, TSV·대표 review·overlay 보존.
- 보호 Visual Sweep은 검증된 원본 글꼴을 공급한 일반 환경이며 Rust 회귀는 개인/추가 시스템 글꼴을 차단한 macOS 최소 글꼴 환경이다. 신규 비교용 HFT ZIP은 설치·혼합하지 않았다.
- 보호 WMF가 요청하는 원본 굴림체의 WASM 공급 누락은 [동일 조건 base/후보 전쪽 대조와 원본 TTC 공급](protected-resource-repair/cause.md)으로 바로잡았다. 점수/PDF/제품 구현을 바꾸지 않았다.
- [fresh WASM 빌드](wasm-build-validation.json)는 Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 빌드 통과로 주장하지 않는다.

[입력·제출 범위 감사](submission-audit.json)와 [렌더 트리 반복 결정성](snapshot-determinism.json)을 확인했다.

[통합 PR #9156](https://github.com/edwardkim/rhwp/pull/9156)의 code head `69f0deaa468f0914c29afa0e9697424a75fb10ae`에서
[Full CI](https://github.com/edwardkim/rhwp/actions/runs/44079335323)·CodeQL·Render Diff·Adapter·Proptest·최종 정책이 성공했다.
[실제 head·job 기록](code-ci.json)과 [제출 raw URL 71개 바이트 검증](submission-raw-link-verification.json)을 보존한다.
Native Skia 본체 lib와 workspace 멤버의 집계 표기만 원 로그에 따라 정정했다.
CI 변경이 없어 같은 PR에 오늘할일·검토 보완의 문서 전용 trailing을 잇고, 최신 검사 뒤 병합·원 PR/이슈 후속 처리를 진행한다.
아직 실행 전인 후속 처리를 완료로 적지 않는다.

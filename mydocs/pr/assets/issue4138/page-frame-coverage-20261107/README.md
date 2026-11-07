# 용지 테두리의 소수 원점과 인쇄 coverage 보정

2026-11-07 KST. 부모 `0510f302953de8acf3f777d55bf892979a67a1fb`, 고정 base
`a7feb849e11642b1438b7e9ee460ff4c8a7c946e` 위의 단계 검증이다.

## 독립 근거와 보정

`pdf/issue4138/interword-space-tight-2024.pdf`의 왼쪽 실선은 x=14.152000427pt,
선폭 약 0.24pt다. PDFium 192DPI의 본문 없는 y=1250에서 x=37/38의 회색 값은
60/194다. 기존 Native는 얇은 선을 화소 중심으로 옮겨 0/255로 만들었다. 글자 영역은
보정 전후 동일하며, 글꼴 실험으로 이를 해결할 수 없었다.

조판기의 용지 테두리가 소수 원점 소유를 발행하고, Rust Native 및 CanvasKit의 공통 선
계획에서 최소 한 장치 화소의 폭과 원래 좌표를 함께 소비한다. 일반 표·도형의 기존 정렬,
점선·복합선·화살표의 기존 경로는 유지한다. 새 속성은 paint JSON과 Studio 타입에도 전달한다.

## 결과

| 독립 1쪽 문서 | Native 전 | Native 후 | fresh WASM 전/후 |
| --- | ---: | ---: | ---: |
| interword-space-wide.hwpx | 98.81412% | 100% | 100% / 100% |
| interword-space-edge.hwpx | 98.36366% | 100% | 100% / 100% |
| interword-space-tight.hwpx | 97.17452% | 100% | 100% / 100% |

대조군 전 출력은 `79ee0477b6365d936a97df320e384451b75dc081` 단계의 실제 산출물이다.
부모 `0510f302`의 추가 변경은 구형 수식의 아래첨자 배치이며 이 대조군에는 수식이 없다.
세 문서 모두 안쪽 화소 영역은 전후 byte 단위로 동일하다. 100%는 canonical 2px 내용
실루엣 raw 값이며 이미지 전체의 byte 일치를 뜻하지 않는다. 대표 overlay의 아래쪽 선에는
약 1px의 차이가 보이나 2px 비교 범위 안이며, 이 차이를 숨기거나 문턱을 바꾸지 않았다.

`saved-split-1x2.hwp` 전197쪽을 양 backend에서 다시 산출했다. 부모와 다른 PNG는 각각 0쪽,
쪽수 197/197·누락 0이다. Native 최저 94.76447%(44·129·130쪽 미달), fresh WASM 최저
94.83414%(129·130쪽 미달)이며 수식 가로 배치 문제는 계속 미완료다.

#8696 원본 3조건 및 독립 글상자 대조군 5조건은 양 backend 16건 모두 100%를 유지했다.
편람의 38·59·130·272쪽 Native 집중 비교도 전후 100 / 100 / 99.96463 / 100%로 동일하다.
편람 결과는 현재 Mac 로컬 글꼴 환경의 집중 검증이며 최소 글꼴 및 전383쪽 gate 완료가 아니다.

## 회귀검사와 범위 게이트

- `native_fractional_glyph_origin`의 새 고정형 검사: 독립 PDF의 화소 위치 ±2px와 채널 차이
  32 이하를 검사한다. 실제 부모 1 FAIL → 보정 후 1 PASS이며 기존 Native 4검사까지 5 PASS다.
  기존 CI의 동일 모듈 호출로 수집하므로 workflow 개별 호출은 추가하지 않았다.
- 기존 집중 검사 117 PASS, CanvasKit 선 계획 5 PASS. 이 수에 새 Native 5건을 중복 합산하지 않는다.
- fmt, workspace all-target Clippy, native-skia lib Clippy, 파생 suite manifest,
  unit tiers(4099검사), Studio build 통과.
- 초기 manifest check는 fmt가 생성 harness를 바꿔 실패했다. fmt 후 `--prepare`를 다시 실행하고
  최종 manifest check를 통과했다. 실패 기록은 ignored output에 유지했다.
- Native 및 fresh WASM을 새로 빌드했다. WASM은 Mac locked wrapper의
  `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

이 문서는 이 단계의 증적이며 최종 전체 회귀·모든 대상 98%·실제 소비 경로 완료 보고가 아니다.
전체 상태·producer hash·독립 입력/PDF hash는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/page-frame-coverage-20261107/validation.json`), 단계 게이트는
gates.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/page-frame-coverage-20261107/gates.json`), 전후 전쪽 수치는 `before/after-*-197.tsv`에 보존했다.

## 대표 그림

- Native tight: [보정 전 review](before-native-tight-review.png) · [overlay](before-native-tight-overlay.png),
  [보정 후 review](after-native-tight-review.png) · [overlay](after-native-tight-overlay.png).
- Native edge: [보정 후 review](after-native-edge-review.png) · [overlay](after-native-edge-overlay.png).
- Native wide: [보정 후 review](after-native-wide-review.png) · [overlay](after-native-wide-overlay.png).
- fresh WASM tight: [보정 후 review](after-wasm-tight-review.png) · [overlay](after-wasm-tight-overlay.png).

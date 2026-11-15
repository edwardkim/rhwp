# SVG Print의 옅은 표 괘선 래스터 차이

Issue: #7732, #7445. 2026-11-15 SVG 보정·실제 소비 확인 기록이다.

## 입력과 독립 기준

- 원본: `mydocs/pr/assets/issue7445/issue1937_rowbreak_footnote_overpagination.hwp`, 전 50쪽.
- 기준: `pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf`.
  한컴 2020의 Print 출력이며 웹한글기안기 PDF가 아니다.
- 96 DPI, canonical `visual_sweep.py`, 색상 경계 보정 전 2px raw 기준을 유지한다.
- 기존 Studio Print SVG 36쪽은 88.253%다. 앞선 Native/fresh WASM PNG의 동일 쪽은 각 100.00%다.
  PNG의 높은 점수로 SVG 미달을 상쇄하지 않는다.

## 원인 분리

처음에는 SVG의 TextRun fallback과 PDF의 HFT 윤곽 차이를 우선 의심했다.
그러나 픽셀을 분리하면 raw 불일치 14,873개 중 14,656개(98.54098%)가
78개의 가는 직교 괘선 주변 2px 구간에 있다. SVG의 0.48px `#bbbbbb` 및
0.64px `#999999` 선은 브라우저의 서브픽셀 coverage로 옅어져 이진 잉크 경계를 넘나든다.

같은 입력의 실제 Canvas2D Print는 99.75605%, 실제 CanvasKit Print는 99.83217%다.
두 raster backend는 `renderer/raster_stroke.rs`의 최소 장치 폭·직교선 coverage 계약을 사용한다.
SVG는 원본 선폭·좌표를 기록하지만 해당 raster 표현 힌트가 없었다.

원본 SVG의 글자·좌표·선폭·색을 그대로 두고 선의 `shape-rendering`만 바꾼
독립 진단 대조군에서 36쪽은 99.7214%가 됐다. 이는 원인 분리 결과이며
보정 제품의 최종 측정으로 사용하지 않는다. 글리프 윤곽만 연결한 실험은 원인을 해결하지 못했으므로
제출 변경에서 제거했다. 해당 실험 코드·로그·미달 결과는 ignored output에 보존한다.

## 보정 범위와 검증

SVG Print/HighQuality의 가는 직교 실선이 공통 `raster_line_geometry`의 최소 장치 폭
처리에 해당할 때 브라우저의 `crispEdges` raster 힌트를 전달한다. 원본 SVG의 끝점·선폭·색과
논리 문서·글꼴 선택·PDF·점수 계산·90% 기준은 바꾸지 않는다.
분수 위치 보존선, 점선·복합 획·화살표·회전/기울임 및 화면 프로필은 기존 계약을 보존한다.

`tests/cases/issue_7732_svg_print_stroke_coverage.rs`는 글꼴 없는 관계형 검사다.
실물 문서의 낮은 기존 점수를 절대 px 핀으로 고정하지 않는다. 최소 글꼴 환경에서
보정 전 실제 1 PASS / 2 FAIL(0.013초), 보정 후 실제 3 PASS / 0 FAIL(0.018초)을 확인했다.
기울어진 그룹에서 정상 형제 선으로 복귀하는 경계와 분수 위치·점선·복합 획·화살표·화면
대조군을 포함한다.

## 제품 재출력

제품 소스 `d26f3bedf4fc3398164ed6a4640275c66ab9ccd9`, 고정 base
`1805d8f08cb3a54bccda8dfa6647432b7799f074`에서 같은 50쪽 원본과 독립 PDF를 비교했다.
Native SVG 전 50쪽 최저는 94.74%, fresh WASM SVG 전 50쪽 최저는 99.16%로
미달·누락 쪽은 없다. 이 수치는 기존 정본 보호 검사인 Native/WASM **PNG** 값과 구분한다.
Native SVG 7쪽의 94.74%는 base와 SVG 바이트가 동일하며, 새 하락이 아니다.

| 경로 | 36쪽 | 37쪽 | 38쪽 |
| --- | ---: | ---: | ---: |
| Native SVG | 99.72% | 99.08% | 99.53% |
| fresh WASM SVG | 99.99% | 99.38% | 99.83% |
| 실제 Studio Print SVG | 99.82661% | 99.15609% | 99.65759% |
| 실제 Studio Canvas2D Print | 99.75605% | 99.07736% | 99.57140% |

실제 Studio 36쪽 SVG는 이전 88.253%에서 99.82661%로 개선됐다.
같은 페이지의 review와 overlay를 함께 열어 괘선·셀 내용·문단 소속을 확인했다.
Canvas2D 세 쪽의 PNG는 보정 전과 바이트가 같아 기존 raster 경로를 바꾸지 않았음을 확인했다.
원본 괘선의 좌표·선폭·색은 유지하며, 옅은 선의 픽셀 coverage만 달라진다.

리베이스 전후 Rust 소스는 같아 기존 동일 경로 자료를 재사용한다.
새 devel의 #9093 글꼴 별칭 E2E와 Studio 단위 검사는 실제 실행해 통과했다.
root pkg·Studio public·실제 브라우저 초기화 응답의 WASM SHA가 같음을 확인했다.
HTTP JavaScript는 Vite의 모듈 변환 응답이므로 디스크 원본과 별도 해시로 기록한다.
Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt`의 로컬 대체 빌드다.

이 절은 새 SVG 경로의 실제 측정이며 전체 Rust 게이트·PR·CI 완료 선언은 아니다.

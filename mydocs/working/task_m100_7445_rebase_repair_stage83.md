# Stage83: 정수 종이 격자의 공통 그림 소비와 회귀 보호

Issue: #7445. 부모 HEAD `9bf0391b8c7c1b5fb985e6d6e2a01c3df40cba69`, 고정 base
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867` 위의 로컬 구현 체크포인트다.
기존 C352/C354/C356만 사용했으며 HWP 보정 대상·기준 PDF·90%·2px 문턱을 늘리거나 바꾸지 않았다.

## 공통 원인과 소비 경계

독립 한컴 Print는 물리 종이 전체를 정수 raster viewport로 투영한다. 이전 구현은 그림의
논리 bbox를 명목 배율로만 축소해, 정수 종이 폭·높이와의 소수 차이가 옅은 선의 표본을 바꿨다.
`PrintPaperGrid`가 원본 bbox를 보존하고 실제 종이·root CTM·정수 출력 크기로 장치 구간만 정한다.
Native PNG, WebCanvas, CanvasKit은 같은 Rust `prepare_bitmap`을 소비한다. Screen·HighQuality,
회전/비축 정렬·소수 쪽 원점·Native PDF/vector target은 기존 경로를 유지한다. invalid 값은
기존 paint로 돌아간다. 공통 image cache는 실제 장치 구간을 포함하고, Native picture cache도
종이 격자를 구분하며 cache 비활성 계약을 보존한다.

CanvasKit의 Print static picture는 실제 device CTM에서 기록하고 역행렬로 한 번만 상쇄해 재생한다.
기록 bounds도 같은 CTM으로 투영한다. 초기 후보의 논리 bounds 재사용은 확대에서 잘릴 수 있어
실제 소비 감사 중 보완했다. cache key는 실제 CTM을 포함한다. Screen 기록은 유지한다.

## Native/fresh WASM 집중 시각 검증

Mac locked wrapper `--target web --out-dir pkg --no-opt` 빌드 exit 0, 312초이며 Docker 최적화
결과가 아니다. Native release-test/native-skia 빌드는 exit 0, 128초다. 실제 로컬 PDFium
152.0.7947.0 / pypdfium2 5.12.1 / `pdfium_smooth_rgb_v1`, 96DPI·raw 2px 조건으로 비교했다.
과거 기록의 PDFium153이라는 설명을 실제 신원으로 대신하지 않았다.

| 기존 입력/쪽 | Native | fresh WASM |
| --- | ---: | ---: |
| C352 1쪽 | 100.00000% | 100.00000% |
| C354 1쪽 | 100.00000% | 100.00000% |
| C356 1쪽 | 100.00000% | 100.00000% |
| 로드맵 153쪽 | 100.00000% | 100.00000% |
| 로드맵 295쪽 | 98.96693% | 98.96600% |
| sample11 2쪽 | 99.83678% | 99.83678% |
| sample11 50쪽 | 93.13188% | 93.13713% |

실제 쪽수는 세 대조군 각각 1/1, 로드맵302/302, sample11 151/151이다. 뒤의 두 문서는 선택한
네 쪽만 비교했으므로 전쪽 완료로 보고하지 않는다. C354/C356의 세 격자·색 strip·옅은 선을
review와 overlay에서 직접 판독했다. Native/fresh WASM의 두 대조군은 엄격 픽셀 지표도 100%다.

## 실제 Studio 소비와 잔여

기존 세 대조군의 실제 문서 열기·합성 화면을 Canvas2D, CanvasKit compat/default 각 3건,
총 9건으로 캡처했다. fallback 없는 실제 backend·software surface·자원 준비·본문 canvas 소유를
확인했고 9건 모두 raw 100%다. root pkg·Studio/public·실제 초기화 JS/WASM 응답의 hash를 대조했다.
Studio 캡처는 CSS 종이 경계 반올림으로 높이 1123px, PDF 1122px이며 흰 여백을 포함해 원점 그대로
비교했다. CanvasKit faint의 한 선에는 1px 경계 차이가 남아 엄격 지표 90%이며 raw 100%와 구분한다.

확대 2배에서 같은 faint를 CanvasKit 두 모드로 추가 확인했다. raw 90.39248%이며 세 띠와 마지막
줄이 보존된다. CSS 종이 경계 및 계조 경계의 잔여가 있고 완전 일치나 98% 완료 페이지로 주장하지
않는다. 확대 비교는 192DPI의 같은 독립 PDF를 사용했다. source/PDF·비교 영역은 변경하지 않았다.

## 자동 회귀와 실행 경로

기존 `renderer::skia::renderer::tests::raster_options_scale_output_size` 안에서 실제
`DocumentCore`의 Print PNG 진입점을 호출해 세 대조군의 종이 크기·양방향 내용 보존을 검사한다.
독립 Print PNG의 최소 채널 232 미만·2px 이웃·raw 98% 문턱을 사용하며 색상 관용으로 미달을
상쇄하지 않는다. 빈 출력·그림 누락도 실패한다. 새 test attribute나 CI 개별 호출을 추가하지 않고
기존 Native Skia `skia --lib` 경로에 들어간다. 테스트용 fixture와 독립 PNG·SHA 출처만 승격했다.
기존 standalone 회귀 제거·15개 assertion의 관계 검사 통합은 Stage80 커밋을 그대로 보존한다.

실제 이전 동결 PNG에 같은 독립 내용 판정을 적용하면 C354 94.57727%, C356 42.89522%로 98%를
충족하지 못하고, 새 PNG는 모두 100%다. 이전 Rust unit binary를 다시 실행한 결과라고 쓰지 않는다.
확장한 실제 Rust unit은 1 PASS / 0 FAIL, 3870 filter 제외, 0.14초(컴파일 포함190.657초),exit0다.
마지막 rustfmt는 이 cfg(test) assertion의 줄바꿈만 바꿨다. 단위 테스트 tier와 suite manifest도 통과했다.

최종 CanvasKit bounds 보정 source의 TypeScript exit 0, Studio 전체 2411건 중 2409 PASS / 0 FAIL / 2 skip
(12.008초)를 확인했다. 전체 Rust 회귀와 기존 50조건 전쪽 검증은 이 체크포인트 뒤에 수행한다.

## source와 증적

새 Native SHA `ffd17865f30dea575a4996f9985ace1fceb1742238b7d5c98d99bb81fee96a49`,
JS SHA `56b2836e71bbbde64097476cb009303b8eeff61777c480cae82a238cf4d47af3`,
WASM SHA `e015798e39c7ce49c6999dc6ded0f1cc4e7b772aabb451bb545bfbb9c18546d3`이다.
빌드 뒤 Rust 변경은 기존 cfg(test) 내부뿐이며 별도 임시 index로 빌드 당시 source를 복원해
6개 제품 파일의 바이트 또는 production prefix 동일성을 입증했다. 실제 UI bounds 변경은 이후
Studio 전체 검사와 실제 화면 캡처에서 검증했다. 준비 전 최종 코드의 전체 검증은 아직 남아 있다.

안정 TSV·대표 review/overlay는 assets의 `print-paper-grid/`에 보존한다. 빌드·실행·source 생산자,
초기화 응답·글꼴·backend 관측, 전후 판정·원본 PNG와 로그는 ignored output의
`print-paper-grid/`, `print-paper-grid-final/`, `print-paper-grid-actual/`에 보존했다.
분석 script의 출력 디렉터리 누락·API 호출 형식 오류와 system Python의 PDFium 부재는
경로 생성·정본 API·기존 venv로 바로잡고 최종 결과를 다시 산출했다. 실패한 분석은 수용 증거가 아니다.

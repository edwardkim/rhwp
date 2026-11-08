# 굴림 결측 점의 측정·paint 공급자 정합(C53)

기준 `upstream/devel`: `f5251e7de4e31ee0fb939f352c6ef144bb340bdb`.
parent: `93985513f007364f20fce729d919afc8fb21a7fc` + 이 commit의 두 Rust 변경.
원격 head 재조회 결과는 기준과 같아 추가 리베이스하지 않았다.

## 근거와 적용 경로

원본 `samples/issue6034/2912735_court_report_form.hwp` SHA-256
`9e68695b504656383f7686b75b3e92d28056201ad967570c28144e4bfce880ac`,
독립 한컴2020 Print `pdf/2912735_court_report_form-2020.pdf` SHA-256
`039f83dda89a7ac1fe3a162b39fdee0afbc43f4cc98af0c48a1223eed38d27a8`를 재사용했다.
두 번째 빈 쪽도 원본의 일부라는 사용자 확인을 유지한다.

Gulim의 cmap에는 U+2024가 없고 독립 Print는 Haansoft Dotum을 사용한다.
기존 `verified_missing_glyph_supplier`의 측정도 이미 HDOTUM 178/1024em을 썼지만
기본 paint 체인에 이 공급자가 없어 실제 Native는 HCR Dotum을 골랐다.
기존 공통 `verified_program_glyph_fallback_families`에 Gulim 공급자만 연결했다.
원본·substFont·명시 환경·embedded 우선순위는 기존 resolver가 유지한다.
문서/페이지/점수 예외나 좌표 보정은 없다. 이 체인은 공통 TextStyle에서 SVG·Canvas·Skia로 전달된다.

실제 Native 렌더러의 공급 글꼴 경로로 수정 전후610개 문자 기록을 비교했다.
source와 layoutMetric은 동일하며, resolved face가 바뀐14개는 모두 U+2024로
HCR Dotum→Haansoft Dotum이다. 다른 문자의 실제 선택은 같다.
54개 입력 조건의 source 슬롯과 Gulim/HDOTUM cmap 차집합 조사에서는 법원 서식의
U+2024만 후보였다. 이는 neutral 슬롯 상속·embedded/명시 환경을 완전히 대체하는
전수 backend 입증이 아니며 현재 Studio Canvas2D/CanvasKit 실제 소비는 미검증이다.

## 실행과 결과

- Native build: `cargo build --locked --profile release-test --features native-skia --bin rhwp`,
  jobs8, exit0,98.594초. binary SHA-256 `b58655f702c93c5a4889e304f5b3e91e3d74b8bb7f2ce4397ed8f09435dd3c8d`.
- fresh WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`,
  jobs8, exit0,166.477초. Docker 최적화 결과가 아닌 Mac 로컬 대체 빌드다.
  `rhwp.js` SHA-256 `009b53a277844df11b6d9f00a0cab0b29e449d3e46f00a24dca42a6fed9696ed`,
  `rhwp_bg.wasm` SHA-256 `358bb713368e1c0cab07a75450f32b74b568db4f69cb4973e631383b73a67738`.
  root pkg와 Studio/public 파일 hash가 같다. 실제 Studio 초기화 응답 검증은 아직 남았다.
- 정식 `issue_7445_original_font_program_ownership`: 신규 공급자 계약의 frozen C52
  수정 전1FAIL→Cargo 수정 후5PASS(기존4건 포함),0.029초,218skip.
  명시 환경의 HCR Dotum 우선순위와 HWP 직렬화 불변도 검사한다.
- 정상 `issue_6389_font_environment`:5PASS,0.678초,207skip.
- fmt/manifest(f525 기준)/unit-tier/all-target Clippy: 모두 exit0.
  Clippy82.701초. 전체 nextest와 나머지 제출 게이트는 이 후보에서 미완료다.

정본 Visual Sweep,192DPI,원본에 맞는 compat2022,2px 이웃 관용 실루엣,
Native Skia PNG 및 fresh WASM Canvas PNG,각 전2쪽,쪽수2/2. 캡처 작업자8이다.
각주 source와 기존 빈 쪽을 바꾸지 않았다.

| 쪽 | Native | fresh WASM |
| --- | ---: | ---: |
| 1 | 90.14068% | 90.15243% |
| 2(빈 쪽) | 100.00000% | 100.00000% |

추가 SVG 소비 대조도 같은 source와 font를 사용했다. Native/fresh WASM SVG를
전체 글꼴 embed 후 Chrome raster로 비교하면 양쪽1쪽90.14943%,2쪽100%,쪽수2/2다.
이는 위 Native Skia/WASM Canvas 결과와 별개이며 실제 Studio CanvasKit 검증은 아니다.
최초 SVG 진단은 font-path에 필요한 embed 옵션 누락으로 즉시 실패해 별도 invalid 경로에
보존했고, 수정한 명령의 전쪽 결과만 인용했다.

전쪽 TSV·manifest·source snapshot·명령/exit·trace는 ignored 로컬
`rebase-f5251e7d-20261016/candidate-c53-gulim-supplier/` 및
`candidate-c53-court-whole/`에 보존한다. 중간 JSON/로그/TSV/폰트는 이 증적에 복사하지 않았다.

## 직접 판독과 잔여

Native/fresh WASM의 [Native review](native-review.png), [WASM review](wasm-review.png)를
독립 PDF와 전쪽 및 각주 확대에서 직접 판독했다. 작은 점의 실제 공급자는 복원됐으나
각주1·3의 계속 줄 선두 공백과 각주4의 내부 공백/줄 소속, 본문 일부 미세 간격 차이는 남는다.
90% 통과를 문서 시각 완료나 PR 준비 완료로 해석하지 않는다.
이 페이지는98% 미만이므로 절대 px 핀을 추가하지 않았다. 신규 검사는 합성 프로그램의
intrinsic cmap/폭·소유 계약이며 법원 서식의 현재 배치를 고정하지 않는다.

C52부터 남은 fixed devel 대비 WASM 하락, 다른 입력의 전쪽 재검증·미달/쪽수 문제,
전체 회귀·actual36은 미완료다. 이 보정만으로 전체 브랜치 수용을 주장하지 않는다.

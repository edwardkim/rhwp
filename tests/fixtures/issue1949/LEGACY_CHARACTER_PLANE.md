# 구형 CHAR 수식의 독립 문자 전진 대조

## 입력과 기준 출력

- `legacy-character-plane.hwpx`: 공개 `issue1949_giant_cell_nested_tables_perf.hwpx`의
  깨끗한 용지·스타일을 사용해 새 문단 186개를 구성한 독립 합성 대조다. 원본 내용은
  대체하지 않는다. 영문 대·소문자와 숫자 62개를 각각 1·2·3회 표시한다. 반복 사이의
  ASCII 공백은 명령 이름 결합을 피하며 명시적 `~`·역따옴표 공간과 다르다.
- 각 수식은 `version=""`, `font="HYhwpEQ"`, `lineMode="CHAR"`, `baseUnit="1000"`,
  `baseLine="80"`, 절대 너비 9,000 HU·높이 2,200 HU, TAC다. 저장 LineSeg를 만들지 않았다.
- [기준 PDF](../../../pdf/issue1949-legacy-character-plane/legacy-character-plane-2020.pdf)는
  같은 바이트를 Windows 한컴오피스 2020 **11.0.0.9136**에서 Print한 전체 9쪽이다.
  MCP job은 `8d0992d1-ea91-4413-82ba-1f77d8e676ea`, `pdf_print_method=0`,
  `pdf_output_mode=hancom2020_pdf_driver_one_up`, `input_preprocess=none`이다.
- `legacy-character-plane.print.json`은 입력/PDF SHA-256과 Print 계약, 원본 문단별
  독립 PDF 쪽 소속·문자·첫 글리프 대비 X 원점을 담은 정식 기대 자료다. 좌표는 PDF의
  pt 원점 차이에 `96/72`를 곱했다. 구현의 전진 표·배치값에서 정답을 만들지 않았다.

## 신규 검사와 선행 증거

[Native/fresh WASM 전체 TSV와 대표 review·overlay](../../../mydocs/pr/assets/issue1949_legacy_character_plane/README.md)를
먼저 확인했다. 제품 source `f3143bf35e71b1d419966d49c7aa54dce27d2a60`의 Native188
전 9쪽 최저는 **99.98690%**, fresh WASM051은 전쪽 **100%**다. 전체 쪽수가 같고
1·5·9쪽의 글리프·간격·기준선을 직접 판독했다. 같은 96dpi에서만 ±2px 간격을 고정한다.

`verified_legacy_equation_character_plane.rs`는 새 이름으로 등록한다.

1. 원본 문단·제어·쪽·글줄 소유, 내용 순서·누락·중복, 본문/글줄 포함을 보호한다.
2. 원본의 선언 너비와 실제 공통 수식 프레임, 변하지 않은 글리프 크기를 확인한다.
3. 같은 수식 안의 독립 Print 글리프 전진을 ±2px로 검사한다. 본문 라벨 글꼴이나
   페이지의 절대 위치를 고정하지 않는다. CI에 사설 글꼴 윤곽을 공급했다고 주장하지 않는다.

이 완료 입력의 검사가 복합 그룹·첨자·분수, LINE 범위 또는 원본 76쪽의 남은
수식 간격을 검증하는 것은 아니다. 해당 대조군은 미달 상태에서 정식 fixture로 채택하지 않았다.

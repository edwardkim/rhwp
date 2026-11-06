# 표지판 관리대장의 원본 TrueType 프로그램 분석

## 독립 기준과 최초 결과

원본 `samples/issue8639/sign_board_register_card.hwp`의 한컴 2020 Print
`pdf/issue8639/sign_board_register_card-print-2020.pdf`는 실제 `SeoulHangangM`·
`SeoulNamsanM` TrueType을 내장한다. 2022 Print는 `Haansoft Batang`을 내장하므로
두 PDF의 글꼴 환경을 구분한다. 2020 기준을 2022 PDF의 높은 점수로 대신하지 않는다.

초기 통합 source `46aca162bcf7fbcbc114cc42a251e6047598b371`의 Native 전쪽 결과는
2020 기준 33.10%, 2022 기준 100.00%다. 원본 이름과 같은 실제 TrueType 프로그램을
환경에 명시한 추가 실험도 33.82779%여서 환경 선언만으로 해결되지 않음을 확인했다.

## 코드 근거와 보정 규칙

- HWP5 FACE_NAME의 하위 비트는 TrueType 종류의 근거가 아니다. 새 base의
  `try_resolve_styles_for_document`가 `source_ttf` 추정을 비우는 사양 정정을 유지한다.
- `DocumentCore::native_otf_font_requests`는 명시적 TTF 또는 HFT만 요청한다.
  메트릭 표가 없는 미분류 HWP5 face는 설치된 원 프로그램이 있어도 요청하지 않는다.
- 실제 시스템의 서울한강체·서울남산체 M은 이름이 대응하고 `glyf`를 가진 TrueType이다.
  파일을 재배포하거나 폭을 상수로 복제하지 않고 실제 paint 공급자의 프로그램을 사용한다.
- 원본 HFT 선언도, 검증된 메트릭 표도 없는 face는 같은 이름의 TrueType 원 프로그램을
  요청할 수 있다. 원본 이름이 다른 대체 face와 CFF는 이 요청의 TrueType 근거가 아니다.
- 요청 자격과 실제 공급·전진 사용을 구분한다. 프로그램을 못 받으면 기존 동작을 유지하며,
  받았을 때만 원 프로그램의 `cmap`·`hmtx`로 미확인 전진을 대체한다. 원본 IR의 TTF/HFT
  선언은 바꾸지 않는다. 검증된 원본 폭·HFT 폭·공백과 대시 규칙도 유지한다.
- Native와 fresh WASM은 같은 요청·등록·스타일 조회·전진 선택 경로를 소비한다.
  특정 문서명이나 서울서체 이름으로 분기하지 않는다.

이는 보정 전 원인 분석이다. 코드 변경과 실제 재측정 결과를 아래에 추가한 뒤 수용 여부를 판단한다.

## 실제 원인 확인과 Native 보정 결과

2026-11-06 19:22 KST에 자동 선택을 재측정했다. 실제 엔진 SHA-256은
`8561be7cbfc5b783ac8ea0ba82759bf75f2f446db6c7399bece7de9b8ef61309`이며,
원본·2020 Print PDF는 위 기준과 같다. 96 DPI·compat 2022·Print profile의 전쪽 Native는
**1쪽 100.00%**, 쪽수 1=1이다. fresh WASM과 회귀·lint는 아직 완료하지 않았다.

- 처음에는 `--font-path`로 실제 두 TTF를 공급해도 자동 환경이 서울체를 한컴바탕으로 치환했다.
- `fontdb`의 `parse_names`는 Typographic Family(ID 16)가 있으면 Family(ID 1)를
  수집하지 않는다. 서울체의 ID 16 `SeoulHangang`·`서울한강체`와 ID 1
  `SeoulHangang M`·`서울한강체 M`은 모두 같은 프로그램의 실제 가족 이름이다.
- `FontEnvironment`·공개 웹폰트 준비·Skia 설치 프로그램 복구가 각각 이 축약 목록을
  조회해, 설치된 원본 가족을 놓쳤다. 단순 시스템 설치 유무나 파일 경로 문제가 아니다.
- 공통 `font_paths::retain_fontdb_family_aliases`는 실제 name 레코드 ID 1·21의 누락된
  가족 이름을 같은 source/index의 조회 뷰로 보존한다. 기존 face·스타일·프로그램은
  유지하며 Full Name(ID 4)·PostScript(ID 6)를 가족 이름으로 승격하지 않는다.
- 글꼴 환경 로더도 명시적 파일 경로를 디렉터리로 잘못 처리하지 않고 파일로 적재한다.
  글꼴 파일 자체는 추가하거나 배포하지 않는다.
- 원본 프로그램을 PostScript 이름으로 지정한 선행 진단도 100.00%였으며,
  보정 후에는 별도 글꼴 환경·개별 경로 지정 없이 전체 시스템 글꼴의 자동 선택으로
  같은 100.00%를 얻었다. 단순 별칭 하드코딩으로 대체하지 않았다.

실제 산출물은 `output/pr-review/planet6897-integration-20261106/` 아래의
`common-fixes-r3-producer.json`, `common-fixes-r3-native/sign-board-2020/`에 있다.
최종 검증 뒤 정식 TSV·review·overlay를 개별 리뷰에 연결한다.

2022 Print는 실제 Haansoft Batang 대체 환경이므로 2020 원 서울체 환경과 구분한다.
해당 대조군은 독립 출력과 같은 대체 선택을 명시해 검사하며, 자동 원본 선택의
2020 결과를 과거 2022 출력으로 대신하지 않는다.

## 정상 대조군의 추가 보정

법제 보고서 Native 대조군에서 1·2쪽이 99.97/99.95%에서 97.92/96.20%로 내려가는 것을
검출했다. 자동 글꼴 환경의 프로그램 목록은 이전과 같았다. 별도로 보존한 r2 엔진에서도
각주 한글 전진이 12.6px에서 11.6px로 바뀌어, 이후 추가한 가족 이름 조회나 각주 번호
경계 보정 이전부터 미분류 TTF 요청 확대가 영향을 주었음을 확인했다.

현재 표시 family에 메트릭 표가 없다는 이유만으로 원본 프로그램으로 인증하면 안 된다.
다른 원본에서 HCR Batang 등으로 대체된 언어 슬롯이 같은 family의 세션 프로그램을
공유하더라도, 원본과 같은 자연 전진을 선택했다는 근거가 아니다.
원본 DocInfo의 글꼴 ID·언어 슬롯과 최종 family의 동일 프로그램 이름을 함께 보존해,
대체된 슬롯을 미분류 원본 TrueType 요청에서 제외하는 보정을 추가했다.
같은 표시 family를 쓰는 두 슬롯이라도 원본이 다르면 요청 권한을 공유하지 않는 합성
계약 검사를 포함했다. 빌드와 실제 재측정 후 이전 수치 유지 여부를 확인한다.

`font_paths.rs`의 최초 분리 컴파일은 프로젝트의 다른 소비 모듈이 없어 6건의 dead-code
경고를 냈다. 제품 코드에 경고 억제 속성을 넣지 않고 정식 Cargo 라이브러리 단위 검사로
다시 실행했다. 정식 Cargo 검사에서 같은 글꼴 경로 검사 6건 모두 통과·경고 0건·exit 0을 확인했다.
다른 라이브러리의 필터 제외 항목을 통과 건수에 합산하지 않는다.


## 실제 Cargo 검사와 대조군 재측정

2026-11-06 19:46 KST 완료 검사에서 아래 정식 경로의 6건 모두 통과했고,
컴파일·실행 경고는 0건, 종료 코드는 0이었다. 최초 단독 파일 컴파일이 남긴
`MAX_NATIVE_CUSTOM_FONT_BYTES` 등 6개 미사용 경고는 실제 crate의 소비자가 없는
진단 구성에서 발생했다. 제품 코드에 `allow(dead_code)`를 추가하지 않았다.

```bash
cargo test --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review \
  --features native-skia --lib renderer::font_paths::tests:: -- --test-threads=1
```

실행에는 공개 글꼴 5개만 노출한 검토 wrapper를 사용했다. 실제 로그는
`output/pr-review/planet6897-integration-20261106/font-paths-identity-unit.log`,
명령·실행 시각·종료 코드는 같은 위치의 `font-paths-identity-unit.json`에 기록했다.
다른 라이브러리의 필터 제외 항목은 통과 건수에 합산하지 않는다.

원본 언어 슬롯 동일성 보정을 포함한 R5 Native 엔진 SHA-256은
`9e33c1b8d9dc2f0740ff7511c1168ba83fcb9323c5ab719bc52d52b8b8ea11e3`이다.
동일한 2020 Print 기준 표지판 관리대장 1쪽은 자동 선택 100.00%, 별도 2022 Print
대조군도 해당 PDF의 실제 대체 환경에서 100.00%를 유지했다.
법제 보고서 10쪽은 최저 99.95%이며, 1·2쪽은 각각 99.96/99.95%다.
처음 1쪽 99.97%와 비교한 0.01%p 차이를 동일 수치로 쓰지 않는다. 각주 번호의
실제 경계 보정이 함께 적용된 producer이며, 이 결과로 fresh WASM이나 최종 확대
판독을 완료했다고 판단하지 않는다.


2026-11-06 20:08 KST에는 R5 보정 코드와 기존 테스트 초기화 6곳의 누락을 보완한
동일 crate에서 원본 언어 슬롯 계약 1건을 실행해 통과했다. 별개 원본을 같은 표시
family로 대체해도 원본 TrueType 전진의 요청 자격을 빌리지 않는 반례다. 이어서
글꼴 경로 6건을 다시 실행해 모두 통과·경고 0건·exit 0을 확인했다. 최신 결과는
`font-paths-identity-unit.log`에 두었으며 이전 R4 결과는 `font-paths-identity-unit-r4.log`다.
원본 슬롯 계약 로그는 `original-font-slots-unit-r5.log`와 `.exit`에 있다.
두 검사는 글꼴 설치 여부에 의존하지 않으며 필터로 제외된 항목을 통과로 합산하지 않는다.

R3와 R5의 자동 서울체 선택 Native PNG SHA-256은 모두
`6a03c9a1a6efc213a93f545d6308d20474a902572b29eeec5a1412b0ab1321e5`로 같다.
동일 원본·독립 PDF·DPI의 같은 그림이므로 R3에서 만든 대표 review·overlay는
R5의 내용 확인에도 재사용할 수 있다. fresh WASM 결과로 재표기하지 않는다.

# 굴림체의 결측 한 점 리더 공급과 전진

부모 `f68e81f858ed09dd66ca224716dc2208c3059d7a`, base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`에서 `굴림체`(GulimChe) regular의 `․`(U+2024) 결측을 보완했다. 같은 원본 전103쪽을 다시 출력했으며 26쪽 하나만98% 미만이다. 이슈·PR 전체 완료는 아니다.

## 원본과 선행 해결의 대조

입력·기준 PDF는 [첫 저장 조각 증적](../opening-cell-frame-20261107/README.md)의 같은 SHA를 보존한다. `samples/issue5679/10857_delegation_rules.hwp`의 글자 모양32는 일곱 언어 슬롯 모두 `굴림체` TrueType을 지정한다. 실제 `gulim.TTC`(SHA `ed7502755424c560f4538647eff0833b738fe7f3f4ba4de8fca9ff8c4d6a776c`)의 GulimChe cmap에 U+2024가 없다.

독립 한컴2020 Print44·86쪽은 이 문자만 Haansoft Batang으로 출력한다. PDF 글리프와 기존 로컬2020 HBATANG의 윤곽·178/1024em 전진이 같다. 44쪽 `교육․홍보`는 `교`264.63999px, `․`291.19999px, `홍`293.60000px의 독립 원점을 갖는다. 검증 기록 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/one-dot-program-20261107/validation.json`)에 원본·PDF·글꼴 해시와 해당 프로그램 대조를 기록했다. 현재 Mac 루트 HBATANG에는 미세한 버전 차이가 있으나 전진은 같으며 글꼴 복사·설치를 변경하지 않았다.

기존 #8419의 제출 PR #8428·통합 PR #8622와 반영 커밋 `6d81fb84ed`는 Batang의 같은 결측을 기존 `verified_missing_glyph_supplier`와 cmap 경로로 해소했다. 현재 공급자·원본을 각각 대조한 뒤 같은 공통 소비 경로에 GulimChe regular U+2024를 추가했다. 일반 Gulim과 GulimChe를 섞지 않으며 다른 문자·굵기·기울임의 공급자를 추정해 확대하지 않는다.

## 구현과 실제 소비

기존 코드는 없는 글리프를 고정폭1em으로 재어 가운데 정렬 문장이 밀렸다. 검증된 결측을 명시하고 기존 Haansoft Batang의 자연 전진을 사용한다. 측정과 paint는 기존 `measured_glyph_supplier`의 같은 결정을 소비한다. 새 글리프 폭 배열·문서/쪽 예외·픽셀 허용값·기준 PDF 변경은 없다. 글꼴 바이너리·복원 표본·윤곽 데이터는 커밋하지 않는다.

## 전103쪽 결과

PDFium96DPI·compat2022·색상 경계 보정 전2px 내용 실루엣 raw, capture 총8의 같은 조건이다. 실제 로컬 글꼴 공급 시각 출력과 macOS 최소 글꼴 sandbox의 의미 검사는 구분한다.

| 쪽 | Native 이전 → 보정 | fresh WASM 이전 → 보정 |
| --- | ---: | ---: |
| 9 | 97.91001 → 99.86222% | 97.90146 → 99.86214% |
| 44 | 97.75656 → 100% | 97.75547 → 100% |
| 51 | 99.51311 → 100% | 99.51281 → 100% |
| 86 | 96.68760 → 99.70650% | 96.68846 → 99.70611% |

각103쪽·누락0·90% 미달0이다. 양쪽 모두62쪽의 수치/PNG가 바뀌고41쪽은 PNG 바이트가 같다. 페이지별0.1%p 초과 하락0이다. 남은26쪽은 Native96.65328% / fresh WASM96.65155%이며 표 하단 차이를 계속 보정한다.

[이전 Native TSV](before-native-silhouette.tsv) · [이전 WASM TSV](before-wasm-silhouette.tsv) · [보정 Native TSV](after-native-silhouette.tsv) · [보정 WASM TSV](after-wasm-silhouette.tsv)

## 회귀 검사와 범위 게이트

[정식 검사](../../../../../tests/cases/issue_7700_source_owned_table_fragments.rs)에 공급자의 실제 전진 관계형과44쪽 앞뒤 문자 원점 고정형을 추가했다. 기존7개는 유지했으며 실제7 PASS/2 FAIL→9 PASS다. 해당44쪽은 Native/fresh WASM 모두100%이고 독립 PDF 원점을96DPI±2px로 비교한다. 기대값을 후보 PNG에서 복사하지 않았다. 검사의 글꼴 전진·원점 의미는 개인 글꼴 파일 없이 macOS 최소 글꼴 sandbox에서도 통과했다. 실제 시각 공급/출력의 전쪽 증거는 위 TSV다.

글꼴 정책·기호 슬롯·기존 결측 공급자·저장 프레임·중첩 표까지160 PASS이며 정식9개·출력용 진단1개를 포함한다. 중복 모듈을 넣었던 초기 임시 묶음은 등록을 바로잡고 해당 묶음만 다시 실행했다. 정식 검사·assertion은 제거하지 않았다. fmt·Native/WASM/전체 target/Skia Clippy·workspace build·manifest·unit tier가 통과했다.

fresh WASM은 Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 결과가 아니다. root pkg/Studio public 파일 일치는 확인했으며 실제 브라우저 초기화 응답·최종 전체 nextest·편람 전쪽·실제 소비 경로 및 다른 문서98% 보정은 남아 있다.

## 대표 review·overlay

- 9쪽 Native 이전: [review](before-native-p009-review.png) · [overlay](before-native-p009-overlay.png); 보정: [review](after-native-p009-review.png) · [overlay](after-native-p009-overlay.png)
- 9쪽 fresh WASM 이전: [review](before-wasm-p009-review.png) · [overlay](before-wasm-p009-overlay.png); 보정: [review](after-wasm-p009-review.png) · [overlay](after-wasm-p009-overlay.png)
- 44쪽 Native 이전: [review](before-native-p044-review.png) · [overlay](before-native-p044-overlay.png); 보정: [review](after-native-p044-review.png) · [overlay](after-native-p044-overlay.png)
- 44쪽 fresh WASM 이전: [review](before-wasm-p044-review.png) · [overlay](before-wasm-p044-overlay.png); 보정: [review](after-wasm-p044-review.png) · [overlay](after-wasm-p044-overlay.png)
- 51쪽 Native 이전: [review](before-native-p051-review.png) · [overlay](before-native-p051-overlay.png); 보정: [review](after-native-p051-review.png) · [overlay](after-native-p051-overlay.png)
- 51쪽 fresh WASM 이전: [review](before-wasm-p051-review.png) · [overlay](before-wasm-p051-overlay.png); 보정: [review](after-wasm-p051-review.png) · [overlay](after-wasm-p051-overlay.png)
- 86쪽 Native 이전: [review](before-native-p086-review.png) · [overlay](before-native-p086-overlay.png); 보정: [review](after-native-p086-review.png) · [overlay](after-native-p086-overlay.png)
- 86쪽 fresh WASM 이전: [review](before-wasm-p086-review.png) · [overlay](before-wasm-p086-overlay.png); 보정: [review](after-wasm-p086-review.png) · [overlay](after-wasm-p086-overlay.png)

가운데 정렬 앞뒤 문자·줄 소속·괘선과 다음 내용을 직접 대조했다. 일부 기존 원 기호·글리프 가장자리·미세한 행 높이 차이는 남는다. raw100%를 RGB 전체 동일 또는 다른 문서의 완료로 확대하지 않는다. 실행 로그·중간 스크립트는 ignored output에만 보존한다.


## 후속 슬롯 보정

이 기록의26쪽 미달은 [후속 공통 슬롯 보정](../reset-span-slot-20261107/README.md)에서
양 backend99.98249%로 개선했다. 새 전103쪽은 Native최저99.21307%·fresh WASM최저99.21221%이며,
위의 이전 기호 단계 수치는 당시 증거로 유지한다.

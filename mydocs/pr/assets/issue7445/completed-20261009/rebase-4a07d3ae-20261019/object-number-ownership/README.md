# 개체 번호 소유·원문 슬롯 검증 증적

최종 소스는 단계18 보고서와 Native607/fresh610 source ledger로 연결한다.
canonical PDFium smooth_rgb_v1/192DPI/compat2022/raw2px90%, 각 입력16/16쪽이다.

|입력|Native 최저|fresh WASM 최저|전체 TSV|독립 Print|
|---|---:|---:|---|---|
|original|95.34478%|95.33483%|[Native](native/original.tsv) / [WASM](wasm/original.tsv)|[PDF](reference/original-2020.pdf)|
|old-none-hwp|99.02719%|98.92066%|[Native](native/old-none-hwp.tsv) / [WASM](wasm/old-none-hwp.tsv)|[PDF](reference/old-none-hwp-2020.pdf)|
|old-none-hwpx|99.03102%|98.92493%|[Native](native/old-none-hwpx.tsv) / [WASM](wasm/old-none-hwpx.tsv)|[PDF](reference/old-none-hwpx-2020.pdf)|
|public-save-hwp|99.02719%|98.92066%|[Native](native/public-save-hwp.tsv) / [WASM](wasm/public-save-hwp.tsv)|[PDF](reference/public-save-hwp-2020.pdf)|
|public-save-hwpx|99.03102%|98.92493%|[Native](native/public-save-hwpx.tsv) / [WASM](wasm/public-save-hwpx.tsv)|[PDF](reference/public-save-hwpx-2020.pdf)|
|inline-unbound|99.03275%|98.92675%|[Native](native/inline-unbound.tsv) / [WASM](wasm/inline-unbound.tsv)|[PDF](reference/inline-unbound-2020.pdf)|
|all-picture|99.02719%|98.92066%|[Native](native/all-picture.tsv) / [WASM](wasm/all-picture.tsv)|[PDF](reference/all-picture-2020.pdf)|
|mixed-picture-none|99.02719%|98.92066%|[Native](native/mixed-picture-none.tsv) / [WASM](wasm/mixed-picture-none.tsv)|[PDF](reference/mixed-picture-none-2020.pdf)|
|duplicate-field-owner|99.02719%|98.92066%|[Native](native/duplicate-field-owner.tsv) / [WASM](wasm/duplicate-field-owner.tsv)|[PDF](reference/duplicate-field-owner-2020.pdf)|
|none-start7|99.02719%|98.92066%|[Native](native/none-start7.tsv) / [WASM](wasm/none-start7.tsv)|[PDF](reference/none-start7-2020.pdf)|
|picture-start7|99.02719%|98.92066%|[Native](native/picture-start7.tsv) / [WASM](wasm/picture-start7.tsv)|[PDF](reference/picture-start7-2020.pdf)|
|cross-caption-category|99.02719%|98.92066%|[Native](native/cross-caption-category.tsv) / [WASM](wasm/cross-caption-category.tsv)|[PDF](reference/cross-caption-category-2020.pdf)|

[입력·PDF 전체 SHA256](inputs-and-references.tsv). 총384쪽, 누락·중복0,90% 미달0쪽.

## 직접 판독

- native inline-unbound 1쪽: [review](native-inline-unbound-p001-review.png) / [overlay](native-inline-unbound-p001-overlay.png)
- wasm inline-unbound 1쪽: [review](wasm-inline-unbound-p001-review.png) / [overlay](wasm-inline-unbound-p001-overlay.png)
- native inline-unbound 15쪽: [review](native-inline-unbound-p015-review.png) / [overlay](native-inline-unbound-p015-overlay.png)
- native public-save-hwp 8쪽: [review](native-public-save-hwp-p008-review.png) / [overlay](native-public-save-hwp-p008-overlay.png)
- wasm duplicate-field-owner 3쪽: [review](wasm-duplicate-field-owner-p003-review.png) / [overlay](wasm-duplicate-field-owner-p003-overlay.png)
- native old-none-hwp 8쪽: [review](native-old-none-hwp-p008-review.png) / [overlay](native-old-none-hwp-p008-overlay.png)
- wasm old-none-hwpx 3쪽: [review](wasm-old-none-hwpx-p003-review.png) / [overlay](wasm-old-none-hwpx-p003-overlay.png)
- wasm public-save-hwpx 8쪽: [review](wasm-public-save-hwpx-p008-review.png) / [overlay](wasm-public-save-hwpx-p008-overlay.png)

본문1쪽 Before1/1과15쪽 After5/5, 중복 캡션2/2, NONE 캡션1,
공개 저장본8쪽 그림4가 독립 PDF와 같다. 번호와 뒤 본문 사이 공백·표/그림 앞뒤 소속을 확인했다.
본문·로고·표 선의 미세한 위치/글리프 차이는 남으며100% 일치를 주장하지 않는다.
[보정 전 review](before-inline-p001-review.png) / [overlay](before-inline-p001-overlay.png)는
원문 슬롯 뒤 여분 공백으로 둘째 번호가194.88CSSpx에 놓인 중간 후보602의 출력이다.
최종607은 독립 PDF188.217997CSSpx의 ±2px 안이며 원문 공백을 label 표시용으로 소비한다.

실행 로그·중간 JSON·생성기·무효 실행은 ignored output에만 남긴다.
전체 브랜치 PR 검증과 fixed devel 비교 완료를 이 부분 증거로 대신하지 않는다.

## 고정 기준 대조: PR 보류

[전체384쪽 기준/후보 대조](base-candidate-comparison.tsv).
고정4a07의 renderer/parser/model/serializer는 현재 기준2e3b4f88과 바이트 동일하고
그 사이 src 변경은 clipboard.rs뿐이다. 이 컴포넌트 재사용을 새 base 전체 fresh 빌드라고 주장하지 않는다.
Native와 WASM의 실제 로드된 글꼴 family/source SHA/SFNT SHA/face/weight/style 집합도 입력별로 같다.
WASM의 명시 null 환경은 후보가 실제 선택한 None과 같으며, 실제 paint 글꼴 공급 동등성을 별도로 확인했다.

0.1%p 이상 하락은27관측: 원본13쪽×양쪽26건과 picture-start7 Native8쪽1건이다.
최대 하락은 원본 Native6쪽4.07228%p. 저장본·대조군의 높은 점수로 이를 상쇄하지 않는다.
[기준 원본3쪽 review](base-original-p003-review.png) / [overlay](base-original-p003-overlay.png),
[후보 review](candidate-original-p003-review.png) / [overlay](candidate-original-p003-overlay.png).
독립 원본 Print는 꼬리말 그림 외곽의 B(fill+stroke) 뒤 device CTM0.120을 쓰고,
같은 그림의 현재 저장본은 그 외곽이 없어0.119935/-0.119869를 유지한다.
HWP3 명세의 그림 옵션 offset14 bit0과 파서의 누락을 추가 원인으로 추적한다.
이 차이를 글꼴 파일 불일치로 분류하거나 허용치를 낮추지 않는다.

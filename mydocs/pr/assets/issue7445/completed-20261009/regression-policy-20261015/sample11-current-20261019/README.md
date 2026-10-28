# sample11 현재 전151쪽 증적과 직접 판독

- 원본: `samples/hwp3-sample11.hwp`, SHA256 `51b743b2823a2df9b6fac243f56aebecedbbd02e2a8baad58ffc2e5a4e695f20`.
- 독립 사용자 한컴 Print: `pdf/hwp3-sample11-ui-print-2020.pdf`, SHA256 `d787942b264a6593a0a5aafc50f44ecc85e7fca70b2617c2ec87c0e7315f7fda`,151쪽.
- parent `b80662087571f746579350d2ef00f3387b54b527`의 미커밋 후보 C360/C377 소스 원장을 사용했다. parent만으로 이 미커밋 후보를 식별하지 않는다. 소스·빌드·글꼴 상세는 Stage8 C399 기록과 ignored producer 원장을 따른다.
- 고정 base `6ed1682c`와 최신 devel `224a3260` 사이 해당 실행 바이트의 동일성 근거 C344를 사용했다. 양 역할 모두 C373의 같은99glyph HFT 공급 조건이다. 제품 기본 글꼴 실제 소비 검증은 별도다.
- canonical Visual Sweep, compat2022,192DPI,기준 PDF pdftoppm, 색상 경계 보정 전2px 실루엣90% 문턱을 유지했다.

| 역할 | Native 최저 | fresh WASM 최저 | 페이지 수 |90% 미만 |
|---|---:|---:|---:|---:|
| 고정 base |20.00112%|19.58063%|151/151|150/148|
| 현재 후보 |90.06416%|90.07462%|151/151|0/0|

현재 양 backend 모두0.1%p 초과 같은 쪽번호 하락은0이다. 이 정량 결과는 직접 판독의 남은 차이를 면제하지 않는다. 원본 sample16의64쪽 자료와 혼동하지 않는다.

## 대표 review와 standalone overlay

| 페이지/역할 | Native | fresh WASM |
|---|---|---|
|2쪽 목차|[review](head-native-p002-review.png) · [overlay](head-native-p002-overlay.png)|[review](head-wasm-p002-review.png) · [overlay](head-wasm-p002-overlay.png)|
|50쪽 도해·최저|[review](head-native-p050-review.png) · [overlay](head-native-p050-overlay.png)|[review](head-wasm-p050-review.png) · [overlay](head-wasm-p050-overlay.png)|
|117쪽 코드·ASCII 도해|[review](head-native-p117-review.png) · [overlay](head-native-p117-overlay.png)|[review](head-wasm-p117-review.png) · [overlay](head-wasm-p117-overlay.png)|
|base50쪽 대조|[review](base-native-p050-review.png) · [overlay](base-native-p050-overlay.png)|[review](base-wasm-p050-review.png) · [overlay](base-wasm-p050-overlay.png)|

16개 PNG를 직접 열었다.2쪽의5.2행은 앞/뒤 목차와 들여쓰기·줄 소속이 유지되고 과거 왼쪽 이탈은 보이지 않는다.50쪽 PC/서버/라우터·회선과 뒤 본문·하단49의 소속은 유지되며 base의 그림 위치/소실·footer 큰 이동은 개선됐다. PDF에만 있는 회색/갈색 면은 기존 독립 인쇄와 사용자 편집기 대조의 범위에서 별도 구분한다.

남은 차이:2쪽 INDEX 자형과 일부 자간, 얇은 상·하 가로선의 색경계/미세 위치,50쪽 연결선·색면·윤곽과 본문 작은 glyph 차이가 보인다.117쪽 route add/Basic routing ASCII 도해는 일부 수평 glyph 원점·공백 간격이 PDF와 다르다. 이것을 단순 글꼴 문제나 전체 시각 완료로 판정하지 않으며 후속 원인 검토 대상으로 남긴다.98% 미만인 이 페이지들에는 절대 픽셀 회귀 핀을 추가하지 않는다.

## 전쪽 원본 TSV

- [현재 Native151쪽](head-native-151-pages.tsv), [현재 fresh WASM151쪽](head-wasm-151-pages.tsv)
- [base Native151쪽](base-native-151-pages.tsv), [base WASM151쪽](base-wasm-151-pages.tsv)

PNG/TSV는 원본과 byte동일하게 복사했다. 전체52조건, 소유/누락/중복 검사 및 실제 제품106요청의 완료를 이 문서로 주장하지 않는다. PR은 모든 필요한 검증과 사용자 승인 뒤 진행한다.

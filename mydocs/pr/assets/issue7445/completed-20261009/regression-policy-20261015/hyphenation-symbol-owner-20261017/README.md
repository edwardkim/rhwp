# U+2027 기호 슬롯 소유 보정 (2026-10-17)

Issue: #7445. base `fe5725dc384689b45d60ef8ea4c83138bc2701d8`,
부모 `f6e716d65856041fcfc4b89035fb3fb242b0b865`의 C75 후보다.
이 자료는 집중 검증이며 전체48조건·전체 회귀·실제 Studio 수용이나 PR 준비 완료를 뜻하지 않는다.

## 입력과 독립 근거

- 원본: `samples/issue6145/worklife_balance_index_156607916.hwpx`,
  SHA256 `7778e5373baf7371a7c643cf4cca73defb065a92ecc941a3e55fff34fcdd2d17`.
- 독립 한컴2020 Print: `pdf/planet-review-20260917/worklife_balance_index_156607916-2020.pdf`,
  SHA256 `0cc7229c1d8da5ef425ef76f3ba143db17171a71449bc4e341579a18a07a47ec`.
- 원본6쪽과 PDF6쪽. Print profile, compat2022, 기존 devel 비교와 같은96DPI,
  canonical Visual Sweep raw2px 실루엣을 사용했다. 다른 DPI의 수치를 섞지 않는다.

독립 PDF5쪽은 휴먼명조 본문 사이의 `‧`를 HFT T14/T19로 출력하며 자연 폭은0.5em이다.
한글/영문/기호 슬롯을 서로 다른 실제 face로 교차한 별도 한컴2020 Print에서도
U+2027은 앞 글자의 언어와 무관하게 기호 슬롯을 따른다. 같은 face인 대조군도 유지했다.
TrueType HY신명조의 결측 공급자는 한컴바탕, 굴림의 결측 공급자는 한컴돋움이며
해당 프로그램의 실제 cmap과 출력 face를 함께 확인했다. HFT 폭에 TTF hmtx를 대입하지 않는다.
추가 probe는 합성 프로그램 진단이며 실물 문서 시각 통과의 대체물이 아니다.

`detect_lang_category` → composition/공통 char-width → 원문 scalar별 TextRunNode →
Native/Canvas paint가 같은 기호 슬롯을 소비한다. 원문을 바꾸지 않는다.
기본/명시 환경만으로 폭을 나누려던 초기 후보는 반증돼 전부 되돌렸다.
기호 소유를 본문 슬롯으로 가정하던 원인을 먼저 고친 결과다.

## 전쪽 확인과 중요한 페이지 보호

|범위|Native 최저|fresh WASM 최저|90% 미만|쪽수|
|---|---:|---:|---:|---|
|일·생활균형 원본6쪽|97.57113%|97.56858%|0|6/6|
|개인정보 명시 환경66쪽|94.12098%|94.12098%|0|66/66|
|개인정보 기본 환경66쪽|94.12098%|94.12098%|0|66/66|

|일·생활균형5쪽|부모|보정 후|devel 기준|
|---|---:|---:|---:|
|Native|98.93457%|99.93989%|99.33098%|
|fresh WASM|98.96869%|99.94147%|99.33957%|

원본 전6쪽 모두90% 이상이며 실제 변경 경계인5쪽은 양쪽98% 이상이다.
같은 HFT 기호 소유를 검증하는 페이지로5쪽을 선택했다. 다른 face/경로의 더 높은
페이지로 경계를 대체하지 않는다. 원문·단일 소유·다음 본문 순서와 함께 독립 원점을
±2CSSpx로 보호한다. 첫 점의 X292.641388pt/기준선83.428825pt, 다음 '공'의
X300.916901pt를96CSSpx로 환산한다. 관계 검사도 유지한다.
review/overlay에서 세 점·앞뒤 본문·표와 후속 내용을 확인했다. 엄격 RGB 일치100%를 주장하지 않는다.

기본/명시 개인정보의 전체 결과는 부모와 동일하며 새 시각 하락은 없었다.
다만 devel 대비 일·생활균형4쪽(-0.15998%p Native), 개인정보22/44쪽 등 기존
하락은 남아 있다. WMF18/35쪽 및 giant 원본 WASM80쪽 차이도 별도 잔여다.
이번 개선으로 이들을 상쇄하거나 PR gate 통과로 판정하지 않는다.

## 실행과 미검증

슬롯 API 회귀는 수정 전6건 중5PASS/1FAIL → 수정 후6PASS/0FAIL,0.027초다.
전쪽 검증 뒤 실물5쪽 px/내용 보호를 포함한7PASS(0.067초), 기존 개인정보50쪽의
단일 긴 run 가정을 교체한1PASS(0.175초)로 정식 검사8건을 통과했다.
새 페이지 검사의 첫 BoundingBox 메서드 오용은 컴파일 오류로 제외하고 수정 뒤 실제 실행했다.
개인정보의 독립 px 기대값을 완화하지 않는다.

Native frozen SHA256 `5c2b9c37aa192aa3727ab323fe5b586d2ea8cf53e2371de07cfc0e96354df44b`.
fresh WASM SHA256 `0e413fc14b6dafa79469f79212d83faf5f9a28734d4ba84ec00f7127c4ae1f96`.
Mac locked wrapper web/no-opt 로컬 대체 빌드이며 Docker 최적화 수용이 아니다.
루트 pkg/Studio public hash는 같지만 새 browser 초기화 응답은 아직 검증하지 않았다.
TSV/producer/입력·PDF·글꼴 hash/실제 로그/한컴 job metadata와 거절한 후보는 ignored output의
`regression-policy-audit-20261015/rebase-fe5725dc-20261017/direction-review-c93/`에 보존한다.

## 비교 이미지

|출력|review|overlay|
|---|---|---|
|부모 Native|[review](before-native-review.png)|[overlay](before-native-overlay.png)|
|보정 Native|[review](after-native-review.png)|[overlay](after-native-overlay.png)|
|보정 fresh WASM|[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|

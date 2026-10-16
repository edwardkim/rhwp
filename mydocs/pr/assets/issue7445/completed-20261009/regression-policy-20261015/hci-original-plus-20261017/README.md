# HCI Poppy '+' 원본 전진 보정 (2026-10-17)

Issue: #7445. base `fe5725dc384689b45d60ef8ea4c83138bc2701d8`,
부모 C75 `bd153fa40fba5e9890efe8dda90d5f3f178579e0`의 C76 보정이다.

## 독립 근거와 공통 소비

입력은 `samples/issue6145/worklife_balance_index_156607916.hwpx`,
SHA256 `7778e5373baf7371a7c643cf4cca73defb065a92ecc941a3e55fff34fcdd2d17`.
독립 PDF는 `pdf/planet-review-20260917/worklife_balance_index_156607916-2020.pdf`,
SHA256 `0cc7229c1d8da5ef425ef76f3ba143db17171a71449bc4e341579a18a07a47ec`다.

4/5쪽 Type3 T10은 ToUnicode002B, Widths605, FontMatrix.001이다.
원본 charPr100/128의 LATIN은 보통체·정체 HCI Poppy다. 기존 표시 Palatino의
0.5em으로 계산하면 같은 줄의 두 '+' 뒤 숫자가 누적해서 약3.3CSSpx 당겨졌다.
기존 부분 `OriginalGlyphMetric`의 확인한 보통체 '+'에605/1000을 추가했다.
원본 glyph metric → char-width 결정 → composition/position → 공통 TextRun/각 paint가
같은 전진을 소비한다. 좌표 보정·문서 ID 분기·다른 face나 미확인 스타일의 폭 변경은 없다.

## 전쪽과 devel 비교

Print profile, compat2022, 기존 devel과 같은96DPI, canonical raw2px 실루엣이다.

|원본6쪽|Native|fresh WASM|
|---|---:|---:|
|전쪽 최저|97.56458%|97.56205%|
|90% 미만|0|0|
|쪽수|6/6|6/6|
|devel 대비0.01%p 이상 하락|0|0|

|쪽|부모 Native→현재|부모 WASM→현재|devel Native/WASM|
|---|---:|---:|---:|
|4|98.74104→99.00620%|98.73373→98.99872%|98.90102/98.93976%|
|5|99.93989→99.94332%|99.94147→99.94496%|99.33098/99.33957%|

3쪽은 부모 대비 각각0.00655/0.00653%p 감소했으며 숨기지 않는다. devel 대비는
양쪽 모두 개선이고0.01%p 하락 기준에 해당하는 쪽은 없다. 평균으로 상쇄하지 않았다.
4쪽은 같은 물리 줄에서 두 '+'의 전진이 누적되는 경계다.5쪽의 더 높은 단일 '+'
페이지와 다른 조건이므로 이 반복 경계의 독립 ±2CSSpx 검사는4쪽에 둔다.
C75의5쪽 기호 소유·원문·위치 보호도 유지했다. 양쪽 review/overlay를 직접 확인했다.

## 실제 FAIL→PASS와 좌표 보호

프로그램 메트릭 검사는8실행7PASS/1FAIL(0.082초) →8PASS/0FAIL(0.079초)다.
전쪽/98% 조건과 직접 판독 뒤 반복 '+' 문서 검사를 추가하고 **같은9개 검사**를
부모의 원본 메트릭과 후보로 실행했다. 부모9실행7PASS/2FAIL(0.085초) →
보정9PASS/0FAIL(0.085초)다. 형식/환경 오류가 아니라 자연 폭과 마지막 숫자 원점에서 실패했다.

독립 PDF의 마지막 `전남(+1.0...)`에서 '1' X347.8113607CSSpx,
다음 '.' X356.1271973CSSpx, 기준선997.9493815CSSpx를 ±2로 확인한다.
수정 전 숫자는344.493334CSSpx로 실제 실패했다. 원문 순서·누락·중복,
단일 source 소유·같은 run의 앞 기호/숫자/소수점 순서와 모델 불변도 검사한다.
측정값을 기대값으로 재인용하거나 허용 오차를 완화하지 않았다.

전후 검사 동안 원본 메트릭 파일 한 개만 안전하게 부모로 교체했고 finally에서
후보를 복원했다. 복원 SHA는 캡처 producer의 동일 파일 SHA와 일치한다.
이후 테스트 서두의 낡은 설명만 고쳤으며 컴파일·실행한 함수 본문은 byte-identical이다.
정식 실행 원문과 주석만 바꾼 동등성 증거도 ignored output에 보존한다.

## 범위와 잔여

Native frozen SHA256 `02553059c38a825be18b4543804db479a6fa0aaebddff1f56f1064e593ec57ed`.
fresh WASM SHA256 `fdb146bf2f6fece92c0c8c00adb14bc1ad32c0c589a9e72aea635cdceba8ecd2`.
Mac locked wrapper web/no-opt 로컬 대체 빌드이며 Docker 최적화가 아니다.
Root pkg/Studio public의 JS/WASM hash가 같음을 확인했다. 새 실제 browser 소비는 별도 잔여다.

TSV/전쪽 비교/실행 로그/원본·PDF·producer SHA와 원점 진단은 ignored output의
`regression-policy-audit-20261015/rebase-fe5725dc-20261017/hci-original-plus-c94/`에 보존한다.
이번6쪽 수용을 다른 문서의 devel 하락이나 WMF raw 미달 해소로 확대하지 않는다.
전체 변경 범위 재검증·전체 회귀·필수 gate·실제 소비 경로가 남아 PR은 아직 보류다.

## 비교 이미지

|출력|review|overlay|
|---|---|---|
|부모 Native|[review](before-native-review.png)|[overlay](before-native-overlay.png)|
|보정 Native|[review](after-native-review.png)|[overlay](after-native-overlay.png)|
|보정 fresh WASM|[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|

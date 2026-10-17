# 표 셀 안의 쪽번호 선언 보존 (2026-10-17)

Issue: #7445. base `b7a5388034ee66f58a7e1d5363974e83b6374f57`,
부모 `bc194ba7c61795254aee3230f27e75eb2ebe7cb8`의 C77 보정이다.

## 독립 근거와 원인

입력 `samples/issue6145/worklife_balance_index_156607916.hwpx`,
SHA256 `7778e5373baf7371a7c643cf4cca73defb065a92ecc941a3e55fff34fcdd2d17`.
독립 Print `pdf/planet-review-20260917/worklife_balance_index_156607916-2020.pdf`,
SHA256 `0cc7229c1d8da5ef425ef76f3ba143db17171a71449bc4e341579a18a07a47ec`다.

원문 첫 쪽의 표 셀에는 `pageNum pos=BOTTOM_CENTER, formatType=DIGIT, sideChar=-`가
한 번 있다. exact base의 공개 IR에서도 구역0/문단1/표0/셀2 안의 `PageNumberPos`를
확인했다. PDF에는1~6쪽 모두 Gulim `- n -`가 있지만 부모 Native/fresh WASM의
12개 render-tree에는 꼬리말 글자가 모두 없었다.90% 이상이어도 직접 판독에서 발견한
누락을 완료로 보지 않고 구현을 보정했다.

두 조판 경로가 공유하는 `collect_page_number_pos`가 본문·표 셀의 마지막 선언을
소스 순서로 선택한다. 기존 머리말/꼬리말 전용 수집 인터페이스는 유지하고 그 내용 안의
선언을 구역으로 승격하지 않는다. 선택 결과 → 두 경로의 `PageContent.page_number_pos` →
`build_page_number`의 공통 스타일·기준선 → Footer → 최종 print TextRun/각 PNG paint로
전달된다. 쪽번호 감추기·새 번호·구역 간 carry의 소비는 바꾸지 않았다.

48조건 원본 IR 중 셀 안 쪽번호 선언은 이 문서에서만 관측됐다. 이는 발동 범위
조사이며 나머지 문서의 시각 검증을 대신하지 않는다. legacy Paginator도 같은 원문을
실행해6쪽 모두 위치5·번호1~6을 선택함을 확인했다. 이 진단을 legacy 전쪽 시각 수용으로
확대하지 않는다.

## 전쪽과 직접 판독

Print profile·compat2022·96DPI·canonical raw2px/90%를 유지했다.

|원문6쪽|Native|fresh WASM|
|---|---:|---:|
|전쪽 최저|97.61944%|97.61696%|
|90% 미만|0|0|
|쪽수|6/6|6/6|
|6쪽|99.99225%|99.99226%|
|1쪽(같은 동작의 다음 후보)|99.99213%|99.99200%|

같은 동작을 검증할 수 있는 쪽 중 양쪽 raw 최솟값이 가장 높은6쪽을 선정했다.
이 쪽은 문서 종료의 쪽번호도 보호한다. 부모의6쪽99.94189/99.94191%보다 개선됐다.
양쪽의 전6쪽에서 쪽번호를 제외한 render-tree JSON이 같았다. export 정밀도의 대조이며
모든 subpixel 배치가 같다는 주장은 아니다.6쪽 review/standalone overlay를 양쪽 모두 열어
본문·표 내용과 단일 쪽번호를 확인했다. 기준선의 작은 차이는 아래에 남겼다.

## 관계·독립 px 회귀

정식 `issue_7445_nested_page_number` 검사 결과: **수정 전1FAIL(0.044초) → 수정 후1PASS(0.066초), 같은 검사 소스**.
같은 검사 소스로 수정 전/후를 실행하며, 임시 원복은 finally에서 후보를 복원한다.
원문 셀의 선언,6쪽 각각의 단일 꼬리말·번호 문자열·밴드 내부 포함·원문 보존을
검사하고 마지막 쪽의 최종 print paint도 직접 확인한다.

독립 Print6쪽 Gulim의 원점283.2864685/797.0089722pt를96CSSpx로 환산한
X377.7152913·기준선1062.6786296을 ±2px로 보호한다. 현재 실제 print paint는
377.853333/1061.382222로, 각각+0.1380/−1.2964px다. 이 잔여를 숨기지 않으며
좌표 이동·허용 오차·DPI를 바꾸지 않았다.

## 출처와 잔여

Native SHA256 `ba342cef5481843f3ec4eda93ab72bbafc8c6ed15caf623b3efe962aec2cc902`.
fresh WASM SHA256 `1023cb137cc7ed480a0c1e4191360b54e8b140c5077e76405fbd94ce5199f483`.
Mac locked wrapper web/no-opt 로컬 대체 빌드이며 Docker 최적화가 아니다.
Root pkg/Studio public의 JS/WASM hash 일치를 확인했다. 새 실제 Studio 초기화 소비는
전체 변경 범위의 후속 검증에 남아 있다.

캡처 후 공통 함수의 들여쓰기만 rustfmt로 정리했다. 보존한 원문에 대한 rustfmt 표준
출력이 현재 소스와 정확히 같음을 기록했다. 최종 head의 새 빌드·전쪽 비교는 별도로 수행한다.
첫 수집 helper의 인자 확장 시도는 머리말 전용 호출 두 곳의 컴파일 오류로 폐기했다.
최종 구현과 정식 FAIL/PASS의 증거로 그 실패를 사용하지 않았다.

ignored `rebase-b7a53880-20261017/nested-page-number-c95/`에 TSV·원본/PDF hash,
producer·선정 후보·양쪽 본문 보존·legacy 진단·실행 원문을 보존한다. 새로운 b7 devel
전쪽 비교·전체 회귀·lint/Skia·실제 소비 경로와 기존 WMF raw 미달/다른 하락 보정은
별도 잔여다. 이6쪽의 개선으로 전체 PR 준비 완료를 주장하지 않는다.

## 비교 이미지

|출력|review|overlay|
|---|---|---|
|부모 Native|[review](before-native-review.png)|[overlay](before-native-overlay.png)|
|보정 Native|[review](after-native-review.png)|[overlay](after-native-overlay.png)|
|보정 fresh WASM|[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|

정상 대조군은 split-paragraph 감추기/새 번호5건·쪽 정보1건·기호 원본/중요 위치9건,
총15건PASS이며 새1건과 합쳐16건PASS다. fresh WASM wrapper가 생성한 Studio glue의
추가 런타임 import39행도 함께 동기화했다. API를 수동 수정한 것은 아니며 실제 초기화
응답 hash 검증은 새 head의 전체 소비 검증에 남긴다.

선택 순서의 추가 대조본3개는 한컴2020 Print queue에 제출했다. 현재 queued 상태이므로
그 결과를 혼합 선언 순서의 독립 수용 근거로 사용하지 않는다. 원본6쪽의 단일 중첩 선언
수용과 구분하며 ignored `order-probes/`의 기존 job ID로 계속 확인한다.

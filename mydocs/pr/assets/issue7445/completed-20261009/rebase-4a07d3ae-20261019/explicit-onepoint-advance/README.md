# 명시한 기호 공급자의 자연 전진 전달

2026-10-19~20 KST, parent `1bb5b3f6b3a63a2daa1deaf661db5b760a657ee3` + runtime patch
SHA `98b021d032c623c7377206279e246a910ddba30fadd5fe8cc780d0eef3be9385`. Native frozen68 CLI SHA
`e48c2c452012545b1935493f32c2ae2d81d334a08091e8b5fdc6f9bcf625c84f`,
fresh69 WASM SHA `18c5d547535ee2aa62c9e2375f63e2ea0f0ebc36c14b77a3925abd22613ebb4f`.
Native/Studio runtime2198파일 ledger를 source와 대조했다. 새 글꼴이나 공급 경로를 추가하지 않았다.

- 원본: `samples/task2097/75544_pii_bunseok.hwpx`, SHA `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`
- 독립 한컴2020 Print66쪽: `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, SHA `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`
- 선언 환경: `mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json`, SHA `21e5c5a0b081bf3101749e5b80c1b053fa3eeff4fcb1ba3f8aa33fd3e2a4d544`

## 원인과 제품 결과

`resolve_single_char_style`가 선언한 glyph 공급자는 paint 체인에 전달했지만 자연 전진의 명시
여부 벡터에는 값을 넣지 않았다. 기존 W3 양성 검사 중 휴먼명조U+2024만 char-miss/전각1000HU가
되었다. 각 언어 슬롯의 같은 환경 결정에서 이 값을 만들어 측정과 실제 후속 원점에 전달했다.
기본 자동 공급·내장 글꼴·HFT 폭 보호는 승격하지 않는다. 집계나 trace의 분류 기준은 바꾸지 않았다.

| 선언 환경 쪽 | Native before→after | fresh WASM before→after |
| --- | ---: | ---: |
| 17 | 98.35193→99.95382% | 98.41237→99.95563% |
| 40 | 97.72414→99.95671% | 97.81731→99.95802% |
| 65 | 98.69031→99.97348% | 98.75526→99.97486% |

192DPI/compat2022, canonical PDFium raw2px·90% 문턱을 유지했다. 별도 pdftoppm audit는
파일명으로 구분하며 canonical과 섞지 않았다. 전쪽 before/after8조건528쪽, 전후264쌍 중
6쌍 개선·258쌍PNG byte동일이다. 현재 보정으로0.1%p 이상 하락0쪽이다.
원본 소제목의 기호 직후 원점은340.8→약324.3CSSpx로 복원됐다. 독립 PDF는
기호240.60pt/후속243.24pt/기준선69.48pt(CSS320.80/324.32/92.64)다.

같은 핵심 경계를 가진17/40/65쪽 중 양쪽 raw 최솟값이 가장 높은65쪽을 선택해 review/overlay를
직접 판독했다. 새 회귀는 해당 쪽의 원본 문단·순서/누락/중복·한점 리더 한 번과 후속 원점 연결,
독립 원점/기준선±2CSSpx를 보호한다. 새 쪽수 검사는 추가하지 않았다. prototype은 기존 대시
1PASS+후속 원점1FAIL→2PASS이며, 정식 NativeSkia suite013도2PASS/247필터skip,
test0.188초·전체10.214초 exit0다. 회귀 선행 범위는 이 기호 변경의 완료 페이지이며 아래의
다른 표 미달을 이 검사로 면제하지 않는다. 기존 대시·폭 양성 검사를 지우거나 완화하지 않았다.
기존 집계 검사 자체도 frozen59에서1FAIL→frozen68에서1PASS다.
관련 기존4묶음은53PASS/959필터skip(test8.820초·전체293.693초), Native lib Clippy-D warnings
66.467초·fmt도 통과했다. test 추가 뒤prepare와실제 target013을 다시 계산했다.

## 남은 표 미달과 고정 base

현재 전66쪽 최저는 Native35.07344%, fresh WASM35.59924%로 이번 수정 전후 같다.
선언 환경은3/36쪽, 기본 환경은3/4/20/21/36쪽이90% 미만이다. 문서 전체 수용이 아니다.
고정 `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`의 Native도 같은 입력·PDF·글꼴 공급·DPI로
전66쪽×2환경을 실행했다. 최저 선언35.58967%(42쪽 미달), 기본35.55311%(47쪽 미달)다.
그와 비교한0.1%p 초과 하락은 5쪽/조건이며 `fixed-base-native-comparison.tsv`에 전쪽을 남겼다.
base의 기존 미달과 현재 하락을 구분하며 원본/검사를 이관·제거하지 않는다. 특히3쪽은 다음
분할 조각의 빈 물리 띠와 현재 닫힘의 소비를 추적 중이다. 아직 보정·완료 판정하지 않았다.

## 실제 공급과 범위

Native 빌드121.340초, Mac locked web/pkg/no-opt fresh WASM267.549초다. Docker 최적화가 아니다.
root pkg/Studio public·새 Chrome155.0.8059.84 초기 JS/WASM 응답을 frozen69와 대조했다.
첫 browser 시도는 추가 응답 body를 읽지 못해 실패 기록을 보존했고 durable network buffer로
새 실행을 검증했다. 문서 CK/actual36 수용을 주장하지 않으며 PII preflight50000 guard도 바꾸지 않았다.
canonical/audit의20개 완료 silhouette_manifest와 실제PNG1320쌍 hash·쪽수·누락/중복을 확인했다.
공급 경로·모든 원 로그/메타데이터는 ignored output의repair67~77에 남긴다.

전체 nextest의 최신 전체 결과는 이전9b1f의20FAIL 이력이다. 집중 복구/새 검사55PASS를 전체
통과로 재인용하지 않는다. SVG 부분집합·표 물리 공간·다른 필수 실패 및 전체/실제 소비 검증이
남아 있으며 PR은 최종 검증 보고와 새 사용자 승인 뒤 진행한다.

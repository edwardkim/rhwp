# 저장 줄 압축에서 원본 들여쓰기 보존

## 기준과 범위

`upstream/devel`의 #7913 `4662c069e557e928ed7968d061121fde90b1f027` 위로218개
로컬 커밋을 충돌 없이 리베이스했다. 후보 parent는 `2d7787f29af697c38a96ed5eac034d237fb64e6a`,
`paragraph_layout.rs`의 source diff SHA-256은
`307b1cafe6c2d885e2107091bd1aad9235d72e0b506ef70d7e98026d736da5a6`다.
사용자 초안 두 파일·기존 PDF·backup ref/stash는 보존했다. 이 단계는 들여쓰기 보정과
중요 위치의 회귀 근거이며 전체 PR 검증 완료를 뜻하지 않는다.

## 원인과 실제 소비 경로

PII 원본은 첫줄 앞 공백 두 칸 뒤의③과 다음 줄 첫 글자를 같은 x에 인쇄한다.
기존 코드는 저장 줄의 자연 폭이 가용 폭을 넘을 때만 원본 접두 영역의 고정을 해제했다.
그 결과 본문 압축이 들여쓰기에도 적용되어 첫 원점을 당겼다. 독립 Print의 두 원점은
각각97.919993/97.919993 CSSpx다. 수정 전 실제 트리는96.722667/98.013333이었다.

`LeadingSpacePlan → first_inline_pos/first_inline_offset → unjustified_leading_spaces_by_run →
정렬 분배 배열 → TextRun의 원점·extra_word_spacing → PageLayerTree/PaintOp → Native/Canvas`
연결에서 음수 정렬 여분에도 같은 원본 접두 영역을 보존한다. 남은 압축은 본문 배열이 소비한다.
TAC·필드·자동 탭이 소유한 접두 경계는 유지한다. 이미 사용하지 않는 선행 trailing_width 계산은
제거했다. 문서 ID·글꼴명·좌표 예외나 clamp를 넣지 않았다.

일반 접두 고정을 아예 없애던 중간 후보 C63은 PII를 고치지 못하고 다른 문서를 악화해 폐기했다.
선택한 규칙은 양수·음수에 같은 원본 들여쓰기 계약을 적용한다.

## 독립 입력과 전쪽 검증

| 문서 | 입력 SHA-256 | 독립 Print SHA-256 | 쪽수 |
| --- | --- | --- | ---: |
| PII 명시 글꼴 환경 | f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1 | 0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770 | 66 |
| 자산관리 | 97b5d6c571a6b7626321c6a53d797e3511978447497bb73309bd23eaa8e7ea77 | 99099b0eb4b829b87e3b55e37a2db9f5aae51008ac7b8b93283db6900b02aecb | 60 |
| 규제분석 | e68d903c20e89ed1325b3a5633c3c0606d21b042345b76a1785cad1e1eca8589 | 65bbfb5a1ae7c4a71774cda4c91156990edd8a41177b6509109ee08ec286f286 | 17 |

각각 `samples/task2097/75544_pii_bunseok.hwpx` / `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`,
`samples/issue6031/3249937_asset_management_rules.hwpx` / `pdf/3249937_asset_management_rules-2020.pdf`,
`samples/issue7445/80250_regulatory_analysis.hwp` / `pdf/80250_regulatory_analysis-hwp-2024.pdf`다.

canonical Visual Sweep,192DPI,Print profile,threshold32,2px 이웃 관용90%를 유지했다.
PII/자산관리는compat2022,규제분석은2024다. Native Skia PNG와 fresh WASM Canvas PNG를
직접 출력했다. PII는 기존 `pii5846-reference-font-environment.json`의 명시 환경이며
실제 기본 글꼴 선택의 수용으로 혼동하지 않는다. 다른 두 문서는 실제 기본 선택이다.
공급 글꼴은 앞 단계와 같고,독립 HFT 한글 공급 SHA
`7257416630313e6989c7690847f2c372a3d5106a2c215a6e16ee03573fe995dd`도 유지했다.

| 전쪽 문서 | Native 최저 | fresh WASM 최저 | 쪽수 일치 | 90% 미만 |
| --- | ---: | ---: | --- | ---: |
| PII 명시 환경 | 93.75247% | 93.75247% | 66/66 | 0 |
| 자산관리 | 91.79644% | 91.79276% | 60/60 | 0 |
| 규제분석 | 94.12693% | 94.07717% | 17/17 | 0 |
| 법원 서식 대조군 | 90.14068% | 90.15480% | 2/2 | 0 |
| 필드 대조군 | 100% | 100% | 1/1 | 0 |

원본 HWP29/104쪽,과거·현재 분할본177/178쪽 각각,HWPX4/5/104쪽,
float-stack-defer전2쪽,저장 되감김18/19쪽도 확인했다.
각 backend는11조건159쪽(전쪽5문서146쪽+경계13쪽)이고 모두90% 이상이다.
이는 기존 전체43조건 또는 전체 제출 범위의 완료가 아니다.
리베이스 직전 C65와 새로 빌드한 C66 Native의 같은159쪽은 점수와 PNG SHA가 모두 같다.
이 비교는 새 devel 기준 전쪽 하락 검증을 대신하지 않는다.

Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드로186.528초다.
Native SHA `de0c4f675efd4443178d34552c7bf0e51c50c68dc022c31bd1647fc17fbc38d9`,
JS SHA `6961f07addf370125d69f096e1dbcafdc9922424537815a14708b2b6309ec409`,
WASM SHA `84fb05759c780bafef263690b8153b7fab03861d0c05d4c4c72e1532264dca76`다.
root pkg/Studio public/동결 pkg hash를 확인했다. Docker 최적화 또는 실제 Studio 초기화 통과는 아니다.

## 중요한 쪽 선정과 정식 회귀

[review.png](review.png)는 Print/수정 전C62 Native/수정 후C66 Native/fresh WASM 순서다.
수정 전 그림은 C62 producer의 실제 재출력이며 새 기준의 전체 통과로 인용하지 않는다.
[PII 상세](pii-detail.png)와 [규제분석 상세](regulatory-detail.png)에서 번호·다음 글줄의
원점과 문장 흐름을 확대 판독했다.

같은 음수 들여쓰기 동작이 있던 쪽을 모두 조사하고 양쪽 점수의 최솟값으로 순위를 정했다.

| 문서 | 후보 쪽의 양쪽 최솟값(내림차순) | 선정 |
| --- | --- | --- |
| PII | 21:99.63145,23:99.52566,45:99.15636,44:99.14543,41:98.60535,22:98.58174,20:98.16097 | 21쪽③ x97.919993 CSSpx ±2 |
| 규제분석 | 6:99.96430,5:99.57508,4:98.94402 | 6쪽 목록 x416.800008 CSSpx ±2 |
| 자산관리 | 7:98.26808,3:97.76725,4:97.37462,8:96.54743,1:95.58246 | 7쪽 원점·내용 관계만 |

좌표는 PDF pt×96/72다. raster2px 실루엣 관용과 좌표 ±2CSSpx는 구분한다.
자산관리7쪽은 최고 점수 후보지만 [상세](asset-detail.png)의 ASCII 따옴표 윤곽이 Print와
달라 아직 시각 완료로 보지 않는다. 해당 px 핀은 추가하지 않고 원인 확인을 계속한다.

[정식 검사](../../../../../../../tests/cases/issue_7445_negative_indent_ownership.rs)는 원본 표/셀/문단·텍스트로
위치를 식별한다. PII 첫두 저장 글줄의 내용 소속·누락/중복·이어받기,자산관리 목록의 같은 원점·
두 문단 전체 내용,규제분석의 오른쪽 셀 소속·두 목록의 같은 원점·전체 내용 순서를 보호한다.
TextRun이 합쳐지거나 나뉘어도 실제 glyph 원점을 비교한다.

새 기준의 parent에서 이 보정만 빼고 빌드한 라이브러리로 **실제3FAIL → 같은 검사3PASS**를
확인했다. marker helper가 run 분할을 가정했던 초기 진단은 바로잡고 같은 의미 검사로
다시 실패·통과를 입증했다. 기존 관련7묶음19건도 통과했다.
manifest를 재생성한 정식 `regression_suite_003` 실행은3PASS/0FAIL,0.177초(231개 필터 제외)다.

## 증적과 남은 검증

ignored 정본은 `output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/`
`rebase-4662c069-20261016/`다. `rebase-result.json`, `range-diff.txt`,
`candidate-c66/{producer,wasm-producer}.json`, `candidate-c66-{native,wasm}-qualification/`의
전쪽 TSV/manifest/PNG, `prefix-semantic-proof-steps.json`과 before/after 로그를 보존했다.
raw TSV/JSON/log와 글꼴 바이너리는 커밋하지 않는다.

새 devel4662의 전쪽 Native/fresh WASM 기준 및 Decimal0.01%p 하락 비교,현재 전체 회귀와
나머지 lint/Skia/Studio/Gym·실제 소비 경로는 아직 남아 있다. PII 기본 글꼴 선택·법원 서식의
미세 차이·자산관리 따옴표를 완료로 표시하지 않는다. 과거10467PASS는 이 후보의 새 전체 통과가 아니다.

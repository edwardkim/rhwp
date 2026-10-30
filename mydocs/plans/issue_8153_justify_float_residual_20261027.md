# #8153: 양수 낱말 여백에서 발생한 수치 잔여와 글자 압축

## 원본·사양·독립 출력

대상은 기존 편람 HWP·HWPX와 같은 독립 2024 Print다. 원본을 수정하지 않는다.
HWP5 문단 모양의 정렬 속성 bit 2~4와 HWPX `align horizontal="JUSTIFY"`를 따른다.
136쪽의 해당 문단은 원본 문단 모양 247, 글자 모양 53, `condense=0`, 저장 줄 폭
38128HU(96dpi에서 508.37333px)다. 구역3의 표 안 문단이며 첫 줄의 순서와 다음 줄의
소유를 보존한다. `p136-raw-source-justification-contract.json`은 실제 XML에서 읽은 값이다.

source `08d87142c8`의 Native 실제 paint에서 낱말 확장은
`1.187878787878798px`인데 추가 자간은 `-0.062004066023816064px`다.
독립 Print의 `카카오톡` 시작은 x=585.59998px이고 현재는 x=582.80515px다.
136쪽은 두 원본 모두 28픽셀이 미대응이다. 현재 전쪽 Native는 717/766조건이 0픽셀이다.
이 단계는 남은 49조건과 원래 1,532조건의 목표를 축소하지 않는다.

## 기존 구현 추적

원 구현은 `b85f8533d28`(기존 작업 #2189)의 셀 오버플로우 보정이다. 공백 축소 한계에
도달해도 남는 실제 음수 여백을 글자 사이에 분배해 잘림을 막는 목적이었다. GitHub에서
연결된 PR은 릴리스 PR #2330이며 구현 근거는 해당 커밋과 기존
`tests/issue_2189_cell_text_clip.rs`다. 이어받은 `989d01fef3a`의 Frame admission,
`34a8b6e86f7`의 선행 공백 소유 보정도 대조했다. 실제 축소 한계가 작동한 경우의 보호는 유지한다.

현재 `compute_line_extra_spacing_with_dash_plan`은 먼저 `raw_ews=slack/spaces`,
`ews=max(raw_ews,min_ews)`를 구한다. 한계가 작동하지 않아도 곱셈 왕복의 잔여
`slack-ews*spaces`가 아주 작은 음수가 되면 셀 자간 수렴으로 진입한다.
실제 측정된 ews와 같은 나눗셈을 만드는 양수 사례는 다음과 같다.

- slack=13.066666666666777px, spaces=11
- raw_ews=1.187878787878798px, 한계는 음수이므로 작동하지 않음
- f64 잔여=-1.7763568394002505e-15px

이 잔여를 실제 초과로 취급한 뒤 런별 반올림 폭의 수렴까지 수행해 유의미한 음수 자간을
만든다. 픽셀 epsilon을 늘리거나 문서별 수치를 넣지 않는다. 실제 낱말 한계가 작동했는지
`raw_ews < min_ews`로 판정한다. 양수 여백의 글자 폭과 실제 음수 여백의 기존 보정을 함께 지킨다.

## 생산·소비 연결과 검증

같은 공통 함수의 반환 → 측정 style의 `extra_char_spacing` → 실제 방출 style의 같은 필드 →
최종 글자 위치·bbox다. 현재 연결은 `paragraph_layout.rs`의 2571·7481·8852줄이며,
수정 후 실제 위치도 다시 대조한다. backend별 좌표 덧셈이나 사후 clamp는 추가하지 않는다.

먼저 원본 136쪽의 같은 source Native·fresh WASM과 독립 Print를 고정한다.
실제 원점의 수정 전 FAIL/수정 후 PASS를 확보하고 글자·공백·다음 줄의 소유·내용을 확인한다.
기존 셀 압축·낱말 꼬리·머리말·혼합 글자 검사를 정상 대조로 실행한다. 이후 전쪽 Native와
fresh WASM에서 새 미대응 증가가 있는지 확인하며, 불확실하면 개선 완료로 기록하지 않는다.
최종 표준 최적화 WASM·전쪽 100%·필수 전체 검증·PR 준비가 남는다.

## source40 실제 검증 결과

`40d36c55993f327f9b1c3bc66b854750c7fa3fad`를 고정한 Native 전 766조건을 완료했다.
HWPX는 361/383조건, HWP는 358/383조건이 raw 100%·정수 미대응 0픽셀이다.
직전 source08의 717조건에서 719조건으로 늘었고, 미대응은 2,272 → 2,216픽셀이다.
미대응 증가 페이지는 0개이며 양쪽 최저 raw는 99.46283%다. 전체 목표 완료는 아니다.

같은 source의 fresh DEV WASM은 원본 18쪽씩과 독립 탭 대조군 3개, 총 39조건을 완료했다.
33조건이 100%·0픽셀이다. 166·228·289쪽의 남은 차이는 각 형식에서 별도로 남으며,
이를 성공 조건에 합산하지 않는다. 두 원본의 136쪽은 Native와 WASM 네 조건 모두
100%·0픽셀이다. 같은 backend의 두 형식 136쪽 PNG는 각각 바이트까지 같아 직접 판독을
공유했다. 최종 표준 최적화 WASM 검증과 전 1,532조건 완료를 뜻하지 않는다.

기존 탭·그림자·bbox·셀 압축·condense·혼합 글자·머리말과 실제 136쪽 검사 29건이 통과했다.
시스템 글꼴을 별도 준비하지 않는 공개 API 진단에서도 source08은 단어 원점 두 건 FAIL,
source40은 두 건 PASS였다. 원점은 582.80515 → 585.53333px이며 독립 Print는
585.59998px다. 원본 셀의 내용·공백·다음 줄 순서도 보존했다.

작업 증거의 공통 디렉터리는
`output/pr-review/issue8153-handbook-raw100-20261022/remaining-raw100-diagnostic/continuous-text-shadow/`다.

- 전쪽 Native: `positive-slack-native/native-whole/whole-result.json`과 각 형식의 `score/silhouette.tsv`.
- fresh DEV WASM: `positive-slack-wasm-dev/wasm-focus-result.json`과 각 형식의 `score/silhouette.tsv`.
- 실행 로그: `positive-slack-native-whole-run.log`, `positive-slack-wasm-focus.log`, `positive-slack-native-validation.log`.
- 136쪽 직접 판독: `positive-slack-native/native-focus/handbook-hwpx/p136/` 및
  `positive-slack-wasm-dev/wasm-focus/handbook-hwpx/p136/`의 review·overlay PNG.

정식 회귀 `tests/cases/issue_8153_positive_word_slack.rs`는 실제 셀·줄의 원문과 공백,
원본 불변, 독립 단어 원점·기준선 ±2px를 함께 보호한다. 같은 파일을 고정 source08의
라이브러리에 연결하면 2건 FAIL, 한자 보정을 함께 포함한 source `ffc7cf81b4`에서는
2건 PASS다. 시스템 글꼴 경로나 복사한 글꼴 fixture를 요구하지 않는다. 이 검사의
확인은 최종 자동 수집 suite와 전체 회귀 실행을 대신하지 않는다.

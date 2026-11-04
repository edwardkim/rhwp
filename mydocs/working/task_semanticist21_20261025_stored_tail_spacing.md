# 저장 본문 끝 간격과 후속 표 원점 보정

## 독립 근거와 원인

원본 `samples/exam_science.hwp`의 독립 기준은
`pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf` 전 4쪽이다.
Native193(`ed427895aa80aff7045242a85cd1bbe8afd328ca`)과 고정 devel은 4쪽이나,
Native195(`f7c8f7001fd944481ded1646708aea89aaafb11a`)부터 5쪽이다.
원 해결·통합 후 저장 TAC 프레임 소유 보정의 전후를 동일 입력으로 추적했다.

문단 42의 마지막 구성 높이는 18.2533333333px, 간격은 5.88px이다.
기준선 보정이 높이를 19HU 늘리고 간격을 같은 만큼 줄였으나, 후속 표의
원점 상속은 보정된 내용 하단에 원본 간격을 더해 19HU를 중복 계상했다.
문단 43의 저장 원점 935.7466666667px 대신936px를 강제한 차이가 다음 단으로
이어져 문항·보기 표 소속과 쪽수가 달라진다.

## 공통 결과의 생산과 소비

`FormattedParagraph.line_spacings` → `line_tail_spacing(end_line)` →
`ParagraphFragmentBounds`/`ParagraphFragment` → 실제 수용 조각의 기록 →
`previous_text_content_bottom` → `stored_tac::prepare`의 inherited origin으로 전달한다.
일반 전체 문단·분할 문단·host 본문·wrap tail은 같은 구성 간격을 사용한다.
내용 하단을 발행하지 않던 decoration/column 조각은 간격도 발행하지 않는다.
원본 저장 사다리는 소유·상대 간격의 유효성 확인에 유지한다. 문서별 상수나
clamp를 추가하지 않으며 후속 최대 원점 선택도 삭제하지 않는다.

원본과 구성 메트릭이 같을 때 `stored_content_bottom` 경로의 간격 차이는 0이다.
구성 메트릭이 달라지면 같은 구성 결과의 내용 높이와 간격 차이를 함께 소비한다.
보충 폭의 수집·소비 eligibility 병합 보정 5386de6e6a도 같은 후보에 포함한다.
실제 `supplemented` 경로는 준비된 전진폭을 소비하고 `em_exact`로 덮어쓰지 않는다.

## 검증 상태

수정 전 실제 trace와 원본·Print SHA, base/candidate 전쪽 결과는 ignored
`output/pr-review/semanticist21-20261019/science-flow-traces-3239-result.json` 및
`failed-input-comparison-3236-result.json`에 보존했다.
현재 변경의 Native208 빌드·실패 검사 재실행·전쪽 Native/fresh WASM 시각 검증은
아직 실행 전이다. 기존 기대값·baseline·쪽수 핀은 변경하지 않았다.
완료 판정은 새 source의 실제 실행 결과로 갱신한다.

## Native208의 실제 재검증

구현 커밋 5ae234786dbcd15e98a6cf35dd0c187af483efc7을 수정 없이 빌드했다.
공용 target/pr-review·잡 8개, 실제 exit 0·357.23초다. 원본 과학 시험지는
Print와4쪽이 일치하며 Native 전쪽 최저 96.42394%다. 2쪽의 새 review·overlay를
함께 직접 판독해 보기 표의 왼쪽 단 꼬리와 문항 10~12의 오른쪽 단 보존을 확인했다.
집중 8모듈 121검사는118 PASS·3 FAIL이며, 보충 측정의 이전 16실패·다단 4실패·
원본 과학 시험지 1실패는 기대값 변경 없이 모두 통과했다. 다른 실패 3개를 숨기지 않는다.
기존49조건 1772쪽 Native 재출력도 전체 90% 이상·독립 쪽수 일치다.
이전 전체 Rust의 79실패를 모두 해결했다는 뜻은 아니다.

실제 원장은 ignored output의 rebase-native208-build-3244-result.json,
rebase-focused-contracts-3246-result.json, failed-input-comparison-3247-result.json,
rebase-candidate-native208-3252-result.json과 native208-base-comparison-3262.json이다.
색상 보정 없는 raw를 썼고 같은 base의 페이지별 하락 두 진단 기준을 따로 기록했다.
최적화 fresh055는04d8c792ee의 Git archive에서 실행 중이다. 이는 구현5ae234786d에
독립 #7216 증적만 추가한 소스이며, 새 출력·최종 Rust/lint·PR 제출 판정은 남아 있다.

### 보충 실행의 종료 결과

fresh055의 표준 release·wasm-opt -O가 1278.83초, exit 0으로 끝났다.
과학 원본의 Native/fresh 최저는96.42394%/96.32059%, 전체 4쪽 일치다.
2쪽 review·overlay를 양쪽에서 직접 확인했고 정식 증적은
mydocs/pr/assets/science_stored_tail_20261025에 보존했다.
수정한 #1937/#4138/#6184/#6624의4모듈 23검사도23 PASS다.
차트 crate 전체 165검사와 -D warnings Clippy가 exit 0으로 종료했다.
이는 기존 전체 79실패 중 공통 구현 보정 21건과 검사 계약 보정 15건의 재검증이다.
기대값을 바꾸지 않은 21건과 독립 Print·원본 단위로 재검토한 15건을 구분한다.
전체 Rust의 최종 실패 수는 전체 재실행 전까지 확정하지 않는다.

# 빈 부동 개체 호스트의 글자 줄 점유 정정

## 독립 근거와 원인

현재 작업 범위의 #5929 원본만 사용한다. 원 해결 `0c2f1cfe3`, 후속 PR #6193,
통합 PR #6209/`254d7263`의 줄 점유 계약을 먼저 확인했다. 같은 현재 원본을 한컴 2020
11.0.0.9136의 PrintMethod 0으로 새로 출력한 PDF는 그림 아래에 간격을 둔 표를 그린다.
코드가 없는 빈 스크립트만 제거한 전처리와 원본/PDF SHA는 아래 3708 자료에 있다.

제품98c2의 원점 추적3729에서 조판 문단8은 줄 높이/간격0+0, 문단 앞뒤 간격20px만
점유했다. 배치는 같은 빈 앵커의 글자 줄34.1333px를 소비해 뒤 흐름이 갈라졌다.
그 뒤 공통 표 배치 계획이 조판 원점에 외곽 여백을 더한618.2px를 발행하여, 배치가
구한 그림 회피 원점보다 위에 표를 그렸다. 표 하단 clamp로 숨길 문제가 아니다.

## 공통 결과와 적용 범위

`reflow_float_host_line::neutral_anchor_metrics`는 잉크 없는 부동 개체 앵커 중 유효한
저장 LineSeg가 없는 경우의 글꼴 줄 메트릭을 계산한다. 개체는 여전히 별도 Shape가
소유하며 글자 줄 높이를 개체 높이로 바꾸지 않는다. typeset 빈 문단 폴백,
HeightMeasurer, paragraph layout이 같은 결과를 소비한다. 앞뒤 간격은 기존 문단
포맷이 한 번만 소비한다.

- 적용: 빈 텍스트, 비-TAC 흐름 중립 개체/마커만 있는 문단, 실제 저장 줄 없음.
- 비적용: 유효 저장 줄/수평 안내 줄, 보이는 텍스트, TAC·자리차지 개체.
- HWP3의 기존 작은 빈 글꼴 cap을 유지한다. 문서 ID나 특정 문단 번호로 분기하지 않는다.

생산한 글자 줄 높이 → `format_paragraph_for_flow`의 줄 점유/현재 높이 →
`place_block_table`이 기록하는 표 배치 원점/점유 끝 → `layout_table`의 확정 원점
소비를 대조한다. 단순히 공통 helper를 호출했다는 사실만으로 완료 판정하지 않는다.

## 검증 계획과 결과

보정 전 원본은 Native95.09370% / fresh95.08965%이지만 그림과 표가 겹친다.
점수 통과로 이 배치 차이를 면제하지 않는다. 같은 입력/PDF의 보정 후 전쪽 TSV와
review/overlay를 확인하고, 기존 두 정확한 검사는 유지한다. 저장 줄·다른 배치 정책과
현재 셀 그리드/부모 프레임 대조군도 함께 확인한다. 새 회귀·기준값 변경은 전쪽90%
선행 조건과 직접 판독을 완료한 뒤 수행한다. 현재는 구현/검증 중이며 완료 판정이 아니다.

같은 소비 경로 점검에서 HeightMeasurer의 다른 불일치도 확인했다. 폴백이 줄 배열을
교체하기 **전**의 합을 `lines_total`에 쓰고 있었다. 최종 배열을 만든 뒤 합계를 계산하게
바꾸어, 셀의 빈 글자 줄이400HU 안내 높이에 남는 경우도 제거한다. TAC 표 별도 축이나
저장 사다리의 수용 조건은 바꾸지 않는다. 이 합계 변경 전의 Native230 결과와 변경 후
최종 Native231 결과를 구분해 기록한다.

## 현재까지 확인한 결과

증적 루트는 `output/pr-review/semanticist21-20261019/`다.

| 검사 | 실제 결과 | 증적 |
| --- | --- | --- |
| #5929 Native 231, 원본 전체 1쪽 | raw 99.94742%, 그림 아래 표 배치와 간격을 직접 판독 | `issue5929-native231-3739/issue5929-native231/` |
| 기존 정확한 #5929 검사 2건 | 공식 nextest 2 PASS, 0 FAIL | `logs/issue5929-official-3742.log` |
| 셀 그리드·부모 프레임·정상 대조군 8조건 | Native 231 모두 1쪽, raw 100% | `cell-frame-native231-3738-result.json` |
| 기존 영향 검사 70건 | 동결 Native 231에서 59 PASS, 11 FAIL, 0 ignored | `native231-focused-3737-result.json` |
| Rust 형식 검사 | `cargo fmt --all -- --check` 종료 코드 0 | `logs/empty-float-host-format-3741.log` |

그림 하단 627.9px와 표 상단 652.3px의 관계는 독립 PDF와 일치한다. 보정 전 표 상단은
618.2px여서 그림과 겹쳤다. 좌표는 진단 관측값이며 새 픽셀 기대값으로 고정하지 않았다.
괘선 두께와 일부 글자 잉크 차이는 이미지에서 구분되므로 완전한 픽셀 일치로 보고하지 않는다.

원본 입력은 `samples/issue5929/table_below_square_pic.hwpx`, SHA-256은
`1b4727d65fcd85bea3fa4e299b32acd6577a612458ca945be582476957c5fd03`이다.
기준은 `mcp-issue5929-current-3708/print/table_below_square_pic-2020.pdf`, SHA-256은
`0e412a15afb1d647a7f772ad8b7b777f97332bdbba1556ad155497a750add58d`다.
Native 231의 실제 명령·소스 해시·실행 파일 해시는 `rebase-native231-build-3734-result.json`,
시각 명령과 결과는 `issue5929-native231-3739/summary.json`을 따른다.

남은 실패는 같은 검사·입력·환경에서 고정 devel과 후보를 실행해 분리 판정한다.
기존 검사에 픽셀 고정 자격이 있는지는 devel의 해당 페이지에서 Native/fresh WASM raw가
각각 98% 이상인지 확인한다. 현재 출력만 높거나 다른 페이지가 낮다는 이유로 검사 의미를
바꾸거나 제거하지 않는다. 관계·내용 소유 검사와 절대 픽셀 핀도 구분한다.

후속 fresh WASM 066은 소스 커밋 `43e2697fe76e495fd6363123b3021270eac54cc7`에서
표준 release와 실제 `wasm-opt -O`를 거쳐 생성했다. 빌드 종료 코드 0, 소스 변경 없음과
패키지 해시를 `rebase-candidate-wasm066-build-3766-result.json`에서 확인했다.
#5929 전 1쪽 raw는 **99.94593%**, 셀 그리드·부모 프레임 대조군 8조건은 모두 **100%**다.
`cell-frame-wasm066-3774-result.json`에 입력·PDF SHA와 실제 명령·TSV를 보존했다.
`common-frame-wasm066-3774/table_below_square_pic-wasm066/`의 review와 standalone overlay를
직접 판독해 그림 아래 표 배치·간격과 본문 보존을 확인했다. 괘선 잉크의 두께 차이까지
완전히 일치한다고 주장하지 않는다. Native 231과 같은 범위의 배치 보정을 fresh에서도 확인했다.

남은 실패 판정과 최종 전체 검증은 완료되지 않았다. 위 결과는 해당 범위의 검증이며
전체 회귀 통과나 PR 준비 완료를 의미하지 않는다.

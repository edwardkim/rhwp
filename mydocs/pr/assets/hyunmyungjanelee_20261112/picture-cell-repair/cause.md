# 그림 전용 셀의 행 높이와 출력 프레임 보정

사용자 지시에 따라 미공개 문서를 대신할 공개 합성 입력을 만들었다. 미공개 원본 3건을 확보하거나 직접 검증한 결과는 아니다. 공개 `samples/issue8576/cell-square-six-pictures-35x25.hwpx`의 그림·서식 구조를 재사용하고 4행 표 및 쪽 경계의 20행 표를 구성했다. 각 host를 빈 문단·공백 7개·글자 X로 바꾸되 셀 내용과 그림 크기·안 여백은 유지했다. 모든 저장 LINE_SEG를 제거했다.

같은 각 입력을 한컴 2020 11.0.0.9136, Print method 0, one-up 경로로 출력했다. 여섯 입력과 PDF의 해시·작업 ID는 `independent-print-receipts.json`에 기록했다.

## 원인과 실제 보정 전 결과

고정 devel `80b1c354`과 통합 코드 `0c9819d6`을 같은 입력으로 대조했다. 4행 공백 host의 독립 Print 뒤 본문 y는 336.13px, devel은 336.2px, 통합 후보는 321.1px다. 후보는 행 합을 175.0px에서 160.0px로 압축했다. 그림 자체 40px에 위·아래 안 여백을 더한 높이는 각 행 안에 필요하지만, 기존 선언 높이 fit의 내용 하한은 안 여백을 뺀 높이만 읽는다. 새 공백 host가 글자 host 경로로 들어가자 이 기존 fit-down 문제가 드러났다. 글자 X host에도 같은 문제가 있다.

20행 공백 host에서 독립 Print와 devel은 2쪽, 통합 후보는 1쪽이다. 독립 Print의 그림 소속은 1쪽 19개·2쪽 1개이며 표 뒤 본문과 END는 2쪽에 한 번씩 놓인다. 따라서 쪽수 핀만 바꾸지 않고 그림 포함·안 여백·행 프레임·내용의 단일 소속을 함께 검사한다. 최소 글꼴 환경에서 새 관계 검사 3건의 실제 보정 전 결과는 0 PASS / 3 FAIL이다.

## 보정 범위

`fit_measured_table_to_declared_height_with_outcome_with_storage`의 내용 하한에서 저장 줄 없는 그림 전용 셀의 물리 안 여백을 보존한다. 글자처럼 취급하지 않는 Square 그림이 본문 흐름을 따르는 경우에 한정한다. 저장 줄을 가진 표의 기존 패딩 회수 및 원본 프레임 보정은 유지한다. 같은 MeasuredTable을 예약·쪽 나눔·paint가 소비하도록 공통 fit에서 보정한다.

보정 후 검사 및 Native/fresh WASM 전쪽 출력은 진행 중이며, 이 문서는 완료 판정을 대신하지 않는다.

## 첫 보정 뒤 남은 쪽 경계 검사

그림 내용 하한에 실제 안 여백을 포함한 첫 보정은 3건 중 2건을 통과했다. 그러나 20행 공백 host는 2쪽을 만들면서도 그림 20개를 첫 쪽에 남겼다. 기존 진단 경로 `RHWP_TABLE_DRIFT`와 `RHWP_DIAG_SPLITSCAN`으로 원인을 확인했다. 행 합 875.2px·host 간격 15.5px·전체 점유 890.7px에 비해 본문 예산은 876.8px였다. 통째 표 판정은 `declared=true`로 수용했고, 그 뒤 본문 아래에서 표가 2.1px, host 줄이 19.3px 넘쳤다.

원인은 `near_measured_rowbreak_fits`가 셀 행만 비교하고 같은 개체의 host·바깥 여백을 뺀 것이다. 저장 프레임 없는 그림 셀에서 같은 물리 소유 질의를 공유해 전체 점유 `table_total`을 비교하도록 추가 보정한다. 기존 저장 표의 2px 반올림 허용값을 바꾸지 않는다. 첫 보정 검사 실패와 진단 기록은 보존하며, 두 번째 보정의 같은 3건은 실제 3 PASS / 0 FAIL(0.034초)다. 기존 공개 사례와 저장 표 대조군 검사는 이어서 진행한다.


## 첫 opening의 원점과 이어받은 조각의 종료 소비

세 번째 Native 검사에서 첫 쪽의 위 바깥 여백 283HU(약 3.77px)를 실제 괘선 원점까지 전달했다. 첫 쪽은 21.85%에서 97.18%로 복원됐지만, 공백·글자 host의 끝 조각 뒤 본문이 각각 86.39%·86.14%로 미달했다. 같은 host의 짧은 표와 이어받은 표의 `본문 윗변 - 표 하단`을 대조하면 빈 host는 반올림 범위에서 같고, 공백·글자 host는 약 3.5px 늘었다.

원인은 첫 opening에 필요한 NextLine 배치를 continuation에도 전달한 것이다. 이어받은 조각의 기존 배치는 이미 위 여백을 소비하지만 새 점유 끝을 추가하면 후속 host 줄에서 이를 한 번 더 사용한다. 원점 보완은 저장 줄 없는 단일 흐름 그림 표의 첫 opening에 적용하고, continuation의 기존 소비는 유지한다. 원래 빈 host의 조각 배치는 변경하지 않는다. 새 `continuation_preserves_the_same_host_gap_after_the_last_picture_row`는 절대 위치 대신 같은 원본 host와 여백의 종료 간격을 대조한다. 보정 전 실제 라이브러리에 연결한 네 검사는 3 PASS / 1 FAIL이며, 수정 코드 재검증은 진행 중이다.

## 최신 공식 기준의 재검증 상태

위 2020 Print 수치는 초기 원인 분리 이력이다. 사용자 지시에 따라 같은 6종을 저장 제품에 맞는 MCP 2024 Print로 다시 출력하고 공식 경로·새 SHA를 `../mcp-print-reference/receipts.json`에 보존했다. 현재 code head `3532e09de2`에 대응하는 Native 전 9쪽의 최저값은 97.18%이며 미달은 없다. 각 짧은 표는 100%, 쪽 경계 그림의 소속과 뒤 본문도 복원됐다. 1쪽의 기존 빈 종료 띠 차이는 같은 devel 출력에도 있으며 새 회귀와 구분한다. continuation 관계 검사 4건을 포함한 최소 글꼴 집중 125건은 전부 통과했다. fresh WASM 및 최종 전체 회귀는 진행 중이다.

## 최종 실행과 현재 리베이스의 재사용 범위

위 단계별 실패·중간 수치는 원인 분리 이력이다. 검증 source `30fa7739f0340f2a248b22542c7dd36d9354f056`에서 다음 결과를 확인했다. #8897 공개 22종·28쪽의 Native/fresh WASM 최저는 모두 97.18%이며 새 하락은 없다. 그림 전용 칸 경계·continuation을 포함한 해당 PR 집중 22건이 통과했다.

최소 글꼴 전체 13,365 PASS / 0 FAIL / 45 skipped와 Native 집중 141 PASS 및 필수 16단계 게이트를 완료했다. [전체 게이트](../final-rebase-30fa7739/gates-validation.json), [집중 검사](../final-rebase-30fa7739/focused-validation.json), [Native 전쪽](../final-rebase-30fa7739/native-validation.json), [fresh WASM 전쪽](../final-rebase-30fa7739/wasm-validation.json), [실제 backend 지원 범위](../final-rebase-30fa7739/actual-backends/summary.json)를 구분한다. RawSvg 차트의 강제 CanvasKit 진단은 기존 미지원이므로 지원 통과로 세지 않는다.

현재 `73d06ed0`은 새 base `a85d3265`의 #9017 HTML 붙여넣기를 반영했다. 기존 조판·입력·PDF·캡처 경로가 동일해 이 시각 증적을 원 producer와 함께 재사용한다. [동일성 감사와 추가 검증 경계](../final-rebase-30fa7739/rebase-a85d3265/README.md)를 따르며, 이전 source의 전체 실행 결과를 새 후보의 실행 결과로 고쳐 쓰지 않는다.

현재 제품 source `73d06ed0`에서도 최소 글꼴 Native 집중 164 PASS, 전체 13,380 PASS / 0 FAIL / 45 skipped(실행 404.945초), 최신 WASM·실제 브라우저 6사례/24회 저장/API11건 및 필수 21단계를 모두 통과했다. [현재 실행 원장](../final-rebase-30fa7739/rebase-a85d3265/gates-validation.json)과 [현재 기능 보존](../final-rebase-30fa7739/rebase-a85d3265/incoming-base-validation.json)을 기존 시각 producer의 자료와 구분한다.

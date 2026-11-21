# #7749·#7750·#7753 원본과 독립 Print 대조 계획

검증 기준은 `upstream/devel`의 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f`다.
작업 브랜치는 `task/7749-protected-document-fidelity`다. 각 입력의 전체 페이지를 Native Skia와
최적화 fresh WASM에서 독립 한컴 Print와 96 DPI로 대조한다. 두 백엔드의 페이지별 raw 실루엣
일치율은 98% 이상을 목표로 하고, 쪽수·순서·내용 소속·누락을 별도로 확인한다.

| 이슈 | 입력과 기준 | 확인할 경계 |
| --- | --- | --- |
| #7749 | `samples/issue3637/regulatory_impact_nested_table_escape.hwpx`와 `samples/issue6697/80550-agricultural-machinery-act-amendment.hwpx`는 SHA-256 `e7b147f7cea66c97bed79085a3d89c2656037e0f711232f659ed3c7344984f62`로 일치한다. 두 경로에 각각 새 한컴 2024 Print와 기존 한컴 2020 Print를 공급한다. | 전 31쪽, 25쪽 표의 하단, 26·27쪽 임금·사업체 조사 내용 소속, 29~31쪽 중첩 표와 캡션·말미 문단. |
| #7750 | `samples/hwpctl_ParameterSetID_Item_v1.2.hwp`와 새 한컴 2024 Print 전 74쪽을 비교한다. | 쪽수와 본문·연속 표의 소속, 표의 행 분할, 아이콘 줄과 저장 줄 사다리. |
| #7753 | 간장 기증자 정책연구의 HWP와 HWPX를 각 형식의 독립 한컴 2024 Print 전 215쪽과 대조한다. | 그림 셀과 캡션, 66·67쪽 표와 각주 이어받기, 말미 내용 소속. |

#7749의 과거 분리는 동일 SHA의 두 원본을 각각 다룬 검토 기록에 근거한다.
[중첩 표와 내용 소속](../pr/assets/issue7445/regulatory3637_blocking_scope_validation.json) ·
[80550 호스트 캡션](../pr/assets/issue7445/host6697_blocking_scope_validation.json).

#7750의 이전 차단은 쪽수 검사까지 포함했다. #7753은 기존 #7505의 분할 표·각주 보정 및
#7538의 큰 문서 후속 검증을 현재 기준의 실측과 대조한다. 원본별 보류 회귀 범위와
유효한 기존 검사는 보정 대상 원인의 대조를 마친 뒤 판정한다.
[ParameterSet 보류 범위](../pr/assets/issue7445/parameter_control_blocking_scope_validation.json) ·
[정책연구 HWP·HWPX 보류 범위](../pr/assets/issue7445/policy_report_blocking_scope_validation.json) ·
[후속 비교 PR #7538](../pr/archives/pr_7538_review.md).

새 인쇄 두 건은 모두 한컴 2024 `13.0.0.3901`의 `PrintMethod=0`,
`frame_print_to_pdf_ex_one_up` 실행이다. #7749의 job은
`94d66329-3999-49a1-b2d5-64bd08a638a5`, #7750의 job은
`da701e83-436f-4b1d-874d-cd0ccf2f4fa9`다. 입력·PDF SHA와 전체 쪽수는 실행별로 고정하고,
저장 줄과 실제 배치가 갈리는 쪽을 비교해 공통 조판 경계에서 원인을 분석한다.

# 저장 줄 없는 도형 문단의 중앙 정렬 보정

원 PR #8920의 `01d09b21b1c23aa918838fbfc6eedc409b3da82f`(통합 `77c42dec4d`)은 쪽에 걸친 표의 온전한 행과 병합 칸에서 사라지던 직선을 복구했다. 새 Windows 한컴 2020 MCP Print와 대조하자 병합 칸 가운데 정렬은 1쪽 87.91%, 2쪽 96.78%였고, 직선이 아래로 내려가는 기존 잔여가 확인됐다.

원본은 저장 줄·글자 없이 non-TAC 도형만 가진 문단이다. 글자 모양은 1000HU, 도형 오프셋 600HU, 높이 1HU다. 합성 안내 줄 400HU와 도형 범위 601HU만으로 정렬 높이를 정해 원본 글자 줄 1000HU의 상자가 사라졌다. `TopAndBottom` 도형은 기존 시각 bottom 질의에서 빠지므로, object bottom이 양수일 때만 보정하면 그 경로에는 적용되지 않았다(V5 실제 집중 검사 124 PASS / 1 FAIL).

공통 문단 질의가 해당 형상의 원본 글자 줄을 반환하고, 높이 측정·배치 양쪽은 개체 범위와 그 줄 상자 중 큰 범위를 한 번만 소비한다. 저장 줄·실제 글자·그림·중첩 표·TAC 도형을 가진 문단은 기존 경로를 유지한다. 좌표 상수나 특정 문서 ID, PDF 수치로 위치를 보정하지 않는다.

새 관계 검사는 원본 글자 크기·안 여백·도형 오프셋 및 실제 칸 상자에서 기대 앵커를 계산하고, 일반 칸과 두 행 병합 칸의 모든 온전한 행을 대조한다. 보정 전 실제 4 PASS / 1 FAIL, 보정 후 관련 집중 125 PASS / 0 FAIL이다. `TopAndBottom`·글앞, 위·가운데 4종의 Native 전 8쪽은 같은 새 MCP PDF로 100%다. TSV·같은 쪽 review/overlay와 생산 SHA는 `native-validation.json`에 연결했다. 최신 `8fe7d1fb8b`의 Native 집중 검사는 126 PASS / 0 FAIL이다. 같은 소스의 fresh WASM도 같은 Print 기준 4종 전 8쪽이 100%이며 [직접 판독](../final-print-8fe7d1fb/wasm-direct-reading.json)에서 병합 칸 화살표의 가운데 정렬을 확인했다. 최종 전체 회귀·게이트는 별도 기록으로 연결한다.

## 최종 실행과 현재 리베이스의 재사용 범위

위 단계별 실패·중간 수치는 원인 분리 이력이다. 검증 source `30fa7739f0340f2a248b22542c7dd36d9354f056`에서 다음 결과를 확인했다. #8920의 위/가운데·일반/병합 칸 4종·8쪽은 Native/fresh WASM 모두 100.00%이고 해당 PR 집중 5건이 통과했다.

최소 글꼴 전체 13,365 PASS / 0 FAIL / 45 skipped와 Native 집중 141 PASS 및 필수 16단계 게이트를 완료했다. [전체 게이트](../final-rebase-30fa7739/gates-validation.json), [집중 검사](../final-rebase-30fa7739/focused-validation.json), [Native 전쪽](../final-rebase-30fa7739/native-validation.json), [fresh WASM 전쪽](../final-rebase-30fa7739/wasm-validation.json), [실제 backend 지원 범위](../final-rebase-30fa7739/actual-backends/summary.json)를 구분한다. RawSvg 차트의 강제 CanvasKit 진단은 기존 미지원이므로 지원 통과로 세지 않는다.

현재 `73d06ed0`은 새 base `a85d3265`의 #9017 HTML 붙여넣기를 반영했다. 기존 조판·입력·PDF·캡처 경로가 동일해 이 시각 증적을 원 producer와 함께 재사용한다. [동일성 감사와 추가 검증 경계](../final-rebase-30fa7739/rebase-a85d3265/README.md)를 따르며, 이전 source의 전체 실행 결과를 새 후보의 실행 결과로 고쳐 쓰지 않는다.

현재 제품 source `73d06ed0`에서도 최소 글꼴 Native 집중 164 PASS, 전체 13,380 PASS / 0 FAIL / 45 skipped(실행 404.945초), 최신 WASM·실제 브라우저 6사례/24회 저장/API11건 및 필수 21단계를 모두 통과했다. [현재 실행 원장](../final-rebase-30fa7739/rebase-a85d3265/gates-validation.json)과 [현재 기능 보존](../final-rebase-30fa7739/rebase-a85d3265/incoming-base-validation.json)을 기존 시각 producer의 자료와 구분한다.

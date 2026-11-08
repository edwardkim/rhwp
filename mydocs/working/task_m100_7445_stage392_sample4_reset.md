# #7445 sample4 저장 본문 조각의 쪽 소속 보정

## 사전 분석과 독립 근거

- 이전 단계의 한컴 2020 Print 36쪽에서 문단 1242의 마지막 두 줄은 36쪽 첫머리에 있으며, 현재 Native는 이를 35쪽에 둔다. 저장 LineSeg도 같은 경계에서 63900 → 0 → 1500으로 재시작한다.
- 앞 조각의 첫 저장 원점은 현재 조판 흐름과 일치한다. 줄 구성·편집 여부와 양수로 증가하는 앞 조각의 사다리를 함께 확인할 수 있다.
- `prepare_forced_page_boundary`는 형식에 관계없이 저장 경계와 흐름을 조회하지만, `stored_body_reset_fragment_matches_current_flow`는 HWP3 유래 문단에 뒤 조각 그림이 있어야 한다고 다시 요구한다. 그림이 없는 정상 본문까지 이 이유로 거절한다.
- 일반 fallback의 음수 cursor 규칙을 완화하지 않는다. 기존 저장 조각 검증에서 문서 계보 때문에 그림 존재를 추가 요구한 조건을 제거한다. 원문 ID·고정 수치로 분기하지 않는다.

## 생산 결과와 소비 경로

- 저장 줄 사다리 + 현재 쪽 흐름 → `stored_body_reset_fragment_matches_current_flow` → `prepare_forced_page_boundary`의 강제 경계 → `decide_whole_fit`의 전체 배치 차단 → 줄 스캔/부분 문단 배치 → renderer가 같은 시작·끝 줄 소속을 소비한다.
- 후보는 기존 유효성·단일 단·저장 구성·원점 일치·양수 증가·정확한 0 재시작 조건을 유지한다. 편집/합성 줄, 흐름 불일치, 자리차지·어울림 개체는 기존 검증에서 거절한다.
- 기대값·baseline·기존 검사 함수는 변경하지 않는다. 새 회귀를 추가하지 않는다.

## 검증 계획

- 먼저 기존 body-overflow partition 10과 HWP3 변환/저장 경계 대조군을 실행한다.
- 같은 독립 Print PDF·글꼴 공급·compat 2022에서 Native/fresh WASM 전체 36쪽 TSV와 35·36쪽 확대 PNG를 대조한다.
- 이 문서는 base에서 이미 29쪽이 90% 미만이다. 경계 수정 통과를 전체 fidelity나 PR 준비 완료로 기록하지 않는다. 기존 검사의 변경·이관 여부는 별도 시각 근거로 판단한다.

## 전체 lint에서 발견한 이전 정리의 잔존 오류

- Native·WASM32 Clippy와 workspace build는 통과했지만, workspace all-target Clippy는 `split_caret_axis.rs`의 단일 항목 반복문으로 실패했다.
- stage389에서 시험지 세 항목을 제외한 뒤 정상 신문 문서 한 항목만 남았는데 반복문 형태를 그대로 두었다. 반복문을 단일 지역 바인딩으로 정리하고 문서 입력·분할·글자 보존 assertion은 유지한다.
- 이 수정은 생산 렌더링을 바꾸지 않으므로 앞서 만든 Native/fresh WASM 출력과 빌드 근거를 그대로 유지한다. 변경된 기존 검사와 실패한 lint를 다시 실행한다.

## 실제 결과와 잔여

- 기존 body-overflow partition 10: FAIL → PASS. 해당 문서의 공차 2px 넘침은 6 → 1건(기존 기준 4건)이다. 새 쪽 경계로 내용이 이동하며 전체 36쪽은 유지된다.
- 영향 focused 17건: 16 PASS / 1 FAIL. 실패한 `issue_1692`는 수정 전후 동일한 본문 y=337 / 대조군 y=339.7이며 이번에 해결된 것으로 기록하지 않는다.
- 이전 시험지 제외의 단일 항목 반복문 정리 후 기존 `split_caret_axis` 10건 모두 PASS. 새 검사·기대값 변경은 없다.
- Native/fresh WASM 전 36쪽: 모두 최저 56.62352%, 미달 22쪽. 35쪽은 88.04609 → 89.11271%, 36쪽은 49.74763 → 99.89458%다. Native만으로 WASM 결과를 대체하지 않고 새 pkg를 빌드하여 실제 Chrome 출력을 확인했다.
- 내보낸 문단 그룹 1261개의 공백 제외 내용은 수정 전후 동일하다. 36쪽 첫 두 줄과 그 뒤 참고문헌의 소속·순서를 review PNG에서 직접 확인했다.
- Mac no-opt fresh WASM build PASS, pkg/public의 JS·WASM hash 동일. Docker 최적화 빌드 통과로 보고하지 않는다.
- fmt, Native Clippy, WASM32 Clippy, workspace build, 현재 harness의 workspace all-target Clippy와 base SHA 고정 manifest·unit 정책이 통과했다.
- 단일 항목 반복문 변경 후 harness 배정 drift가 확인되어 `--prepare`를 재실행했다. 중단된 lint-attempt2와 stale harness의 attempt3을 완료 증거로 사용하지 않고 attempt4 결과를 기록한다. generated harness는 커밋하지 않는다.
- 35쪽 배경/캡션과 다른 미달 쪽의 보정은 아직 남았다. 전체 문서 90%나 최종 전체 회귀 통과·PR 준비 완료로 판정하지 않는다.

## 증거

- 요약: `mydocs/pr/assets/issue7445/completed-20261009/sample4-stored-reset-validation.json`.
- Native/fresh 35·36쪽 PNG: 같은 디렉터리의 `sample4-reset-<backend>-p<page>-review.png`.
- 실행 정본과 전체 TSV: ignored output의 `stage392-sample4-reset/` 및 `lint-attempt4/`.
- `rhwp-studio/public/rhwp.js`에는 빌드가 기존 Rust 글꼴 API 두 메서드를 동기화한 별도 변경이 남아 있다. 이 경계 보정 커밋에 섞지 않고 별도로 검증·반영한다.

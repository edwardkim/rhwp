# 최신 devel 반영 후 보존 검증 계획

이전 기준 `170b886f167b705b6e630c5b34bb638d92fae0a9`에서 시작한 검토 도중 #8659가 merge되어 기준을 `5716a226b562b7300193dd0b573ded9cd71ac890`으로 갱신했다. 이전 전체 검증은 끝나기 전에 중지했고 이전 완료 4개 lint를 새 후보의 통과로 재사용하지 않는다.

새 로컬 통합 head는 `3cdc91947ef8bf888e6c80896a0fb5f414279188`이다. merge-tree와 실제 merge 모두 충돌이 없으며 원 PR 5개 head가 그대로 조상으로 남는다. 새 base와 겹친 직접 파일 `src/serializer/control.rs`의 변경은 겹침 글자 표지 저장이고, #8656의 그림 꼬리 기록과 다른 함수다. Studio source·Cargo 입력·crate/vendor는 새 base와 같다.

공용 target에서 새 base Native → 후보 Native → 후보 optimized WASM을 순차 빌드한다. 같은 원 사례 6개·경계 2개·이전 정상 2개를 저장 버전별 Print와 다시 비교한다. 기존 한컴 Print PDF는 입력 바이트가 같고 실제 Print 출처가 확인된 자료로 재사용한다. 바뀐 base를 예전 TSV의 source로 바꾸지 않는다.

#8659의 18개 원 PR은 공개 실제 API 12그룹의 저장·재열기와 formal 회귀, 원 주장별 주요 18개 입력의 Native base/후보·fresh Canvas 전쪽 비교로 보존을 확인한다. #8537은 원 input/PDF가 #8536과 동일 SHA인 공유 근거를 사용한다. #8616은 겹침과 덧말을 둘 다 포함한다. 바뀐 공통 helper와 실제 소비 경로가 없는 Studio 전용 동작은 source 동일성 및 기존 실제 API 계약과 구분한다.

최종 전체 nextest·Native 공식 3종·모든 Rust lint·새 sample 보안·383쪽 편람을 이 후보에서 완료한다. 코드가 바뀌면 영향받는 경계와 시각·예산 검증을 다시 실행한다. 이 문서는 완료나 원격 merge 승인이 아니다.

추가 경계 점검에서 `i32::MAX` 오프셋에 양수 바깥 여백을 더하면 i32 합계가 감싸져 양수 가로 좌표가 `-28632994.4px`가 되는 것을 실제 render-tree로 재현했다. 해당 합계를 정확히 표현할 수 있는 f64에서 계산하도록 보정한다. 유효한 보통 좌표의 연산 순서는 유지하며 clamp·포화·문서별 예외를 사용하지 않는다. 합성 최대 오프셋 입력은 정수 경계 진단이며 한컴 조판 일치 기준으로 등록하지 않는다. 최종 실제 출력에서 부호 보존을 재검증하고 원 사례의 전쪽 Native/fresh WASM을 다시 측정한다.

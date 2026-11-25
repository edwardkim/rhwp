# #9318 병합 후 리베이스와 재빌드 준비

새 base `622af208ee12c4cb16357d465e4802e7fd4726bc`로 75개 커밋을 리베이스했다.
직후 head는 `2393eb7ae6041d40489295b914c5b8d8dc93f827`이다.
[전체 tree 대조](source152-rebase-result.json)는 Git이 시작 전 head와 새 base를 자동 병합한
tree `3bd2061f201d32ad716d2868c216796a5ec8d3b5`와 정확히 일치한다.
[range-diff](source152-rebase-range-diff.txt)는 동일 patch 72개와 원장 적용 순서 차이 3개다.
제품 구현을 수동으로 덮어쓰거나 기준값을 완화하지 않았다.

[해소 근거](source151-expected-fixture-merge.json)와 [두 중간 충돌 이벤트](source151-rebase-resolution-events.json)는
4개 fixture 원장을 독립 자동 병합 blob으로 해소한 기록이다. 첫 실패한 시도는 abort했고
원 head와 clean 상태에서 다시 시작했다. 최종 파일과 tree 일치로 해소 결과를 검증했다.

[공간 확보](source150-inactive-incremental-cleanup.json)는 종료된 증분 cache 121개만 제거했다.
Cargo/rustc/WASM builder 0개·열린 파일 참조 0개·잠금 검사를 확인했다. 예상 할당량
14,226,780,160 bytes와 실제 여유 공간을 구분해 기록했다. 고정 target의 프로필·deps·
fingerprint·실행 파일은 보존했다.

리베이스 성공은 새 head 기능 검증의 완료가 아니다. 새 base·후보의 실제 재빌드와
Native/fresh WASM·전체 회귀·Studio·독립 Print 대조를 이어서 수행한다. PR 준비까지만 진행한다.

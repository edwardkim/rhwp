---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase125-native-wasm-api-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase125-native-wasm-api-contracts/goal-checkpoint.md` · 원문 SHA-256: `81df08dbecf93749dc22f1836850f5be01590e96b5bf354f406bd14dd6995839`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 125차 기록

직접 검토는 11,868개 함수·helper, Rust 테스트 7,061개 / 870개 파일입니다. 미검토 9,703개·증거 대기 581개이며 전수 조사는 미완료입니다. 보정 필요 확정은 112개에서 113개 함수 항목으로 늘었습니다.

이번에는 src/wasm_api/tests.rs의 선택 root 16개 667줄·helper 4개 53줄과 source Table 분할·병합·export 소비를 읽었습니다. 실제 Native 원 검사는 13 PASS / 0 FAIL / 3 ignore입니다. 수동 corpus sweep과 profiler 두 개는 ignore를 풀거나 실행하지 않았습니다.

새 검출 공백은 test_get_table_bbox의 필수 fixture 누락 return입니다. 정상 원본의 bbox JSON을 확인했고 별도 빈 cwd에서도 입력이 없다는 문구와 1 PASS를 실제 확인했습니다. 정상 bbox의 117.2/251.0/555.4/95.0을 새 px golden으로 복사하지 않습니다. 필수 입력 누락은 FAIL하고 source Table 소유·구조화 bbox 계약은 보존하도록 보정할 항목입니다.

기존 Native unit binary는 실제 2546c8b clean build입니다. 현재 01ced3d에서 다시 빌드했다고 표시하지 않았습니다. rustc depfile의 1,309경로를 정규화한 1,305개 컴파일/Cargo/lock/build 입력이 old/current Git blob 및 실제 작업 파일과 같고, 실행 파일 SHA와 runtime 원본 3개가 유지됨을 확인해 재사용했습니다. 변경된 문서·시간 정책 스크립트는 컴파일 입력 밖입니다. 추가 빌드는 0입니다.

KPS의 split·페이지 전파 출력과 giant의 캐럿 질의 시간은 진단입니다. 진단 PASS를 페이지 배치·성능 상한 또는 전쪽 Native/fresh WASM 시각 통과로 확대하지 않습니다. 행/셀 개수·cut·repeat_header/caption·원 높이의 관계는 source 계약이며 실물 px 핀이 아닙니다. 별도 KPS 배치 회귀의 pending은 유지했습니다.

표 기능 #3831·본문 편집 #3255·캐럿 비용 #4265·row-local 거부 #6694·생성 helper #1482의 실제 통합 조상을 확인했습니다. bbox 원 함수는 최초 import f0f7f1a4b에 이미 같은 누락 guard가 있었고, rustfmt line blame을 원 해결 PR로 오인하지 않았습니다. import 이전 PR은 확인하지 못했습니다. 제품·테스트 수정·이름 변경은 0이며 사용자 JS와 원본·공유 target을 보존했습니다.

전수 조사 후 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.

---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase131-page-frame-and-margin-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase131-page-frame-and-margin-contracts/goal-checkpoint.md` · 원문 SHA-256: `3e647283bc0629d26a581654e7ccc700b471a97a147b2d1f58167d29e5c7d4fe`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 131차 기록

직접 점검은 11,939개 함수·helper, Rust 테스트 7,107개입니다. 보정 필요 확정은 121→124개 함수 항목이며 전수 조사는 미완료입니다. 미점검 9,661개·증거 대기 572개가 남습니다.

검사 14개와 helper 16개를 분석하고 기존 Native 검사 14 PASS를 완료했습니다. 현재 head의 공통 WASM을 locked wrapper no-opt로 한 번 만들었으며 기존 root pkg와 Studio 파일은 보존했습니다. Native 추가 빌드는 없습니다. 현재 두 backend의 선택 110쪽과 추가 전쪽 144행을 구분했고, 선택 PNG 재사용의 동일 producer·hash와 전쪽 재산출 TSV를 확인했습니다. 최종 대상별 고유 페이지는 각 backend 123쪽입니다.

정상 반칸 공백 2쪽 양쪽100, 1133 원본/대조군 3쪽 양쪽100, 스타일 6쪽 양쪽99.99143, 각주 25쪽 양쪽99.98대, sample5의5/6쪽 양쪽100을 확인했습니다. sample5 전64쪽은 Native93.37483/WASM93.37577 이상입니다. 정상 ±2 좌표·내용/소유·source 크기 계약은 유지합니다.

#6133의 생성 slice는 저장 제품 null이며 Native1쪽과 2020/2024 Print2쪽·동일한 제목 위치 차이가 있습니다. 원본4쪽의83.1 핀을 slice의 독립 검증 없이 쓰면 안 됩니다. #6143의75.6+100 창과 #6753의1009.2+8 창도 실제 source/Body·조각 소유 관계로 보정할 대상으로 기록했습니다. 관련 전쪽 미달을 기대값 교체로 숨기지 않습니다.

위성 보도자료 8쪽은5쪽만90 미만입니다. 같은2018저장본을 새2020 Print로 대조해도5쪽 N84.13091/W84.07531이므로 오래된 기준 PDF만의 문제로 처리하지 않습니다. 정확한6/8쪽 기준선은 유지하고 전체90 조건은 미충족으로 남겼습니다.

원 PR #6169/#6170/#6762와 실제 통합 #6191/#6774, 후속 #7923/#7913/#7941/#8191의 출처를 구분했습니다. 제품·테스트 소스 수정과 이름 변경은 아직0입니다. 전수 조사 후 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.

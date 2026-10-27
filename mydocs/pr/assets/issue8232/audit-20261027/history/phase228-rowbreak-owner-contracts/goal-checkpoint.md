---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase228-rowbreak-owner-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase228-rowbreak-owner-contracts/goal-checkpoint.md` · 원문 SHA-256: `424c562c3af878b330f6594d7a4d41c1174f512bfe2408d2743052a16d436a90`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 228차 기록

실제 동일 HEAD Native로 11쪽의 RenderTree를 다시 읽었다. 기존 cell_box caller의 대상 5쪽은 모두 고유 셀이었지만, 중첩 표가 있는 58쪽에는 주소가 중복되는 경우가 6개였다. (0,0)은 바깥 상자와 중첩 3표에 존재하며 주소-only/last 선택은 표 소속을 인증하지 못한다. 현재 제품이 잘못된 셀을 골랐거나 원 Rust root에 변형을 주입해 PASS했다고 주장하지 않는다.

독립 CFB에서 후속 문단 vpos 15006/20884/700과 바깥 상자 위 여백 283HU, 페이지 top/header 합 5668HU를 확인했다. 기존 기대의 5670HU는 2HU(0.02667px) 근사 차이가 있다. 123쪽 종료 셀은 저장 줄 끝 66020+1200HU와 padding 282HU에서 나온다. 이 5건은 근거 없는 화면 절대 px 핀으로 제거하지 않는다.

증거 보류 428→422개, 보정·정리 함수 지점 321→322개다. 현재 파일의 닫는 선 100HU와 rowspan 하한 3건 등 5항목은 보류를 유지한다. 새 빌드·원 Rust root 실행·시각 캡처·소스 수정·개명·외부 게시 없이 감사 근거만 갱신했다. 전체 감사는 진행 중이다.

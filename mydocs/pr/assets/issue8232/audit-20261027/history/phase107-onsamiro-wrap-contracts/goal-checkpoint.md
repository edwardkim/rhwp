---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase107-onsamiro-wrap-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase107-onsamiro-wrap-contracts/goal-checkpoint.md` · 원문 SHA-256: `162a1e93994908f257369771580b29a0a3171b39258e864c7333a39c0662d5db`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 107차 기록

직접 검토 10,821개 함수, Rust 테스트 6,691개/847개 파일이다. 미검토 10,490개, 증거 대기 568개, 보정 확정 97개로 전수 조사는 미완료다.

온새미로 검사 623줄·16개 함수 중 신규 15개와 기존 1개를 조사했고 현재 8 PASS / 0 FAIL / 0 IGNORE다. 동일 HEAD의 기존 실행 파일을 재사용해 추가 빌드는 0이다. 사전 조사기가 ignored 생성 suite를 git show HEAD로 읽으려던 오류는 Rust 시작 전 실패였고 별도 INVALID에 보존했다. 수정한 준비만 재개했으며 제품 검사를 반복하지 않았다.

목표 렌더 줄이 0개여도 offenders 공집합으로 통과하는 검사와 첫 7줄이 0/3개여도 take(7).all이 통과하는 검사 2개를 보정 대상으로 확정했다. 이는 source predicate의 논리 반례이며 제품의 실제 본문 누락이나 mutant Rust 실행으로 세지 않는다. 원본 HWP raw/HWPX XML에는 9줄과 첫 7개의 850/20999 HU가 일치한다. 관련 본문에는 LS가 없지만 익명화 fixture 내부 셀에는 9개의 LS record가 남아 있음을 구분했다.

현재 두 형식의 Native 6쪽은 100%, 35쪽은 91.61686%다. 실제 4쪽을 캡처했고 HWP/HWPX Native/PDF PNG는 바이트가 동일하다. 정본 review/overlay 두 쌍을 직접 판독했으며 35쪽의 문자 전진 차이는 남는다. compat=false는 실제 Rust 기본값과 같다. 6쪽 고정 x 상한의 fresh WASM/완료 페이지 자격은 추가 근거가 필요하다.

소스/테스트 수정·이름 변경은 0이며 기존 JS 변경은 보존했다. 미검토/보류 조사 완료 후 최종 이슈 갱신과 issue_8232 명명의 필요한 보정·실제 재검증을 수행한다.

---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase18-survey-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase18-survey-report.md` · 원문 SHA-256: `600be6256e81ddf6ac8143c6c2e563cfe2caf32569054268772f815c482819b8`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 18차 전수 조사·검사 검출력 대조

head `238de6de41a208648f4ade6b494fe433a81c1fce`, 브랜치 `audit/8232-px-regressions-20261025`에서 진행했습니다. 제품·테스트 소스 수정/개명은 아직 0건이며 PR/push는 하지 않았습니다. 전수 조사 후 실제 보정과 새 명명·호출/등록 갱신·필수 검증까지 수행하는 목표는 유지합니다.

## 직접 분류

16개 파일의 검사 196개와 helper 93개, 총 289개 함수를 직접 읽어 분류했습니다.

- 첫 7개는 Codex/strategist/visual-regression 스킬 계약, ontology, root binary 공용 helper, render-diff JSON, redact/sanitize입니다.
- 다음 9개는 EPS/AI 변환, 구조 auto 우선순위, nextCall 수복, 문단 시작 break, CLI 종료 코드와 공용 helper, fields/explain/search입니다.
- 작성 EPS BoundingBox/행렬·폰트 크기·색상, 원본 break bit·문자 오프셋·논리 구조/검색 주소, API 기본 임계값과 실물 페이지 px 핀을 구분했습니다. 이 숫자들을 모두 ±2로 바꾸지 않습니다.
- 20개의 보정 후보 함수 기록에는 missing-fixture 공유 helper/caller가 포함돼 있습니다. 독립 결함 20건이나 보정 완료 20건이 아닙니다.

## 실제 기존 검사에서 확인한 허위 통과

| 기존 검사 | 정상 대조 | 잘못된 응답 대조 | 관측 |
| --- | --- | --- | --- |
| capabilities 검색 | 1 PASS | 실제 CLI exit1/빈 출력인데 1 PASS | 실패 분기의 return |
| render-diff 읽기 실패 | 1 PASS | 실제 usage exit2인데 1 PASS | runtime1/usage2 혼용 |
| 표 셀 검색 | 1 PASS, 실제 cell 주소6개 | 같은 응답에서 cell 주소 전부 제거해도1 PASS | optional if-let로 필수 소유 주소 검사 생략 |

현재 실제 등록 목록의 binary 경로/SHA와 source SHA를 각각 고정했습니다. 정상과 반례 합계 6회의 기존 검사 실행입니다. 제품은 정상 응답을 반환합니다. 제품 회귀·과거 바이너리 재현·새 정식 검사 FAIL→PASS가 아닙니다. 원본은 수정하지 않았습니다.

Codex/visual 검사는 원 이슈 #5318/#5312 전체 본문·댓글 → 원 PR #5328/#5320 → 실제 통합 #5444/e79f113 → 현재 파일 이력을 대조했습니다. 두 허위 통과 조건이 실제 통합 시점부터 있었음을 확인했습니다. 자료 contract와 기존 CLI 호출을 연결하는 목적은 유효하지만 잘못된 단언까지 보존하지 않습니다. 검색 주소 검사는 실제 수정 전에 원 해결 계보의 나머지 확인이 남습니다.

## 본문 응답 검사

`document_text`는 text 키가 없어도 빈 문자열을 반환합니다. 현재 frozen Native의 읽기 전용 field-01 응답은3쪽/156글자입니다. missing/null이라는 서로 다른 잘못된 응답을 기존 predicate가 같은 빈 본문 값으로 바꾸는 반례를 기록했습니다.

첫 proxy 주입 시도에서는 runtime env를 바꿨지만 해당 helper가 compile-time env!를 사용하여 proxy를 호출하지 않았습니다. 그 두 PASS는 변형 응답 검출력의 증거가 아니며 INVALID 기록과 원 driver를 보존했습니다. 최종 근거는 읽기 전용 실제 응답과 source predicate 대조로 제한합니다.

## 바이너리 증적 정정

기존 15/17차 producer의 캐시된 이전 binary SHA를 실행 바이너리 증거로 잘못 연결한 부분을 정정했습니다. 원 producer와 당시 receipt는 정정 전 snapshot으로 보존하고 실제 nextest 요약과 SHA 출처를 분리했습니다.

- 17차는 같은 소스·옵션의 현재 metadata, 추가 컴파일 없음, binary mtime이 당시 producer보다 이전임을 대조한 연결 근거를 별도 기록했습니다. 기존 33 PASS/0 FAIL 요약은 변경하지 않았습니다. qualification의 producer SHA도 새 기록으로 갱신했습니다.
- 15차의 과거 binary SHA는 실행 증거에서 철회했습니다. 현재 metadata의 정확한 suite016 binary/source SHA를 고정해 기존27개를 다시 실행했습니다. 실제 27 PASS/0 FAIL, exit0, 1.217초입니다. 전체 회귀·새 검사·지침 적합 완료가 아닙니다.

## 원장과 남은 작업

원장10,346그룹/2,105파일, 미분류4,443, 직접 의미 판정5,118, 독립 근거 대기446, 의미 메모127, 파일 문맥212입니다. 직접 판정5,564개는 모두 원장에 연결됐고 누락0입니다. 이번289개 source SHA도 현재와 대조했습니다.

기존 전쪽 시각 목록96조건/각 backend2,505쪽은 변경하지 않았습니다. 새 캡처나 시각 개선으로 보고하지 않습니다. 미분류·독립 근거·동적 호출 폐쇄가 남아 전수 조사 미완료입니다. 판정을 마친 뒤 실제 보정·issue_8232 새 명명·등록/호출 정리·필수 검증을 이어갑니다. 잘못된 기존 단언의 유지를 전제로 삼지 않습니다.

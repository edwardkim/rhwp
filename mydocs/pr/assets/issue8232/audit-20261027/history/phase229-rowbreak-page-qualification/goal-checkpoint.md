---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase229-rowbreak-page-qualification/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase229-rowbreak-page-qualification/goal-checkpoint.md` · 원문 SHA-256: `1a4a86432ce9271ad59ef0ff80dd513796ad4d24f38a90cc48dfa87cb907168b`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 229차 기록

세 원본 Native/fresh WASM 전쪽456행을 현재 동일 HEAD 빌드 재사용으로 산출했다. CBTA204쪽은 Native 최저91.87577%, WASM91.82266%이고 미달0이다. 전직 면제6쪽은95.84490/95.96239%로 미달0이나2/3쪽은98미만이다.70833은17.28781/17.08861%이며 양쪽 모두5·6·7·9·10·15쪽이 미달이다.10쪽 문단 이월 차이를 직접 판독했으며2쪽 고득점으로 상쇄하지 않는다.

현재154·195쪽은 양쪽99%대이며 독립 Print·올바른 라벨의 full review/focus overlay를 직접 판독했다.100HU는 원본 필드가 아닌 실측 상수다. 기존 구현 재인용 .5px 핀2개는 독립 PDF 좌표·현행±2px·내용 소속/앞뒤 경계 계약으로 보정할 대상으로 판정했다.5128HU source 선언·저장 두 줄2882HU와 원래 행 경계는 유지한다. 다른3검사의 선언/내용/남는 밴드 하한은 관계 검사로 보존한다.

최초 그림 생성기의 page index를 확대율로 오인한 p004 라벨 오류는 INVALID 폴더로 분리하고 page-1로 보정해 다시 산출·직접 판독했다. 원본 PNG/TSV·제품 소스에는 영향이 없다.

증거 보류422→417개, 보정·정리 함수 지점322→324개다. 해당 분할 표 파일의 보류5건은 모두 판정했으나 전체 전수 조사는 진행 중이다. 새 빌드·원 Rust root 실행·소스 테스트 수정·개명·외부 게시는0이다.

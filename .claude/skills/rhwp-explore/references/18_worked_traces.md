# 18 — 재현 트레이스

트레이스는 `fixtures/traces/Txx.json` 이다. 각 파일은 시나리오의
DocFacts 와 `build_menu` 가 만든 봉투를 그대로 싣는다. 바이너리
없이 계약을 재현한다.

| ID | 시나리오 | 제목 | 첫 항목 |
| --- | --- | --- | --- |
| T01 | S01 | 특수 없음 → digest | triage-overview |
| T02 | S02 | 서식 → fields | form-fill |
| T03 | S04 | 표 → export-tables | table-extract |
| T04 | S06 | 조문 → export-structure | structure-outline |
| T05 | S08 | 차트 → chart-to-csv | chart-extract |
| T06 | S13 | 주입 → injection, 본문 금지 | security-sweep |
| T07 | S14 | 은닉 → hidden-text | security-sweep |
| T08 | S15 | 둘 다 → injection 명령 | security-sweep |
| T09 | S16 | 보안이 서식·표보다 앞 | security-sweep |
| T10 | S17 | 암호 풀림 → encrypted why | triage-overview |
| T11 | S11 | 10쪽 medium long-doc | long-doc-digest |
| T12 | S12 | 40쪽 high long-doc | structure-outline |
| T13 | S23 | 9쪽은 long-doc 없음 | triage-overview |
| T14 | S26 | 8개 전부 | security-sweep |
| T15 | S40 | 우선순위 계약 | security-sweep |
| T16 | S20 | HWP3 개요 | triage-overview |
| T17 | S19 | HWPX 표 | table-extract |
| T18 | S22 | 0쪽 로드 성공 | triage-overview |
| T19 | S09 | 각주·미주 | note-structure |
| T20 | S36 | 서식이 표보다 앞 | form-fill |
| T21 | S28 | medium 보안도 1번 | security-sweep |
| T22 | S30 | 암호 장문 | structure-outline |
| T23 | S07 | 조문 medium | structure-outline |
| T24 | S32 | 차트 1개 | chart-extract |
| T25 | S33 | 미주만 | note-structure |
| T26 | S37 | 빈 파일 레이블 | triage-overview |
| T27 | S03 | 누름틀 11 | form-fill |
| T28 | S25 | 표+차트+조문 | table-extract |
| T29 | S27 | 장문 논문 | structure-outline |
| T30 | S35 | 주입 1건 high | security-sweep |
| T31 | S18 | 암호 서식 | form-fill |
| T32 | S31 | 표 1개 | table-extract |
| T33 | S05 | 병합 없는 표 | table-extract |
| T34 | S21 | HML | triage-overview |
| T35 | S34 | HWPX 서식 | form-fill |
| T36 | S38 | 알 수 없음 형식 | triage-overview |
| T37 | S39 | 쪽 0·표 1 | table-extract |
| T38 | S24 | 20쪽 high | long-doc-digest |
| T39 | S10 | 각주만 | note-structure |
| T40 | S29 | DRM 레이블 | triage-overview |

T15 는 `tests/cases/explore_menu_contract.rs` 의 우선순위 표본과 같다.

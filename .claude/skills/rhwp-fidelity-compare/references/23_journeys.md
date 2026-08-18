# 23 — 실사용 여정 (이 스킬 범위)

아래 여정은 **한컴 PDF 대조** 만 다룬다. bug-hunter 의 접수·패치
여정이 아니다. 각 줄의 정지 ID 는 `fixtures/journeys.json` 과 같다.

## 짧은 표

| ID | 제목 | 정지 |
| --- | --- | --- |
| J01 | plan 전수 35쪽 | F03 |
| J02 | 독립 PDF 없음 | F01 |
| J03 | text-only 215쪽 | F02 |
| J04 | Chrome 부재 | F10 |
| J05 | venv 부재 | F09 |
| J06 | Windows 경로 | F03 |
| J07 | break-system-packages 요청 | F15 |
| J08 | 암호화 PDF | F13 |
| J09 | 쪽수 35 vs 37 | F11 |
| J10 | 두부 시트 | F14 |
| J11 | korexam A3 창 | F03 |
| J12 | math 6–11% | F03 |
| J13 | bunjang 참고 PDF | F17 |
| J14 | direct pair 플래그 누락 | F12 |
| J15 | PUA #3385 | F04 |
| J16 | 각주 owner shift | F02 |
| J20 | run-state incomplete | F12 |
| J21 | 편집 전후 오인 | F01 |
| J22 | 여정 방법론 요청 | F08 |
| J23 | gym pack 요청 | F06 |
| J24 | 새 CLI 요청 | F06 |
| J25 | visual-regression 수정 | F07 |
| J30 | 암호화 + text-only | F13 |
| J56 | 맞춰찍기 축소 PDF | F17 |
| J71 | CI diff% 게이트 | F05 |

전체 80 줄은 fixtures. 이 장은 대표 절차만 풀어 쓴다.

## J01 plan 전수

1. 독립 PDF 확인 — `pdf/*-2022.pdf`, 등급 한컴 2022
2. venv 인터프리터 확인
3. 최신 `RHWP_BIN` (방금 렌더러를 고쳤다면 rebuild)
4. `plan 0 34 --out-dir /tmp/rhwp-fidelity-plan`
5. run-state complete
6. 두부 아님
7. report.tsv 상위 시트 감사
8. 실질만 이슈 초안, 유지자 (F05)

중간에 Chrome 없으면 J04 로 갈라져 `--text-only` 전수 + 상위만 나중에.

## J03 긴 문서 1차

1. direct pair 세 플래그 + grade
2. `--text-only --export-all-svg --layout-ledger`
3. text-report / boundary / page-count
4. 시트는 창만. 215장 PNG 금지
5. 확정 문장 금지 (F02)

## J10 두부

1. 전면 □ 확인
2. 랭킹 폐기
3. FONT_PATH + 새 out-dir
4. p0–p2 만 재실행
5. 본문 생존 확인 후 범위 확대

## J21 / J22 오인

편집 전후만 있으면 F01 인계. "버그 헌팅 해줘" 이고 원장이 필요하면
여기를 도구로 쓴 뒤 hunter 로 인계 (F08). 두 스킬 파일을 고치지 않음.

## 에이전트가 여정을 추가할 때

`fixtures/journeys.json` 과 이 장과 `stop_rules.json` 의 id 가
같아야 한다. 계약 시험이 검사한다. gym pack id 를 빌려 오지 않는다.

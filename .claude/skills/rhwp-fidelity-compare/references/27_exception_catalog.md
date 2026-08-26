# 27 — 예외 카탈로그

정지 규칙과 예외 경로를 한 장에 모은다. SKILL.md 표의 확장이다.

## 환경

| ID | 신호 | exit | 다음 |
| --- | --- | --- | --- |
| F09 / E-VENV | `pypdf가 필요합니다` / `pypdfium2가 필요합니다` | 2 | 저장소 venv. break 금지 |
| F10 / E-CHROME | `Chrome/Chromium을 찾을 수 없습니다` | 2 | `CHROME_BIN` 또는 `--text-only` |
| F15 | 사용자가 `--break-system-packages` | — | 거절 |
| E-RHGP | `rhwp 실행 파일을 찾을 수 없습니다` | 2 | `RHWP_BIN` / release-test |
| E-RANGE | `요청 끝 쪽이 기준 PDF 마지막 index를 넘습니다` | 2 | 끝 쪽 수정 |

## 입력

| ID | 신호 | 다음 |
| --- | --- | --- |
| F01 | 공식 PDF 없음 | visual-regression |
| F13 / E-ENCRYPT | encrypted / password | 정지. 우회 금지 |
| F17 | 동반 PDF · 맞춰찍기 축소본 | 참고 등급. 승격 금지 |
| F18 | 원본 경로에 쓰려 함 | `--out-dir` 만 |

## 산출

| ID | 신호 | 다음 |
| --- | --- | --- |
| F02 | `--text-only` 원장 | 확정 금지 |
| F03 | report.tsv 상위 | 시트 감사 |
| F04 | PUA/FFFD/□ (국소) | 글꼴 먼저, 그다음 후보 |
| F11 / E-PAGECOUNT | ledger delta ≠ 0 | owner 조사. 전역 패치 금지 |
| F12 | run-state incomplete, SVG 없음 | 누락 쪽. 전수 포장 금지 |
| F14 / E-TOFU | 전면 두부 시트 | 랭킹 폐기, 글꼴 재실행 |
| F05 | 감사 끝 | 유지자 |
| F16 | 질문 이미 답 | 다음 단 금지 |

## 범위

| ID | 신호 | 다음 |
| --- | --- | --- |
| F06 | gym / 새 CLI | 거절 |
| F07 | visual-regression 수정 | 거절 |
| F08 | bug-hunter 재작성 | 거절 |

## 하드 vs 소프트

하드(계약을 깨면 즉시 거절): F06 F07 F08 F09 F13 F15 F18.
소프트(다른 단으로 내릴 수 있음): F01 F02 F10 F11 F12 F14 F16 F17.
F03 F04 F05 는 진행 중의 읽기 규칙이다.

`fixtures/exception_catalog.json` 과 `stop_rules.json` 이 같은 id 를
가진다. 새 예외를 추가하면 세 곳(이 장, SKILL 표, fixtures) 을 같이
고친다.

## 우선순위

동시에 여러 예외가 나면:

1. F13 암호화 — 비교 자체가 성립하지 않음
2. F09 venv / E-RHGP — 실행 불가
3. F10 Chrome — 시트만 포기 가능
4. F14 두부 — 숫자 폐기
5. F12 incomplete
6. F11 쪽수 · F04 국소 두부
7. F03 랭킹 감사

암호화된 채 시트를 만들려 하지 않는다. 두부 시트의 랭킹을 쪽수
논의보다 먼저 버린다.

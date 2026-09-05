# 요청 → 레시피 판단 나무

이슈: #5331. 라우터 장 `00_tree.md`.
정본 디렉터리: `mydocs/manual/recipes/`.
gym 이 아니고 새 CLI 도 없다. 07·08 을 만들지 않는다.

## 한 줄

사용자 요청을 읽고 **레시피 한 장**을 고른 뒤, 그 장의 첫 명령만 치고 이웃 스킬로 넘긴다. 이 스킬은 채움·표 왕복·스윕·배치·회귀를 다시 쓰지 않는다.

## 살아 있는 동사는 이 여덟 장

| 번호 | 짧은 이름 | 첫 수 | 다음 스킬 |
| --- | --- | --- | --- |
| 01 | 서식 | `rhwp fields <file> --json` | rhwp-form-fill |
| 02 | 표 | `rhwp export-tables <file> --json` | rhwp-table-exchange |
| 03 | 마스킹 | `rhwp edit redact <file> --dry-run` | rhwp-security-sweep |
| 04 | 수신 점검 | `rhwp info <file> --json` | rhwp-doc-triage |
| 05 | 메일머지 | `rhwp fields <file> --json` | rhwp-form-fill |
| 06 | 시각 회귀 | `rhwp render-diff <file> --via hwpx` | rhwp-visual-regression |
| 09 | 대량 추출 | `rhwp batch info --json` | rhwp-bulk-pipeline |
| 10 | 송신 스윕 | `rhwp inspect hidden-text <file> --json` | rhwp-security-sweep |

07·08 은 표에 없다. 결번이다.

## 분기

1. 요청이 07/08 또는 없는 번호 → R02/R03, 파일을 만들지 않는다.
2. 정본 파일의 `last_verified` 가 30일보다 오래됨 → R04, 순서를 메우지 않는다.
3. 트리거가 두 장과 맞음 → R05, 사용자에게 고르게 한다.
4. 한 장만 맞음 → 그 카드의 첫 수 → nextSkill.
5. 출처 모르는 첨부 + 채움/추출 → 04 가 앞 (R06).

## 금지

- recipe/route 하위명령 발명
- gym pack 으로 실무 레시피 대체
- 이웃 스킬 SKILL.md 재작성
- 정본 레시피 밖의 명령 사다리 창작

# 26 — 인계

이 스킬은 한컴 PDF 대조만 닫는다. 인접 작업은 이웃 스킬에 넘기고
그 파일을 고치지 않는다.

## 표

| 상황 | 넘길 곳 | 이 스킬이 넘기는 것 |
| --- | --- | --- |
| 공식 PDF 없음 / 전후 레이아웃 | `rhwp-visual-regression` | "오라클 없음" 한 줄 |
| 원인 미확정 실사용 결함 | bug-hunter | out-dir, 상위 쪽, provenance |
| 미지 문서 파악만 | `rhwp-doc-triage` | 비교하지 말 것 |
| export-svg 단건 디버깅 | `rhwp-cli` | `--font-style` 를 같이 쓰라는 힌트 |
| 편집 자체 | `rhwp-safe-edit` / `rhwp-form-fill` | 끝난 뒤 공식 PDF 가 있으면 복귀 |
| 배포 전 숨김/주입 | `rhwp-security-sweep` | 무관. 여기서 스윕하지 않음 |
| 온보딩 / MCP | 해당 스킬 | 무관 |
| gym | 거절 F06 | 없음 |

## 복귀

form-fill 로 채운 뒤 "한컴 원본 PDF 와 채워진 산출이 같나" 는
**다른 문서** 비교라 이 하네스의 정직한 입력이 아닐 수 있다.
채워진 HWP 와 한컴이 채운 PDF 가 둘 다 있으면 direct pair.
한컴 PDF 가 원본 빈 서식이면 글자가 다른 것이 정상이다. 그 경우
visual-regression 전후가 맞다.

## 문구

인계 때 이웃 스킬의 정지 ID 를 흉내 내지 않는다. 이쪽 F01 만 적고
"visual-regression 스킬을 여세요" 라고 한다.

이웃 `SKILL.md` 존재는 계약 시험이 확인한다. 내용을 이 PR 에서
바꾸면 F07/F08 위반이다.

## 에이전트 금지

- 인계 대신 이웃 스킬 본문을 이 references 에 복사
- 한 응답에서 render-diff 와 fidelity 를 한 판정으로 합치기
- hunter playbook 단계를 여기 사다리에 끼워 넣기

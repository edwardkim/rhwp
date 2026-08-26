# 21 — 인계

이 스킬은 헌팅만 한다. 이웃 스킬 본문을 여기서 재작성하지 않는다
(금지 기본값). 포인터만.

기계본: `fixtures/handoff.json`.

## 언제 떠나는가

| 상황 | 가는 곳 | 돌아올 때 |
| --- | --- | --- |
| 누름틀 채움 수단 | rhwp-form-fill | 채운 산출을 정답지와 대조 |
| 전후 레이아웃 px | rhwp-visual-regression | 한컴 충실도는 여기. render-diff 는 자기 일관성 |
| 표 CSV 왕복 | rhwp-table-exchange | 되돌린 표 재독 |
| 배포 전 숨은 글 | rhwp-security-sweep | 스윕 후 제출 직전 대조 |
| 미지 문서 파악 | rhwp-doc-triage | 파악 후 여정 선택 |
| 폴더 수백 건 | rhwp-bulk-pipeline | 실패 1건을 여정으로 승격 |
| 수정 PR 절차 | rhwp-contributor | 요청받은 뒤에만 |

## 떠나지 않는 것

- playbook 실행 계약
- 정답지 확보와 provenance
- fidelity_compare 호출
- 이슈 템플릿 3필수
- 접수 거부 / UTF-8 파일 비교

visual-regression 의 STRUCT_MISMATCH 경로 읽기는 그 스킬의
루브릭이다. 여기로 복사하지 않는다. 한컴 PDF 축이 필요하면
그 스킬이 아니라 이 스킬의 12장을 쓴다.

## 수정 인계 문장

```
헌팅 이슈 #<n> 를 고치는 작업은 별도 PR 이다. bug-hunter 스킬은
닫는다. DocumentCore 변경은 이 스킬의 범위가 아니다.
```

# 00 — 판단 트리

권위는 playbook 한 장이다. 이 트리는 그 6단을 에이전트가 빼먹지 않게
상자로 펼친 것이다. 점수표를 새로 만들지 않는다.

살아 있는 동사는 기존 `rhwp` CLI 와 `tools/fidelity_compare` 뿐이다.
`bug-hunt` 같은 하위명령을 발명하지 않는다.

```
질문이 "버그를 고쳐라" 인가?
  └ 예 ──▶ F12. 이 스킬을 닫고 contributor 로 인계. DocumentCore 금지
질문이 "gym 과제/채점" 인가?
  └ 예 ──▶ F01. 거부. 이 스킬은 실 에이전트 헌팅
질문이 "실사용 기준으로 찾아라" 인가?
  └ 아래 6단
```

## 6단 (강제 순회 — 질문이 이미 답이면 그 단에서 정지)

```
1. 여정 선택
   playbook 카탈로그(예시 1–7 + 다음 후보)에서 사용자 가치 순
   samples/ 무작위 스윕이면 F01 로 되돌림
        │
2. 정답지 확보 (여정 실행보다 먼저)
   한컴 공식 PDF / 법정 서식 / 실제 제출 요건
   한컴이면 도구·버전·출력 경로·폰트·원본/산출 경로를 기록 (F03)
   없으면 F04: render-diff 자기 일관성만 + 한계 기록. 충실도 이슈 금지
        │
3. 최종 산출물까지
   info → fields/export-tables → edit → export-svg/pdf/hwpx
   중간 info 만 보고 멈추면 F05
   실제 접수/로그인/실명인증이면 F13 즉시 거부
        │
4. 대조 (축을 섞지 않음)
   픽셀/시각          → 후보. 최종은 maintainer
   문자 멀티셋        → 소실/과잉/치환 후보 (F06·F07·F08)
   기록값 재독        → 기계 확정
   종료 코드·JSON     → 기계 확정
   --verify 4/4       → 멈추지 않음. ZIP 이름 집합 (F09)
   콘솔 깨짐          → 결함 아님 (F10)
        │
5. 이슈화
   재현 명령 + 파일:라인 + 정답지 근거
   증상만이면 F11. devel 에 이미 있으면 F14
        │
6. 같은 여정 다음 격차 → 마르면 다음 여정
   playbook 4단 예시 추가를 제안
   수정은 별도 PR
```

## 이 상자가 재는 것 / 안 재는 것

재는 것: 에이전트가 playbook 순서를 빠뜨렸는지, 정답지 없이 충실도
이슈를 쓰려 하는지, 발명 CLI 로 새 오라클을 만드는지.

안 재는 것: 엔진이 맞는지. 그건 정답지 대조의 결과이지 이 트리의
점수가 아니다.

## 관련 픽스처

- `fixtures/tree.json` — `ladder`, `livingVerbs`, `secondRubricForbidden`
- `fixtures/stop_rules.json` — F01–F16
- `fixtures/command_ladder.json` — 허용 argv

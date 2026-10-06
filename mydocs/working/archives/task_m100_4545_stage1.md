# task_m100_4545 stage1 — 6년 축 게이트 (Y6-M1+M3)

- 이슈: #4545 (설계서 #4449 착공) / 브랜치: task_m100_4545 (스택 최상단)
- 산출물: src/policy_gate.rs + cmd_gate + 계약 4종 + 7+1표면 + 대전 76명령 +
  **대량 문서** mydocs/tech/policy_gate_guide.md(운영 대전서 — 판정 키 사전
  전체·시나리오 6종·TOCTOU·배치 전략·FAQ)

## 설계 이행

- 연산자 4개 고정(감사 가능성 우선) · 미지 키/연산자 로드 시점 exit 2(항상-참
  구멍 차단) · deny 기본(빈 규칙=거부) · **재계산 원칙**(신고 불신 — 계보
  걷기·서명 검증·앵커 조회·--deep 재실행, 참조된 판정만 지연 계산) ·
  unavailable 위반(모르는 것은 통과 아님 — reproduced 는 --deep 없이 판정
  거부) · targetSha256(TOCTOU) · 정책 서명 보고(M3, 4년 축 재사용).

## 실측 (gate_contract 4/4 첫판)

- 전 5축 규칙 allow(평가 6조건) → 후행 공백 변조 → R3-서명·R4-앵커 위반
  명세와 함께 deny — 한 명령이 1~5년 축 전부를 소비함을 실측.
- 오타 키·미지 연산자 exit 2 / 빈 규칙 deny / keyring 없는 서명 규칙·deep
  없는 재현 규칙 = unavailable 위반 / policySigned 3상태.
- 가드: 사전 +7행(218 유니크 — policy 기존 행 재사용 실측)·봉투 45·node
  466(gate 단일 함수 — toCamel 직행이라 우산 표 불필요)·대전 76명령 델타
  2장·미분류 자동 소멸.

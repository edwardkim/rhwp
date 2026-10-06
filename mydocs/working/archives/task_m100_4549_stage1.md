# task_m100_4549 stage1 — 7년 축 연합 (.lineage-bundle) Y7-M1~M2

- 이슈: #4549 (설계서 #4450 착공) / 브랜치: task_m100_4549 (스택 최상단)
- 산출물: src/lineage_bundle.rs + bundle export/verify + 7+1표면 + 계약 2종
  (왕복+3공격) + 대전 77명령 동행

## 설계 이행 (horizon_year7_federation.md 그대로)

- zip 컨테이너(신규 포맷 발명 없음 — HWPX 선례): manifest(파일별 SHA-256)·
  capsules/(폐쇄집합)·signatures/·anchor/proofs.json·domain.json.
- **F2 방어가 급소**: 서명 판정은 동봉 keyring 절대 불신 — 수신자가 자기
  경로로 받은 trust-domain 의 keyring 만. 앵커도 동봉 체크포인트가 아니라
  도메인 선언 체크포인트와 루트 대조.
- 머클 증명 동봉: 로그 줄 원문+경로 → 수신자는 로그 없이 잎→루트 재계산.
- 수신 정책(⑥)은 gate 단독 실행으로 위임(v1 범위 명시).

## 실측 (bundle_contract 2/2 첫판)

- 왕복: 폐쇄집합 2·서명 2·증명 2 export → 5단 verify 전건 green
  (checkpointTrusted true).
- 3공격 검출: 운송 변조(zip 내 후행 공백)→containerOk false / 조상 은닉
  (부모 항목+매니페스트 제거)→closureOk false / **낯선 도메인**(F2)→
  signed.invalid 2, 전부 exit 3.
- 가드: 스윕 레시피 2종(export→verify 순서 의존)·사전 +8행(226)·봉투 46·
  node 466(패리티 우산 bundle)·대전 77명령 델타 2장 멱등·clippy 0.

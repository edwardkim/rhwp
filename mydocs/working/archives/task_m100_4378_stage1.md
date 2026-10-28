# Task M100 #4378 Stage 1 — R22×R24 CAS 구현 (적층: #4330)

- 이슈: #4378 · 브랜치 task_m100_4378(fork/task_m100_4329 적층 — planSchemaVersion 범프를 레지스트리 단일 출처에서)
- 2026-08-10 KST · 구현·검증 완료 · R23(체인 전체)은 범위 밖(W1 저널 #4361 이 씨앗)

## 구현
- run 계획서: preconditions.inputSha256(선택) — 형식 검사(usage)·파싱 전 대조,
  불일치 = 실행 0·저장 0·exit 2 + invalid[]{code:preconditionFailed, expected, actual}.
- edit --expect-sha256(기함: replace-text): 불일치 exit 3 + preconditionFailed 봉투,
  검사는 파싱 전. **다른 edit 하위명령 확장은 후속**(fill 계열은 공용 코어가 파일을
  읽는 구조라 배선 지점이 달라 별도 이슈로 — 정직 범위 축소).
- planSchema: $defs.Preconditions 신설 + 루트 preconditions 등재, PLAN_SCHEMA_VERSION
  1.0→1.1(minor, 이력 주석은 schema_registry).
## 검증
- 신규 계약 6/6 첫 실행 통과(red: 거부·저장0·exit2/3 실증, green: 완주, 형식,
  자기서술 1.1). 인접: run_plan·dry_run·schema_registry 계약 green,
  plan_schema_contract 는 버전 고정 1건을 1.1 로 정합화. clippy 0. fmt 적용.

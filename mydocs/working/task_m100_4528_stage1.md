# task_m100_4528 stage1 — rhwp-work-receipt 스킬 (검증 사다리 실행 규약)

- 이슈: #4528 / 브랜치: task_m100_4528 (base devel)
- 산출물: .claude/skills/rhwp-work-receipt/SKILL.md

## 설계

- devel 실재 명령만 참조(replay·audit·lineage·run) — 서명 계열(4년 축 #4511)은
  머지 후 2부로 확장한다고 경계 절에 명시. 표류 가드(#4508/#4510)가 이 스킬의
  명령 실재를 상시 보호하는 구조(가드 규약 준수: frontmatter name=폴더명·
  description 상세·실행 참조 다수).
- 절차 4종(단건 영수증→캡슐·체인→폴더 감사→계보 검증) + 요청별 권장 흐름 +
  함정 실록(캡슐 불변·--parent 상대경로는 캡슐 기준·부모 덮어쓰기 거부·
  attest 는 output 경로 미생성).
- 판정 규약 공통 절: exit 3 = 봉투 데이터, 실패 stdout 0바이트.

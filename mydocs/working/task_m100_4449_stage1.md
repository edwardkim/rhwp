# task_m100_4449 stage1 — 6년 선행 축 정책 게이트 설계서

- 이슈: #4449 ([#3907 지평]) / 브랜치: task_m100_4449 (base devel, 코드 0줄)
- 산출물: mydocs/tech/agent_roadmap/horizon_year6_policy.md
- 내용: admissionPolicy(연산자 4개 고정, deny 기본, 미지 키 exit 2)와 rhwp gate. 판정 재료는 자기 신고가 아니라 재계산. 정책 자체의 서명(4년 재사용)과 TOCTOU 방어(targetSha256).
- 착수 조건: 5년 축 머지(최소 4년) — [지평] 등급, 완료 표기는 머지 링크와 함께만.
- 검증: check_markdown_links, check_document_metadata 통과. 교차 참조는 이슈/PR URL 만 사용.

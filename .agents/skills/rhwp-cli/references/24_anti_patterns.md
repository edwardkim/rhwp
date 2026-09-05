# 금지 패턴

| id | 패턴 | 대신 |
|---|---|---|
| A01 | 새 서브커맨드 `rhwp layout-debug` | 기존 6단 |
| A02 | DocumentCore 패치를 이 스킬에서 | 이슈→브랜치 |
| A03 | gym/ 팩 실행 | 실파일 + 기존 CLI |
| A04 | 다른 스킬 SKILL.md 수정 | 인계만 |
| A05 | 페이지 기본값을 1로 문서화 | 0 |
| A06 | 한컴 호환을 테스트 초록으로 단정 | 19장 |
| A07 | oracle 없이 generated 두 번 비교 | 한컴 저장본을 받기 |
| A08 | 실패 경로를 빈 성공 봉투로 합성 | stdout 0바이트 |
| A09 | export-png 를 skia 없이 재시도 루프 | 재빌드 안내 |
| A10 | ir-diff 차이를 예외 throw | 데이터 |
| A11 | dump 전체 문서를 컨텍스트에 덤프 | -s -p 로 좁히기 |
| A12 | 편집 fill/redact 를 이 스킬에 흡수 | 해당 스킬 |

이 표의 id 는 픽스처 `anti_patterns.json` 과 같다.

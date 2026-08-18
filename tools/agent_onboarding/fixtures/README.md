# agent_onboarding fixtures

이 폴더는 닥터의 **실패·계약 픽스처**다. 성공 시연용 정상 HWP 를 두지 않는다.
정상 문서는 저장소 `samples/basic/english.hwp` 를 쓴다.

## samples/

고의로 깨진 입력. `classify_sample` 과 `bad_sample` 예외 경로 테스트용.

| 파일 | 기대 |
|---|---|
| empty.hwp | 0바이트 |
| tiny.hwp | 하한 미만 |
| not_hwp.txt | 문서가 아님 |
| text_named_hwp.hwp | 확장자만 hwp |
| truncated_ole.hwp | OLE 매직 8바이트만 |
| zeros.hwp | NUL 패딩 |

## envelopes/

명령별 required 키. 닥터 상수와 같아야 한다.

## mcp/

호스트 모양 스니펫. 포트·인증 필드가 없어야 한다.

## reports/

리포트 형태 메모. 런타임 골든이 아니라 필드 계약.

## recipes/

첫 5분 단계가 읽기 전용 기존 CLI 만 인용한다는 인덱스.

# 중위 첨자 문법 경계 독립 대조군

Issue: #7642

정식 회귀 fixture가 아닌 구조 진단 자료다. 원본 HWPX를 한컴 Office 2020에서 정상 HWP로 저장한 뒤 Print로 출력했다. 출력은 `11.0.0.9136`, `PrintMethod=0` (1-up)이다.

- 정상 저장 job: `cc3bde1b-3234-4fe7-a427-8b759df04ff2`
- Print job: `0ea24572-8cce-43e0-84b1-7986c93c324e`
- 1–3·8–9쪽: 선행 항 유무, 오류 구문의 그룹·행 범위와 후속 본문.
- 4쪽: 첨자 피연산자 누락.
- 5쪽: 유효한 왼쪽 첨자 뒤의 본문 보존.
- 6–7쪽: 오른쪽 SUB/SUP의 중위 명령.
- 10쪽: 위·아래 왼쪽 첨자의 역순 연쇄.
- 11–12쪽: 근호/적분 인자의 소속. 바깥에서 첨자를 덧붙이는 가정의 반례다.

이 자료의 미검증·미달 범위는 활성 작업 기록에 남긴다. 출력 점수 없이 정식 회귀를 추가하거나 golden을 갱신하지 않는다.

## SHA-256

- `left-script-boundary-hancom-2020.hwp`: `b9f0c4049d3dcc6b3b3abde3095f733728826822a9e79b2db8da28dbeae4664a`
- `left-script-boundary-hancom-2020.pdf`: `aec2d14231e38a8b3a32d6cd2ea92b1efc58ff5154aaf84132b5d70d3eb6f938`
- `left-script-boundary-input.hwpx`: `4682ce44bc24206b9caa2a6fb01f067b069112dce813c8edb2a712b069a0ef8b`

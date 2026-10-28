# task_m100_3370 처리결과 보고서 — 에이전트 실무 대체 예제집 (1차 조각)

- **이슈**: [#3370](https://github.com/edwardkim/rhwp/issues/3370)
- **브랜치**: `pr/task-3370-agent-playbook` (upstream/devel `4a39f7cc0` 직분기)
- **범위**: `mydocs/manual/agent_task_playbook.md`(신규), `mydocs/manual/README.md`(색인 1항목)
- **분류**: 문서 (사용자 온보딩 — 실무 대체 카탈로그)

## 1. 배경

CLI 는 조회 8축·batch·검증 게이트를 갖췄지만, 최종 사용자(실무자와 그들의 에이전트)가
**자기 업무를 도구에 매핑할 카탈로그**가 없었다. "1등 에이전트 도구"의 기준은 기능 수가
아니라 **줄어든 실무 시간**이고, 그 병목은 사람이 스크린샷을 보며 고치는 잔여 루프다.

## 2. 설계 결정

- **4단 구성 고정** — 사람의 업무 → 에이전트 시퀀스 → **기계 검증** → 절감 포인트.
  기계 검증(종료 코드·재독 대조·자기서술)이 눈 검증을 대체하는 것이 문서의 중심 원칙.
- **전 예제 실측** — 7개 시나리오 전부 저장소 samples 실문서로 실행 검증한 것만 수록
  (658건 batch 수치, 5필드 메일머지 코드포인트 대조, 분장사무 표→CSV 등).
- **미래 기능은 정직하게 표기** — fill-fields(#3345)·batch 확장(#3346)·truncated(#3353)·
  ingest 오류(#3358) 등 미머지 항목은 반영 시점을 명시.
- **로드맵 연계** — 편집 축(#2659 §7.3)의 우선순위를 "대체되는 사람 업무 크기"로 재배열해
  제안하고, 신규 축 DoD 에 예제집 항목 추가를 포함하자고 제안.

## 3. 검증

- 전 명령 시퀀스 실측 재현 (2026-07-26, v0.8.0 릴리스 + devel 빌드)
- 문서 링크: 같은 디렉터리 상대 링크 2건(cli_commands, cli_json_pipeline_guide) 확인
- front matter 4필드(kind/status/canonical/last_verified) 준수, manual README 색인 등재

## 4. 남긴 것

- 예제 1(fill-fields)은 #3345 머지 후 "현재 동작" 으로 승격.
- edit replace-text/set-cell/batch fill-fields 가 붙으면 커버리지 표의 "제안" 행들이
  예제 항목으로 이동한다.

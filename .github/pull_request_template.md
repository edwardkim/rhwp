> **PR base 브랜치가 `devel` 인지 확인해주세요** (`main` 아님 — GitHub 기본 선택이 main 일 수 있습니다).
> 작업 브랜치는 최신 `upstream/devel` 에서 생성합니다. 상세: [CONTRIBUTING.md](../CONTRIBUTING.md)

## 변경 요약

이 PR이 해결하는 문제와 변경 내용을 간결하게 설명해주세요.

## 관련 이슈

closes #

## 테스트

- [ ] `cargo test` 통과
- [ ] `cargo clippy -- -D warnings` 통과
- [ ] 관련 샘플 파일로 SVG 내보내기 확인
- [ ] 웹(WASM) 렌더링 확인 (해당하는 경우)
- [ ] `.claude/agents/`, `.claude/skills/`, `.agents/skills/` 변경 시: [capability 카탈로그](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/agent_capability_registry.md)의 등록·검증 규칙을 반영

## 성능 영향 및 측정 결과 (해당하는 경우)

- 예상 영향: <!-- 개선 / 회귀 가능성 / 영향 없음 / 미확인 -->
- 재현·측정: <!-- 공개 sample, 명령, 환경, 변경 전후 관측값. 측정 환경이 없으면 "미측정" -->

> 특정 장비의 절대 성능 수치나 메인테이너 전용·비공개 벤치마크 통과는 PR 제출 조건이 아닙니다.
> 공개된 결정적 성능 회귀 테스트와 GitHub required checks는 기존과 같이 적용됩니다.

## 스크린샷

변경 전후 비교가 필요한 경우 첨부해주세요.

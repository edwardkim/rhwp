# Task M100 #2072 Stage 27 - 최신 기준 최종 감사

## 목표

Stage 24~26 보완과 최신 `upstream/devel` rebase 이후 이슈 #2072 본문 및 maintainer 코멘트의 완료
조건을 다시 대조한다. 이전 Stage 22의 중간 수치를 최종 근거로 재사용하지 않고 현재 트리에서 모든
검사를 다시 수행한다.

## 감사 항목

- manual·tech 지도와 canonical/detail/history 경계
- CLI, 시각 검증, PR, 스펙 정오표, IR 표준의 단일 진입점
- 이슈별 investigation, 확정 troubleshooting, 장기 기술 문서의 분리
- 저장소 `AGENTS.md`와 짧은 `CLAUDE.md` 부트로더
- 제한된 redirect와 이동 경로 내부 재참조
- 디렉터리 자동 링크·메타데이터 CI
- 제품 소스와 동작 변경 여부

## 검증 계획

- 최신 `upstream/devel` 기준 ahead/behind 확인
- 기본 링크, 전체 메타데이터, Python 구문, workflow, diff 검사
- 문서·redirect·investigation 수와 제품 소스 변경 정량 확인
- 완료 조건 대조표와 최종 보고서 작성

## 최신 원격 반영

`upstream/devel`이 2커밋 전진해 #2072 커밋을 최신 기준 위로 rebase했다. `CLAUDE.md`에서 원격의
대형 중복 본문과 짧은 부트로더가 충돌했으며, canonical 문서로 절차를 위임하는 짧은 부트로더와
원격에서 추가된 rhwp-studio UI convention 링크를 함께 보존했다. rebase 후 behind는 0이다.

## 완료 조건 대조

| 조건 | 판정 | 현재 근거 |
| --- | --- | --- |
| manual·tech 지도에서 권위/상세/역사 구분 | 충족 | `mydocs/README.md`, `manual/README.md`, `tech/README.md`와 전체 front matter |
| CLI·시각 검증·PR·정오표·IR canonical 명확화 | 충족 | 각 문서 지도와 canonical manifest |
| 이슈 조사와 장기 기술 문서 분리 | 충족 | `tech/investigations` 32개 이슈 디렉터리, 91개 Markdown |
| investigation과 troubleshooting 경계 | 충족 | 미확정 조사와 확정 재발 방지 자료의 역할을 두 지도에 명시 |
| 기존 링크와 제한된 redirect 보존 | 충족 | redirect 31개, 내부 redirect 재참조 0건, 상대 링크 검사 통과 |
| 저장소 부트로더 확정 | 충족 | 개인 경로 없는 `AGENTS.md`, 짧은 `CLAUDE.md` |
| 새 문서 자동 검증 | 충족 | 디렉터리 단위 링크 383개·메타데이터 378개 검사 |
| 코드 동작 변경 없음 | 충족 | 제품 소스·테스트 변경 0건 |

## 정량 결과

- `mydocs/manual`: 143개 Markdown
- `mydocs/tech`: 170개 Markdown, 루트 53개
- `mydocs/troubleshootings`: 64개 Markdown
- `tech/investigations`: 32개 이슈 디렉터리, 91개 Markdown
- redirect stub: 31개, 모두 `status: superseded`
- 기본 검사 범위에서 redirect를 가리키는 내부 링크: 0건

`tech` 루트의 조사형 이름은 이동 안내용 redirect뿐이다. 비-redirect 번호 문서인
`ci_cache_policy_1664.md`와 `hwp_ole_chart_renderer_architecture_decision_1251.md`는 특정 실험 로그가
아니라 장기 정책·아키텍처 결정이므로 front matter와 tech 지도에서 역할을 명시하고 안정 경로를 유지한다.

## 최종 검증 결과

- `python3 scripts/check_markdown_links.py`: 383개, 이상 없음
- `python3 scripts/check_document_metadata.py`: 378개, 이상 없음
- `python3 -m py_compile scripts/check_markdown_links.py scripts/check_document_metadata.py`: 통과
- `actionlint .github/workflows/docs-link-check.yml`: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- 활성 부트스트랩·문서 지도의 개인 절대경로: 0건
- 제품 소스·테스트 변경: 0건

문서와 문서 검증 인프라만 변경했으므로 Cargo 회귀 테스트는 수행하지 않았다.

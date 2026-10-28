# Task M100 #2072 Stage 22 - 완료 조건 최종 감사

## 목표

이슈 #2072 본문과 maintainer 코멘트의 완료 조건을 최신 `upstream/devel` 기준 전체 변경과 대조한다.
문서 지도, canonical 경계, history 보존, investigation 분리, CI 범위가 서로 모순되지 않는지 확인한다.

## 감사 항목

1. 루트·manual·tech·Codex 문서 지도가 authority/detail/history를 구분하는지 확인
2. CLI, PR, 시각 검증, parser/IR, 스펙 정정 canonical이 하나씩 지정됐는지 확인
3. 조사 자료와 troubleshooting이 운영 가이드에서 분리됐는지 확인
4. 이동 문서의 redirect와 내부 상대 링크가 보존됐는지 확인
5. OWPML reference 자산과 문서 메타데이터가 검사 대상에 포함됐는지 확인
6. CI가 일반 링크·메타데이터만 검사하고 일회성 이전 manifest에 종속되지 않는지 확인
7. 전체 변경에 제품 소스와 동작 변경이 섞이지 않았는지 확인

## 검증 계획

- GitHub issue #2072 본문과 핵심 코멘트 재확인
- 문서 트리, front matter, redirect, investigation/OWPML 자산 정량 검사
- 기본 링크, 전체 변경 Markdown 링크, 메타데이터, Python 구문, `actionlint`, `git diff --check`
- `upstream/devel` 대비 변경 경로 분류

## 완료 조건 대조

| 완료 조건 | 판정 | 근거 |
| --- | --- | --- |
| manual·tech 지도에서 권위/상세/역사 구분 | 충족 | `mydocs/README.md`, `manual/README.md`, `tech/README.md`의 manifest·경계 표 |
| CLI·시각 검증·PR·정오표·IR canonical 명확화 | 충족 | 각 지도에서 canonical과 detail 문서를 분리하고 front matter로 검사 |
| 이슈 조사와 장기 기술 문서 분리 | 충족 | `tech/investigations/` 27개 이슈 디렉터리, 조사 문서 73개 |
| 기존 링크와 redirect 보존 | 충족 | redirect 29개 유지, 활성 내부 Markdown 링크의 redirect 재참조 0건 |
| 코드 동작 변경 없음 | 충족 | 전체 변경 331경로에 제품 소스 없음. 문서·부트스트랩·문서 검사만 변경 |

## 코멘트 보완 조건 대조

1. 문서 링크 검사는 Stage 0에서 추가됐고 현재 CI는 일반 링크·메타데이터 두 명령만 실행한다.
2. `kind`와 `status`를 분리하고 `canonical`, `last_verified`를 독립 필드로 검사한다.
3. Stage 19~21에서 Codex memory와 활성 환경·온보딩·배포 가이드의 현행성을 감사했다.
4. 저장소 루트 `AGENTS.md`를 재현 가능한 부트로더로 추가하고 개인 전역 설정을 계약에서 제외했다.
5. redirect는 외부 이력 호환용으로만 남기고 이동 stage의 `--forbid-path` 검사를 일회성으로 수행했다.
   영구 migration manifest와 문서별 workflow 수정은 제거했다.
6. `tech/investigations/`와 `troubleshootings/`의 미확정 조사/확정 대응 경계를 지도에 명시했다.
   OWPML reference는 XML 7개와 PowerShell 검증 스크립트 1개를 `manual/OWPML SCHEMA/`에 보존하고
   `owpml_schema_reference.md`에서 기술 권위 문서와 사용 경계를 결정했다.

## 루트 잔존 문서 감사

- `tech` 루트 Markdown은 77개이며 이 중 19개는 외부 이력용 redirect다.
- 번호가 포함된 비-redirect 문서는 다음 두 개뿐이다.
  - `ci_cache_policy_1664.md`: `decision` / `historical`
  - `hwp_ole_chart_renderer_architecture_decision_1251.md`: `decision` / `active`
- 두 문서는 이슈별 가설·실험 로그가 아니라 장기 정책·아키텍처 결정이므로 루트에 유지하고, 역할과
  생명주기를 front matter 및 `tech/README.md`에 명시했다.

## 최종 검증 결과

- 기본 Markdown 링크 검사: `374개`, 이상 없음
- `upstream/devel` 이후 전체 변경 Markdown 링크 검사: `328개`, 이상 없음
- redirect 29개 대상 `--forbid-path` 검사: 재참조 0건
- 문서 메타데이터 검사: `211개`, 이상 없음
- `python3 -m py_compile scripts/check_document_metadata.py scripts/check_markdown_links.py`: 통과
- `actionlint .github/workflows/docs-link-check.yml`: 통과
- `git diff --check`: 통과
- 활성 부트스트랩의 개인 절대경로·종료 브랜치 참조: 0건
- 제품 소스 변경 후보: 0건

문서와 문서 검증 인프라만 변경했으므로 Cargo 회귀 테스트는 수행하지 않았다.

## 결론

이슈 #2072 본문 완료 조건과 2026-07-11 maintainer 코멘트의 여섯 보완 조건을 모두 충족한다.
추가 문서 이동은 이 이슈의 완료 조건이 아니라 각 legacy 클러스터의 독립 현행성 감사로 진행할 수 있다.

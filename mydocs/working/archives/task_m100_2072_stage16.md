# Task M100 #2072 Stage 16 - 최신 devel 기준 최종 완료 감사

## 기준

- 이슈: [#2072](https://github.com/edwardkim/rhwp/issues/2072)
- 보완 코멘트: [issuecomment-4943891081](https://github.com/edwardkim/rhwp/issues/2072#issuecomment-4943891081)
- upstream 기준: `4817f2c1b`
- 작업 브랜치: 최신 `upstream/devel` 위에 #2072 단계 커밋을 rebase한 로컬 `devel`

## 완료 조건 대조

| 요구사항 | 확인 결과 |
| --- | --- |
| manual·tech 문서 지도에서 권위·상세·역사 문서를 구분 | `mydocs/README.md`, `manual/README.md`, `tech/README.md`에서 역할과 진입점을 구분 |
| CLI·시각 검증·PR·정오표·IR canonical 명확화 | manifest와 각 문서 front matter의 `kind`, `status`, `canonical`, `last_verified`로 명시 |
| 조사 문서와 장기 스펙·아키텍처 분리 | tech 루트의 실질 `task_*` 조사 문서 0개, 이슈별 자료는 `tech/investigations/issue-*`로 분류 |
| 기존 링크 보존 | 내부 링크 갱신과 제한된 redirect stub 28개를 함께 적용 |
| 코드 동작 변경 없음 | 변경 경로는 문서, 문서 검사 스크립트, 문서 전용 workflow에 한정되며 제품 소스 변경 없음 |

## 보완 코멘트 대조

1. 링크 검사를 Stage 0 문서 전용 CI로 먼저 도입했다.
2. 문서 역할과 생명주기를 분리한 메타데이터를 200개 감사 대상에 적용하고 CI로 검사한다.
3. CLI help, 종료 로컬 브랜치 표현, SVG 예제와 분류 문서의 현행성을 함께 감사했다.
4. 저장소 루트 `AGENTS.md`를 일반 경로만 사용하는 부트로더로 추가하고 CI trigger에 포함했다.
5. 외부 이력 호환이 필요한 redirect만 남기고, stub 유무와 무관하게 옛 경로 64개의 신규 참조를 금지했다.
6. investigation과 troubleshooting 경계를 지도에 명시하고 OWPML XML schema·PowerShell 자산의 reference
   위치와 권위 관계를 확정했다.

## 최종 검증

- Python 검사 스크립트 구문 검사
- 메타데이터 200개 양성 검사와 필수 4필드 누락 음성 검사
- 기본 상대 링크 검사
- 전체 `mydocs` 이전 경로 금지 검사
- redirect 28개와 금지 경로 64개 집합 대조
- canonical 문서의 개인 절대경로·종료 `local/task*` 표현 검사
- `actionlint .github/workflows/docs-link-check.yml`
- `git diff --check`

문서와 문서 CI 스크립트만 변경했으므로 cargo 빌드·테스트는 실행하지 않았다.

## 판정

최신 `upstream/devel` 기준으로 이슈 본문의 완료 조건과 보완 코멘트의 요구사항을 모두 충족한다.
변경 PR이 merge된 뒤 #2072를 close할 수 있다.

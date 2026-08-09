---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4332 검토 — 릴리스 바이너리 설치 진입점

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4332](https://github.com/edwardkim/rhwp/pull/4332) / @kevin9327 |
| base | `devel` |
| 원 PR head | `5237006d60b9f136383ae1279336df2c194712a6` |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4332` |
| 원 변경 규모 | `README.md` 1파일, contributor 커밋 1개 |

원 변경은 릴리스 아카이브 다운로드와 첫 실행 예제를 루트 README에 추가한다. 실행 코드,
renderer, fixture, baseline은 바꾸지 않으므로 visual sweep 대상은 아니다.

## 발견한 차단 결함과 메인터너 보정

`.github/workflows/release-binary.yml`은 `dist/rhwp` 디렉터리 자체를 tar/zip에 넣는다. 따라서
압축 해제 뒤 최상위 `rhwp`는 실행 파일이 아니라 디렉터리다. 원 예제의
`./rhwp capabilities`는 POSIX에서 디렉터리를 실행하려 해 실패한다.

메인터너 보정은 실제 아카이브 구조에 맞춰 첫 실행 경로를 `./rhwp/rhwp capabilities`로
수정한다. contributor 커밋은 rewrite하지 않고 원 head 뒤의 single-parent 후속 커밋으로
추가한다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| 릴리스 workflow의 POSIX 패키징 명령 대조 | `tar ... -C dist rhwp`로 최상위 `rhwp/` 확인 |
| Windows 패키징 명령 대조 | `Compress-Archive -Path dist/rhwp` 확인 |
| README 명령 경로 계약 검사 | 압축 해제 뒤 `rhwp/rhwp`를 실행하도록 확인 |
| Markdown 상대 링크 검사 | 변경 README와 review 문서 통과 |
| `git diff --check origin/pr/4332..HEAD` | 통과 |
| 시각·Cargo 검증 | 생략. 문서 명령 한 줄과 review 기록만 변경 |

## 리스크와 권고

- 실제 GitHub release 자산을 내려받는 네트워크 E2E는 로컬에서 실행하지 않았다.
- push 뒤 최신 head의 required checks와 mergeability를 다시 확인한다.
- merge는 작업지시자의 별도 승인 전에는 수행하지 않는다.

**최신 head CI 통과와 실제 release 자산 구조 재확인 후 조건부 merge 권고.**

---
kind: pr_review_impl
status: active
canonical: mydocs/pr/pr_4385_review.md
last_verified: 2026-08-10
---

# PR #4385 메인터너 보정 실행 기록

## 커밋 경계

| 구분 | SHA / 내용 |
| --- | --- |
| contributor source | `fb0b069803a14bcaa5ba8c99d965633d0e22d23f` |
| first maintainer correction | `a595e58c8c5bfcfe0e48c5f373341844aca86196` — `docs(roadmap): distinguish #4385 evidence states` |
| follow-up docs correction | `5b32cff82a930c845859b2386e6fc742f7ac6e70` — `docs(roadmap): correct #4385 W32 evidence states` |
| trailing review update | 이 문서를 포함한 `docs(pr): update #4385 W32 status review` commit |

## 실행 내용

1. 각 arXiv ID의 abstract/version history를 대조하고 2604.17025의 v1/v2/v3 날짜를
   다시 고정했다.
2. GitHub에서 #4385 source head와 #4330/#4364/#4371/#4381의 current state/head를
   읽기 전용으로 확인했다.
3. 1차 보정 `a595e58c...`을 보존하고 Markdown·SVG의 W32 절대 시점, atlas,
   R67/R83, R101~R200와 기본 run/CAS 상태를 후속 docs commit에서 정정했다.
4. Markdown 링크·metadata·날짜·상태 문구와 SVG XML well-formed·위험 요소를 검사했다.
5. 이 review update를 docs correction 뒤의 별도 single-parent commit으로 추가하고,
   원 head부터 merge commit 없는 선형 history인지 확인했다.

## rollback

후속 상태 보정에 문제가 생기면 trailing review update와 `5b32cff8...`을 역순으로
revert한다. 메인터너 보정 전체를 제거해야 할 때만 그 뒤 `a595e58c...`도 revert한다.
contributor source는 amend, rebase, reset 또는 force-push하지 않는다.

# PR #2316 최종 보고 — legacy /web 개발 앱과 current tooling 결합 제거 (#2313)

- PR: https://github.com/edwardkim/rhwp/pull/2316 (postmelee)
- 결정: **merge** (dc6d94c2, 2026-07-17)
- 검토 기록: `pr_2316_review.md` (구조 검토 + 작업지시자 요청 정밀 평가)

## 경과

1. 검토: tracked `web/` 18 entries(616KB) 제거 + CI/metrics/font contract 의
   `/web` 결합 정리. 로컬 재실증 전부 green. 충돌은 orders 문서 add/add 1건.
2. 작업지시자 정밀 평가 요청 → 4개 축 전수 실증:
   - 소비처 스캔: Pages(`studio/dist`만)/npm(pkg 신선 빌드)/release/확장
     4종/e2e 전부 소비 0
   - 제거물 분류: stale wasm glue 377KB(이중 진실) + 휴면 legacy 앱
     224KB(2026-04-07 이후) + Python 서버·tracked private key + 호환 심링크
   - metrics 함수 단위 대조: 함수 -149/CC -828/CC>25 -4건·-207 = legacy 그룹
     수치와 정확 일치, **non-legacy 7개 그룹 전 수치 완전 동일**
   - 문서: 규범 매뉴얼 갱신 + 잔존 언급은 기준-commit 스냅샷(불변 원칙)
3. 충돌 union 해소를 head 브랜치에 직접 push(maintainer edit, f94a0d85) →
   CI 전 항목 재통과 → approve + merge + 리팩토링 성과 감사 코멘트.

## 남은 항목

- #2313 close 승인 대기 (devel 반영 검증 후)

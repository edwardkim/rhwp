# PR #2388 검토 — 템플릿 문서 PrvText placeholder 보정 (donggyun112 2번째)

- PR: https://github.com/edwardkim/rhwp/pull/2388 — Closes #2387 (자기 등록)
- 직렬화/한컴 호환 영역 — 트러블슈팅 사전 검색 수행 (관련 기록 없음, 스펙
  문서의 PrvText 정의만 확인)

## 변경 본질

템플릿 생성 문서가 placeholder PrvText("\r\n")를 물고 나가 탐색기·한컴
미리보기에 빈 문서로 보이던 버그 — `supplement_preview`: **원본이 실재하면
불변**(라운드트립 보존), 없거나 placeholder 일 때만 본문 문단으로 생성
(표/글상자 제외, 한컴 실측 ~1000자 절단). PrvImage 는 렌더러 경유 필요로
범위 외 명시 — 스코프 규율.

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| red→green | devel src 원복 → 2/2 FAIL → 복원 2/2 |
| `cargo test --tests` | 실패 0 (roundtrip 계열 포함) / clippy 0 |
| fmt | 1건 — maintainer edit 정리 push(d921c465) 후 통과, CI 전 항목 green |

## 판단

**merge 권고.** 원본 보존 우선 원칙과의 정합(보존 조건 명시)이 정확 —
같은 날 두 번째 기여, 둘 다 이슈 자기 등록 + red→green.

## 처리 결과 (2026-07-18)

merge 완료(admin) + PR·이슈 #2387 메인테이너 코멘트 게시. donggyun112 같은 날
두 번째 기여 완결 — #2387 close 는 승인 대기.

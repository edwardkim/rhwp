# PR #9273 검토 — fix(table): 쪽보다 길게 자란 중첩 표 행을 쪽 경계의 글줄 사이에서 나눈다

## 2026-11-21 메인테이너 보완 착수

원 [PR #9273](https://github.com/edwardkim/rhwp/pull/9273)의 head는 `7d8d6dc56caaef30893e1ed1cb6442a27597dfee`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

기여자가 보고한 70문단 사례는 3쪽이 한컴보다 한 줄 일찍 끝난다. 4쪽 Native 87.67%로 페이지별 90% 기준에 미달한다. 이어받은 행의 점유와 줄 경계를 보정한 뒤 4쪽까지 줄의 소속·누락·중복을 독립 Print와 Native·fresh WASM으로 재검증한다.

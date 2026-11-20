# PR #9265 검토 — fix(clipboard): 각주·개체 뒤 논리 캐럿의 내부 붙이기와 표·개체 글자 모양 상속

## 2026-11-21 메인테이너 보완 착수

원 [PR #9265](https://github.com/edwardkim/rhwp/pull/9265)의 head는 `0a94a8710270d219590a5ae73678df0f220cbd64`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

인접한 수식 사이에 캐럿을 놓고 내부 붙여넣기할 때 입력이 어느 수식 뒤로 가는지 확인했다. 공개 선언에 pasteInternalLogical도 빠져 있다. 인접 개체 사이의 논리 위치를 보존하고 본문 붙여넣기 및 공개 선언을 검증한다.

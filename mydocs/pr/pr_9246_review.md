# PR #9246 검토 — fix(typeset): 단 영역을 넘친 각 단 아래 각주는 같은 쪽 다음 단에 잇고, 오른쪽 단 아래 각주는 오른쪽 단을 다 쓴다

## 2026-11-21 메인테이너 보완 착수

원 [PR #9246](https://github.com/edwardkim/rhwp/pull/9246)의 head는 `c5cbd9851e24832d037fe0ec69fbbb8ac21e3dae`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

8개 HWP가 포함되어 있고 기여자가 측정한 사례는 Native 최저 98.07%다. 구역 끝 overflow-e2-tail의 Native 실루엣과 fresh WASM 전쪽 검증이 빠졌다. 8개 원본의 대응 한컴 Print와 각주 이월·줄 재배치·HWPX 재열기 상태의 양 backend 결과를 보완한다.

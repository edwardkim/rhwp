## 요약
- HWP3-origin sample16 3쪽 본문 세로 배치를 한컴 3mm 격자 기준에 맞춰 보정했습니다.
- 2022 저장본의 BCP 꼬리 줄 처리와 k-water 2024 RowBreak 표 절단 위치를 정정했습니다.
- `HCI Poppy` 등 legacy Latin 폰트가 HWP3/HWP5 변환본에서 일관되게 한컴 HFT 치환 결과로 해석되도록 보정했습니다.
- 최종 보고서를 추가하고, 내부 타스크 PR 생성은 별도 승인 후 진행한다는 절차를 문서화했습니다.
- PR #1120 CI에서 발견된 `spacing_before` 사전 차감 회귀를 수정해, 일반 문서 경로와 HWP3-origin 예외 경로를 분리했습니다.
- #1116의 SVG/browser 폭 고정 출력 변경에 맞춰 SVG golden 스냅샷을 갱신했습니다.

## 검증
- cargo test --lib
- cargo test --test issue_1116 -- --nocapture
- cargo test --test issue_1105 -- --nocapture
- cargo test --test issue_1086 -- --nocapture
- cargo test --test issue_1035_alignment -- --nocapture
- cargo test --test issue_713 -- --nocapture
- cargo test --test svg_snapshot -- --nocapture
- cargo fmt --all -- --check
- cargo build --bin rhwp
- git diff --check
- sample16 변환본 5종의 3mm SVG를 재생성하고, Latin glyph가 `Palatino=6`, `HCI=0`으로 해석되는지 확인했습니다.

관련 이슈: #1116

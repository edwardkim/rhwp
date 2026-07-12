# PR #1927 검토 — Task #1880: convert-HWP 자리차지 표 host_before 비대칭 + BinData placeholder

- 작성일: 2026-07-05 / planet6897 → devel / 시간순 2/7 (04:16)

## 요지
① `format_table` 자리차지 판정을 원시 attr 비트에서 **의미 필드(text_wrap) + is_hwpx_source
게이트**로 교체 — HWPX 파스(attr 미채움)와 HWP5 재파스(raw attr)의 비대칭 해소. 3075729
convert 렌더 p12→p13(한컴 2022 오라클 정합), A/B 2,005건 신규 divergence 0.
② BinData 로드 실패 시 빈 placeholder 등록으로 pic 컨트롤 왕복 보존(#1917 연계, 103.7MB
실표본 PASS 실측).

## 검토
- 컨트리뷰터가 **#1924와의 중복(#1917 상한)을 스스로 정리** — 배치/개별 PR 정합 관리 양호.
- ir-diff 사각지대(원시 attr 직독) 진단이 정확. 잔존 3건 별개 클래스 분리 기록.
- 참고(비차단): is_hwpx_source 분기 1곳 신설 — 리팩토링 Phase P(Provenance) 수렴 대상
  인벤토리에 추가될 사이트. 기능 수정으로는 정당(결정론 마커 계열).

## 판단
머지 권고. #1928과 동일 라인 충돌 → 메인테이너 통합(아래 참조).

## 결합 게이트 결과 (devel `d982067b` + 7건 시간순 통합, 2026-07-05)

fmt 통과 / clippy 경고 0 / `cargo test --profile release-test --tests` **2,889 통과·실패 0**
(신규 핀 11건 포함) / OVR baseline 5샘플 **개체 회귀 0건** (기준 00014ecf)

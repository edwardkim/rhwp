---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4967.md
last_verified: 2026-08-25
---

# Issue #4967 — W8 font face 교정 qualification

이 디렉터리는 W8 tracker의 첫 process canary인 rank 8 `KoPubWorld바탕체 Light`의 교정 적격성 증거를
보존한다. 현재 단계는 제품 font mapping 변경이 아니라 기존 W3·W4·W5·W7.5 증거의 호환성과 실사용
cohort를 판정하는 query 단계다.

## Stage W8-Q0 경계

- private W3 journal을 다시 parse하거나 10k corpus를 재실행하지 않는다.
- rank 8을 실제로 사용한 문서의 경로·이름·본문·hash는 owner-only local output에만 둔다.
- tracked baseline에는 aggregate, evidence digest와 privacy gate만 남긴다.
- W5 exact/subst/missing Hyper-V ladder는 재사용하며 이번 단계에서 VM을 실행하지 않는다.
- v2 registry와 다섯 runtime projection은 읽기 전용이다.

재현 도구는 `scripts/font_rank8_qualification.py`, 계약 테스트는
`scripts/tests/test_font_rank8_qualification.py`다. local-only 입력이 있는 메인테이너 환경에서 projector를
실행하면 `rank8_private_cohort.json`은 mode `0600`, 공개 baseline은 mode `0644`로 생성된다.

# PR #6629 검토 - 1x1 wrapper 표 padding

- 원 PR head: `965297c219c29e3908c600b76b2b7683654840b6`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: `394ffc23b` 및 이 trailing maintainer 보정
- reviewer: `jangster77` 요청 완료

## 판정: 메인터너 보정 됨 수용 가능

원 변경은 wrapper cell의 padding을 inner table과 외곽 테두리에 반영했다. 실제 CI/전체 회귀에서 native HWP5 wrapper의 선언 높이가 padding 처리 뒤 축소되던 반례가 드러났다. 보정은 HWP5 stored-pagination profile에만 선언 outer height를 하한으로 보존하며, HWPX 및 일반 padding 경로는 바꾸지 않는다.

## 검증 및 증적

- wrapper padding과 nested border focused 회귀 `4/4` 통과, HWPX scope 회귀 `1/1` 통과.
- 전체 nextest: `8951 passed`, `46 skipped`; host/WASM clippy, workspace build, test manifest/tier, native WASM build 통과.
- `exam_social.hwp` 1쪽과 한컴 2022 PDF 직접 비교의 전체 page diff는 `23.84%`다. 이는 표 내부의 기존 text shaping 차이를 포함하므로 wrapper 외곽선·선언 높이 판정은 focused 좌표 계약으로 한정했다.
- [stable review PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6629_issue6621_exam_social_p001.png)

원 PR은 직접 merge하지 않고, 이 메인터너 보정을 포함한 승인된 통합 PR에서만 수용한다.

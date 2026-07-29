# PR #3534 검토 기록 — HWPX/HWP3 무손실 결함 6건

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3534](https://github.com/edwardkim/rhwp/pull/3534) — 저장·파싱 무손실 결함 6건 (#2790, #2778, #2782, #3050 및 CDATA) |
| 작성자·검토자 | `@kevin9327` · `@jangster77` |
| source head / 통합 commits | `8d5a171d005d6aebbc592a31311a26ed24eaba32` / `b767004a1`, `f4c7de75e`, `e2b99c43e` |
| 적용 순서 | #3539가 이 PR 위에 적층됐으므로 세 고유 commit을 먼저 `-x` 체리픽 |

공용 도형 `textFlow`, 수식 크기 기준·overlap, 본문/양식 개체 CDATA, HWP3 `ch=5` field code의
저장·파싱 손실을 고친다. HWP3 dispatch는 control을 직접 push하지 않고 tail에서 정확히 한 번
생성하도록 역할을 분리한다. HWPX 속성은 파서가 IR에 보존한 값을 serializer가 하드코딩으로 버리지
않도록 연결한다.

## 검증과 판정

- #3539와 의도적으로 같은 dispatch 문맥을 공유한다. #3534를 먼저 적용한 뒤 #3539의 고유 `ch=6`
  commit만 적용해 중복 source 적용을 피했다.
- CDATA 저장/재파싱, HWP3 control 개수, HWPX serializer 범위는 release library와 전체 test gate로
  통합 확인한다.
- 원 PR이 닫으려던 #2790·#2778·#2782·#3050은 원 PR merge가 아닌 통합 PR 실제 merge 후에만 후속 처리한다.

공통 결함 클래스만 묶고 각 경로에 회귀를 둔 변경으로, 전체 gate 조건 아래 **기술적 수용 가능**이다.

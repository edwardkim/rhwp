# PR #3539 검토 기록 — HWP3 책갈피 control 중복 생성

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3539](https://github.com/edwardkim/rhwp/pull/3539) — HWP3 `ch=6` bookmark control 중복 push (#3538) |
| 작성자·검토자 | `@kevin9327` · `@jangster77` |
| source head / 통합 commit | `86b1cb173affc0a41b96d94336c15cb7ee05baac` / `e82bc568c57e911de14dfe403d9b9087d245a254` |
| 적층 처리 | source에 포함된 #3534 세 commit은 이미 적용돼 고유 마지막 commit만 체리픽 |

`ch=6` bookmark가 dispatch와 tail catch-all에서 두 번 Control로 push되어 U+FFFC 하나 뒤의
문자/control 정렬이 계속 밀리던 결함을 고친다. dispatch는 raw info만 전달하고 tail에서 bookmark
Control을 한 번 생성한다. 이는 #3534 `ch=5` field code와 같은 제어 흐름을 공유하지만 이름·종류
payload의 바이트 의미는 유지한다.

## 검증과 판정

- 고유 회귀는 bookmark control·data record가 각각 정확히 하나인지와 이름·종류 보존을 단언한다.
- #3534와 적층을 해제한 통합 적용 순서에서 불필요한 source 중복과 conflict churn이 없다.
- release library·전체 test gate에서 HWP3 parser와 control 정렬 범위를 추가 검증한다.

control 생성 책임을 단일화하고 기존 payload를 보존하므로 **기술적 수용 가능**이다.

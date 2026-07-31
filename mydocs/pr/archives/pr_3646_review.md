---
kind: pr-review
status: active
---

# PR #3646 검토 — 수식 `instid`의 스키마 비허용 방출 제거

| 항목 | 값 |
| --- | --- |
| 작성자 / reviewer | `@JamesPsh` / `@jangster77` |
| 원 PR / 관련 이슈 | [#3646](https://github.com/edwardkim/rhwp/pull/3646) / [#3543](https://github.com/edwardkim/rhwp/issues/3543) |
| 원 head 참고값 | `2be15696bd7adba6622ac1cbe584662e2c13a5b3` |
| 통합 후보 | [#3659](https://github.com/edwardkim/rhwp/pull/3659) `00450ccb1cb075c688b037462c760e1f4dd23700` |
| 원 변경 규모 | 1 file, +17 / -1 |
| 권고 | #3659로 수용 |

## 변경과 통합 판정

`hp:equation`은 공용 도형 컴포넌트가 아니므로 `instid` 속성을 가질 수 없다. 이 PR은 수식
직렬화 경로에서 그 속성만 제거하고, `id`는 유지하는 `equation_omits_instid` 회귀를 추가한다.
파서도 수식의 `instid`를 모델에 보존하지 않으므로, 변경은 기존 IR 왕복 모델과도 일치한다.

원 head의 `devel` 병합 commit `2be15696b`은 통합에서 제외했다. 기능 commit `4b4eed64a`는
통합 branch에서 `30d850618`으로 patch 동등하게 적용됐으며, 충돌은 없었다. `hp:chart`의
별도 스키마 위반 가능성은 공용 경로의 범위가 달라 이번 PR에 섞지 않았다.

## 검증

| 검증 | 결과 |
| --- | --- |
| 원 #3646 head CI | lint, Native Skia, default-feature 8 shards, `Build & Test`, CodeQL success |
| 통합 #3659 code head CI | 동일 full CI, CodeQL 및 `Build & Test` success |
| 체리픽 동등성 / diff | `git range-diff` 동등, `git diff --check` 통과 |
| 추가 로컬 Cargo | 원 PR 및 정확한 통합 head CI와 중복되므로 작업지시에 따라 실행하지 않음. 성공 근거로 사용하지 않음 |

원 PR CI는 [CI run](https://github.com/edwardkim/rhwp/actions/runs/30627555187), 정확한 통합
code head CI는 [#3659 CI run](https://github.com/edwardkim/rhwp/actions/runs/30627974311)에서 확인했다.

## 증적 범위

이 변경은 HWPX serializer의 속성 계약만 바꾸며 renderer·layout·typeset 경로를 건드리지 않는다.
따라서 PDF 정합을 새 성공 근거로 만들지 않았다. 대신 회귀는 `render_equation` 결과에 `instid`가
없고 필수 `id`는 남는지를 직접 고정하며, 통합 CI의 Canvas visual diff도 성공했다.

## 권고와 merge 전 조건

**권고: 수용.** #3659의 code head full CI는 성공했고 상태는 작성 시점 `CLEAN`·`MERGEABLE`이다.
archive review·통합 계획·오늘할일만 추가한 review-only head의 fast-pass를 확인한 뒤 #3659을
merge한다. merge 뒤 #3543 close 상태, #3545 open 상태, 원 PR #3646의 supersede 처리와
contributor 결과 comment, `devel` 동기화와 검토 ref 정리를 확인한다.

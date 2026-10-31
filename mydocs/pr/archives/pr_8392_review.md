# PR #8392 리뷰 — verdict protocol의 자동 CI 실행 제외

## 최종 판정

**승인 — #8391의 선택적 제외와 다른 검사 보존을 확인했고, 정확한 code head의 Full CI가 성공했다.**

이 판정은 self-review 문서 기록이다. GitHub의 공식 APPROVE 리뷰는 제출하지 않았다.
사용자의 게시·self-review·정상 병합 승인을 확인했다. merge 전에는 이 문서를 포함한 최신 trailing
head의 필수 검사 성공, mergeability와 exact head를 다시 확인한다. 보호 규칙 우회는 사용하지 않는다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| PR·작성자·base | [#8392](https://github.com/edwardkim/rhwp/pull/8392) / edwardkim / devel |
| 관련 이슈 | [#8391](https://github.com/edwardkim/rhwp/issues/8391), `Closes #8391` |
| 검증한 code head | `64c2c4d4e165408965fad3b4476edba16297902c` |
| 구현·검토 base | `cb9c45cf774a03eddb5d56fdb1fd28919f10a969` |
| 작성 시점 원격 상태 | Open, MERGEABLE, CLEAN, reviewer 지정 및 공식 review 없음 |
| 절차 | collaborator_self_merge + intake_and_review + local_validation + review_only_fast_pass + post_merge |

## 변경과 검토 결과

- 내부 `cargo test --workspace … --lib`, workspace build, workspace all-targets Clippy의
  세 명령에 `--exclude llm-verifier-verdict-protocol`만 추가했다.
- 기존 내부 테스트 제외 집합에 대상 하나만 더해졌고, build·Clippy의 다른 workspace 멤버는
  모두 유지됐다. `--lib`, `--all-targets`, `-D warnings`도 유지됐다.
- 명시적 verifier 계약 7개, `claim_bind`, `criteria_decomp`, root archive의 `--package rhwp`와
  native·WASM32 제품 Clippy가 유지됐다. 기존 workflow 계약 실행 경로에서 새 검사를 수집한다.
- Cargo manifest·lockfile·workspace 구조, 대상 소스·스키마·2,240개 합성 코퍼스를 변경하지 않았다.
  수동 `cargo test -p llm-verifier-verdict-protocol` 경로도 유지됐다.
- 공통 fmt와 CodeQL 등 보안 검사, event, 권한, trusted classifier, required check와 배포 정책을
  변경하지 않았다. 자동 실행 제외 범위는 이슈에서 정한 build/test/Clippy다.
- 실제 diff와 호출 경로에서 수정이 필요한 결함이나 범위 밖 변경을 발견하지 못했다.

## 검증 입력과 결과

[구현·로컬 검증 기록](../../working/task_m100_8391_stage1.md)을 대조했다. 선택적 제외 회귀는
기존 workflow에서 제외 누락으로 실패했고 수정 후 통과했다. 실제 Cargo artifact에서도 내부
테스트는 나머지 5개, build·Clippy는 나머지 9개 workspace 멤버를 선택했다.

| 로컬 검사 | 결과 |
| --- | --- |
| Python CI·보안·배선 계약 | 112개 통과, 선택적 제외 회귀 6개 포함 |
| CI impact Node 계약 | 95개 통과 |
| 내부 Rust 테스트 | 229개 통과 |
| 유지 대상 verifier 7개 | 222개 통과 |
| 대상 패키지의 수동 전체 테스트 | 단위 19개 + integration 11개 통과 |
| workspace build·all-targets Clippy·공통 fmt·YAML·diff 검사 | 통과 |

code head `64c2c4d4e165408965fad3b4476edba16297902c`에 대한 원격 결과도 확인했다.

| GitHub Actions | 실제 결과 |
| --- | --- |
| [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/41058179664) | Full 실행 성공. Lint, archive A~E build·test, Native Skia, frontend package 성공. archive/worker 총계 11,995/11,995 일치 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41058179754) | Rust·Python·JavaScript·Actions 분석 성공 |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/41058179635) | 성공 |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/41058179753) | 성공 |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/41058179609) | 성공 |

성공한 원격 Lint 로그에서 내부 Rust 229개와 verifier 7개의 222개 테스트 실행도 직접 확인했다.
코퍼스 integration test가 종전 CI에서 실행됐거나 판정 정답을 검증했다고 해석하지 않는다.

## 조판 원칙·시각 증적·검증 입력

비해당: CI 패키지 선택과 그 workflow 계약만 바꾸며, 제품 소스·데이터·조판 소비 경로에 변화가 없다.
별도 HWP/HWPX/PDF 시각 검증 입력을 사용하지 않았고 렌더링 회귀·golden·baseline도 바꾸지 않았다.
새 base 반영을 위한 rebase·source 보정은 없었다. 원격 Full CI가 확인한 제품 전체 회귀를 다시
로컬에서 중복 실행하지 않았다.

## 남은 병합 절차와 복구

이 review와 오늘할일만 single-parent trailing commit으로 추가한다. push 전 최신 base와 trailing
head의 merge-tree 종료 코드, 공백·merge tree 링크·기존 오늘할일 보존을 검사한다. push 뒤에는
trusted controller의 실제 reuse/Full 판정과 최신 head의 필수 CI를 확인한 후 정상 squash merge한다.
이 소형 변경의 실행 순서와 rollback은 이 문서로 충분해 별도 review_impl은 만들지 않았다.

되돌릴 때는 세 Cargo 명령의 제외와 대응 회귀 계약을 함께 revert한다. 다른 검사나 package 소스의
삭제·재구성은 필요하지 않다. 원격 작업 브랜치는 사용자 지시에 따라 보존하고 배포는 수행하지 않는다.

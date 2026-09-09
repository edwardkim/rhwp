# PR #6773 검토 기록

## 판정: 승인

메인터너 보정이 원 PR에 반영된 코드 head `66b69a46a6b9736c35893246c56be4148c525e84`를 승인한다.
원 contributor head의 보류 사유는 해소됐다. 이 판정은 아직 생성하지 않은 문서 trailing head의 CI 성공이나
병합 완료를 뜻하지 않는다. 최종 head의 CI와 `MERGEABLE/CLEAN`을 다시 확인한 뒤 병합한다.

## 검토 대상

| 항목 | 확인 내용 |
| --- | --- |
| PR | [#6773 셀·글상자 안 표 삭제 by_path](https://github.com/edwardkim/rhwp/pull/6773) |
| 작성자 | `zlzlzlmo` (신승훈), rhwp 첫 기여. 확인 시점 이전 병합 PR 없음 |
| 관련 이슈 | [#6771 셀 안 표 컨트롤 삭제 API 부재](https://github.com/edwardkim/rhwp/issues/6771), 본문 `Closes #6771` |
| 대상 브랜치 | 잘못 지정된 `main`을 `devel`로 변경 |
| 원 contributor head | `77c8dc68a94605285bd1010fa80d2b51f2e0579d` |
| devel 호환 병합 | `1e6edb482`, 기준 `92f6242af96f51ede912588fa6fe35f5447709bc` |
| 메인터너 보정 | `66b69a46a6b9736c35893246c56be4148c525e84` |
| 코드 고유 diff | 5개 파일, 추가 368줄 / 삭제 11줄. source 3개, integration test 2개 |
| 로컬 검토 브랜치 | `review/pr6773-maintainer-20260909` |
| 원격 source | `zlzlzlmo/rhwp:feat/6771-delete-cell-table-by-path`, `maintainerCanModify=true` |
| CI 완료 후 참고 상태 | `MERGEABLE`, `CLEAN`; 최종 문서 head에서 재확인 필요 |

## 기여 구현과 메인터너 보정

- 기여 구현은 `deleteCellTableControlByPath` WASM API와 native wrapper를 추가했다. 기존 그림 삭제와
  공통 헬퍼를 사용하고 컨트롤 종류를 검사한다. 이슈에서 제시한 경로 기반 삭제 대안을 충족한다.
- 기존 CI 실패는 삭제 헬퍼에 `section.raw_stream = None`이 없어서가 아니라, 공통화한 두 wrapper가
  무효화 guard에 미분류였기 때문이다. 두 wrapper를 실제 헬퍼에 대한 `DelegatesTo`로 등록했다.
  guard 삭제, skip, `Pending` 예외 또는 기준선 완화로 우회하지 않았다.
- 표 삭제에도 `PictureDeleted`가 기록되던 의미 오류를 수정했다. 표에는 `CellTableDeleted`와 정확한
  중첩 경로·내부 컨트롤 번호를 기록하고, 기존 그림 삭제 이벤트 계약은 유지했다.
- 공개 실물 문서의 저장·재열기, 중첩 셀·글상자 경로, 텍스트와 오프셋 보존, 양방향 잘못된 종류 거절,
  기존 그림 삭제 회귀를 보강했다. source-side `cfg(test)`를 늘리지 않고 integration test에 추가했다.
- devel 동기화 충돌에서 기존 main 쪽 Gym workflow와 Native Skia timeout을 되살리지 않고 devel 정책을
  유지했다. 최종 PR 고유 diff에는 workflow 변경이 없다. contributor 이력은 rewrite하지 않았다.

## 완료한 로컬 검증

검토 전용 `CARGO_TARGET_DIR=target/pr6773-maintainer-20260909`를 사용했다. 최종 전체 회귀는 8 threads였다.

| 검증 | 실제 결과 |
| --- | --- |
| #6771 집중 회귀 | 6개 통과 |
| #2724 passthrough 무효화 guard | 5개 통과; 위임 대상의 실제 무효화까지 확인 |
| 전체 integration 회귀 | `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr6773-maintainer-20260909 --tests --no-fail-fast --test-threads 8`: 9,358개 통과, 실패 0, skip 46, 341.658초 |
| 테스트 suite | `--prepare` 후 최종 `--check` 성공, 48/48 integration targets |
| source-side 정책 | 4,205 tests / 298 modules, 기준선 증가 없음 |
| 포맷 | `cargo fmt --all -- --check` 성공 |
| Clippy | native, WASM32 library, workspace all-targets 모두 `-D warnings` 성공 |
| workspace 빌드 | `cargo build --locked --workspace` 성공 |
| WASM 패키지 빌드 | `scripts/wasm-pack-locked.sh --target web` 성공, wasm-opt 완료 |
| 실제 WASM JavaScript API | 공개 HWP에서 표 삭제 후 HWP 257,024바이트 / HWPX 281,459바이트로 저장·재열기 성공; 같은 표 재삭제 거절 |
| Native Skia library | rhwp 3,930개 통과 / 13개 ignored, workspace 보조 library 15·165·2개 통과 |
| Native Skia 집중 검사 | 그림/PNG 계약 2개, 직접 PDF export 4개 통과 |

초기 추가 테스트의 이벤트 JSON envelope 가정과 합성 문서 첫 문단의 section 정의 위치를 수정한 뒤 집중
6개를 재실행했다. 최종 all-target Clippy의 `Box::default()` 스타일 보정 뒤에도 suite를 재준비하고
위 전체 회귀를 다시 완료했다. 중간 실패 또는 0개 실행을 통과 증거로 사용하지 않았다.

## 코드 head의 GitHub CI

아래 결과는 모두 `66b69a46a6b9736c35893246c56be4148c525e84`에 대응한다.

- [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34357106608): 성공.
  Lint, Native Skia, Archive A/B/C/D builder·test worker와 실행된 frontend package gate 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34357105471): Rust·JavaScript/TypeScript·Python 분석 성공.
  GHAS CodeQL check도 최종 성공을 확인했다.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34357105284): Canvas visual diff 성공.
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34357105445): 성공.
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34357105562): 성공.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/34358850487): 성공.
- workflow 조건상 WASM Build·frontend unit·promotion·PR duration refresh skip은 실제 수행한 로컬
  WASM 빌드 및 실행된 다른 CI worker 결과와 구분한다.

## 증적 범위와 잔여 위험

- 실물 fixture는 기존 공개 `samples/2022년 국립국어원 업무계획.hwp`의 section 0 / parent paragraph 20 /
  cell 0 / cell paragraph 2 / inner control 0이다. HWP/HWPX 재열기 후 삭제 상태와 주변 텍스트 보존을 확인했다.
- 이번 변경은 표 컨트롤 삭제 API와 이벤트 계약이며 레이아웃 알고리즘 수정은 아니다. 로컬 판단은 문서
  구조·저장·API 실행 증거에 근거한다. 별도의 한컴 PDF 대조나 Studio 화면 visual sweep은 실행하지 않았다.
  CI Render Diff 성공을 해당 fixture의 삭제 전후 화면 직접 검토로 바꾸어 표현하지 않는다.
- 이슈 작성자의 미제공 원 문서 5개 위치를 전수 검증했다고 주장하지 않는다. 공개 실물 fixture와
  중첩 셀·글상자 합성 fixture로 요청된 경로 기반 삭제 기능을 확인했다.
- `deleteControlAt`의 모든 본문 밖 리스트 지원이나 Studio UI 연결까지 확장한 PR이 아니다.
  호출자는 새 `deleteCellTableControlByPath` API를 사용한다.
- 로그, 임시 WASM package, 파생 suite, 중간 JSON·PNG·SVG는 커밋하지 않는다.

## Merge 후 contributor PR comment 계획

1. 이 리뷰와 오늘할일만 같은 source branch의 trailing commit으로 push한다. 최신 head의 preflight,
   required aggregate, CodeQL 및 필요한 worker 성공/정책상 skip을 확인한다. 문서라는 이유로 CI를 우회하지 않는다.
2. 최종 SHA를 다시 확인하고 일반 merge commit으로 병합한다. devel 동기화와 merge SHA의 실제 CI를 확인한다.
3. [#6771](https://github.com/edwardkim/rhwp/issues/6771)의 auto-close 상태와 기존 댓글을 확인한다.
   같은 merge SHA·검증의 완료 댓글이 없으면 UTF-8 body file로 한 번 기록하고, OPEN이면 종료한다.
4. 원 PR에는 첫 기여 감사·환영, 기여자의 경로 기반 API, 메인터너의 guard·이벤트·회귀 보정을 구분해 적는다.
   확정 merge SHA 링크, 최종 PR/devel CI URL, 이 리뷰의 merge SHA 고정 링크, 실제 로컬 결과를 포함한다.
5. 별도 시각 asset은 생성하지 않았으므로 가상의 이미지·PDF·pixel 수치를 게시하지 않는다.
   API·저장 검증 범위와 미실행 화면 검증을 정확히 적는다. 게시 후 API로 본문을 재조회하며 중복 게시하지 않는다.
6. 기본 작업공간이 clean이고 실행 중 작업이 없으며 merge SHA의 devel 포함을 확인한 뒤 이번 local branch와
   전용 target만 정리한다. 별도 worktree·upstream 임시 branch는 만들지 않았다.
   원격 정리 승인과 별개로 첫 기여자 지침에 따라 contributor fork branch는 보존한다.

처리 순서와 보정 SHA가 위에 명시된 소형 단일 PR이므로 별도 implementation 문서는 만들지 않는다.
문서 trailing push·병합·후속 댓글·정리에 대한 작업지시자 승인을 받았다. 이 기록의 작성 시점에는 병합과
이슈 종료를 아직 수행하지 않았다.

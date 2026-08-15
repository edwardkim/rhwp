# PR #4798 검토 - 손상된 위치 축의 HWPX LineSeg 저장 억제

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#4798](https://github.com/edwardkim/rhwp/pull/4798) |
| 통합 PR | [#4801](https://github.com/edwardkim/rhwp/pull/4801) |
| 관련 이슈 | `Closes #4778, #4677` |
| 작성자·검토 방식 | `planet6897` · collaborator 체리픽 통합 self-review |
| 원 base / head | `devel` / `bfe531ad50dd7a48e065f6013a274c3d572cc70a` |
| 적용 commit | `c7e2f1fe5` |
| 통합 code candidate | `c7e2f1fe5586eca576daeb58495da5718f7eebfc` |
| 규모 | 1 file, +80/-12 (원 PR 기준) |
| 라우팅 | collaborator external PR · intake/review · local validation |

원 기능 commit은 최신 `upstream/devel@44bcba400072128bdc4e4d6c05bf822e3ff60996` 위에 충돌 없이
체리픽했다. 이 기록은 code candidate Full CI 성공 뒤 추가하는 review-only trailing 문서이므로,
merge 직전에는 이 문서 head의 fast-pass와 최신 mergeability를 다시 확인한다.

## 변경 범위와 판단

- 문단 run의 위치 축이 연속적이지 않으면 저장된 `linesegarray`를 방출하지 않는다. 손상된 위치로
  재사용되는 line segment가 한글 본문을 폐기하는 경로를 차단한다.
- 위치 축이 온전한 문단의 기존 LineSeg 보존은 유지한다.
- HWPX control slot의 `U+FFFC` marker는 별도 위치 슬롯을 차지하는 기존 #3739 계약을 유지해,
  marker가 있는 정상 문단의 LineSeg를 잘못 억제하지 않는다.

기존 #3739 경계 보존과 함께 검토했으며, suppression 범위가 정상 위치 축까지 넓어지는 결함은
발견하지 못했다. 메인터너 보정은 필요하지 않다.

## 완료된 검증

- `cargo test --profile release-test --target-dir target\pr-review --lib issue4778_broken_position_axis_suppresses_stored_linesegs -- --nocapture`
  를 통과했다(1 passed).
- `cargo test --profile release-test --target-dir target\pr-review --test issue_3739_hwpx_same_char_shape_boundary -- --nocapture`
  를 통과했다(4 passed).
- `git diff --check upstream/devel...HEAD`를 통과했다.
- code candidate `c7e2f1fe5`의 GitHub Actions에서 Lint, Build & Test 전체 shard, Native Skia,
  Canvas visual diff, CodeQL이 모두 성공했다.

## 위험과 후속 범위

- 이 변경은 위치 축이 깨진 입력에서 저장 LineSeg를 보수적으로 버린다. 손상 입력의 layout 재구성
  정밀화는 별도 serializer 진단 범위다.
- 다른 HWPX control 종류의 위치 모델은 이 PR에서 확장하지 않는다.

## 최종 권고

수용을 권고한다. #4801의 review-only trailing head가 fast-pass 조건을 만족하고 최신
`MERGEABLE/CLEAN` 상태를 재확인한 뒤, 작업지시자 승인에 따라 통합 PR로 병합한다.

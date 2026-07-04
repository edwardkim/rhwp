# PR #1679 검토 - HWPX 커넥터 선 도형 파싱

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| PR | [#1679](https://github.com/edwardkim/rhwp/pull/1679) |
| 제목 | `[IR 복원] HWPX 커넥터 선 도형 파싱` |
| 작성자 | `humdrum00001010` |
| base <- head | `devel` <- `codex/pr1573-connect-lines` |
| 문서 작성 시점 참고값 | `MERGEABLE` / `BEHIND`, draft 아님, maintainer 수정 가능 |
| 규모 | 1 파일, +123/-6 |
| 원 PR 커밋 | `b51ff99484bc` |
| 로컬 적용 커밋 | `0bc7171bf` (`git cherry-pick -x`) |

## 2. 변경 범위

HWPX XML의 connector line shape를 실제 line control로 복원하는 parser/IR 보강 PR이다. PR 본문도 시각 렌더링 자체가 아니라 HWPX XML 기반 Format/IR 복원으로 분류한다.

- body paragraph 안의 `hp:connectLine`을 인식한다.
- group container 안의 `hp:connectLine`도 동일하게 파싱한다.
- connector endpoint, subject reference, control point, line type을 보존한다.
- 변경 파일은 `src/parser/hwpx/section.rs` 1개다.

## 3. 검토 의견

커넥터 선은 diagram/form 계열 HWPX에서 시각적으로 중요한 요소지만, 이 PR의 직접 역할은 parser가 control을 잃지 않도록 하는 것이다. downstream renderer가 모든 connector geometry를 정확히 그리는지는 별도 문제로 분리해야 한다.

단일 파일 변경이고 범위가 명확하다. group 내부 파싱이 추가되므로 기존 group child ordering이 보존되는지, shape/control list에 들어가는 순서가 문서 구조와 맞는지 확인할 필요가 있다.

이 PR은 다른 렌더 보정 PR과 달리 경험적 layout 계산을 크게 바꾸지 않으므로 회귀 표면은 비교적 좁다. 다만 IR에 새 control이 살아나면 renderer 단계에서 이전에는 없던 선이 나타날 수 있으므로 visual diff는 필요하다.

## 4. 로컬 적용 상태

`upstream/devel` 기준 로컬 일괄 검토 브랜치 `local/humdrum-pr-batch-review`에 원 커밋 1개를 `-x`로 체리픽했다. 충돌은 없었다.

원 PR은 `BEHIND` 상태다. 일괄 브랜치에서는 #1676-#1678 다음 순서로 적용했다.

## 5. 검증 상태

- 완료: `cargo build --release`
- 완료: `cargo test --release --lib` (2037 passed, 0 failed, 7 ignored)
- 완료: `cargo test --profile release-test --tests` (통합 테스트 전체 통과)
- 완료: `cargo fmt --check`
- 완료: `git diff --check`
- 완료: `cargo clippy --all-targets -- -D warnings` (최초 공통 검증 18m 25s warning 0, TIFF 보강 후 최종 재실행 17m 28s warning 0)
- 완료: `cargo test --doc` (0 passed, 0 failed, 1 ignored)
- 완료: `cargo test --test svg_snapshot` (8 passed)
- 완료: `cd rhwp-studio && npx tsc --noEmit`
- 완료: `cd rhwp-studio && npm test` (153 passed)
- 완료: `wasm-pack build --target web --out-dir pkg` (1m 25s)
- 중단/무효: `cargo check --all-targets --message-format=short`는 workflow 문서에 없는 명령이라 중단했으며 검증 결과로 기록하지 않는다.

### 5.1 PR 내용별 targeted 검증

2026-06-30 로컬 일괄 브랜치 `local/humdrum-pr-batch-review`에서 #1679 주장별로 다음을 확인했다.

| 주장 | 검증 |
|------|------|
| `hp:connectLine`을 `LineShape` control로 materialize | `cargo test --profile release-test --lib test_parse_hwpx_connect_line_materializes_connector` 통과 |
| connector line type 보존 | 같은 테스트에서 `LinkLineType::StraightOneWay` assertion 통과 |
| start/end subject id/index 보존 | 같은 테스트에서 `start_subject_id/index`, `end_subject_id/index` assertion 통과 |
| control point 보존 | 같은 테스트에서 control point 개수, 좌표, point type assertion 통과 |

단, 이 targeted 검증은 parser/IR 복원까지만 보장한다. 이번 snapshot/visual sweep에서는 page count mismatch가 없었지만, connector-specific fixture가 아니므로 canonical renderer와 같은 라우팅/화살표/겹침 순서까지 보장하지는 않는다.

### 5.2 시각/브라우저 검증

2026-06-30 로컬 일괄 브랜치에서 자동 시각 gate를 추가 확인했다.

- `cargo test --test svg_snapshot`: 8개 golden snapshot 통과
- `cargo test --profile release-test --test visual_roundtrip_baseline`: 3개 visual roundtrip baseline 통과
- `python3 scripts/task1274_visual_sweep.py --target all`: 15개 target 모두 SVG/PDF page count 일치, page count mismatch 0건
- 자동 sweep flagged 후보: 5개 target. connector-specific fixture가 아니므로 커넥터 라우팅/화살표의 canonical 일치까지 보장하지는 않지만, 기존 snapshot/sweep에서 새 page count mismatch는 확인되지 않았다.
- browser/WASM 경로: `rhwp-studio` TypeScript/test와 `wasm-pack build --target web --out-dir pkg` 통과

## 6. 잠정 판단

수용 후보. parser/IR 복원 주장은 targeted 검증으로 확인했다. snapshot/visual sweep도 page count mismatch 없이 통과했지만, merge 판단 시에는 "connector를 파싱한다"와 "canonical renderer와 완전히 같은 선을 그린다"를 분리해서 기록해야 한다.

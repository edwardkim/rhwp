# PR #3123 검토 — Native Skia cache writer 정합성 복구

| 항목 | 내용 |
| --- | --- |
| PR | [#3123](https://github.com/edwardkim/rhwp/pull/3123) |
| 작성자 | postmelee |
| 관련 이슈 | [#2431](https://github.com/edwardkim/rhwp/issues/2431), C 단계 |
| base / 규모 | devel / 6 files, +233 -10 |
| 문서 작성 시점 참고값 | base 85ac3f294f595896db3e35381867bcdd7a1aac8f, head f8eebc4f230409a0aecad9275caa60b197172c0e. #3232의 문서-only devel 갱신을 Update branch로 반영했으며, 최신 head의 GitHub Actions가 모두 성공했다. |
| contributor 협업 | postmelee/rhwp fork, maintainerCanModify=true, reviewer jangster77 지정 |

## 관련 이슈와 변경 범위

- #2431에서 owner가 우선 승인한 C 단계다. writer가 사라진 Linux-cargo namespace를 되살리지 않고 Native Skia job도 rust-cache 체계로 통합한다는 결정과 일치한다.
- Native Skia job의 actions/cache restore step을 Swatinem/rust-cache v2로 바꾼다. native-skia shared key로 default-feature archive 및 lint cache와 분리하고, PR은 restore-only, devel/main push만 save하게 한다.
- Native Skia 패키지 설치, test 명령, release/release-test profile 분기, 선행 gate 및 Build & Test 집계는 변경하지 않는다.
- 계획서, 구현계획서, 단계 보고서 및 당시 오늘할일 기록을 함께 추가한다. cache 삭제, PR-ref cleanup, npm cache 정책 변경, action SHA pinning은 명시적으로 범위 밖이다.

## 렌더 영향과 검증

- workflow와 운영 문서만 바꾸며 renderer, 샘플, golden, PDF/SVG 출력은 변경하지 않는다. visual sweep 대상이 아니다.
- 최신 격리 worktree에서 git diff check와 actionlint .github/workflows/ci.yml을 통과했다.
- Ruby YAML.load_file 파싱을 통과했다.
- Swatinem/rust-cache v2 action 정의에서 shared-key가 job 기반 key를 대신해 여러 job에 안정적으로 쓰이는 입력이고, save-if=false가 restore-only 동작임을 확인했다.
- Update branch 뒤 이전 head 3d393390의 진행 중 CI와 CodeQL은 force-cancel했고, 이미 성공한 Render Diff는 유지했다.
- f8eebc4 최신 head에서 Build & Test(8개 default-feature shard 포함), Lint, Native Skia tests, Frontend package gates, CodeQL 3개 분석, Canvas visual diff가 모두 성공했다. WASM Build는 조건부 skip이다.

## 리스크와 판단

- C PR 자체에서는 trusted writer가 실행되지 않아 cold restore가 정상이다. merge 후 devel push에서 native-skia cache save, 후속 PR에서 warm restore를 확인해야 lifecycle 검증이 완료된다.
- 새 native-skia cache와 legacy Linux-cargo cache가 일시 공존할 수 있다. legacy cache 삭제와 PR-ref cache cleanup은 #2431의 별도 승인 범위이므로 이 PR에서 수행하면 안 된다.
- rust-cache SHA pinning은 D 단계로 분리한 owner 결정에 따른 것이므로, 이 PR의 v2 tag 사용은 범위 위반으로 보지 않는다.

## 최종 권고

- 최신 head CI가 성공해 수용했고, squash merge `0f0d92e687b1a0006e4d33c3892f883fb718f810`로 `devel`에 반영했다. 추가 코드 보정은 필요하지 않다.
- merge 뒤에는 devel writer save와 후속 PR warm restore를 관측하고, #2431은 A, B, D 및 legacy cache 정리 범위를 계속 추적한다.

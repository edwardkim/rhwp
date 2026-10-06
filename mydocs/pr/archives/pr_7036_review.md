# PR #7036 self-review — 분할 표·반복 제목 셀 대각선

## 1. 대상과 절차

| 항목 | 검토 시점 값 |
| --- | --- |
| PR / Issue | [#7036](https://github.com/edwardkim/rhwp/pull/7036) / #7028 |
| 작성자 / 검토 | edwardkim, 메인테이너 지시의 작성자 self-review; reviewer 별도 지정 없음 |
| base | devel `376c6b605c6be3b735bf6b8b9464fcd16b833a10` |
| 녹색 candidate | `d59e83c798d1fef800575b648a1ef71781376791` |
| 최초 제출 규모 | 9파일, +1008/-3. Rust 제품 +65/-3, 테스트 +280, 나머지 문서 |
| GitHub 상태 | OPEN, MERGEABLE / CLEAN. 작성 시점 참고값이며 merge 전 재확인 |

기본 경로: `collaborator_self_merge`. 보조: `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `review_only_fast_pass`, `rework_and_exceptions`.
PR 라우터·선택표와 위 자식 문서를 확인했다. 최신 base의 동작 기반 회귀 지침도 확인했다.
1,000줄 초과이므로 코드·시각·시뮬레이션 검토를 수행했으며 즉시 admin merge하지 않는다.
메인테이너는 결과보고서·PR 생성에 이어 이번 self-review 진행을 승인했다.

## 2. 코드와 보호 반례 검토

`table_partial.rs`의 공통 대각선 생성기 연결은 기존 일반 표 의미를 재사용한다.
페이지에 표시되는 원본 행 범위를 연속성까지 검사하며, 내용 clip 확장이 아닌 격자 셀 좌표를 쓴다.
가로쓰기와 세로쓰기 분기 전에 한 번 생성하고 각 경로에서 한 번 추가한다. 제목행만을 위한
하드코딩이나 페이지네이터·높이·텍스트 진행 수정은 없다.

- 선 없음 / type=0은 기존 생성기의 계약으로 선을 만들지 않는다.
- 실제 잘린 행·높이 override·불완전한 rowspan은 새 조각 대각선을 만들지 않는다.
- 활성 대각선 zone은 유효 영역과 교차하는 셀에만 영향을 준다. 무관하거나 비활성인 zone으로
  전체 표의 사선을 억제하지 않는다.
- `tests/cases/issue_7028_partial_table_diagonal.rs`는 실제 `DocumentCore`의 RenderTree/SVG를
  호출한다. 6쪽의 셀 끝점·정확히 1개 방출, 비반복 제목행, 일반 본문 셀, 세로쓰기, zone,
  온전한 rowspan 및 실제 cut/경계 반례를 검사한다. 소스 문자열 검사로 대신하지 않았다.
- Stage 2의 수정 전 음성 대조와 수정 후 결과를 확인했다. 초기 cut 행의 반복 횟수 전제 오류는
  테스트에만 정정했으며 제품을 그 잘못된 기대값에 맞추지 않았다. fixture 미존재는 실패한다.

검토에서 승인 범위의 차단 결함은 발견하지 못했다. 실제 분할 셀·분할 zone의 완전한 대각선
지원은 이번 성과가 아니다. 셀의 선행 빈 문단 배치는 #7032로 분리하며 이 PR로 닫지 않는다.

## 3. 완료한 검증

전체 로컬 검증 SHA `4592264c007a4f6616f38d7fda30784f7936fc51`와 녹색 candidate 사이에는
`mydocs/` 차이만 있다. 로컬 fmt·Clippy 3종·workspace build·manifest, nextest 9,481개,
Skia lib 4,112개·PNG 2개·PDF 4개 및 Docker WASM 성공은
[Stage 3](../../working/task_m100_7028_stage3.md)의 결과를 재사용했다.

이번 self-review에서 동일 제품 후보의 기존 review worktree로 다음 집중 검사를 다시 실행했다.

```bash
node scripts/run-rust-test.mjs issue_7028_partial_table_diagonal -- \
  --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target
```

**9/9 통과**, 실행 0.163초, 컴파일 2분 11초. 필터로 제외된 183개를 통과 수에 넣지 않았다.
로그: `output/7028/self-review-focused.log`. nextest 권장 버전/JUnit 옵션 경고는 기존과 같았다.
신규 테스트·제품 수정·파생 파일 제출은 없다.

정확한 candidate의 다음 `pull_request` run이 모두 성공했다. CI API의 PR 번호 7036, head와 base를
대조했다. CI 변경 전후를 추정한 것이 아니라 실제 성공한 실행을 확인했다.

| 검증 | 실행 |
| --- | --- |
| CI: Lint·Rust archive/shard·Native Skia·Build & Test | [34590787844](https://github.com/edwardkim/rhwp/actions/runs/34590787844) |
| CodeQL Rust 분석 | [34590787922](https://github.com/edwardkim/rhwp/actions/runs/34590787922) |
| Render Diff / Canvas visual diff | [34590787658](https://github.com/edwardkim/rhwp/actions/runs/34590787658) |
| Proptest roundtrip | [34590787923](https://github.com/edwardkim/rhwp/actions/runs/34590787923) |
| Adapter inter-diff | [34590787976](https://github.com/edwardkim/rhwp/actions/runs/34590787976) |
| CI Impact Policy Controller | [34590787782](https://github.com/edwardkim/rhwp/actions/runs/34590787782) |

동일 후보의 전체 회귀·Skia를 다시 중복 실행하지 않았다. 새 code/test/fixture 변경 없이 집중 검증과
시각 대조를 수행하는 정본 재사용 경로다. 브라우저 재시도는 새 탭 `Network.enable` timeout으로
종료되어 **미완료**다 (`output/7028/self-review-studio.log`). 브라우저 통과로 승격하지 않는다.
이 제한이 공개된 보고서를 메인테이너가 승인했으며 아래 native 직접 대조·메인테이너 대각선 확인을
동등한 시각 근거로 사용한다. GitHub Canvas visual diff 성공이 해당 수동 탭 검사의 성공을 뜻하지는 않는다.

## 4. 시각 증적과 남은 차이

renderer/layout 사용자-visible 변경이므로 직접 시각 확인이 필요하다. 기존 프로젝트 fidelity 비교
자료를 재사용해 이번 검토에서 **1·2·6쪽 이미지 자체를 다시 열었다**. 첫 쪽과 반복 제목 셀의
좌상단→우하단 사선이 보인다. 텍스트/본문 행 배치와 선 두께의 기존 차이는 그대로 남아 있다.
Stage 2에서 대각선 외 native 6쪽 노드·텍스트 원장이 이전 출력과 동일함을 확인했고,
메인테이너도 대각선을 확인한 뒤 빈 문단 문제를 분리하도록 지시했다.

- 원본: `samples/task2146/21761835_jeonjik_exemption_table.hwp`.
  SHA-256 `d8e7e38b206f6e53b7d2d260115396f8a0dd94bd8b712bbc4428dc11bf66e5dc`.
- 기준: `pdf/task2146/21761835_jeonjik_exemption_table-hwp-2020.pdf`.
  SHA-256 `99d150b26805a29f3d3153528638f363e33f882d0e38ad11edb374baccdac127`,
  SHA-1 `3b623963b9c6aa62fd08fb74114e9ce80ac80430`.
  Creator Hwp 2022 0.0.0.0 / Producer Hancom PDF 1.3.0.550, 6쪽 A4(595×841pt),
  기존 기준 자료를 그대로 사용했다. 파일명으로 생성 버전을 추정하거나 새 PDF를 만들지 않았다.
- 임시 비교: `output/7028/stage2/visual/cmp-p000.png`, `cmp-p001.png`, `cmp-p005.png`.
  native 산출 후보 `1ae766455a`의 직접 대조이며 최신 통합 tree를 새로 raster한 자료로 표기하지 않는다.
  그 뒤 우리 제품 변경은 없고 최신 통합의 자동 Render Diff 성공은 위 표에 별도로 기록했다.
- 보존 대표: [1쪽](../assets/pr_7036/hancom-native-p001.png), [2쪽](../assets/pr_7036/hancom-native-p002.png).
  각각 SHA-256 `a6be47316b16ef815617cd6899884c2cd72ebdbbedaee5065958ccc97bd6a2b3`,
  `41ef6db1b725d161335376704504dfd391cd81c9861887529d11166263e52aa3`.
  비교 도구의 한글 라벨과 diff 수치는 판독 가능하다. 최종 패널 두 장만 보존하며 SVG/JSON/log는 제외한다.
- fidelity pixel diff: 1쪽 14.02%, 2쪽 16.56%, 6쪽 14.84%.
  **Visual Sweep의 pixel_match·visual_accuracy_proxy_percent·flagged 후보 수는 미산출**이다.
  서로 다른 지표로 환산하거나 문서 전체 한컴 일치·내용 정확도 수치로 보고하지 않는다.

## 5. Merge 후 contributor PR comment 계획

이 PR은 작성자 self PR이므로 외부 기여자에게 별도 판정을 게시할 대상은 없다. 승인된 후속 절차에서
PR #7036 / 이슈 #7028에 완료 근거를 남길 경우 다음 내용을 사용한다.

- 정본: [Visual Sweep GitHub merge comment](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment).
- 실제 판정 범위: 6쪽 native 원장, 직접 이미지 1·2·6쪽, 메인테이너 대각선 확인.
  위 fidelity diff만 기록하고 정본 proxy·pixel_match·flagged는 미산출로 명시한다.
  빈 문단 #7032 및 기존 전체 fidelity 차이는 해결 완료로 쓰지 않는다.
- 대표 이미지 URL 형식:
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7036/hancom-native-p002.png`.
- asset이 실제 merge commit을 통해 devel에 반영된 뒤 승인받은 `gh ... --body-file`로 게시하고
  API로 Markdown/URL을 재조회한다. 지금은 댓글·GitHub approve·close·merge를 수행하지 않는다.

## 6. 최종 판정

**승인** — #7028의 온전한 셀·반복 제목 셀 대각선 복원 범위에 한한다.

CI 녹색 후보 뒤에는 보관 중인 제출 기록 및 이번 review·대표 PNG만 잇는다.
현재 base와의 merge simulation은 clean이다. `mydocs/` 허용 범위의 single-parent 후행 commit으로
push하며, 문서 때문에 devel을 merge/rebase하지 않는다. 코드·테스트·baseline은 추가로 변경하지 않는다.

병합 전 조건: 후행 문서까지 포함한 **최신 head CI/required aggregate 성공**, 최신 head/base와
mergeability 재확인, 메인테이너 병합 승인. 이 self-review 승인은 병합 승인을 대신하지 않는다.
추가 구현·체리픽·복잡한 rollback은 없어 별도 review_impl은 작성하지 않는다.

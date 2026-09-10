# PR #6991 검토: 검증된 current-base merge와 문서 후속 커밋의 CI 재사용

## 판정: 로컬 검토 승인, GitHub CI 완료 전 머지 보류

- 대상: [PR #6991](https://github.com/edwardkim/rhwp/pull/6991), [Issue #6815](https://github.com/edwardkim/rhwp/issues/6815).
- 기준 devel: `2a780e0d296846df577866eba6ac8f388527551b`.
- 검증 코드: `0462bdf826fea5065ee77c5cb5645e1b67c56f98`.
- 경로: collaborator self-review. 저장소 owner를 reviewer로 자동 지정하지 않았다.
- 로컬 계약 검증에서 미해결 실패는 없다. 최신 GitHub required check와 실제 운영 재사용 성공은 아직 확인하지 않았으므로 머지 또는 이슈 전체 종료를 승인한 기록이 아니다.

## 원인과 보정 범위

#6813의 문서 충돌 해소 merge 뒤 post-merge 후보 탐색과 #6990의 PR preflight 사례를 함께 다뤘다. #6990에서는 current-base merge `cc11b55e69656450a57629f2b7a2122732b08dc9`의 Full CI가 성공했지만, 문서 trailing head가 해당 merge 자체를 후보에서 건너뛰어 Full CI를 재실행했다.

CI·CodeQL·Render Diff·Adapter·Proptest의 후보 탐색에 merge 자체를 포함하고, merge 이후 후보와 이전 후보를 구분했다. 이전 후보를 재사용할 때의 독립 merge-tree 증명은 유지했다. head에서 후보까지의 부모 계보, 저장소·브랜치·SHA, 실행 완료와 성공을 확인한다.

post-merge collector와 verifier는 신뢰 base 코드로 Git 객체와 필수 worker 증거를 검사한다. 증거가 없거나 worker가 누락·실패·진행 중인 경우 재사용하지 않는다. 문서 밖에서 들어오는 rename, symlink·실행 파일 및 실행 계약 변경은 문서-only로 인정하지 않는다. CodeQL 분석과 보안 check, CI B/C/D artifact 및 timing 갱신 계약도 보존했다. PR head 코드는 실행하지 않는다.

## 실제 검증

| 검증 | 결과 |
| --- | --- |
| Node verifier·실제 preflight/collector 계약 | 220개 통과, 실패 0, skip 0 |
| Python workflow 계약 | 233개 통과, 실패 0 |
| 최신 devel 정렬 후 위 두 검증 재실행 | 모두 통과 |
| actionlint 6개 workflow | 기준선에도 있는 SC2016 경고 1건을 제외하고 통과 |
| git diff whitespace 검사 | 통과 |

실행 명령:

```sh
node --test scripts/tests/verify-trusted-postmerge-ci-reuse*.test.mjs scripts/tests/verify-trusted-postmerge-review-bridge.test.mjs
python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'
actionlint -ignore 'SC2016' .github/workflows/ci.yml .github/workflows/codeql.yml .github/workflows/render-diff.yml .github/workflows/adapter-diff.yml .github/workflows/proptest-roundtrip.yml .github/workflows/trusted-postmerge-ci-reuse.yml
git diff --check
```

SC2016은 수정 전 ci.yml에서도 재현한 기존 경고다. 무경고 통과라고 주장하지 않는다. 초기 CodeQL 계약 실패 6개는 테스트 모형의 실제 SHA 형식 및 head/getCommit.sha 누락을 보완한 뒤 해소했다. 성공 판정을 위해 제품 검증 조건을 완화하지 않았다.

CI 정책 변경 범위이므로 Rust 전체 회귀·WASM 빌드·문서 시각 sweep은 실행하지 않았다. 원시 로그·임시 JSON 등은 제출하지 않는다. 제품 화면 변경이 없어 PNG/PDF 증적은 불필요하다.

## 적용 경계와 남은 검증

이 PR은 CI 실행 정책을 변경하므로 자체 Full CI를 통과해야 한다. 로컬 mock·Git 객체 계약 성공은 GitHub 운영 성공과 구분한다. 병합 후 별도 코드 PR의 문서 trailing/current-base merge 사례에서 heavy skip과 timing 재사용을 실제 run으로 확인해야 #6815를 최종 완료할 수 있다.

#6990의 [PR CI](https://github.com/edwardkim/rhwp/actions/runs/34469847029)는 중복 Full 실행 사례다. 이후 [devel CI](https://github.com/edwardkim/rhwp/actions/runs/34472004648)와 [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34472004649)의 재사용 성공은 기존 코드의 결과이며, 아직 병합하지 않은 #6991의 개선 효과가 아니다.

## 후속 코멘트와 정리 계획

- PR 및 #6815에는 실제 merge SHA, 최종 PR/devel CI run URL, 확인된 재사용/Full fallback과 timing 갱신 결과를 기록한다. 미완료 운영 검증을 완료로 적지 않는다.
- 기존 #6815 코멘트를 갱신하여 동일 내용의 중복 코멘트를 만들지 않는다. 종료 조건을 충족하기 전에는 이슈를 열린 상태로 유지한다.
- UTF-8 body-file 방식으로 게시하고 API로 본문을 다시 확인한다. 시각 변경이 없으므로 무관한 PNG/PDF를 첨부하지 않는다.
- post_merge.md의 gate 충족 뒤에만 이 작업의 local/upstream `fix/6815-green-merge-review-tail-20260910` 및 전용 worktree를 정리한다. 기본 작업공간·다른 작업은 보존한다.

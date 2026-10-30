# PR #8145 — 셀 영역의 병합 끝 칸 채우기 수용

## 판단

**독립 검증·승인·머지 완료.** 사용자의 기존 결함 분리·수용 지시에 따라 Native issue1880 p1의 동일한 제목 누락은 이번 PR의 머지 차단 사유에서 제외한다. 원래 자동 gate와 수치는 보존한다. 일반 90% 기준을 완화하거나 새 회귀를 허용하는 결정은 아니다.

- [원 PR](https://github.com/edwardkim/rhwp/pull/8145), pblsketch.
- 원 head `a88d758d6f80e7b4b5a788f4dc70df7c865b6516`.
- current base `698906bf4aeaa6d0d7101a4fd4d7c4a9229f6bb2`.
- 독립 후보 `9e3a672474cc3e424cd068bfa998285dd172e0e0`, tree `6aaeeefc13c5db204aafb319ffd09353b3d7290c`.
- 다른 보류 PR을 포함하지 않은 clean merge다. 기여자 원 코드·검사를 수정하지 않았다.

## 보류 판단의 재검토

[이전 보류 댓글](https://github.com/edwardkim/rhwp/pull/8145#issuecomment-6512992492)의 Native 90% 미달은 해당 PR이 만든 회귀가 아니었다. 최신 base와 #8145 독립 후보를 새로 빌드해 전체 페이지를 비교했다. Native issue1880 p1은 양쪽 45.43888%, PNG 바이트 동일이다. fresh WASM의 같은 쪽은 양쪽 100%다. 메인터너가 이 기존 결함을 분리해 수용하도록 지시했으며, 필요한 추가 검증을 이번에 완료했다.

## 구현과 독립 근거

셀 영역의 끝 주소는 칸 주소다. #6619의 기존 테두리 처리와 같이 끝 칸의 span을 반영하는 것이 독립 한컴 Print와 맞는다. `src/renderer/layout/table_layout.rs`의 cellzone 끝 칸 조회 → col_span/row_span을 반영한 grid 끝 인덱스 → row_col_x/row_y의 좌표 → 배경·대각선 paint가 같은 경계를 소비한다. 끝 칸 없음은 기존 계산을 유지하고 grid 범위를 제한한다. pagination·이어받기 컷·줄 높이 측정은 변경하지 않는다.

원 사례는 한컴으로 다시 저장한 공개 재현본이며 독립 메인터너 Print를 사용했다. cellzone p1의 가로 병합 채우기, issue1880 p9의 세로 병합 채우기, issue6542 p4의 대각선을 base/후보/Print로 직접 판독했다. issue6298 p6은 경계 색 차이만 남고 새 배치·내용 차이는 보이지 않는다. 기존 글꼴·각주 차이는 해결했다고 기록하지 않는다. 칸 목록 조회 비용이 추가되지만 별도 성능 측정은 하지 않았으므로 속도 영향이 0이라고 단정하지 않는다.

## 검증

[전쪽 원본 TSV·새 생산자·해시·직접 판독](../assets/pr_8145_maintainer/README.md).

- 6문서 Native 36쪽 + fresh WASM 36쪽을 base/후보 각각 렌더했다. 독립 PDF 쪽수 일치, 누락 없음.
- 원 사례 Native 76.17115% → 99.54886%, WASM 76.32898% → 99.55637%.
- 2%p 초과 하락 0쪽. 최대 하락 0.01673%p는 경계 색 차이다.
- 유일한 90% 미달은 위 Native p1이며 `re_review_required` 상태를 그대로 기록했다.
- #8165 정상 대조군은 전쪽 PNG가 수정 전후 동일하다.
- 기존 자동 수집 집중 회귀 26 PASS / 0 FAIL, 1,393 filtered out. `git diff --check` 통과.
- [원 head Full CI](https://github.com/edwardkim/rhwp/actions/runs/39687357171)의 성공을 재확인한다. code head 변경이 없고 clean merge이므로 전체 Cargo/Native suite를 중복 실행하지 않는다. 승인 직전에 정확한 head·실제 devel ref·필수 check·merge-tree를 다시 확인하고 head를 고정해 merge한다.

## Merge 후 contributor PR comment 계획

원 PR에 감사와 수용 사실, 이전 보류의 변경 이유, 실제 merge SHA, 26개 집중 회귀, 6문서 전쪽 비교 결과와 기존 Native 결함의 한계를 기록한다. 원 기여자 asset은 code merge SHA로 고정하고 메인터너의 새 이미지·TSV는 운영 기록 SHA로 고정한다. 서로 다른 PDF/생산자의 수치를 혼합하지 않는다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 링크하고 대표 PNG를 Markdown 이미지로 게시한 뒤 API 본문·blob을 확인한다.

## 후속 처리

별도 close 대상 이슈는 없다. Refs #6619는 유지한다. maintainer 일반 경로의 archive·asset·오늘할일만 devel 운영 기록으로 반영한다. 다른 3개 보류 PR이 담긴 로컬 검토 브랜치와 contributor fork는 보존한다. 검토 worktree·루트 output·사용 종료한 debug는 필수 증거 보존 뒤 정리한다.

## 머지 확정

- Merge SHA: `d456f3dd25074baed4b291d665ef4cae6c199747`, 2026-10-30T04:27:58Z. 원 기여 이력을 보존하는 merge 방식이다.
- 실제 merge tree가 위 독립 검증 후보 tree와 정확히 같다.
- [승인 리뷰](https://github.com/edwardkim/rhwp/pull/8145#pullrequestreview-5708077932)를 게시했고 원 PR은 MERGED다.
- 원 head의 Full CI·Native Skia·lint·개별 CodeQL 언어·Render Diff·Adapter·Proptest 성공을 승인 직전에 확인했다.

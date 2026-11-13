# 최신 devel #9052 병합 결과의 추가 검증

후보 `1cf0ff99d73d0426ead4089a239e4c6da549dffa`의 제품 코드는 `d66d5f05f9`이며 모든 필수 검증을 완료했다. 제출 직전 `devel`에 그림·도형 CLI #9052(`d7b927d073489b8c5ea95655224b04096c939534`)가 추가되어, 후보를 바꾸지 않고 실제 병합 tree `c4c6b5aae90068fa57ff21b62874d37b5790408d`를 detached worktree에서 검증했다. parent 두 개의 merge-tree로 재구성할 수 있다.

- 실제 Native producer를 빌드하고 최소 글꼴에서 #9019 7건·#9020 7건·머리말 그림 5건·이번 원본/경계 9건, 총 28건을 통과했다.
- 같은 병합 producer로 원본·BodyX 모델 전 4쪽 Native를 다시 출력했다. 독립 Print 대비 모두 100.00%이며 후보의 실제 PNG 4쌍과 RGB가 같다.
- 새 base는 CLI metadata/입력 검사와 별도 JSON 검사 helper·re-export를 추가했다. renderer·model·WASM 진입점·기존 문서 열기/렌더/저장 소비자·Cargo manifest/lock은 후보와 같다. 기존 WASM API와 렌더 소비자가 새 CLI 검사 helper를 호출하지 않음도 대조했다.
- 후보의 코드·test·fixture를 rebase/edit하지 않았으므로 후보에서 완료한 전체 필수 검증을 유지한다. 이 추가 결과를 최신 base 전체 회귀를 반복한 결과라고 부르지 않는다. 제출 후에는 해당 exact head/merge ref의 GitHub 필수 CI 성공을 별도로 확인해야 한다.
- 임시 worktree는 이 검증 종료 후 제거하며, shared target의 원 후보 un-hashed binary/library는 검증 전의 정확한 hash로 복원했다. 다른 worktree는 보존한다.

[명령·실제 source·검사 수·TSV·픽셀 대조](results.json), [원본 전 2쪽](original.tsv), [BodyX 전 2쪽](edited.tsv).

# Kang-T PR 3건 누적 검토 계획

## 범위와 고정한 출처

원 [#8566](archives/pr_8566_review.md)·[#8595](archives/pr_8595_review.md)·[#8632](archives/pr_8632_review.md)를 오래된 순서로 검토한다. 각각 #8555·#8556·#8557이며 공통 대표 이슈가 없으므로 날짜 기반 통합 branch `review/integration-20261106-kang-t-picture-editing`를 사용한다. 기준 devel `45684640be5eddcf5972b063260250345b34d928`와 원 head를 개별 review에 고정했다. 원 commit을 재작성하지 않고 merge ancestry로 보존한다.

## 실행할 검증

1. 원 증상이 최신 devel에서도 남아 있는지 실제 API·재조판·그림 drag 흐름으로 재현하고 선행 해결 PR→통합·보정→독립 근거를 추적한다.
2. 개별 3건의 merge-tree는 모두 충돌 없이 생성됐다. 누적 적용도 순서대로 확인하며, 충돌이 있으면 해결 방침을 구체화해 작업지시자 승인 뒤 보정한다.
3. 세 원 입력과 Mac 한컴 Print PDF를 그대로 재사용한다. 4·5·4쪽 전쪽의 base/후보 Native와 fresh optimized WASM TSV를 같은 조건으로 비교하고 90%·2%p 하락·쪽수·직접 판독을 확인한다. 생성 저장본의 재열기를 편집 검증과 구분한다.
4. baseline 정렬 0/위/가운데/아래, 칸 삽입의 소속/빈 경로/Storage 한도·실패 원자성·저장, 재배치의 본문↔칸·중첩 경로·다른 쪽·같은 쪽·앞뒤 방향과 Undo/Redo를 해당 공통 계약으로 확인한다. 최신 devel의 그림 삽입·분할·겹침 방지·저장 기능도 보존한다.
5. Windows Chrome CDP `http://127.0.0.1:19222`에서 테스트가 만든 탭만 사용한다. 현재 사용자 탭·문서·세션은 보존한다. WASM 원본·public 동기화와 실제 초기화 때 받은 bundle 출처를 고정하고 API 소비와 실제 mouse drag를 구분한다.
6. 집중 회귀와 경계를 먼저 확인한 뒤 공유 절대 target `/home/edward/mygithub/rhwp/target/pr-review`에서 Cargo/WASM을 순차 실행한다. 최소 글꼴 전체 회귀·Native 공식 3종·Rust lint·Studio unit/type/build·관련 E2E·Native 편람 383쪽 gate를 최종 후보 한 단위로 완료한다. 생성 suite와 로그는 stage하지 않는다.
7. PR별 판정·증적·첫 기여 환영 댓글 초안을 완료한다. 수용 가능한 원격 후보가 구체화되면 push·PR·CI·merge·댓글·종료 정리의 별도 승인을 요청한다.

기여자 제공 CI/코퍼스 집계와 메인테이너의 실제 실행은 구분한다. 원 제공 Mac Print는 유효한 인쇄 자료로 재사용하며 버전 표기만으로 재변환하지 않는다. Linux에서 편람 gate를 직접 확인한다. 현재 원격 작업은 reviewer 지정만 수행했고 기타 원격 변경은 승인 전이다.


## 누적 통합에서 확인한 보정 방침 — 승인 완료

#8566과 #8595는 최신 devel에 clean merge했다. #8632는 개별 merge-tree에서는 충돌이 없었지만, #8595와 누적하면 다음 두 파일에서 같은 삽입 위치를 사용한다. 원본 코드를 임의로 선택하거나 한 기능을 삭제하지 않고 두 기능을 함께 보존한다.

| 충돌 파일 | 승인받을 보정 | 유지할 계약 |
| --- | --- | --- |
| `rhwp-studio/src/core/mutation-method-registry.ts` | `insertPictureInCell`과 `relocateFloatingPicture` 둘 다 MUTATING_METHODS에 등록 | 기존 그림/표 편집의 mutation route·캐시 무효화·Undo/Redo |
| `src/wasm_api.rs` | 같은 위치에 추가한 두 WASM method를 각각 완전한 함수로 보존 | 각 Native API·parse_cell_path·인수·오류 전파와 기존 `insertPicture` |

원 head 세 개와 실제 source commit은 재작성하지 않는다. 충돌한 임시 merge는 보정 승인 전에 abort해 앞선 clean 통합과 문서만 유지했다. 충돌 원문·해시와 두 API를 함께 유지하는 patch는 ignored `output/pr-review/kang-t-20261106/proposed-conflict-resolution.patch` 및 JSON에 보정 전 기록으로 준비했다. 아래 승인 기록의 통합 commit에서 적용을 완료했다.

보정 뒤에는 문단 세로 정렬점·행 나눔의 줄/개체 점유·현재 devel의 겹침 방지/분할 그림을 함께 확인한다. 신규 API의 Storage 자원 한도·경로 오류·실패 원자성·컨트롤 스트림 좌표와 공개 WASM 선언도 대조한다. 실제 출력에서 추가 문제가 드러나면 이 세 원 PR의 동일 계약 안에서 원인을 보정하고 재검증한다. 필수 검증 범위를 축소하거나 기존 올바른 검사를 완화하지 않는다.

기여자 제공 Mac Print 원본 전 13쪽의 Linux Native/fresh optimized WASM base/후보 비교와 직접 확대 판독, Windows Chrome CDP 19222의 실제 다른 쪽 drag·같은 쪽 이동·본문/칸 이동·Undo/Redo·저장 재열기, 집중/전체/Native/Studio/lint/편람 gate를 한 단위로 마친다. 원격 push·새 PR·공개 댓글·workflow 재실행·merge는 이번 로컬 보정 승인 범위와 구분한다.

2026-11-06 작업지시자가 위 두 API 보존·메인테이너 보정과 필수 로컬 검증을 한 단위로 승인했다. 승인된 patch와 충돌 원문 해시를 대조해 적용했고 통합 commit `46af1d5d22e4fd2fd567a893bd36ce43f12d2ab6`에 기록했다. 세 원 head의 ancestry와 clean worktree를 확인했다. 이후 실측·테스트 결과는 각 원 PR review와 공통 검증 증적에 연결한다.

## 최신 devel 보존 검증과 최종 실행 출처

검토 중 #8696이 `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`로 devel에 반영됐다. 원 기여 SHA를 보존하기 위해 충돌 없는 merge `226f0d487cc8d72305979ceb8ab46832017634d0`로 통합했다. 같은 Rust source digest의 Native·optimized WASM을 다시 생성했고 이후 Studio-only 포인터 보정은 `63236b9cc5`다.

최신 후보의 전체 회귀 12,778건·Native 공식 3종·Rust lint 묶음·Studio 2,745건·타입·build·실제 CDP·원 13쪽 Native/WASM·정상 대조군 10쪽·SVG·편람 383쪽이 통과했다. 기준 devel도 별도 detached worktree에서 같은 profile/글꼴/입력으로 빌드해 비교를 완료했다. 정상 그림 배치 20쌍과 새 글꼴 사례 6쌍의 PNG가 동일하고 원 사례 52개 쪽/조건 대조에서 후퇴가 없다. 과거 후보의 결과를 새 기준 출력으로 표시하지 않으며, 입력 생성·폰트 프로그램·후보 source·실제 실행 근거는 [공통 증거](assets/kang-t-20261106/README.md)에 모은다.

`insertPictureInCell`/`relocateFloatingPicture`를 함께 보존하는 최초 승인과 같은 계약의 경계 보정을 한 단위로 수행했다. 새 menu 구현·기존 private Windows 경로 검사나 오래된 커서 경고 수정을 별도 기능으로 확장하지 않았다. 원격 push·새 PR·공개 댓글·merge는 최종 로컬 결과가 구체화된 뒤 별도 승인을 확인한다.

## 승인된 원격 처리 결과

작업지시자 승인 뒤 PR #8725를 생성했고 제출 head `07510f602a6081d90c94397fe29613420f3e7f63`의 CI·CodeQL·Render Diff·Adapter·Proptest·최종 정책·self-review를 완료했다. merge `331521a3b1fb5de5dfa39b0707c0b295b90f9dc4`와 원 3개 PR의 자동 MERGED를 확인했다. [운영 기록](assets/kang-t-20261106/remote/completion.md)을 따라 후속 처리를 완료한다.

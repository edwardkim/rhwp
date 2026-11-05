# 고정 code head와 전진한 devel의 실제 병합 호환

2026-11-05 KST. 원 제출 코드의 필수 로컬 19단계가 끝난 뒤 devel에 #8630과 #8633·#8634 기록이 추가됐다. source branch를 다시 merge/rebase하지 않고 고정 code head와 현재 base의 실제 병합 트리를 별도 worktree에서 실행했다. 준비 완료의 근거는 충돌 없는 계산과 컴파일만이 아니라 아래 실제 기능·출력 결과다.

| 대상 | 고정값 |
| --- | --- |
| 전체 검증을 완료한 제품 source | `584c9503baa651c62d0403dad9cb77c5cb9ffde6` |
| 그 검증을 보존한 문서 head | `74d37978091007eae783134d530e689064404fd1` |
| 현재 base | `a95d1d5afad1bfd61998794fa78fb0db100d690a` |
| 충돌 없는 merge tree | `1469367a0ad7ba14a63d2a4627ab50ee25df6e57` |
| 실제 실행용 detached probe | `06bf0855b49d6a448b70987016489a65e546aff3` |
| probe의 src tree | `0b4c9740b505f8b5dff4b791f454a01885837206` |

[계산과 실제 source](validation/current-base/current-base-compatibility.json), [Native·최적화 WASM 빌드](validation/current-base/merge8630-builds.json), [WASM manifest](validation/current-base/merge8630-wasm-manifest.json)를 연결한다. probe는 원 기여 10개와 현재 devel을 조상으로 보존한 로컬 검증 객체이며 원 source branch에 반영하거나 원격에 push한 commit이 아니다.

## 추가 변경과 소비 경로

[#8630](https://github.com/edwardkim/rhwp/pull/8630)은 원문 순서의 실제 글상자만 편집 주소를 소비하도록 공통 indexed helper를 연결했다. 변경은 `model/shape.rs`의 글상자 순회, `document_core`의 cursor·문자/inline control·clipboard·hyperlink·중첩 표 소비, Studio cursor와 Esc다. `RenderNode.textbox_source`는 `serde(skip)` 편집 metadata이며, `shape_layout.rs`의 자식 주소 offset과 빈 소유 본문 시작 caret은 인쇄 좌표·clip·flow를 바꾸지 않는다. 이번 개요 ID·장식 속성·표/계산식·각주/미주·문단 머리 경로와 공통 파일/타입이 있으므로 실제 두 기능 묶음을 같이 실행했다.

`#8633`·`#8634`는 운영 기록이다. upstream의 CI 변경은 단계 전후 `df -h` 관측과 기존 검사기의 명령 추출 보정이며, 이번 PR이 workflow를 수정한 것은 아니다. 최종 PR의 고유 diff에서 CI 변경 여부를 다시 확인한다. branch protection 조회에서 `Build & Test`, `strict=false`를 확인했으며 현재 base를 source branch에 반영하도록 강제하는 조건은 없었다. merge 전에는 다시 조회한다.

## 실제 실행 결과

- [집중 Native Rust 18건](validation/current-base/merge8630-focused.json): 새 묶음 글상자 8건과 기존 #1892 공백 5건·#919 hit-test 5건이 모두 PASS다. 921 skipped는 선택한 세 harness에서 이 모듈만 실행한 필터 결과이며 IGNORE로 합산하지 않는다. 최소 글꼴 환경에서 실행했다.
- [실제 브라우저 결과](validation/current-base/group-textbox-browser.json)와 [초기화 artifact 대조](validation/current-base/group-textbox-browser-artifacts.json): 383쪽 문서를 실제 파일 입력으로 열고, 3쪽의 제목/본문 별도 소유·입력·undo/redo·Enter/Backspace·드래그 삭제·4쪽 본문 입력·묶음 선택 후 Esc와 원 소유 본문 입력을 확인했다. 편집 소유 경고는 없다.
- [API·Studio 실행 10단계](validation/current-base/merge8630-runtime.json): 이번 WASM API 10묶음/한컴 계산 37값·양쪽 저장·원자적 오류, 실제 계산식 대화창, Canvas2D 6장면·CanvasKit 효과, TypeScript·Studio **2,721 PASS·2 기존 SKIP**·bindings·build가 모두 성공했다.
- [이번 10건의 46상태·전 124쪽](validation/current-base/summary.json)을 probe의 Native/fresh WASM으로 다시 출력했다. 최저는 **96.49% / 97.10%**, 전 248개 PNG가 앞서 고정한 후보와 byte 단위로 같다. 누락·쪽수 차이·90% 미만·점수 하락은 없다.
- 새 편집 경로의 편람 **3·4쪽은 Native/fresh WASM 각각 100.00%**다. [Native TSV](validation/current-base/book-native.tsv)와 [WASM TSV](validation/current-base/book-wasm.tsv)를 보존했다. Native 두 PNG는 앞서 검증한 편람 프로그램 출력과도 byte 단위로 같다.

## 직접 판독과 재사용의 경계

[3쪽 review](images/current-base/book-p003-review.png)·[4쪽 review](images/current-base/book-p004-review.png)의 동일 영역을 직접 대조했다. 제목·안내문·본문 줄·목차 번호/점선·장식과 미세 간격이 유지된다. [실제 3쪽 편집](images/current-base/page003-edit.png)에서 `이 편X람`의 X가 실제 그림에도 표시되는 것을 확대 판독했고, [4쪽 본문](images/current-base/page004-body.png)의 내용을 확인했다. UI의 대체 글꼴 선택은 기능 확인 환경이며 독립 Print 측정의 명시 글꼴과 구분한다. [이미지 SHA와 crop](validation/current-base/image-index.json)을 연결한다.

source 584c9503ba의 전체 Rust 12,629건·Native 4,064건·정식 release 검사 19단계와 이전 530쪽/편람 383쪽을 probe에서 다시 실행했다고 기록하지 않는다. 원 code head는 변하지 않았으며, 새 편집 metadata의 적용/비적용 경로와 실제 current-base 호환을 위 집중 검사·fresh 프로그램·이번 124쪽·영향 2쪽으로 확인했다. source나 실제 호환 보정이 필요해지면 새 code head의 필수 검증을 갱신한다. 원격 승인 뒤 생성할 PR은 현재-base 전체 CI와 self-review를 통과해야 하며, 로컬 성공으로 이를 대체하지 않는다.

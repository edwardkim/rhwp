# PR #6763 검토: 개체 속성 무변경 확인 시 원래 치수 보존

## 판정: 승인

2026-09-05 갱신. 원 PR CI 조회 결과와 이번 로컬 실행 결과를 분리한다. 메인터너 보정은 `902a208b515e83024502f004a2adaf84c33f18de`로 커밋했다. 통합 PR은 작업지시자 승인에 따라 생성하는 단계이며 merge는 하지 않았다.

## PR 정보와 적용 이력

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#6763](https://github.com/edwardkim/rhwp/pull/6763) |
| 작성자 | `lpaiu-cs`, 기존 기여자 |
| 리뷰어 | `jangster77`, fetch·체리픽 전에 할당 |
| 원 head | `00422f4683d47cda7a46f5e2a8fb47bf7bc51fdb` |
| base / draft | `devel` / `false` |
| 변경 규모 | 3개 파일, +111 / -4 |
| 원 head 조회 당시 병합 참고 상태 | `MERGEABLE` / `CLEAN` |
| 기준 devel | `2c144b180dd776aa450c499778510199ae6cdf89` |
| 로컬 검토 브랜치 | `review/ci-green-6759-6768-20260905` |
| 체리픽 커밋 | `9d161e12d`, `ba245487d`, 원본 출처를 `-x`로 보존 |
| 메인터너 보정 전 체리픽 HEAD | `d87b3037e5aeb6b662904b0182c361d5a2929108` |
| 메인터너 보정 commit | `902a208b515e83024502f004a2adaf84c33f18de` |

관련 [#6758](https://github.com/edwardkim/rhwp/issues/6758)은 높이 1 HWPUNIT인 선에서 속성을 바꾸지 않고 확인해도 200 HWPUNIT으로 부풀어 오르는 문제다. 원 PR의 자기리뷰 보완 커밋까지 두 개 모두 포함했다.

## 조회한 원 PR CI

- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/33954204741/job/101275340768): `SUCCESS`.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33954553197): `SUCCESS`.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33954204647/job/101274465061): `SUCCESS`.
- CodeQL 분석 worker는 성공했지만 [플랫폼 CodeQL check](https://github.com/edwardkim/rhwp/runs/101274897623)의 원 결론은 `NEUTRAL`이다. 이를 `SUCCESS`로 바꾸어 기록하거나 알림을 자동 dismiss하지 않는다.
- 위 결과는 원 head 기준이다. 새 통합 head의 CI·로컬 검증 결과가 아니다.

## 완료한 검증

- 원 PR의 표시 문자열 공용화 커밋까지 두 개 모두 적용했다. 변경하지 않은 너비·높이를 치수 수정 명령에 넣지 않는 범위다.
- 통합 Studio 테스트 1,403건 통과, 1건 skip, TypeScript 검사 및 새 WASM 빌드가 성공했다.
- 새 WASM을 사용하는 실제 Chrome Studio에서 `samples/group-box.hwp`의 선 개체 `section=0, paragraph=2, control=0`를 열었다.
- 실제 속성 창은 너비 `62.50` mm, 높이 `0.00` mm로 표시됐다. 실제 `설정(D)` 버튼으로 무변경 확인한 뒤 모델 너비 `17716`, 높이 `1` HWPUNIT이 그대로임을 assertion으로 확인했다.
- 확인 전·속성 창·확인 후 PNG를 직접 열어 보았다. 전후 선의 외관은 유지되며 PC 전체 화면이 아니라 문서 canvas 또는 Studio 앱 영역이다.

## CDP 검증 완료와 보류 해소

Chrome `Chrome/152.0.7977.82`의 CDP에서 실제 Studio 속성 폼을 사용한 검증이 **exit 0**으로 완료됐다.
원본은 `samples/group-box.hwp`, SHA-256은
`d05b13579a40be6cdcb1251c80e84cd19076e497a01fdef834e04c3a206d5bfc`이며 새 WASM SHA-256은
`f531f4d540839b4d2630f3ffb120d5704ef31659e17acd28236f3d7328743321`다.

| 단계 | 모델 너비 (HWPUNIT) | 모델 높이 (HWPUNIT) | 표시값 또는 확인 내용 |
| --- | ---: | ---: | --- |
| 원본 선 | 17716 | 1 | 너비 62.50 mm, 높이 0.00 mm |
| 무변경 설정 | 17716 | 1 | 원래 치수 보존 |
| 높이 1.00 mm 명시적 설정 | 17716 | 283 | 높이만 변경, 너비 보존 |
| 재열기 및 다시 무변경 설정 | 17716 | 283 | 재열기 표시 62.50 / 1.00 mm, 변경된 치수 보존 |

CDP `Runtime.evaluate`로 화면에 열린 높이 input의 값을 `1.00`으로 채우고
`input`·`change` 이벤트를 발생시켰다. 비율 유지는 해제해 높이만 바꾸는 조건을 명시했다.
이후 실제 `설정(D)` 버튼을 클릭하고 WASM 모델을 읽어 위 값을 assertion으로 확인했다.
`setShapeProperties` 등 모델 setter를 직접 호출해 결과를 만든 검증이 아니다.

### 이전 실패의 원인과 메인터너 보정 범위

- 두 번째 속성 창 timeout: 임시 스크립트가 포커스를 받지 않는 canvas에 `focus()`를 호출했다. 첫 실행은 기존 textarea 포커스 덕분에 열렸지만, 두 번째는 `P`가 `BODY`로 전달됐다. 실제 `문서 편집 입력` textarea 포커스와 속성 창의 초기 자동 포커스를 기다리도록 보정했다.
- 숫자 입력 실패: 숫자 칸의 부분 선택을 전체 선택으로 오인해 `.00`이 남았고, `1.00` 타이핑 후 실제 입력은 `100.00`이 됐다. CDP로 정확한 폼 값을 채운 뒤 적용 전 값도 단언하도록 보정했다.
- 이전 실패를 삭제하거나 성공으로 바꾸지 않는다. 최종 CDP 검증이 위 실패들과 별도로 성공하여 보류 사유를 해소했다.
- **제품 코드 보정은 필요하지 않았다.** 이번 보정은 메인터너의 임시 검증 스크립트에만 적용했다. 엔진 최소 치수 clamp나 원 PR 구현을 우회하지 않았다.
- 이번 확인은 치수 폼의 적용·무변경 왕복 경로다. 숫자 칸의 모든 키보드 편집 UX, 위치 오프셋 왕복 오차, 다른 개체 종류까지 검증했다고 확대하지 않는다.

## 원 PR·이슈 처리 범위

원 PR #6763과 #6758에는 무변경 치수 보존, 높이 1.00 mm 적용 및 재확인 결과를 구분해 기록한다.
무변경 전·창·후 PNG 세 개와 명시적 변경 후 다시 연 속성 창 PNG 한 개만 코멘트에 사용한다.
승인된 통합 merge와 실제 devel CI 성공 뒤 closing reference·auto-close 상태를 확인하고 필요한 후속 처리를 한다.
지금은 GitHub approve·merge·comment·close를 실행하지 않았다.

## 공통 검증과 승인 경계

전체 실행 명령, 첫 실패와 보정 후 결과, lint·Native Skia·WASM·Studio 결과는 [통합 검증 기록](pr_6759_review_impl.md)에 구분했다. 검증 대상은 체리픽 HEAD에 당시 미커밋 메인터너 보정을 더한 작업 트리였으며, 해당 보정은 이후 `902a208b515e83024502f004a2adaf84c33f18de`로 보존했다. 이를 순수 원 head 또는 최종 통합 PR CI 성공으로 대신 기록하지 않는다.

검토 판정은 GitHub approve·merge 권한 행사와 다르다. commit·push·통합 PR 생성은 작업지시자 승인 범위에서 진행한다. 최종 head CI와 시각 판정의 작업지시자 확인·merge 승인은 별도다. 현재 원격 comment·close·merge는 실행하지 않았다.

## Merge 후 댓글 작성 방식

승인된 통합 merge와 실제 devel CI 성공 뒤에만 [후속 처리 절차](../../manual/pr_review/post_merge.md)를 따른다. 원 PR 수용 출처·merge SHA·실제 PR/devel CI를 적고, 같은 merge SHA의 기존 댓글이 있으면 새 댓글 대신 수정한다. UTF-8 `--body-file`로 게시한 뒤 API로 body를 재조회한다.

아래 대표 PNG만 코멘트 이미지로 사용한다. `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr_6759_6768_20260905/<상대 PNG 경로>`를 Markdown 이미지로 넣어 댓글 안에서 직접 표시하고, [시각 대조 기록](pr_6759_6768_visual_sweep.md)과 공개 재현 HWP를 함께 연결한다. 존재하지 않는 merge SHA나 미완료 시나리오를 완료 증적으로 게시하지 않는다.

![#6763 검토 증적 1](../assets/pr_6759_6768_20260905/studio-6763-before.png)

![#6763 검토 증적 2](../assets/pr_6759_6768_20260905/studio-6763-dialog.png)

![#6763 검토 증적 3](../assets/pr_6759_6768_20260905/studio-6763-after.png)

![#6763 명시적 높이 변경 후 재열기 확인](../assets/pr_6759_6768_20260905/studio-6763-explicit-edit-confirmed.png)

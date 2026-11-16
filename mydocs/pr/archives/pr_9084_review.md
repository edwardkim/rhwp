# PR #9084 리뷰 — 값이 든 누름틀 끝 표식의 HWP5 저장 순서

## 최종 판정

**승인.** 최신 devel `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0`를 실제 병합한 source `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`에서 필수 로컬 검증을 완료했다. 전체 회귀 13,632개·Native 공식 4,073개·모든 lint·목표 Native/fresh WASM 전쪽 100.00%·편람·공개 공통 검사가 통과했다.

특별 보호 3,542쪽은 승인된 Native 10쪽 1행의 기존 차이(99.86% → 99.85%)만 예외로 수용한다. 다른 하락·누락은 0건이며 [정확한 결정](../assets/johndoekim-20261116/native-floor-decision.md)을 연결한다. [최종 실행·전쪽 증적](../assets/johndoekim-20261116/latest-devel-revalidation.md)을 기준으로 한다.

작업지시자가 push·Open 수용 PR·해당 head CI·self-review·merge와 후속 처리를 승인했다. 수용 PR #9126의 code/trailing head CI·실제 fast-pass·셀프 리뷰를 완료했고 `fb7367574f13e65111f162bb79238cb0af45b36e`로 병합됐다. [후속 원장](../assets/johndoekim-20261116/closure.md)에서 실제 종료 상태를 확인한다. 원 기여 SHA·fork branch와 #9058 OPEN을 보존한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#9084](https://github.com/edwardkim/rhwp/pull/9084) / johndoekim / devel |
| 원 head | `81dd920099bae78d8d66148ddf9614785760deac` |
| 최초 검토 base | `7ee4fe0327d0cdfc8dec12c450e5dd5b5369b826` |
| 갱신한 base·검토 후보 | `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0` / `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb` |
| 관련 이슈 | [#9082](https://github.com/edwardkim/rhwp/issues/9082) 해결 대상. [#9058](https://github.com/edwardkim/rhwp/issues/9058)은 기존 줄 소속 문제로 구분한다. |
| 규모·reviewer | 원 기여 3 commits·4 files / edwardkim 지정·재조회 완료 / Open, non-Draft |

검토 후보는 원 #9081 → #9084 head를 순서대로 merge한 로컬 누적 후보다. #9084는 #9081의 renderer 수정에
의존하지 않으므로 원 PR 단독 수용이 가능하다. 원 기여 commit과 fork branch를 보존한다.

## 변경과 소비 경로

문단의 마지막 글자에서 닫히는 필드가 이미 본문 루프에서 열렸으면, HWP5 저장기가 그 끝 표식을 뒤 컨트롤
앞에 쓰지 못하고 문단 끝에 썼다. 재파싱하면 누름틀이 뒤의 표·책갈피까지 감싸게 된다.

`try_serialize_para_text_with_slots`가 `trailing_end_after_ctrl` 중 이미 지난 닫기 위치를 정렬해 먼저 방출한다.
파서가 저장한 `control_idx + inner_slot_count`를 근거로 하며 샘플 ID·좌표·높이 상수로 분기하지 않는다.
새 목록은 `Storage::collect`를 사용해 기존 자원 예산과 오류 전달을 유지한다.

같은 직렬화 함수의 `control_stream_slots`·`field_stream_ends`는 인라인 컨트롤 삽입·이동의
`caret_slot`·`remove_slot`·`insert_slot`에서도 소비된다. 이 경로의 기존 집중 회귀를 함께 실행했다.
기존 baseline의 `hwpctl_API_v2.4.hwp` 필드 내부 슬롯 발산 1행 삭제는 해소된 저장 손실의 상한 제거다.
렌더링 품질 하한이나 허용치는 변경하지 않았다.

## 검증 입력과 결과

- `samples/issue9082/textbox_filled_field_tac_tables.hwp`: 한/글 2022 `12.0.0.535`의 실제 저장본.
  원 head에 포함된 SHA-256은 `a4dd2bd52c02ca3d25414e63e8c8af7cefc96d53b00e49fa635ed6f1610cb069`이다.
- `mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp`: 본문의 값이 든 누름틀 뒤에 책갈피가 있는 기존 대조군.
- #9058의 네 글자 값 실제 저장본과 #9081의 안내문 입력은 보조 대조군으로 사용했다.
  공개 원본·해시는 [공유 증적](../assets/johndoekim-20261116/README.md)에 보존한다.

| 실행 | 결과 |
| --- | --- |
| 실제 CLI `edit insert-text` → HWP 저장·재열기 → `edit delete-text` → 재저장·재열기 | 한 글자·네 글자·안내문 3입력 모두 원본의 필드/표 스트림 순서 보존 |
| 위 산출물의 HWPX 저장 → 재열기 → HWPX 재저장·재열기 | 6개 XML의 `fieldBegin → 값 → fieldEnd → tbl → tbl` 순서와 필드 값 보존 |
| 최소 글꼴 환경 focused | 집중 30건과 전체 회귀 13,632개의 필드·개체·IR baseline 통과 |
| 원 head GitHub CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/43651800059), CodeQL·Adapter·Proptest·정책 성공 |
| 충돌·공백 | 최신 devel 누적 merge 성공, `git diff --check` 통과 |

최신 devel에서는 한 글자 입력의 원본 순서 `시작 0 · 값 8 · 끝 9 · 표 17 · 표 25`가 실제 편집 저장 뒤
`시작 0 · 값 8 · 표 9 · 표 17 · 끝 25`로 바뀌었다. 후보는 두 번의 편집 저장에서도 원본 순서를 유지했다.
네 글자 입력도 원본의 끝 12·표 20·표 28을 보존한다. 안내문 대조군은 수정 전후 동일하다.
글자 값 조회는 수정 전에도 통과하므로 단독 판정 근거로 사용하지 않았다.

[집중 실행 영수증](../assets/johndoekim-20261116/validation-record.md)과
[실제 HWP 저장 순서](../assets/johndoekim-20261116/validation-record.md),
[HWPX 순서](../assets/johndoekim-20261116/validation-record.md)를 연결한다.
`bwrap`으로 시스템·사용자·Windows/HFT 글꼴 경로를 가렸고 저장소의 공개 글꼴만 노출한 실행이다.

원 head와 누적 후보의 `src/serializer/body_text.rs` Git blob은 같다.
[component 증거](../assets/johndoekim-20261116/validation-record.md)를 확인했으며
#9084의 제품·test·baseline 보정은 없다. 누적 후보에서는 별도 #9081의 주석을 보완했으므로
같은 검증 입력으로 전체 Rust lint·release-test 13,632개·Native 공식 4,073개를 모두 통과했다.
실제 실행 당시 로컬 SHA와 제출 코드 SHA의 입력 동일성은 공유 검증 기록에 있다. 원 head의 녹색 CI를 새 제출 후보의 GitHub CI로 대체하지 않는다.

같은 최적화 WASM의 공개 API에서도 세 입력을 실제 삽입·HWP 저장·재열기·삭제·재저장·재열기 후 HWPX로 교차 저장했다. 독립 CFB 토큰 판독과 필드 값·HWPX XML 순서가 CLI와 같은 결과임을 확인했다. [실제 WASM 저장 기록](../assets/johndoekim-20261116/latest-devel-revalidation.md)을 연결한다.

## 조판 원칙 적용과 기존 이슈의 구분

이번 수정은 필드 끝 표식의 저장 구조 보존이다. renderer의 저장 줄·개체 주소 계산은 수정하지 않는다.
따라서 [접수 절차 2.7](../../manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토)의 구조 보존 범위를 적용한다.
렌더링 회귀·좌표 pin·시각 baseline 변경은 비해당이다.

#9058의 기존 배치 문제는 `Paragraph::control_source_positions_with_storage`가 짝 있는 FIELD_END 슬롯을
세지 않는 별도 원인이다. 실제 원본 3종의 base/후보 PNG는 Native·fresh WASM 각각 3쌍 모두 같았다.
이 PR로 #9058의 화면이나 한컴 재열기 전체 호환이 해결됐다고 주장하지 않는다. #9058은 OPEN을 유지한다.
해당 진단의 낮은 시각 점수를 구조 보존 회귀의 새 정상 golden으로 등록하지 않았다.

## Merge 후 contributor PR comment 계획

통합 PR의 exact head CI·self-review·merge가 허용되면 merge SHA와 원 #9084의 완료 상태 확인 → archive review·저장 증적·오늘할일을
메인터너 운영 기록으로 반영 → devel 동기화 → #9082 완료 상태 확인·필요한 완료 댓글 → 원 PR 감사 댓글 순서로 처리한다.
이 PR의 수용 근거는 저장 구조이며, renderer 변경 #9081의 이미지를 #9084 해결 증거로 게시하지 않는다.
문안은 한국어 존댓말 `--body-file`로 전달하고 API로 본문·issue 상태를 재조회한다.
원 fork branch는 보존한다. 원격 처리와 후속 범위는 승인받았으며 새 수용 PR의 exact-head CI·self-review 뒤 실행한다.

## 수용 PR code CI 완료

[수용 PR #9126](https://github.com/edwardkim/rhwp/pull/9126)의 code head `529ba31acbdea187a2ed5d9528be419f26686132`에서 [필수 CI와 실제 실행](../assets/johndoekim-20261116/code-ci-529ba31.md)을 모두 확인했다. 셀프 리뷰 기록 뒤 trailing head 검사·merge·후속 처리를 진행한다.

## 원격 통합 결과

[수용 PR #9126](https://github.com/edwardkim/rhwp/pull/9126)은 최종 head `2d80477a8c817041576157aa2d786293b877fe97`의 모든 check·정책·`CLEAN`과 최신 base 병합 tree를 재확인한 뒤 정상 merge commit `fb7367574f13e65111f162bb79238cb0af45b36e`로 병합됐다. 원 PR의 간접 MERGED와 연결 이슈의 completed 자동 종료도 확인했다. 댓글·duration·정리의 실제 결과는 [후속 원장](../assets/johndoekim-20261116/closure.md)에 기록한다. 앞 절의 남은 조건은 각 단계 당시 기록이다.

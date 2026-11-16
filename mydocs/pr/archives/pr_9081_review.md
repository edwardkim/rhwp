# PR #9081 리뷰 — 글상자 안 숨긴 누름틀 안내문과 표의 줄 구성

## 최종 판정

**메인터너 보정 후 수용 가능.** 최신 devel `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0`를 실제 병합한 source `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`에서 필수 로컬 검증을 완료했다. 전체 회귀 13,632개·Native 공식 4,073개·모든 lint·목표 Native/fresh WASM 전쪽 100.00%·편람·공개 공통 검사가 통과했다.

특별 보호 3,542쪽은 승인된 Native 10쪽 1행의 기존 차이(99.86% → 99.85%)만 예외로 수용한다. 다른 하락·누락은 0건이며 [정확한 결정](../assets/johndoekim-20261116/native-floor-decision.md)을 연결한다. [최종 실행·전쪽 증적](../assets/johndoekim-20261116/latest-devel-revalidation.md)을 기준으로 한다.

작업지시자가 push·Open 수용 PR·해당 head CI·self-review·merge와 후속 처리를 승인했다. 수용 PR #9126의 code/trailing head CI·실제 fast-pass·셀프 리뷰를 완료했고 `fb7367574f13e65111f162bb79238cb0af45b36e`로 병합됐다. [후속 원장](../assets/johndoekim-20261116/closure.md)에서 실제 종료 상태를 확인한다. 원 기여 SHA·fork branch와 #9058 OPEN을 보존한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#9081](https://github.com/edwardkim/rhwp/pull/9081) / johndoekim / devel |
| 원 head | `3a2fcbf7d12d4f57fa3fcff33721ff7ed9059738` |
| 최초 base·누적 코드 | `7ee4fe0327d0cdfc8dec12c450e5dd5b5369b826` / `3843f897ae410fdf67347651b39214d9dd9cafff` |
| 최신 base·통합 후보 | `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0` / `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb` (주석 보완) |
| 관련 이슈 | [#9076](https://github.com/edwardkim/rhwp/issues/9076). #9057의 누름틀 없는 경우와 #9023의 안내문 처리를 선행 분석했다. |
| 규모·reviewer | 원 기여 4 commits·22 files / edwardkim 지정·재조회 완료 / Open, non-Draft |

## 구현 근거와 실제 소비

한/글은 미기입 누름틀의 안내문을 저장 LineSeg에 포함하지만 Print에서는 이를 숨긴 뒤 줄을 나눈다.
공개 재현물과 누름틀만 지운 실제 한/글 사본은 독립 Print PDF의 표 선분 20개가 동일하다.
따라서 안내문이 사라진 현재 표시 내용에 안내문을 포함한 저장 줄 분할을 그대로 적용할 수 없다.

`hidden_guide_object_rows_need_reflow`는 글자 없음·안내문 잔재·인라인 개체를 판정한다.
가로쓰기·양수 안쪽 폭인 글상자의 기존 사전 단계에서 `textbox_reflow::in_frame`으로 줄을 만들고,
흐름 끝이 달라지면 `recalculate_section_vpos`로 뒤 문단을 이어 준다. 이어지는 조합기의
`control_line_indices`와 인라인 표 줄 계획·실제 배치가 그 결과를 소비한다.
원본 문단의 저장 LineSeg를 영구 변경하거나 특정 문서의 좌표를 clamp하지 않는다.
기존 행렬 글상자 사전 단계와 글자가 있는 문단의 저장 줄 계약을 보존한다.

## 검증 입력·시각 결과

아래 두 원본과 PDF는 원 head에 포함돼 있으며 실제 해시를 확인했다. 한/글 2022 `12.0.0.535`의
저장본과 동일 제품의 Microsoft Print to PDF 출력이다. 두 문서 모두 전체 1쪽이고 인쇄 글자는 없다.

| 입력·독립 Print PDF | 입력 SHA-256 | PDF SHA-256 |
| --- | --- | --- |
| `samples/issue9076/textbox_unfilled_field_tac_tables.hwp` · `pdf/issue9076/textbox_unfilled_field_tac_tables-2022.pdf` | `cbab3774fedc4d9944cccf6b3e365ed147c8624375f6cffd3f301967f6f6570b` | `3bd2fa3656ec337120898eb46cfb5ce7cbb7214fadaa7939298a8dd485df21a8` |
| `samples/issue9076/textbox_unfilled_field_tac_tables_nofield.hwp` · `pdf/issue9076/textbox_unfilled_field_tac_tables_nofield-2022.pdf` | `055629eea0baa5ffe5090cf682357fc6278b34304c02cd8daefa12bd160099f9` | `1da00c4cc3fc2849153e96c519beae26e5c49a6eeb5e4fce4c26e5b8ed2feed0` |

`visual_sweep.py --silhouette-only --compat 2022 --dpi 96`의 색상 경계 보정 전 2px raw를 사용했다.
Native는 실제 Skia PNG, WASM은 최적화 완료 web package를 Chromium에서 실행한 Canvas PNG다.
입력·독립 PDF·DPI·인쇄 프로필·출력 backend를 바꾸거나 영역을 제외하지 않았다.

| 입력 | devel Native / WASM | 후보 Native / WASM | 쪽수 |
| --- | --- | --- | --- |
| 안내문 있음 | 29.43% / 29.43% | 100.00% / 100.00% | 양쪽·PDF 각 1쪽 |
| 누름틀 없음 | 100.00% / 100.00% | 100.00% / 100.00% | 양쪽·PDF 각 1쪽 |

[전쪽 TSV](../assets/johndoekim-20261116/README.md)와
[Print·base·Native·fresh WASM 확대 비교](../assets/johndoekim-20261116/guide-table-zoom.png)를 직접 판독했다.
base는 둘째 표가 오른쪽으로 밀려 보이지 않았고 첫 표도 앞 줄에 놓였다. 후보의 두 표는 Print와 같은
순서·열 구성·상하 간격으로 글상자 안에 들어간다. 작은 획의 raster coverage 차이와 실제 줄 배치를 구분했다.

#9058의 별도 실제 저장본 3종은 적용 조건 밖의 진단 대조군이다. Native·WASM 각각 원본 3쌍의
base/후보 PNG가 바이트 단위로 같았고, 기존 줄 소속 문제는 남았다. 이 입력을 새 정상 렌더 회귀로 등록하거나
#9058 해결로 판정하지 않았다. 관련 공개 입력·Print·전체 TSV·해시는 공유 증적에 보존한다.

## 로컬·CI 검증과 최신 base 반영

- 최소 글꼴 환경의 집중 9묶음 30건과 전체 회귀 13,632개가 통과했다. 전체 회귀에는 인접 필드·개체 이동·IR field baseline도 포함한다. Native 공식 4,073개·모든 Rust lint·workspace build를 완료했다.
- [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/43644452017)는 성공했지만 새 제출 후보의 CI로 대신 쓰지 않는다.
- #9102 공개 공통 검사와 #9106의 실제 분할 셀·Print TTF em·괘선·Native SVG 글꼴 준비·CanvasKit 예산의 선행 근거와 현재 소비 경로를 대조했다. 새 base와 후보의 각주 50쪽·중첩 표 17쪽은 Native/WASM 각각 PNG 67개가 모두 같다.
- Native 편람 383쪽과 공개 공통 Native/WASM 8개 실행이 통과했다. 특별 보호 3,542쪽은 Native 10쪽 한 행의 승인된 기존 차이 외 하락 0건이다.
- 실제 source·프로그램 해시·전쪽 TSV·직접 판독·독립 글꼴 복원은 [최종 기록](../assets/johndoekim-20261116/latest-devel-revalidation.md)에 구분했다.

## 조판 원칙 준수 검토

| 항목 | 판정·근거 |
| --- | --- |
| 제출 전 검증·PR 상태 | 충족 — 목표·집중·전체 회귀 13,632개·Native 4,073개·편람·공개 공통·lint 통과, 보호 1행 예외 승인 |
| 기존 이슈·해결 PR 선행 분석 | 충족 — #9057·#9023의 계약과 #9058/#9082의 다른 원인을 분리 |
| 근거·일반성 | 충족 — 실제 저장/Print의 안내문 차이에 기존 공통 재조판 적용, 샘플별 수치 분기 없음 |
| 측정·배치 일관성 | 충족 — 재조판 줄 → 뒤 문단 흐름 → 조합기 줄 소속 → 표 배치 추적·실행 |
| 최신 base 기능 보존 | 충족 — #9106까지 실제 병합하고 Native/WASM 각 67쪽 PNG 동일·관련 경계 검사 통과 |
| 최종 backend 소비 | 충족 — 같은 source의 Native Skia·최적화 WASM Canvas를 실제 실행 |
| 분할·이어받기 | 비해당 — 컷·예약 높이·continuation 알고리즘은 변경하지 않음 |
| 줄 소속·점유 높이 | 충족 — 저장 줄과 안내문 없는 현재 줄을 구분하며 표·후속 문단 흐름 공유 |
| 독립 근거·사례 | 충족 — 한컴 저장본 2종·독립 Print 2종, 관계 검사·직접 판독 |
| 기준값 | 충족 — 시각 기준·허용치 수정 없음, IR baseline 1행은 별도 저장 손실의 기존 상한 |
| 렌더 회귀 선행 조건 | 충족 — 목표 두 문서 Native/WASM 모두 전쪽 100.00% |
| 확대 판독 | 충족 — 표 순서·포함·선분·간격·안내문 비표시를 같은 배율로 대조 |
| 최종 주장 범위 | 충족 — 원 사례 개선·대조군 보존·보호 1행의 승인된 기존 차이만 수용, 기존 #9058을 해결 범위로 포함하지 않음 |

## 메인터너 주석 보완과 남은 조건

원 회귀 파일은 전체 1쪽 고정 검사와 관계 검사를 함께 하지만 첫머리에 관계형만 적고 갱신 시각도 없었다.
`e958314dc9078fdb55e7cde04227a1a22054a72d`는 `관계형·고정형`, 실제 갱신 시각
`2026-11-16T03:27:25+09:00`와 직전 파일 commit `3a2fcbf7d12d4f57fa3fcff33721ff7ed9059738`
(`2026-11-15T05:28:15+09:00`), 메인터너의 실제 측정 근거를 기록했다. 제품과 test body는 바이트 단위로 같다.

구형 수식 글꼴은 독립 문자표 Print로 복구했고 77개 원본 프로그램이 이전 provenance와 일치한다.
직전 후보에서는 공식값 하락 0건이었으며 새 #9106 하한에서는 Native 10쪽 한 행의 기존 차이를 별도로 승인받았다. 기준값·허용치는 변경하지 않았다.
[공급·빌드 기록](../assets/johndoekim-20261116/validation-record.md)과
[보호 전쪽 TSV](../assets/johndoekim-20261116/protected-validation.md)를 연결한다.

전체 회귀 13,632개와 Native 공식 4,073개를 모두 통과해 수용 후보를 확정했다.
실행 당시 임시 SHA와 공개할 제출 코드의 Rust·test·검증 입력은 같으며 [실행 기록](../assets/johndoekim-20261116/validation-record.md)에 구분했다.
작업지시자의 원격 승인은 확보했다. 새 PR의 exact-head GitHub CI·self-review를 실제 완료한 뒤 병합한다.

## Merge 후 contributor PR comment 계획

현재는 게시하지 않는다. 원격 승인과 새 후보 CI·merge 뒤 merge SHA와 기록 반영이 확정되면, 원 사례 전 1쪽의
Native·WASM raw와 두 원본의 전쪽 TSV, 각 backend review·overlay를 정확한 SHA의 raw image URL로 게시한다.
그림은 `mydocs/pr/assets/johndoekim-20261116/`의 실제 측정본을 사용하고, PR 본문에도 각 backend 증적이
보이는지 확인한다. #9076 완료와 #9058 OPEN 유지의 범위를 구분하고 `--body-file` 게시 뒤 API로 재확인한다.

## 수용 PR code CI 완료

[수용 PR #9126](https://github.com/edwardkim/rhwp/pull/9126)의 code head `529ba31acbdea187a2ed5d9528be419f26686132`에서 [필수 CI와 실제 실행](../assets/johndoekim-20261116/code-ci-529ba31.md)을 모두 확인했다. 셀프 리뷰 기록 뒤 trailing head 검사·merge·후속 처리를 진행한다.

## 원격 통합 결과

[수용 PR #9126](https://github.com/edwardkim/rhwp/pull/9126)은 최종 head `2d80477a8c817041576157aa2d786293b877fe97`의 모든 check·정책·`CLEAN`과 최신 base 병합 tree를 재확인한 뒤 정상 merge commit `fb7367574f13e65111f162bb79238cb0af45b36e`로 병합됐다. 원 PR의 간접 MERGED와 연결 이슈의 completed 자동 종료도 확인했다. 댓글·duration·정리의 실제 결과는 [후속 원장](../assets/johndoekim-20261116/closure.md)에 기록한다. 앞 절의 남은 조건은 각 단계 당시 기록이다.

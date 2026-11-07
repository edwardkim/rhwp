# R45 리베이스와 충돌 해소 기록

## 완료한 Git 작업과 검증 범위

이전 base `5077b216cbc51d2046da07350de562f6f521a7c1`에서 새 devel
`7f76cee60e38fceb63feacaff706cf984b19c427`로 142개 커밋을 전용 작업 트리에서
리베이스했다. 마지막 `git rebase --continue`의 실제 종료 코드는 0이며,
결과는 `0e8e2a6059130a19e37398545e9da8433e74b670`이다.

리베이스 전 head는 `741a9a524e374da83c2ec0b6917317518007e0bb`다.
142개 전체의 저자·이메일·저자 시각·제목·순서를 Git에서 대조해 모두 보존했음을 확인했다.
원 기여자 27개 PR의 56개 커밋과 메인터너 보정·기록 커밋을 합친 수다.
원 PR을 다시 체리픽하거나 원격 head를 수정하지 않았다.

이 기록 시점에는 기본 작업 트리에서 위 리베이스 전 head의 최종 영향 쪽 WASM 검증이
진행 중이다. 리베이스 후 실행과 섞지 않으며, 새 후보를 기본 작업 트리에 반영한 뒤 실제
검증 결과를 아래에 추가한다. Git 충돌 해소와 format 성공은 기능 보존의 증거가 아니다.

## 충돌 처리

| 위치 | 충돌 파일 | 보존한 계약과 실제 확인할 경계 |
| --- | --- | --- |
| 글꼴 registry | `assets/font-rules/font_rule_registry_v2.json`, `assets/font-rules/font_rule_projection_manifest.json`, `scripts/tests/font_rule_projection_gen.test.mjs`, `tests/cases/issue_4966_font_rule_projection.rs` | 새 base의 한겨레 웹폰트·CanvasKit 규칙과 원 PR의 윤고딕250 메트릭을 정본 reducer로 합쳤다. change set 순서를 21로 정하고 parent hash·projection을 공식 도구로 생성했다. active 855·retired 8, Node 계약 16건 통과. sealed v1은 보존했다. |
| 본문 넘침 원장 | `tests/fixtures/body_overflow_baseline.tsv` | 새 base에서 복원한 `issue5679/10857_delegation_rules.hwp`의 1과 원 PR의 `issue8596/policy_committee_agenda.hwp`의 1을 함께 보존했다. 허용값을 늘리지 않았다. |
| 글자 겹침 원장 | `tests/fixtures/text_overlap_baseline.tsv` | 같은 전결규정의 1과 `issue8641/field_table_after_page_box.hwpx`의 6을 함께 보존했다. 서로 다른 입력의 추가 행이며 기존 수치를 바꾸지 않았다. |
| 공백 줄바꿈 | `src/renderer/composer/line_breaking.rs` | 새 base의 `first_interword_separator` 제외 조건을 유지하면서, 해당하지 않는 줄 끝 공백의 원 PR #8639 `cut`·`hung_extra` 소비를 보존했다. 첫 낱말 구분 공백·문단 끝 공백·#7418 경계의 실제 결과를 함께 확인한다. |
| 표 host 이월 | `src/renderer/typeset/table/block/prepare.rs` | #8652의 보이는 되감긴 host는 방금 낸 줄을 거두고 표와 함께 다음 쪽에서 다시 준비한다. 그 밖의 경로에서는 새 base의 계산된 빈 host 선행 채움을 유지했다. 빈 host와 보이는 host의 소유 차이를 같은 조건으로 합치지 않았다. |
| 구형 수식 | `src/renderer/equation/layout/script_geometry.rs`, `src/renderer/equation/layout/source_glyph_runs.rs` | 원본의 font/version 문자열이 실제 생략된 수식과 HYhwpEQ를 명시한 구형 표시를 모델 속성으로 구분했다. 전자의 한글·수학 기준선과 후자의 아래첨자 0.28em 계약을 각각 유지하고, 새 base의 DPI 전달을 보존했다. 두 표시 세대의 독립 Print와 회귀를 대조한다. |

충돌 파일 10개의 6차 중단을 해소했다. 처리한 Rust 파일 네 개는 `rustfmt --check
--edition 2021` 종료 코드 0을 확인했다. 이를 전체 Cargo format·컴파일·회귀 통과로 쓰지 않는다.

## 새 base에서 보존할 기능

새 base는 [PR #8754](https://github.com/edwardkim/rhwp/pull/8754)의 squash 결과다.
그 PR의 최종 review·실제 diff·전쪽 원장을 먼저 확인했다. 새 체리픽 대상에 추가하는 작업이
아니며, 현재 PR과 공유하는 조판·글꼴·그리기 경로의 리베이스 보존 검사다.

- 셀 도형 편집의 실제 선택·이동·삭제·Undo 및 저장 재열기.
- 명시 구형·현대 수식의 글리프·글줄·앞뒤 문항 보존과 생략 속성 수식의 별도 계약.
- 표 첫 조각·분할·이어받기·종료와 후속 본문의 소유·점유.
- 원본 TTF → TTC의 일치 face → OTF 순서와 실제 family·style·coverage에 따른 공급.
- 실제 Native Skia·Canvas2D·지원 CanvasKit 소비와 현재 원 PR의 변경 영역.

새 devel과 후보는 같은 입력·독립 Print·글꼴 공급·DPI·compat에서 대조한다. 기존 PR
#8389·#8397의 표시값 하락 금지 및 현재 PR 전쪽 90% 조건을 그대로 유지한다.
과거 producer나 Mac의 점수를 현재 Linux 후보의 새 실행 결과로 옮겨 쓰지 않는다.

## 로컬 로그와 관측 오류

아래 로그는 ignored `output/pr-review/planet6897-integration-20261106/` 아래에 있다.

- `logs/rebase-r45.log`: 최초 실제 리베이스와 첫 충돌, 종료 코드 1.
- `logs/rebase-r45-font-registry-resolution.log`: 정본 projection 생성·Node 16건, 종료 코드 0.
- `logs/rebase-r45-continue-01.log`: 첫 충돌 해소 뒤 본문 넘침 원장 충돌까지의 실제 출력.
  이 호출은 부모가 종료 코드를 수집하지 못했으므로 코드를 추정하지 않는다.
- `logs/rebase-r45-continue-02.log`부터 `05.log`: 다음 충돌에서 각각 실제 종료 코드 1.
- `logs/rebase-r45-continue-06.log`: 142/142 완료, 실제 종료 코드 0.
- `logs/rebase-r45-conflict-format.log`: Rust 충돌 파일 format, 종료 코드 0.
- `rebase-r45-all-commit-map.tsv`, `rebase-r45-commit-preservation.json`: 실제 Git 대응표와 저자·순서 보존 확인.

sparse 작업 트리에서 기존 정본·Print가 보이지 않아 projection 명령이 중단됐을 때는
해당 HEAD 또는 stage된 새 파일의 정확한 Git blob만 materialize해 재실행했다.
입력을 새로 만들거나 검증 조건을 우회하지 않았다. 실패 로그도 구분해 보존했다.

WASM 실행 준비의 `PermissionError`는 검토용 셸 스크립트의 실행 비트 누락이었다.
같은 스크립트를 `bash`로 명시해 재개했으며 제품 코드를 바꾸거나 이미 통과한 58건을
반복하지 않았다. 재개 전 실패를 최적화 WASM 빌드 성공으로 세지 않는다.

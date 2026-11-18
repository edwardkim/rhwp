# PR #9057 리뷰 — 글자 없는 글상자 문단의 표 줄 소속

## 최종 판정

**메인터너 보정 후 수용 가능 — #9021 해결 범위의 로컬 필수 검증 완료.**

보완 code head `3d3bd2b41d28bb0d9f86f45351f74cafd680e0d8`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/43580492773), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43580492668), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43580492683), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/43580492673), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/43580492685)와 최종 CI Impact Policy가 모두 성공했다. Archive A~E·Native Skia·lint·Frontend package 검사를 포함한다. [실제 체크 원장](../assets/issue_9021_textbox_tac_tables/reviewer-20261114/merge/code-ci.json).

문서 trailing head의 실제 CI·mergeability와 정확한 head를 확인한 뒤 승인·병합한다. 미래의 병합을 완료로 기록하지 않는다.

## 접수와 기여 구분

- 원 PR: [#9057](https://github.com/edwardkim/rhwp/pull/9057), 작성자 `johndoekim`, 관련 이슈 [#9021](https://github.com/edwardkim/rhwp/issues/9021).
- 원 제출 head `079ec7bc893fd2e68bdb9d28c96c824149bdddc6`, oracle·PDF 정리와 충돌 해결 뒤 head `f42f34dc45e58c5a4602af0c8df4976d918a9d58`.
- 원 fork `johndoekim/rhwp:fix/issue-9021-textbox-control-only-tac-lines`, `maintainerCanModify=true`. 사용자 요청에 따라 원 PR에 보완 기록을 넣고 직접 승인·병합하는 collaborator 9.3.1 경로다. 기여자 커밋을 재작성하지 않는다.
- 제품 수정은 원 기여자의 공통 줄 소속·기준선 소비 변경이다. 메인터너 보정은 독립 PDF의 canonical 이름, 누락된 쪽수 oracle 1행, devel 충돌 해결, 회귀 파일의 관계형·고정형 유형 표기와 검증 기록이다. 제품 조건·수치·기대값을 추가로 바꾸지 않았다.

## 원인과 조판 원칙 검토

실제 한/글 2022 저장본의 문단은 글자 없이 표 3개를 포함한다. `char_count=25`는 제어 슬롯 3×8과 문단 끝이고, 저장 LineSeg의 `text_start=0,16`은 첫 줄 표 2개와 둘째 줄 표 1개의 소유를 나타낸다. 문자 번호에 투영하면 두 시작이 0으로 겹치므로 기존 `char_start`만으로 소유를 정할 수 없다. 독립 Desktop 한/글 2022 12.0.0.535의 Microsoft Print to PDF가 두 줄 배치와 짧은 둘째 표의 기준선 정렬을 확인한다. 웹 Print나 수정 구현의 좌표를 정답으로 사용하지 않았다.

| 생산·소비 경로 | 확인 결과 |
| --- | --- |
| `composer/frame_geometry.rs::control_line_indices` → `shape_layout.rs`의 `owner_lines`, `ctrl_line`, `tac_line_plan` | 저장 줄은 제어 슬롯 축, 합성 줄은 공통 조합 결과를 소비한다. 글자 없는 문단의 여러 줄이 한 줄로 합쳐지지 않는다. |
| `stored_tac_frame::line_table_owner` → `companion_table_top` → `fallback_y` → `table_y` → `layout_embedded_table` | 표 셀·글줄에서 이미 사용하는 공유 기준선을 재사용한다. 선언 높이와 유효한 저장 줄을 확인하며 문서 ID 분기가 없다. |
| `inline_origin.map_or(fallback_y, ...)` | 뒤의 원점 선택도 확인했다. 이 재현물은 저장 줄 경로에서 공통 기준선 결과를 소비한다. 편집 후 합성 줄은 기존 inline 원점 계약을 유지한다. |

새로운 휴리스틱이나 특정 문서용 85% 보정이 아니다. 기존 #7049·#7150의 공통 계약을 글상자에도 적용한다. 이 PR은 pagination 컷·rowspan·continuation 예산을 바꾸지 않는다. 기존 #7524의 저장 줄 없는 그림, #6465의 둘째 줄 인라인 개체와 정상 대조군을 함께 실행했다.

## 검증 범위와 생산 출처

최신 base `88170fd75f9d0ce156fe49d79aeabea9bb3a4bcc`와 원 source `f42f34dc45e58c5a4602af0c8df4976d918a9d58`의 병합 후보를 검증했다. 임시 producer `b6da239caeaf551947a9a6300d25a42f0cd86ab6` 위의 회귀 머리말 보완을 포함한 tree는 `1675188eb5478913195895a17d3b07c997c401f6`다. 원 가시성 branch에서 만든 보완 code head `3d3bd2b41d28bb0d9f86f45351f74cafd680e0d8`를 최신 base와 합친 tree와 완전히 같다. 임시 merge commit은 fork에 push하지 않았다. [트리 대조](../assets/issue_9021_textbox_tac_tables/reviewer-20261114/merge/source-head-bridge.json).

[최종 증거 묶음](../assets/issue_9021_textbox_tac_tables/reviewer-20261114/merge/README.md)에 전체 TSV, 입력/PDF·실행 파일 해시, 최소 글꼴 환경, 직접 판독 PNG를 보존했다.

| 검사 | 실제 결과 |
| --- | --- |
| 최소 글꼴 전체 nextest | 13,526 통과, 45 정책상 제외 |
| Native lib | rhwp 3,852 통과·12 ignored, workspace 라이브러리 182 통과 |
| Native 누락 그림·직접 PDF | 2/2, 36/36 통과 |
| fmt·Native/WASM Clippy·workspace build·all-target Clippy·suite 정책 | 모두 통과 |
| 보호 문서 전쪽 Native/fresh WASM | 18문서 × 2경로, 3,542쪽, 최저 98.09%, 보호 하락·쪽수 불일치·누락 0 |
| 같은 조건의 최신 base 전쪽 비교 | 양쪽 3,542쪽 공식 raw 값 동일. Native 1,771 PNG는 바이트까지 동일 |
| WASM PNG 직접 차이 판독 | 1,768쪽 바이트 동일. 나머지 세 쪽에서 각각 경계 픽셀 하나의 명암만 다름(최대 채널 차이 1·9·3). 독립 PDF와 3배 확대에서 배치·내용 변화 없음 |
| 별도 필수 `--handbook-gate` | HWPX Native 383쪽 passed, 최저 99.96%, 보호 하락 0 |
| #9021 전체 1쪽 | Native/WASM 각각 base 61.61% → 후보 100.00%, 두 일반 시각 gate passed |

보호 TSV는 silhouette-only의 공식 보호값 검사다. 이를 36개 일반 시각 gate 실행으로 표현하지 않는다. #9021은 별도로 일반 시각 gate·review·overlay를 실행했다.

검증 중 #9007이 devel에 추가되어 새로운 base에서 다시 빌드·전체 회귀·lint·전쪽 검증했다. 무늬 HWP/HWPX 각 1쪽, hwpspec 178쪽, 59043 37쪽의 양 경로 434쪽을 base/후보 모두 출력했고 공식 값은 전부 같다. 무늬 방향·격자·무늬 없음과 hwpspec 23쪽의 표·뒤 본문도 직접 판독했다. 59043의 기존 90% 미만 8쪽은 [#7778](https://github.com/edwardkim/rhwp/issues/7778)에 속하며 base와 동일하다. 이 PR의 개선 완료 문서로 분류하거나 검사·기대값을 바꾸지 않았다.

앞서 `b565d9c54a`와 `52fdce92f6`에서 #9042의 7개 oracle 행과 #9021 1행을 보존했다. [이전 devel 동기화 증적](../assets/issue_9021_textbox_tac_tables/reviewer-20261114/devel-sync/README.md)의 8문서 15쪽 양 경로 100.00%는 당시 source에 귀속한다. 이후 a210 기준 #9069/#9063 관련 집중 16개와 대표 4문서 6쪽의 Native/WASM 보존을 확인했으며, 최종 전체 nextest에서도 해당 검사가 통과했다. 과거 실행을 이번 producer의 실행으로 바꾸어 기록하지 않는다.

최소 회귀 환경은 실제 OS 글꼴 237개, 명시적 저장소 테스트 자원만 노출하고 사용자·공유 글꼴과 시각 검증 전용 공급은 차단했다. 초기 실패는 테스트가 생성한 임시 글꼴의 접근 제한으로 확인해 해당 생성 경로를 허용했고, 전체 13,526개를 다시 실행해 통과했다. 제품이나 assertion은 바꾸지 않았다.

공유 target에서 base 산출물이 후보와 동일해진 실행을 해시와 실제 #9021 재현으로 발견해 폐기했다. 유휴 rhwp 패키지 캐시를 비우고 base의 명시적 manifest·작업 디렉터리에서 Native와 최적화 WASM을 재빌드했다. 최종 base는 두 경로 모두 수정 전 61.61%를 재현하며, 최종 비교에는 이 출처가 확인된 산출물만 사용했다. sparse CLI 문서 누락은 해당 Git blob 그대로 복원했다.

## 글꼴 환경과 직접 판독

사용자가 알려 준 T7 글꼴 저장소에서 774개 파일을 확인했다. 한꺼번에 공급하면 `native_pua_fonts.rs::MAX_PROGRAMS`의 256개 snapshot 한도로 필요한 뒤쪽 HFT 글꼴이 빠지므로 기존 문서별 목록으로 공급했다. 공개 KoPub와 원 HFT를 구분하고, 저장소에 이미 기록된 독립 PUA 92개·legacy 산술 기호·정규/굵은 삼각형의 원 윤곽과 advance 해시를 검증했다. 대상 문서 PDF에서 글꼴을 뽑아 답을 맞추거나 윤곽·너비·배치를 바꾸지 않았다. 글꼴 바이너리는 커밋하지 않는다.

#9021 1쪽은 최신 base Native/fresh WASM 61.61%에서 후보 Native/fresh WASM 모두 100.00%로 개선됐다. 전쪽 쪽수는 독립 PDF와 같은 1쪽이다. 2배 확대에서 첫 줄 표 2개, 둘째 줄 표 1개, 낮은 표의 공유 기준선, 글상자 내부 포함·비겹침을 확인했다. 이 원본은 텍스트와 수식이 없는 표 재현물이다. 괘선 두께·래스터 차이는 남으므로 raw 100%를 픽셀 동일성으로 설명하지 않는다.

sample16 HWPX p22 사각형 기호와 p43 산술 기호 및 주변 간격, sample10 p81·351 삼각형과 제목·본문, nested p4 중첩 표도 Native/WASM/독립 PDF의 같은 영역을 직접 대조했다. 정본 보호값과 본문 흐름을 확인했으며 점수만으로 직접 판독을 대신하지 않았다.

현재 대상 원본을 참조하는 검사와 corpus를 검색했다. 다섯 기존 함수는 이미 줄 소속·기준선·포함·비겹침 관계 검사다. 잘못된 절대 px 핀을 바꾼 것은 없으며, 독립 PDF의 1쪽을 확인하는 정확한 고정 쪽수와 내용 개체 보존 검사를 유지했다. 쪽수 oracle은 `1 → 1`로 등록돼 공식 16개 partition 검사에 포함된다.

## 기존 미해결 범위와 통합 경로

원 PR이 별도로 명시한 글상자 그림 앵커 관찰과 #9058의 FIELD_END 제어 슬롯 경계는 이번 해결 범위가 아니다. 기존 공통 helper의 미해결 관찰을 새 회귀나 해결 완료로 분류하지 않는다. 현재 실제 저장본 #9021의 표 배치·기준선·포함은 독립 Print와 두 backend에서 확인했다.

원 PR 직접 보정·trailing 기록 경로를 유지한다. 기여자 commit·author를 재작성하거나 별도 통합 PR로 옮기지 않는다. 사용자 요청에 따라 최신 필수 CI 확인 → 정확한 head 승인 → merge 방식 통합 → issue·댓글·로컬 정리로 진행한다.

## Merge 후 contributor PR comment 계획

최종 head CI와 실제 merge SHA를 확인한 뒤 원 기여에 감사하고 제품 수정과 메인터너 보정을 구분해 설명한다. #9021은 표 줄 배치 범위가 해결됐는지 확인해 닫고, 본문에서 명시적으로 제외한 `insertPicture` 앵커 관찰을 해결했다고 쓰지 않는다.

댓글에는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), 검토 문서, 최종 merge SHA로 고정한 #9021 Native/WASM review·overlay 이미지와 2배 확대 증거를 넣는다. raw 응답의 PNG 형식·SHA를 확인하고 구조화된 도구 인자 또는 `--body-file`로 한국어 존댓말 본문을 게시한 뒤 API 본문과 실제 이미지 표시를 확인한다.

PR head의 review·asset·오늘할일로 후속 기록을 처리한다. 병합 뒤에만 확정되는 merge SHA·duration 결과·issue/comment URL은 GitHub 후속 기록으로 연결한다. 새 검증 CI를 병합 후에 시작하지 않는다. 기본 devel fast-forward와 사용자 변경 보존을 확인한 뒤 이 작업 전용 worktree·branch·output 및 유휴 공유 debug를 정리한다. 다른 작업 소유 자료와 contributor fork branch는 보존한다.

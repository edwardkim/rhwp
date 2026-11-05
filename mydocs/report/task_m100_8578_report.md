# Issue #8578 최종 PR 검토 — 묶음 글상자의 클릭·Esc 편집 복귀

## 최종 판정

**머지 보류 — 필수 로컬 검증과 사용자 정상 동작 확인을 완료했고, PR 생성 승인을 받았다.**
원격 PR·CI·병합은 이 기록 작성 시점에 미실행이다. 기술 검토에 남은 보정 요구는 없다.
이 문서는 이슈 기준 제출 전 로컬 검토이며, 채번·code head CI 성공 후의 정식 PR 기록과 구분한다.

검토 갱신: 2026-11-05 KST. 아래 원격 상태는 작성 시점의 관측이다.

## 제출 범위와 신원

- 이슈: [#8578](https://github.com/edwardkim/rhwp/issues/8578), 편람 3·4쪽 클릭·Esc 편집 전환.
- 작업 위치·branch: `/Users/tsjang/rhwp`, `task/8578-group-text-editing`.
  사용자 지시에 따라 이전 `rhwp-8578-group-text-editing` 작업트리를 제거했다.
- 최신 base: `6e1de406209ba9a58032f6ca6de45b18aa687d1c`.
- 실제 컴파일·전체 회귀·브라우저 검증 head: `2c0432cc0646cf2ad0b277b50e4c685ab7092a80`.
- 이 보고서와 최종 증적 commit은 위 코드에서 source/test tree를 바꾸지 않는다.
- [producer와 파일 해시](../pr/assets/issue_8578/20261105/producer.json),
  [source tree](../pr/assets/issue_8578/20261105/source-equivalence.json),
  [최종 결과](../pr/assets/issue_8578/20261105/verification.json)를 연결한다.

이번 수정은 **편집 소유 주소와 Esc 복귀를 보정**한다. 기존 문서의 조판 수치·쪽 나눔·인쇄 좌표·
paint 규칙을 바꾸지 않는다. 입력 후에는 선택한 자식 글상자의 원본 폭을 사용해 기존 재조판
경로가 같은 소유 내용을 처리하도록 한다. 다른 HWP 문서의 조판 보정은 범위 밖이다.
리베이스 기능 보존을 넓게 적용해 시작했던 다른 문서 시각 검사는 사용자 범위 확인 후 중단했다.
그 중간 PNG·TSV·진단 기록은 ignored output에만 두고 제출 판정과 증적에서 제외했다.

## 결함과 수정 경로

| 원인 | 공통 수정·실제 소비 근거 |
| --- | --- |
| 묶음의 제목·본문에 같은 글상자 번호 0을 부여 | `ShapeObject::textbox_shape_count/at/at_mut`가 원본 자식 순서의 실제 글상자를 구분. 장식·중간 Group은 번호 미소비, 실제 빈 글상자는 소비 |
| Group 주소를 독립 도형 조회로만 해석 | indexed helper를 커서·텍스트 편집·clipboard·중첩 표·inline control·hyperlink에 공유. normalization cache에도 글상자 번호 포함 |
| 장식의 bbox가 빈 글상자 클릭 후보가 됨 | 실제 글상자 배치에서만 내부 `textbox_source`를 생산하고 클릭이 해당 소유 경로를 소비. metadata는 직렬화하지 않음 |
| Esc가 삭제용 인접 문단 착지를 사용 | `returnToSelectedObjectOwner()`로 원 소유 본문/셀 복귀를 분리. 삭제·붙여넣기의 인접 이동은 유지 |
| 빈 Group 소유 본문에 캐럿이 없음 | 확정 본문 흐름 원점에 기존 숨은 문단 표시기를 재사용. Column/Body만 적용, Header/Note 주소 비누출, 실제 공백 보존 |

문서명·쪽 번호·특정 문자열로 구현을 분기하지 않고 Group을 해제·복제하지 않는다.
기존 #1892 공백 보존과 #919 중첩 표의 바깥 글상자 소유 계약은 기대값을 유지한 구현 보정으로
복구했다. 원 PR·통합 보정과 중간 실패는 [단계 기록](../working/task_m100_8578_group_text_editing.md#전체-회귀에서-발견한-호환성-보정)을 따른다.

## 입력과 최종 검증

원본 [편람 HWPX](../../samples/2025%20행정업무운영%20편람%28최종%29.hwpx)의 SHA-256은
`c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`이다.
독립 [한컴 Print PDF](../../pdf/2025%20행정업무운영%20편람%28최종%29-hwpx-kopub-2024.pdf)의 SHA-256은
`299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`이다.
기존 #8389의 383쪽 자료를 그대로 사용했고 이번 작업의 새 MCP 출력은 없다. 원본·PDF·글꼴·허용치를 변경하지 않았다.

| 검증 | 실제 결과 |
| --- | --- |
| Linux 최소 글꼴 전체 nextest | **12,591 PASS / 0 FAIL / 45 skip**, 실행 936.659초, exit 0 |
| Native Skia lib | 본체 3,858 PASS·12 ignore, 다른 libs 182 PASS, 실패 0 |
| Native Skia PNG·PDF | PNG 2 PASS, PDF 22 PASS |
| 집중 계약 | 새 #8578 8건 + 기존 #1892·#919 각 5건 = 18 PASS, Mac Native·최소 글꼴 Linux 모두 통과 |
| Mac gates | fmt·Native/WASM/all-target Clippy·workspace build·manifest·unit tiers·E2E manifest·TypeScript 통과 |
| Studio 단위 검사 | **2,721 PASS / 0 FAIL / 2 skip** |
| 실제 Studio Canvas2D | 원본 383쪽 열기, 3쪽 입력/드래그/삭제/Enter/undo/redo, 4쪽 본문 입력, 개체 선택→Esc→원 본문 입력 통과; 관련 소유·캐럿 경고 0 |
| 사용자 확인 | 사용자가 정상 동작을 검증했다고 확인. 사용자 환경의 프로그램 hash를 별도로 단정하지 않음 |
| 변경 쪽 Native/fresh WASM | **3·4쪽 각각 100.00%**, 새 devel과 후보 PNG SHA·TSV 모두 동일 |
| 동일 편람 Native 추가 gate | **383쪽 최저 99.96%·130쪽**, 미달·누락·base 대비 하락 0, 전383쪽 PNG SHA 동일 |

서로 다른 실행 묶음을 고유 테스트 총수로 합산하지 않는다. Linux 회귀는 시스템·사용자 글꼴을
차단하고 프로젝트/fixture 프로그램 64개만 노출했다. Mac 시각 비교는 기존 Print와 같은 글꼴을
공급한 환경이며 최소 글꼴 시각 정합으로 바꾸어 주장하지 않는다.

새 devel의 #8609·#8622 등은 코드 통합을 포함하므로 과거 전체 통과를 재인용하지 않았다.
위 전체 회귀·Skia·실제 편집 경로와 해당 편람 출력은 새 source에서 다시 검증했다.
다른 PR이 주장한 문서 전체를 이번 이슈의 새 보정 범위로 확장하지 않는다.

### WASM 산출물 검증

공유 target에서 기준 WASM을 만든 뒤 첫 후보 호출이 캐시를 재사용하여 기준 프로그램이 후보
pkg에도 남은 것을 실제 글상자 API에서 발견했다. 해당 artifact는 무효로 분리했다.
공유 target을 삭제하지 않고 자신의 `src/lib.rs` mtime만 갱신해 byte 변경 없이 재컴파일했다.
최종 프로그램에서 제목과 본문을 별도로 읽고 실제 브라우저 편집까지 통과했다.

- Native 후보 SHA-256: `14a96dc997c1c3da23b8e263c2ba7754afdba5c175f159320f4ef6f5d42904a8`.
- 최종 WASM SHA-256: `d98be3e9a846e33ebc43a792070afe47b0ad742774c17652fdc56abaaaa48cde`.
- pkg/public JS SHA-256: `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`.
- 실제 초기화 응답을 확인했다. Vite 변환 JS는 served source와 대조한다.
- Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

## 조판 원칙 준수 검토

| 항목 | 근거·판정 |
| --- | --- |
| 로컬 검증·PR 상태 | 필수 Rust·Studio 검증 충족. PR 생성 승인 받음; GitHub CI·merge 미실행 |
| 선행 해결 분석 | #8578 원본 구조·한컴 편집기 기대 동작, #919 원 해결 `14920220ac`, #1892 원 PR #1905·후속 `6097253468` 확인 — 충족 |
| 일반성 | 원본 자식·실제 글상자 소유를 공통 helper로 해소. 장식·빈 글상자·중첩 표·Header 반례 — 충족 |
| 측정·배치 | 확정 본문 원점과 해당 자식 원본 폭을 소비. 전진·paint 규칙 변경 없음; 동일 편람 출력 보존 — 충족 |
| 리베이스 기능 보존 | 새 source의 전체 회귀·Skia와 해당 편집 소비 경로 재검증. 다른 HWP 시각 확대는 사용자 지시로 중단 — 충족 |
| 실제 소비 계약 | Native API·fresh WASM 실제 응답·Canvas2D 클릭/키보드·저장/reopen을 구분해 확인 — 충족 |
| 분할·이어받기 | 컷·예산·이월·쪽 할당 정책 변경 없음 — 비해당 |
| 줄 소속·높이 | 줄 높이 정책 변경 없음. 입력 후 실제 자식 폭으로 재조판하고 형제 문단·내용 보존 — 충족 |
| 독립 근거 | 원본 XML 소유 구조·기존 Print·실제 UI·합성 경계 반례를 구분 — 충족 |
| 기준값 변경 | baseline·golden·90%·하락 허용폭·글꼴 변경 없음 — 비해당 |
| 신규 검사 | 편집 소유·순서·내용 보존 관계 검사, 실물 px 좌표 고정 없음. 해당 3·4쪽 양쪽 100% — 충족 |
| 직접 판독 | 같은 배율의 제목·안내문·목차·장식·줄 배치 review/overlay에 새 차이 없음 — 충족 |
| 주장 범위 | #8578 편집 오류 해결과 기존 출력 보존. 편람 전체 피델리티 개선·새 후보 전383쪽 WASM 완료는 주장하지 않음 — 충족 |

## 직접 증적

| 쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 3 | [review](../pr/assets/issue_8578/20261105/native-p003-review.png) / [overlay](../pr/assets/issue_8578/20261105/native-p003-overlay.png) | [review](../pr/assets/issue_8578/20261105/wasm-p003-review.png) / [overlay](../pr/assets/issue_8578/20261105/wasm-p003-overlay.png) |
| 4 | [review](../pr/assets/issue_8578/20261105/native-p004-review.png) / [overlay](../pr/assets/issue_8578/20261105/native-p004-overlay.png) | [review](../pr/assets/issue_8578/20261105/wasm-p004-review.png) / [overlay](../pr/assets/issue_8578/20261105/wasm-p004-overlay.png) |

[Native 383쪽 base](../pr/assets/issue_8578/20261105/native-base.tsv) / [후보](../pr/assets/issue_8578/20261105/native-head.tsv),
[fresh WASM 3·4쪽 base](../pr/assets/issue_8578/20261105/wasm-base.tsv) / [후보](../pr/assets/issue_8578/20261105/wasm-head.tsv)를 제공한다.
130쪽 99.96%는 동일한 기존 Native 차이다. Native 나머지 382쪽의 두 자리 표시 100.00%와
반올림 전 정확한 100% 개수 376쪽을 구분한다. 변경3·4쪽은 두 backend의 raw 100.00%다.

[실제 UI 결과](../pr/assets/issue_8578/20261105/browser-result.json)는 3쪽 본문 `parentParaIndex=4, cellIndex=1`,
4쪽 본문 `paragraphIndex=8`, Esc 후 `paragraphIndex=4`다.
[3쪽 화면](../pr/assets/issue_8578/20261105/studio-page003-edit.png)·[4쪽 화면](../pr/assets/issue_8578/20261105/studio-page004-body.png)은 검사 중 입력 문자를 포함하며 인쇄 PNG는 미편집 원본이다.

## 원격 단계

사용자가 승인한 PR 제출을 진행한다. push 후 SHA 고정 raw 다운로드와 게시 본문을 API로 재확인한다.
정확한 PR head CI 성공 전에는 CI·정식 review/trailing·병합을 완료로 적지 않는다.
이번 승인에 merge 권한까지 포함됐다고 확대 해석하지 않는다.

---
kind: report
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-16
---

# PR #7913 self-review — 선택적 PDF 가져오기와 HWP/HWPX 저장

## 최종 판정

**승인 — 메인테이너가 정한 devel 기반 통합 범위의 구현·검증 완료.** 정확한 code head의
GitHub CI가 성공했고, 사용자는 CI 성공 후 Ready 전환과 self-merge를 승인했다.
후속 백로그의 완료나 비신뢰 PDF 배포 준비 완료를 뜻하지 않는다. 이 문서만 추가한
trailing head의 required checks·현재 base와의 병합 가능성을 확인한 뒤 병합한다.

- PR·작성자·base: [#7913](https://github.com/edwardkim/rhwp/pull/7913), `edwardkim`, `devel`.
- 검토 code SHA: `d6bde7307d29bef99b725ca3533a721605146eda`.
- 고정 base: `375aff29498166260a22796d54e67f508c84aada`; 재조회 시 변경 없음.
- 관련 이슈: [#7911](https://github.com/edwardkim/rhwp/issues/7911), `Refs` 관계. 후속 범위가 남아 열어 둔다.
- 경로: collaborator self-merge, intake, local validation, visual fixture evidence,
  review-only fast-pass, post-merge. 별도 reviewer를 지정하지 않았다.
- Ready 전환 전 exact head는 `mergeable=true/clean`, check 33 success·5 정책상 skipped와
  `CI Impact Policy=success`였다. Ready 전환은 사용자 승인 후 수행했다.

## 코드와 주장 범위 검토

`FileFormat::Pdf` 감지 → 공통 parser → 요청별 추출/인식/lowering → Document IR →
DocumentCore·서비스·CLI·WASM이 같은 문서를 소비한다. HWPX를 중간 직렬화 포맷으로 거치지
않는다. `sourcePages`는 명시적인 0-based 증가 배열이며 원본 쪽 대응은 가져오기 시점의
진단으로 보존한다. 기본 빌드의 PDF 비활성 오류와 opt-in 성공 경로를 각각 검증했다.

그림은 `lowering.rs`에서 Embedding BinData와 같은 storage ID를 생성한다. HWP 내보내기는
`convert_if_hwpx_source`의 PDF 분기에서 저장 snapshot의 5.1 wire 구조를 생성하고,
PDF 출처를 HWPX 출처로 바꾸지 않는다. `issue_7911_pdf_frontend`의 저장/재열기 검사는
그림 bytes·참조·본문·쪽 대응과 원 문서 불변성을 확인한다. 실제 한컴 열기와 Print는 별도 증거다.

사각형 clip은 PDF operator의 q/Q·CTM·Form BBox에서 page 좌표의 visible bbox로 전달하고,
그림 frame과 원 bitmap의 crop이 이를 함께 소비한다. 회전/반사/shear와 비사각 clip은
현재 지원으로 가장하지 않는다. 일반 표의 물리 여백도 기존 `physical_margin`의 CHAR
단위로 변환해 저장하며, PDF 쪽 번호나 특정 문서 ID로 배치를 보정하지 않았다.
명시적 0 글상자 여백은 `text_margin_explicit`로 XML 부재와 구분한다.
상세 호출 경로·수정 전후 증거는 [3단계](../../working/task_m100_7911_stage3.md) 7–10절에 있다.

| 공통 검토 항목 | 판정과 근거 |
| --- | --- |
| 구현 근거·일반성 | 충족: HWP Embedding/wire 구조, 물리 여백 단위, PDF clip 의미를 적용. clip을 전체 bitmap 축소나 셀 clamp로 대체하지 않음. |
| 측정·배치 일관성 | 현재 가져오기 범위 충족: 공통 IR의 frame/crop·줄 위치를 기존 엔진이 소비. 임시 줄당 문단 정책은 진단에 공개하며 논리 문단 복원은 백로그. |
| 최종 출력 소비 | 충족: Native 및 실제 Docker WASM 전쪽 출력, 실제 Worker·편집 저장/재열기와 한컴 열기를 각각 확인. 전 backend/환경 조합은 미검증. |
| 분할·이어받기 | 비해당: 기존 pagination·rowspan 컷 알고리즘을 변경하지 않음. 입력 lowering과 저장 여백 보정의 후속 내용은 11쪽 그림/캡션과 전쪽 출력에서 대조. |
| 줄 소속·점유 | 현재 가져오기 범위 충족, 저장본의 한컴 재조판 1·11쪽은 미충족이며 사용자 지시로 B10-1 후속 처리. |
| 독립 사례·반례 | 원본 PDF, 생성 HWP/HWPX와 독립 한컴 Print를 구분. 선택 쪽·빈 쪽·실패 후 정상 요청·기존 세션 보존·명시적 0/부재 계약 검증. 합성 검사를 시각 증거로 승격하지 않음. |
| 기준값·회귀 선행 | 기존 baseline·허용치 완화 없음. 추가 검사는 API·저장 구조·자원 계약이며 실물 배치를 px/golden으로 고정하지 않음. |
| 직접 판독 | 3쪽 숨김 영역과 세 그림 조각 연결, 11쪽 그림 9·10/캡션 보존을 직접 확인하고 사용자 시각 통과와 연결. B10-1의 자동 미달은 유지. |

자원 경계 보완은 요청 context에 연결된 실제 예약 지점을 대상으로 한다. vendored 전체
할당·CPU 상한 검증이 완료됐다고 주장하지 않으며 `parserAllocationBoundsProven=false`와
default-off를 유지한다. 세부 미완료 범위는 [백로그](../../plans/task_m100_7911_backlog.md)에 있다.

## 검증 입력·실행 결과

원본 `pdf/issue5875/nested_table_text_caption-2020.pdf` 12쪽, 저장본
`samples/issue7911/pdf-import-nested-table.{hwp,hwpx}`, 독립 한컴 Print
`pdf/issue7911/pdf-import-{hwp,hwpx}-hancom2020.pdf`는 검토 commit에 포함돼 있다.
[입력 설명](../../../samples/issue7911/README.md)의 해시와 실제 실행 파일을 대조했다.
50 MiB 이상 PDF와 로컬 전용 글꼴·로그·TSV·WASM은 제출하지 않았다.

원본 PDF와 저장본 전쪽 비교의 최저값은 Native 90.14609%, fresh Docker WASM 90.16832%,
12쪽 일치·미달/누락 0이다. 통합 후 각 backend의 전쪽 픽셀은 기존 승인본과 동일했다.
devel은 새 제품 PDF 가져오기 경로가 없으므로 동일한 PDF import의 base TSV는 비해당이다.
기존 HWP/HWPX/HML 3문서 5쪽은 Native/WASM 각각 devel과 SVG bytes가 같았으며,
이 대조군을 전체 기존 문서의 무회귀로 일반화하지 않는다.

저장 HWP의 독립 한컴 재조판과 비교하면 1쪽 Native/WASM 68.40217%/68.72149%,
11쪽 78.54899%/78.74329%다. 사용자가 이를 B10-1로 명시적으로 이관하고 현재 기반 통합을
진행하도록 지시했다. 원본 PDF와의 90% 통과로 이 차이를 상쇄하거나 해결로 보고하지 않는다.

전체 nextest 10,435 pass·45 skip, Native Skia library 4,043 pass·12 ignored, 그림/PDF 2·4 pass,
Studio 2,014 pass·2 skip, 실제 API/Worker 83항목·5 Worker와 편집/대조 19항목이 통과했다.
fmt·Native/WASM/workspace Clippy·base 고정 정책 검사를 완료했다.
명령·source와 산출물 해시·재사용 범위는 [4단계](../../working/task_m100_7911_stage4.md)에 연결한다.

이전 CI에서 발견한 네 폰트의 공개 목록 누락은 같은 검사의 FAIL→PASS로 보정했다.
폰트 bytes는 upstream manifest와 같고 고지 전문을 보완했다. 추적 `public/rhwp.js`는
검증한 Docker default-off 산출물과 동기화했으며 실제 기본 WASM의 HWP 열기·PDF 비활성도
확인했다. Rust 제품 변경 없이 동일 빌드를 반복하지 않았다. 패키지 빌드 4개와 관련 검사 17건 통과.

## 정확한 code head의 원격 CI

모두 `d6bde7307`에서 완료됐다. skipped는 정책상 비실행 job이며 실패를 면제한 것이 아니다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/38856414501): lint, frontend package, Native Skia, 네 archive builder/worker, Build & Test 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38856414563): workflow와 GHAS check 성공.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38856414497), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/38856414540), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/38856414536), [Skill router](https://github.com/edwardkim/rhwp/actions/runs/38856414506): 성공.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/38856412452): 성공.

## 병합 절차와 남은 범위

사용자의 Ready·self-merge 승인을 적용한다. 검증된 code 뒤에는 이 review·오늘할일·단계/계획
상태만 single-parent trailing commit으로 추가한다. 현재 base와 실제 merge tree의 링크·기존
오늘 기록 보존 및 code head 이후 변경의 공백을 확인하고, push 후 exact trailing head의 CI·fast-pass를 확인한다.
실패나 base/head 변화가 있으면 그 경계를 다시 검증한다. self-approve API나 admin 우회를
승인으로 대신하지 않고 일반 squash merge를 사용한다.

PR 본문의 대표 PNG 8개는 최종 head로 고정한다. 병합 뒤 local devel 동기화와 duration
갱신 상태를 확인하며 #7911은 열린 상태로 유지한다. 공개 npm/Studio 자동 연결, 추가 자원
감사·논리 문단/그림 복원·B10-1 등은 이번 통합의 완료 범위에 포함하지 않는다.


전체 base→head diff의 공백 검사는 vendor 소스에서 기존 code head에도 있던 trailing
whitespace 15곳을 보고한다. code→trailing merge tree에는 같은 진단이 추가되지 않으며
문서 변경 범위는 공백 검사에 통과한다. 이를 없애기 위해 검증된 vendor 코드를 재포맷하지 않는다.

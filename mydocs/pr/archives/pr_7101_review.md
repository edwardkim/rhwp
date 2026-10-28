---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7101 — 템플릿 자동화 self-review

**최종 판정: 승인.** 승인된 A/B/C/D 자동화 계약에 대한 판단이다. 모든 문서의 시각 동등성이나
Studio 재편집 문제 해결을 뜻하지 않는다. 이 기록은 GitHub approve·merge·이슈 close가 아니며,
trailing 기록을 포함한 최신 HEAD의 required CI와 메인테이너의 별도 병합 승인이 남는다.

## 1. 대상과 절차

| 항목 | 검토 기준 |
| --- | --- |
| PR / Issue | [#7101](https://github.com/edwardkim/rhwp/pull/7101) / [#3587](https://github.com/edwardkim/rhwp/issues/3587) |
| 작성자 / 검토 | edwardkim / 메인테이너 self-review, 외부 reviewer 미지정 |
| branch / base | `task_m100_3587` / `devel` |
| CI 통과 HEAD | `ea2d3541a9e0555fe5491fdf21ff0ba1cf629c47` |
| 조회한 최신 base | `1ae5ca295bddcb31b846affc62834a2a3023d24d` |
| 로컬 전체 검증 내용 | `6a8aeb9ffac5d574e7a590ce66aba5eae9f7f374`; 위 CI HEAD까지 Markdown 6개만 변경 |
| 검토 전 규모 | 170개 파일, +19,411/-219; 작업 범위 109 commit |
| 기본 / 보조 경로 | collaborator_self_merge / intake_and_review, local_validation, visual_fixture_evidence, rework_and_exceptions, review_only_fast_pass |

공통 PR 절차와 위 역할별 자식 문서를 적용했다. 대형 변경이므로 단계별 계약·실측과 핵심
변경 경로를 대조했다. 19,411줄 전부를 이번 self-review에서 새로 독립 감사했다는 주장은 아니다.
구현·시험 이력의 정본은 [최종 보고서](../../report/task_m100_3587_report.md)와
[Stage 25](../../working/task_m100_3587_stage25.md)다.

## 2. 코드 검토와 지원 경계

- `paragraph_block/repeat.rs`, `import.rs`, `template_edit.rs`: 원형과 대상 경계의 검증·분리된
  준비를 먼저 수행한다. source의 용지 설정을 대상에 덮어쓰거나 기존 빈 문단을 삭제하지 않는다.
- `model/identity.rs`, `clone_identity/remap.rs`: 소유 구조 전체를 순회하고 ID 할당과 참조 연결을
  분리한다. 복사본별 대응표, 앞으로/뒤로 향한 참조, 기존 raw ID 예약·모호한 참조 거부를 확인했다.
- `template_operation.rs`, WASM·CLI adapter: 공통 DocumentCore로 위임한다. JSON 8 MiB,
  CLI source 64 MiB, 깊이·개수 예산과 source SHA 사전조건을 사용한다. dry-run도 같은 준비를 거친다.
  WASM 동일 핸들의 중복 borrow는 호출자 사전검사가 필요하다. 외부 파일 교체 경합이나 프로세스
  OOM까지 복구하는 트랜잭션이라고 주장하지 않는다.
- `rows_geometry.rs`, `validation.rs`: 겹치는/0 크기 셀, 행 경계를 넘는 병합, 제목 행 반복과
  자원·참조의 지원 제한을 검증한다. 의미를 모르는 raw 확장을 임의 정규화하지 않는다.
- `import/resources/binary.rs`: 이미지 ordinal과 storage ID를 구분하고 실제 bytes·metadata로
  재사용한다. 외부 링크·충돌은 거부하며 문서 간 스타일/바이너리 대응은 import 준비 단계에 있다.
- `clipboard.rs`, `serializer/form_identity.rs`, serializer 변경: 기존 붙여넣기도 공통 식별자
  복제 경로를 사용한다. 저장 전 필요한 form ID를 문서 단위로 할당하며 원본 raw를 무조건 재생하지 않는다.
- `renderer/typeset.rs`: Before/After/SectionEnd의 지연 표 배출 지점을 구분한다. 앞의 지연 표와
  다음 표 문단 사이에 가시 텍스트가 있으면 기존 float 흐름을 유지한다. continuation과 저장된
  host 위치/이동한 원점을 구분하며 특정 파일명·문단 번호·Enter 횟수로 분기하지 않는다.

원 제안의 네 API를 그대로 이식한 것이 아니라 승인된 자동화 범위로 재설계했다. 공개 계약은
[사용법](../../manual/template_automation.md)을 따른다. Gym은 선택적 소비자이며 제품·CI 의존성을
추가하지 않았다. Studio #7065, #7084, #7090은 이번 수용 범위에서 제외한 별도 문제다.

## 3. 조판 원칙 검토

| 검토 항목 | 직접 대조한 근거와 제한 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 원형·한컴 cp-01/cp-02와 복제 후 표 owner 순서를 대조. 지연 표 경계/가시 텍스트 반례를 구분하고 출력 은폐나 샘플 상수 없음 | 충족 |
| 측정·배치 일관성 | 반복/가져오기 commit 뒤 기존 recalculate/recompose/paginate 경로 사용. typeset의 동일 표 fragment와 cursor를 소비하며 backend별 보정 추가 없음 | 충족 |
| 줄 소속과 점유 높이 | 기존 저장 LineSeg/재조판의 줄 구성을 유지하고 deferred flush의 host frame·continuation·선언된 표 높이 처리만 변경. 새 줄 분할/글자폭 산식은 비변경; 표 성장 3계약 재실행 | 충족 |
| 사례와 증거의 독립성 | 실제 한컴 복사본과 메인테이너의 양 형식 정상 판정, 자동 계약을 구분. PDF 잔여 차이는 아래에 공개하며 모든 재편집 시각 일치로 확대하지 않음 | 충족 |
| 기준값 변경 | 신규 쪽수 5행은 한컴 PDF 근거. 신규 넘침 2행은 순수 devel과 후보의 동일 JSON·stderr 근거. 기존 행/허용치 완화 없음 | 충족 |
| 주장과 검증 범위 | 코드 SHA·전체 검증·이번 집중 실행·CI와 시각 산출물을 분리 기록. 지원 외 raw 구조 및 Studio 증상은 해결 주장하지 않음 | 충족 |

### 신규 baseline 두 행의 독립 확인

`samples/issue3587/b-table-repeated.hwp`, `.hwpx`의 `body_overflow=1`은 새 입력 등록이다.
순수 base `1ae5ca295`와 후보에서 원본 `samples/hwp_table_test.hwp` 및 두 복사본의 전체 진단
JSON·stderr가 동일했다. 원본 3쪽 중 2쪽의 기존 현상이 복사본 4쪽 중 3쪽에 나타난다.
노드는 `Page/Body/Column0/Table9`, bbox `(113.3866667, 871.1466667, 555.36, 142.76)`,
하단 초과 `4.7866667px`이며 off-canvas/text-overlap은 0이다. 기존 원본 baseline도 1이다.
이번 수정으로 넘침을 해결했다거나 실패를 숨기기 위해 기존 허용치를 올렸다는 뜻이 아니다.

## 4. 실행 증적

| 검사 | 실제 결과 |
| --- | --- |
| Stage 25 전체 nextest | 9,729 PASS / 0 FAIL / 51 skipped, run `cfafe6ae-ac41-4391-8adf-9887e2a19b90` |
| 집중 / 신규 입력 보안 | 192 PASS / 문서 41개 명시 입력 6 PASS |
| Rust 제출 게이트 | fmt, native/WASM32/all-target Clippy, workspace build, manifest·unit tier PASS |
| Native Skia | root 3,930 PASS/13 ignored, 내부 182 PASS, picture 2 PASS, 직접 PDF 4 PASS |
| Docker WASM | 표준 최적화 빌드 PASS; wasm SHA `c31a359626b096d5f38d44fe6b1ae9a7e58093655ed4bb6c698c969a3d3e854b` |
| 이번 표 흐름 집중 재검증 | `issue_3587_repeated_table_flow`: 3 PASS, run `376f6662-7cbd-4c58-b5b3-be1bfb32ed09` |
| 이번 실제 WASM 재검증 | 두 입력 형식 dry-run/apply, 오류·alias 사전조건·0회 및 HWP/HWPX 저장 계약 PASS |
| 선택적 Gym | 양 형식 각 13검사와 음성 대조 3건 통과. 최종 bytes는 메인테이너 판정본과 동일 |

이번 집중 실행은 detached `/home/edward/mygithub/rhwp-review-3587`의 정확한 EA HEAD에서
`node scripts/run-rust-test.mjs issue_3587_repeated_table_flow -- --cargo-profile release-test
--target-dir /home/edward/mygithub/rhwp/target/pr-review`로 수행했다. 선택되지 않은 200건은
이 집중 명령의 필터 제외이지 전체 회귀 실패/누락 수가 아니다. nextest 0.9.137의 권장 버전
0.9.140 경고가 있었지만 실행 exit 0이다.

WASM은 `node mydocs/tech/investigations/issue-3587/probes/import-wasm-contract.mjs
output/3587/review-7101/wasm-contract`로 exit 0을 확인했다. 브라우저 UI를 다시 조작한 검사는 아니다.
source/test가 동일하고 exact HEAD의 Full CI가 성공했으므로 전체 회귀를 문서 검토 때문에 반복하지 않았다.

Gym은 마지막 문단에 복사하여 실제 suffix가 0개다. suffix 보존은 B/D 계약으로 검증했으며
Gym 하나가 모든 경계 사례를 검증했다고 주장하지 않는다. 외부 clipping 92개 묶음은 미실행이다.
D3 100회 가져오기 native 중앙값 15~20ms/CLI 전체 263~379ms는 제한된 템플릿 비용이며
렌더러 전체 성능 영향도나 기존 대비 속도 개선 증거가 아니다.

### EA HEAD의 GitHub CI

- [CI Full](https://github.com/edwardkim/rhwp/actions/runs/34751189482): Lint·Build & Test·archive workers·Native Skia 등 success. 단순 fast-pass 성공이 아니다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34751189470): JS/Python/Rust success; [GHAS 후속 check](https://github.com/edwardkim/rhwp/runs/103708263284)도 success.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34751189334): success.
- [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34751189430), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34751189621), [Skill Router](https://github.com/edwardkim/rhwp/actions/runs/34751189358): success.
- [최종 CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/34751836104): success. 확인 당시 PR MERGEABLE/CLEAN.

## 5. 검증 입력 커밋과 시각 증적

**검증 입력 커밋 확인: 충족.** [MANIFEST](../../../samples/issue3587/MANIFEST.json)의
73개 대응/37개 고유 경로를 실제 파일·EA HEAD blob·기록 SHA-256으로 재대조하여 모두 일치했다.
기존 3개 재사용과 신규 34개(문서 29/PDF 5)이며 `samples/rnote` 원본 12개도 추적 보존되어 있다.
새 integration source는 `tests/cases` 21개만 포함하고 generated suite/manifest는 제출하지 않는다.

Stage 25의 대표 review PNG 두 장을 이번 self-review에서 직접 다시 열었다. 같은 bytes를
아래 PR asset으로 보존한다. 완전한 PDF 동등성 PASS로 읽어서는 안 된다.

| 대상 | 완료/누락/flagged | 대표 페이지 pixel match / 내용 픽셀 보조값 | 직접 확인한 잔여 차이 |
| --- | --- | --- | --- |
| labnote p1, p2 | 2/0/0 | p2 91.08866% / 0.39487% | 세 표와 내용은 있으나 표 사이 간격·세로 선·텍스트 위치 차이 |
| textbox 원본 p4 → 가져온 문서 p1 | 1/0/0 | 92.02321% / 64.48282% | 글상자/표/그림/텍스트 존재, 배경·dash·폰트·위치 차이 |

labnote p2 코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 0.39%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

textbox 코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 64.48%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

- [labnote 대표 review](../assets/pr_7101_labnote_p002_review.png), SHA-256 `ab964a2377b570fe399e77bcabecf93e41aaf8f5533ee6a146cbb30c2f5c0a54`.
- [textbox 대표 review](../assets/pr_7101_textbox_p001_review.png), SHA-256 `2882e45cb6626f74785ae3ddbe6791dd5e6cf0d976d81fa24d2054d569af8ab4`.

원본 비교 경로는 `output/3587/submission-stage25/visual-labnote/labnote/`의
`compare/compare_002.png`, `overlay/overlay_002.png`, `review/review_002.png`와
`visual-textbox/textbox/`의 대응 `_004.png`다. Linux native debug+rsvg, pixel threshold 32의
비교이며 Chrome/Studio 웹폰트가 적용된 비교가 아니다. 정확한 base/head 시각 차분도 아니다.
flagged=0은 사람 판정 PASS가 아니다. 저장 파일의 한컴 정상 판정과 렌더링의 잔여 차이를 분리한다.

## 6. 제출·병합 전 게이트와 merge comment 계획

검토 시작 시 base/EA HEAD의 merge-tree는 exit 0, tree
`d2bb42282331bf6072b7d52ddbff9da805a301d8`이었다. 최신 오늘할일 기존 네 항목은 양쪽에서
동일함을 확인했고 이번 항목만 추가한다. review·오늘할일·보고서·대표 PNG만 single-parent
trailing commit으로 잇는다. 제품 코드를 재병합하거나 기준값을 다시 변경하지 않는다.
push 전 최종 base/head merge-tree·공백·문서 링크·기존 오늘할일 보존을 검사한다.
push 후 최신 head의 실제 fast-pass·required aggregate와 mergeability를 다시 확인한다.

병합 후 별도 승인된 comment는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결하고,
위 페이지·수치·환경·잔여 차이를 요약한다. 이미지는 실제 merge SHA를 조회한 뒤
`https://raw.githubusercontent.com/edwardkim/rhwp/<MERGE_SHA>/mydocs/pr/assets/pr_7101_labnote_p002_review.png`
및 textbox 대응 경로로 고정한다. 실제 파일과 comment 내용을 API로 재조회한다.
이 계획은 comment 게시나 병합 승인을 대신하지 않는다. #3587 종료는 원 제안 네 API 전체가
아니라 승인된 수정 범위와 최종 병합 증거를 기준으로 판단한다.

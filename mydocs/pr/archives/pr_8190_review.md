# PR #8190 셀프 리뷰 — PDF 이미지 코덱 개별 opt-in

## 최종 판정

**승인.** 승인된 #8183 범위에서 머지를 막을 코드 결함을 발견하지 않았다. 개별 선택, 실제
decoder의 요청 예산, 지원 정보, 실패 후 복구와 저장 후 이미지 보존을 구현·실행 증거와 대조했다.
두 반투명 사례는 [메인테이너의 명시적 결정](../../working/task_m100_8183_alpha_visual_review.md#메인테이너-결정--2026-10-24)에
따라 RHWP 출력을 수용한다. 한컴 Print의 미달 수치를 통과로 바꾸지 않는다.

사용자가 2026-10-24 셀프 리뷰와 머지를 승인했다. 이 문서 작성 시점의 code head CI는 모두
완료됐으며, 남은 실행 조건은 문서-only trailing head의 실제 fast-pass·필수 검사 성공,
최신 base와의 병합·링크 검사 및 exact-head 머지다. GitHub의 본인 PR approve review는 게시하지 않는다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| PR·작성자·대상 | [#8190](https://github.com/edwardkim/rhwp/pull/8190) / `edwardkim` / `devel` |
| 역할·절차 | 메인테이너 본인 PR, `collaborator_self_merge` + intake/local validation/visual/fast-pass/post-merge |
| 관련 이슈 | [#8183](https://github.com/edwardkim/rhwp/issues/8183), `Closes #8183` |
| 검토 code head | `9952e55fbe9e9f220ff4adf0c3c19a800d1faa17` |
| fetch 후 base | `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867` |
| 실행 source | 제품 `cfc5be8fe` / 최종 Rust 검사 `2a58c5f3741334a182c466d3eaa1586e93697ff3` |
| 규모 | code head 기준 351 files, +27,548 / -286. 대부분 vendor 원본·라이선스·검증 자료다. |
| 작성 시점 참고값 | Open, Draft 아님, reviewer 없음, `MERGEABLE` / `CLEAN`. 머지 직전 재조회한다. |

## 코드와 증거의 대조

- `Cargo.toml`의 `pdf-jbig2`·`pdf-jpeg2000`은 각각 `pdf-import`와 backend의 해당 feature를 켠다.
  default 목록은 그대로이며 backend는 codec의 기본 image integration·SIMD를 켜지 않는다.
- `vendor/rhwp-pdf-import-backend/src/codec_resource.rs::decode`가 같은 `ImportBudget`을 동기
  thread-local scope에 연결한다. `rhwp-codec-resource`의 checked product·fallible Vec 예약·work와
  실제 hayro fork의 호출을 대조했다. scope는 중첩 복원되고 Send/Sync가 아니며 sticky 오류가
  EOF/parse 오류로 사라지지 않는다. 전체 parser·프로세스 메모리의 상한 증명으로 확대하지 않는다.
- `extractors/images.rs`의 filter resolution → JBIG2 globals·차원 검증·고정 크기 row collector,
  JPX decoder → palette/PDF Indexed·8-bit sample·Gray/RGB alpha·mask 소비를 확인했다.
  미선택·미지원 입력은 명시적으로 거부하며 alpha=2는 premultiplication을 복원한다.
- `pdf_support::pdf_import_support()`는 root alias만 보지 않고 backend의 실제 합산 feature를 조회한다.
  CLI·metadata·WASM 사전 조회가 이를 공유한다. 별도 Rust 앱의 feature union은 이전 source에서
  `accepted=true, declared=false`, 보정 source에서 `true/true`였다. [실행 기록](../../working/task_m100_8183_stage3.md#실제-소비와-공개-지원-정보)을 대조했다.
- 정식 회귀는 source sample·전체 독립 bitmap·저장 후 리소스/쪽 소유·작업 한도 직전/정확/직후·다음 요청
  복구를 검사한다. decoder의 계산값을 그대로 정답으로 재인용하는 배치 px pin·전체 SVG hash는 추가하지 않았다.
  실제 Worker 진입·취소·deadline·복구와 동기 Studio 열기를 구분했다.

source·test·manifest·JS 입력은 `2a58c5f37..9952e55fb`에서 동일하다. 이후에는 문서·시각 비교판·결정과
PDF 원본 바이트용 `.gitattributes`만 추가했다. 외부 contributor 보정·체리픽 통합은 없으므로 별도
review implementation 계획서는 만들지 않는다.

## 완료된 검증과 CI

정확한 code head의 아래 GitHub Actions는 같은 PR·repository·branch·SHA에서 모두 완료되어 성공했다.

| 검사 | 실행 |
| --- | --- |
| Full CI: lint, Archive A–D 및 각 shard, Native Skia, frontend package, Build & Test | [40047278135](https://github.com/edwardkim/rhwp/actions/runs/40047278135) |
| CodeQL: 4개 언어 Analyze 및 GHAS check | [40047278067](https://github.com/edwardkim/rhwp/actions/runs/40047278067) |
| Canvas Render Diff | [40047278187](https://github.com/edwardkim/rhwp/actions/runs/40047278187) |
| Adapter inter-diff | [40047278040](https://github.com/edwardkim/rhwp/actions/runs/40047278040) |
| Proptest roundtrip | [40047278045](https://github.com/edwardkim/rhwp/actions/runs/40047278045) |
| CI Impact Policy Controller | [40047278019](https://github.com/edwardkim/rhwp/actions/runs/40047278019) |

WASM Build·frontend unit 등 정책상 skipped인 job을 실행 성공으로 세지 않았다. required check는
`Build & Test`이며 성공했다. 별도 선택 코덱의 실행 증거는 로컬 네 구성 검사다. default CI가 두
opt-in을 켜서 실행했다고 주장하지 않는다.

제출 전에 [3단계 기록의 명령·결과](../../working/task_m100_8183_stage3.md#회귀와-검증-명령)로
필수 lint·workspace build, 전체 nextest **11,366 PASS / 0 FAIL / 45 skip**, Native Skia·doc tests,
양쪽 opt-in 정식 회귀 **7 PASS**, scoped resource **4 PASS**, CLI·fresh WASM 네 구성 각각
19건, 실제 Studio·Worker, TypeScript와 npm **2361 PASS / 0 FAIL / 기존 2 skip**, 설치 후 소비를
완료했다. 이번 셀프 리뷰에서는 이 실행 자료와 현재 code diff를 대조했고, 녹색 Full CI 뒤에 같은
전체 로컬 회귀를 반복하지 않았다. 제출 전 로컬 검증을 생략한 것이 아니다.

## 시각 증거와 입력 보존

[원본 PDF·생성 출처·해시](../../../pdf/issue8183/provenance.json),
[한컴 Print·재저장 대조군](../../../pdf/issue8183/hancom-print/provenance.json),
[전체 TSV·대표 PNG](../assets/issue_8183/README.md),
[TSV 해시](../assets/issue_8183/provenance.json)를 연결한다.
검토 commit `9952e55fb`의 원본 PDF 20개와 기본 TSV 34개 등 manifest의 57개 항목을 실제 파일의
SHA-256 및 `git show <head>:<path>` 바이트와 다시 대조했다. 대용량 PDF 예외는 없다.
HWP/HWPX 저장 후 재열기는 제품이 생성한 출력의 보존 검사이며, 외부 원본을 숨긴 자료가 아니다.

- 원본 PDF 기준 17쪽의 Native/fresh WASM raw 최저 **각각 100%**, 누락·쪽수 차이 없음.
  같은 저장 HWPX에 대한 base/후보 34행도 전쪽 100%, 하락 0pp다.
  base가 선택 코덱 원본을 직접 가져왔다고 해석하지 않는다.
- 독립 기준은 원본 PDF이며 JPX alpha·Indexed 세 사례만 원본을 Ghostscript로 독립 정규화했다.
  PDFium 자체의 JPX 해석 차이·초기 잘못된 스캔 strip은 [원인과 재실행](../../working/task_m100_8183_stage3.md#독립-자료와-원본-pdf-대조)을 확인했다.
- 한컴 2024 13.0.0.3901 Print method 0의 9쪽 대조는 7쪽이 양쪽 99% 이상이다.
  `jpx-alpha`·`jpx-premult` p1은 각각 **27.39104%·52.99065%**이며 원래
  `re_review_required`를 그대로 보존한다. 한컴 재저장 HWP에도 RGBA가 남지만 Print가 1-bit mask를
  만드는 두 4×2 사례다. 메인테이너가 RHWP를 정답으로 결정해 그 Print를 alpha oracle로 쓰지 않는다.
  일반 시각 gate나 다른 입력·쪽의 예외가 아니다.
- 이번 검토에서도 반투명 비교판과 WASM 200 DPI 스캔 review를 실제로 열었다. 연속 alpha·preblend,
  표의 외곽·본문·마지막 내용 보존을 확인했다. 기존 9종 확대 판독도 제품 source가 동일해 재사용한다.
  2px 실루엣 100%와 픽셀 완전 일치율을 구분한다. 수식 조판 수정이나 OCR 개선을 주장하지 않는다.

## 공통 원칙 준수 검토

| 항목 | 판정·근거 |
| --- | --- |
| 제출 전 검증·Open PR | 충족 — 위 로컬·CI·시각 실행 근거 |
| 기존 해결 이력·독립 근거 | 충족 — #8183 계획·2/3단계, 기존 backend의 미선택 거부 계약, PDF 1.7 Table 89·원본 sample과 독립 PDF 대조 |
| 구현 근거·일반성 | 충족 — codec/filter/색상·alpha 계약별 분기, 문서 ID·우연한 좌표 조건 없음 |
| 측정·배치·줄 높이·LineSeg | 비해당 — 줄 구성·페이지 조판·좌표 보정 변경 없음 |
| 리베이스 반영 기능 보존 | 비해당 — base가 전진하지 않았고 rebase/merge를 하지 않음. 같은 HWPX의 base 표시 대조는 완료 |
| 최종 출력 소비 | 충족 — Native Skia·fresh WASM Canvas·저장/재열기·Studio 실제 소비를 각각 확인 |
| 분할·이어받기 | 비해당 — pagination·rowspan·continuation 변경 없음 |
| 독립성·회귀 선행 조건 | 충족 — source sample/독립 bitmap·PDF 기준, 전쪽 양쪽 raw 100%. 두 Print는 사용자 결정으로 잘못된 alpha oracle임을 명시 |
| 기준값 변경 | 비해당 — 기존 baseline·golden·허용치 완화 없음 |
| 확대 판독·주장 범위 | 충족 — 본문·경계·mask·색상·alpha 직접 판독. 스캔 원본의 글자를 새로운 텍스트 조판으로 해석하지 않음 |
| 실제 호출·경계·복구 | 충족 — 네 구성의 CLI/WASM 실제 진입과 work 한도·손상·정상 후속 요청, feature-union 수정 전 실패/수정 후 성공 |
| 검증 입력 commit 포함 | 충족 — formal fixture/provenance, TSV 해시와 실제 commit 바이트 대조 |

## 한계와 이후 범위

관측한 JPEG 성능에는 일관된 큰 후퇴가 없지만 모든 입력의 속도를 보장하지 않는다. 양쪽 코덱 WASM은
default 대비 +553,400 bytes(+2.544%)다. [154+154회 성능 원문](../assets/issue_8183/performance/README.md)의
프로필·중앙값·min–max·RSS/선형 heap을 구분했다. 고해상도·복잡한 입력은 기존 예산으로 거부될 수 있다.
`parserAllocationBoundsProven=false`, 전체 allocator/CPU 상한 미증명, 동기 Studio의 중단 한계는 유지한다.
#8174의 MSRV·registry 패키징 결정과 0.9.0 배포는 별도다. 기존 lockfile yank 경고를 이번 코덱의 신규
취약점이나 배포 안전 판정으로 바꾸지 않는다. source preflight는 통과했으나 `publish_ready=false`다.

## Merge 후 contributor PR comment 계획

정확한 trailing head CI와 merge tree를 확인해 squash merge하고, 실제 merge SHA·devel 포함 여부·
issue 종료 상태를 확인한다. review·오늘할일은 이번 PR에 포함하므로 후속 문서 PR은 만들지 않는다.
PR 및 issue에 한국어 존댓말로 실제 merge SHA와 code/trailing CI, 위 검증 범위·alpha 결정·남은 한계를
`--body-file`로 게시하고 API 본문을 재조회한다. 시각 비교 정본은
[Visual Sweep merge comment](../../manual/verification/visual_sweep_guide.md#github-merge-comment)다.

대표 이미지는 `mydocs/pr/assets/issue_8183/native/scan-jbig2-200_review.png`,
`wasm/scan-jpx-200_review.png`, `alpha-review/overview.png`를 사용한다.
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_8183/...`
형태의 실제 merge SHA URL로 표시한다. 원본 기준 17쪽×2 raw 최저 100%와 Print 두 미달을 모두
기록하고 점수를 사람의 판정 정확도로 표현하지 않는다. 추가 CI를 병합 뒤 시작하지 않는다.
duration 갱신 상태·devel 동기화를 확인한 뒤 이번 전용 worktree·branch와 임시 산출물을 정리한다.
필수 원본·최종 증적은 저장소에 보존하고 실행 원문은 로컬 영구 기록에 보존한다. 다른 활성 작업은 삭제하지 않는다.

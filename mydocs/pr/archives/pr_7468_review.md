---
kind: snapshot
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-08
---

# PR #7468 검토 — 빈 문단 뒤 서식 변경과 저장 쪽 경계

## 최종 판정

**메인테이너 보정 후 수용 가능.** 원 서식 API 수정에 최종 C/D 배치, 빈 문단 삭제·재열기 및 실제 저장 경계 편집 검사를 보강했습니다. GitHub CI와 최종 merge 조건은 제출 후 별도로 확인합니다.

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#7468](https://github.com/edwardkim/rhwp/pull/7468) / lidge-jun (jun) / devel |
| 원 head | `20b388d9c14f8342986fd4277817eb4e843a7822` |
| 관련 이슈 | [#7467](https://github.com/edwardkim/rhwp/issues/7467), 통합 PR 반영 후 종료 |
| 기존 review | [최종 배치 보강 요청](https://github.com/edwardkim/rhwp/pull/7468#pullrequestreview-5339590395), [저장 경계 편집 요청](https://github.com/edwardkim/rhwp/pull/7468#pullrequestreview-5339982282) |
| 작성 시 원 PR 상태 | OPEN / head 변동 없음 / 조직 fork source 보존 |

## 생산 값과 최종 소비 경로

두 API는 변경 전 `paragraph_flow_end`를 보존합니다. `apply_char_format_native`가 기존 line-seg를 먼저 비워 원본 시작점 정보를 잃던 동작을 없애고, 두 API 모두 기존 `reflow_line_segs`가 저장 시작점을 상속한 뒤 `recalculate_section_vpos`로 뒤 문단의 진행을 복구합니다. 실제 저장 0 경계는 기존 composer guard가 보존합니다. rebuild → compose/typeset → RenderTree의 TextLine 원점·쪽 소속에서 결과를 검증했습니다. 임의 위치 clamp, 문서별 조건 및 출력 은폐는 추가하지 않았습니다.

새 검사는 page count만 확인하지 않습니다. 입력의 13pt×160% 진행으로 C/D 간격과 B/C의 빈 20pt 줄 점유를 확인하고, 빈 줄 삭제 시 C와 D가 같은 줄 전진만큼 이동하는지, 모든 텍스트 순서·문단/쪽 소속·누락/중복·겹침 및 저장·재열기를 함께 검사합니다. 허용 오차 0.05px는 부동소수 연산 정밀도이며 실제 출력의 절대 좌표 golden이 아닙니다.

실제 한컴 저장 70줄/2쪽 대조군에서는 원본 tag의 양수 vpos 뒤 0 경계를 찾아 두 인접 문단을 두 API로 편집합니다. 모든 70줄의 소속 쪽/문단과 편집 전 상대 위치, 전체 2쪽 및 재열기를 확인합니다. 수동 tag/vpos 조작은 없습니다.

## 주장과 검사 대응

| 주장 | 실제 적용·소비 경로 | 독립 기대값·검사 | 결과·제한 |
| --- | --- | --- | --- |
| 빈 줄 뒤 서식 변경의 가짜 쪽 나눔 방지 | 두 서식 API의 저장 시작점과 후속 vpos → compose/typeset → TextLine | 한컴 Print 1쪽, 텍스트 순서·문단/쪽 소속·줄 겹침 | 두 API의 live/재열기 1쪽, Native/WASM 전쪽 100% |
| C/D 및 빈 줄 점유 복구 | reflow/recalculate 뒤 최종 RenderTree | 입력의 13pt×160% C/D 진행, B/C의 20pt 빈 줄 점유 | 수정 전 ID 경로 간격 FAIL, 수정 후 정식 PASS |
| 빈 문단 삭제·저장 왕복 | 기존 delete API → compose/export/reopen | C와 D가 같은 20pt×160%만큼 이동, 삭제 후 13pt 진행 | 두 API 및 재열기 PASS, 독립 삭제 Print 100% |
| 실제 저장 0 쪽 경계 유지 | 원본 저장 양수→0 경계 양쪽 API edit → compose/export/reopen | 한컴 재저장 70줄/2쪽, 모든 줄의 소속·순서·상대 원점 | 편집 네 경로·재열기 PASS, 함초롬바탕 최종 Native/WASM 전 2쪽 100.00000% |
| 측정/배치·분할/이어받기 | 새로운 fit/cut/fragment 경로 없음; 기존 composer의 저장 reset 경계를 재사용 | 최종 TextLine·2쪽 경계 및 기존 전체 회귀 | 새로운 임의 컷·배치 보정 없음. sample16 전쪽 미달은 #7445에 유지 |

## 검증 입력과 초기 시각 판독

HWP 3개와 독립 한컴 Print PDF 3개를 같은 PR에 포함했습니다. [fixture 설명·SHA-256·Print job](../../../tests/fixtures/pr7468/README.md)에 생성 절차와 각 입력/기준의 해시가 있습니다. 실제 한컴 `11.0.0.9136`, Print method 0, `hancom2020_pdf_driver_one_up`을 확인했습니다. 생성 및 재저장 입력은 공개 영문 합성 내용입니다.

| 입력·전체 쪽 | Native 최저 2px 관용 내용 실루엣 | 새 WASM 최저 | 직접 판독 |
| --- | --- | --- | --- |
| edited / 1 | 100.00000% | 100.00000% | 빈 줄 점유 및 C/D 순서·배치 대응 |
| deleted / 1 | 100.00000% | 100.00000% | 빈 줄 삭제 뒤 C/D 간격 및 이동 대응 |
| stored-control / 2 | 99.95857% | 99.95857% | Line 041은 1쪽, Line 042는 2쪽, 끝 Line 070까지 누락·중복 없음 |

초기 전체 4쪽을 두 backend에서 비교하고 review/overlay 16개를 직접 열어 판독했습니다. 현재 [영구 PNG와 설명](../assets/pr_7468/README.md)은 아래 사용자 글꼴을 공급한 최종 재실행 결과로 대체했습니다. 내용 줄 위치와 쪽 경계는 대응하지만 Linux 글꼴/획 raster 차이가 남습니다. 엄격한 내용 픽셀 일치는 16.58933–27.66885%이며 관용 실루엣 결과를 완전한 글꼴 일치로 해석하지 않습니다. 90% 미달이나 폰트 예외를 사용하지 않았습니다.

새 renderer 검사·fixture를 저장소에 추가하기 **전에** 최종 생산 코드 Native와 fresh 최적화 Docker WASM의 위 전쪽 90% gate를 완료했습니다. 이후 안정 경로의 입력으로 full sweep를 다시 만들었습니다. 수정된 두 API의 live/reopened WASM 페이지·좌표는 Native와 같고, 실제 저장 경계의 5상태 HWP export 바이트도 Native와 동일했습니다.

## 수정 전 음성 대조

최신 devel `f0e7228f6…`의 `formatting.rs`만 격리 복사본에 복원하여 같은 공용 target의 dev profile로 빌드했습니다. 제품 작업공간의 source는 바꾸지 않았습니다. 최종 세 검사를 같은 입력으로 실행한 결과 1 PASS / 2 FAIL였습니다.

- apply-char-format: 빈 문단 뒤 저장 원점이 빈 문단 끝보다 앞서므로 FAIL.
- set-char-shape-id: 실제 최종 C/D 간격 42.6667px, 입력의 13pt×160% 관계 27.7333px에 불일치하여 FAIL.
- 실제 저장 2쪽 경계 대조군: PASS. 정상 쪽 나눔을 없애야만 새 검사가 통과하는 구조가 아닙니다.

현재 생산 코드의 같은 검사 세 개는 Native 진단에서 모두 PASS였으며 아래 정식 runner로도 확인합니다. 음성 대조의 검사 본문은 최종 case와 같고 include 경로만 격리 실행용 절대 경로입니다. 로그는 ignored output에 보존합니다.

## 기존 sample16 범위

원 PR의 sample16 로드 64쪽 단언은 실제 저장 경계 인접 편집을 검사하지 않았습니다. 현재 devel은 로드/편집 모두 65쪽이며 독립 기준은 64쪽입니다. 이 전쪽 차이와 p23/p24 시각 미달은 이미 upstream의 [#7445 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868738187)과 commit `f141f8fe7`에서 이관되어 있습니다. 전역 synthetic-tail 변경 제외 후에도 같아 이번 서식 API 변경의 새 로드 회귀가 아닙니다.

새 경계 검사는 전쪽 시각 gate를 충족하는 실제 한컴 재저장 2쪽 대조군으로 강화했습니다. 기존 sample16의 upstream 검사·기준은 유지하고 64→65 완화나 ignore를 추가하지 않았습니다. 원 sample16의 68/69 문단은 편집 뒤에도 물리 2/3쪽에 속했지만 전문서 피델리티를 통과했다고 주장하지 않습니다. #7445는 계속 OPEN입니다.

## 초기 빌드·실행 출처

- 초기 시각 빌드 이후 candidate는 검사·fixture 및 composer import 줄바꿈만 추가했습니다. 이후 아래 함초롬바탕 최종 비교에서는 Native candidate binary와 기존 fresh WASM을 실제 재실행했습니다.
- 초기 Native 생산 코드: `282d5d77f…`, release-test binary SHA-256 `c644304f1820d5fa65730424da53ecf59d44b639dfb37285dcca6892b55b18f4`.
- 최적화 WASM: Rust 1.93.1 Docker 이미지와 표준 `scripts/wasm-pack-locked.sh --target web`, wasm-pack 0.15.0 / bindgen 0.2.127 / Binaryen 117. 완료 31m44s. product Dockerfile/compose는 바꾸지 않았고 managed 환경의 CA/proxy·UID·동일 절대 target 경로를 제공하는 외부 compose를 사용했습니다.
- WASM SHA-256 `596c3edea6ba3f9af9e7a5fcf651d50b33d3f3ba02f0fd3a9352d92065d01a80`; JS `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`. pkg/Studio public 각각 동일합니다.
- 인쇄 profile·96dpi의 프로젝트 Sweep/exporter를 그대로 사용했습니다. 시스템 Chromium의 file URL 정책에 맞춘 외부 읽기 전용 localhost HTTP 전송 adapter로 동일 HTML/폰트 바이트를 제공했습니다. 폰트/레이아웃/비교 알고리즘을 바꾸거나 browser policy를 변경하지 않았습니다.
- 관련 진단 Rust 3 PASS, fresh WASM 9상태 PASS, Studio 검증은 [#7464 검토](pr_7464_review.md)에 있습니다. 정식 Rust focused/전체/Native Skia/lint 결과는 아래 최종 검사 표에 기록합니다.
- 로컬 로그·중간 JSON·TSV: `/workspace/rhwp/output/pr-review/pr7464-pr7468/`; permanent summary·PNG와 실제 입력/PDF만 PR에 포함합니다.

## 통합 경로와 검증 대상

- 경로: `collaborator_external_pr` 9.1.1의 최신 devel 기반 cherry-pick 통합. 사용자께서 두 PR의 보정·통합 PR 생성·병합을 요청하셨습니다.
- 기준 devel: `f0e7228f6dd2ea1437724e53ad640c40c56d204b`; branch `review/lidge-pr7464-pr7468-20261008`, 기본 작업공간 `/workspace/rhwp`.
- 실행 계정 `postmelee`는 push 권한이 있으나 admin/maintain 권한은 없습니다. 원 `lidge-ai/rhwp` 조직 fork에는 push하지 않습니다. 원 commit의 author와 `cherry picked from` 계보를 보존하고 merge commit은 제외했습니다.
- 메인테이너 코드 보정 `282d5d77f6f7167a70047170dbad1a10bcce6ba5`, 회귀·독립 Print 입력 보강 `fdea8a4cbe203f6ec93bbbaf47382ebddacad6c1`. 최종 문서 commit은 생산 코드·검사·입력을 바꾸지 않습니다.
- `formatting.rs` import 충돌은 최신 `restamp_indentation`과 새 flow helper를 함께 보존했습니다. 최신 batch deferred rebuild 및 stored-reset guard를 유지했습니다. 원 PR의 전역 synthetic-tail 제외 변경은 최신 upstream에서 불필요하여 통합 결과에서 제외했습니다. 최종 Rust 생산 변경은 두 서식 API에 한정됩니다.
- 공용 Cargo target은 `/workspace/rhwp/target/pr-review` 하나입니다. Cargo 실행은 순차로 진행하고 파생 suite/manifest, 로그·중간 JSON·TSV는 커밋하지 않습니다.

## 원 기여 계보

| 통합 commit | 원 commit | 내용 |
| --- | --- | --- |
| `41c87755f579a1173b0f85c46a6263394380f0d4` | `d334a1f8a7404ecc569beb23625a4a84fa614182` | 수정: 셀 블록 지우기 키와 삭제 확인 동작 정합 |
| `c6be116b83805610b044cf668e27c3a95d567c35` | `8474698c1d89f8eec3781df882b968d197d7e5ed` | 수정: 셀 블록 지우기가 표 명령 모듈을 불러오지 않게 커서 보정 헬퍼 분리 |
| `c1261e6a959c64dad2f36e67c54ec3a65a1074d0` | `3f1dbff08b9f4cfc2407fb903253594db9d449f6` | 테스트: 빈 문단 뒤 글자 크기 변경 쪽나눔 회귀 재현 |
| `606fc5f4ccd158b9483f0f7357f2f94282e91b97` | `72a47a1c864c72c47e5430f9834853b29d9447bb` | 수정: 글자 크기 변경 뒤 합성 원점의 가짜 쪽나눔 방지 |
| `8cee9363a215587723067f6353e6e495dfe40017` | `1875b509167664a54f6d02e6ea62df827cab38bf` | 정리: 글자 모양 회귀 테스트 서식 맞춤 |
| `8f318e4a45d1b22998e42d5f134dacdc98b285c8` | `68c1e7f367464f25ba123eb345d9e4fea36fdcc5` | 테스트: 빈 문단 삭제 후 저장 재열기까지 확인 |
| `91fd3c29d8265beef6a19ff2e90f81d5b49ed540` | `20b388d9c14f8342986fd4277817eb4e843a7822` | 정리: 삭제 후 재열기 회귀 검사 서식 맞춤 |

위 기능 commit의 저자는 모두 jun입니다. 원 PR 작성자는 lidge-jun이며 기능 기여를 메인테이너 보정과 구분하여 보존합니다.

## Merge 전 조건과 후속 처리

최종 head GitHub CI와 required checks, 최신 devel의 merge-tree 및 head SHA를 확인한 후 일반 merge를 진행합니다. branch protection을 우회하지 않습니다. 통합 PR이 실제 devel에 반영된 뒤에만 원 #7464·#7468을 대체 안내와 함께 close(merged=false)하고 관련 #7462·#7467을 확인·종료합니다. #7445는 OPEN으로 유지합니다.

merge 후 comment는 한국어 존댓말의 `--body-file`로 원 기여, 보정 이유, 실제 로컬/CI 결과, 통합 PR·merge SHA와 SHA 고정 시각 asset을 안내하고 API로 게시 내용을 재확인합니다. 이 archive와 오늘할일은 같은 통합 PR에 포함하며 별도 기록 PR을 만들지 않습니다. 최종 devel sync, 이 작업에서만 만든 clean 임시 branch와 실행 서버 정리, duration refresh 확인까지 수행합니다. contributor fork branch와 공유 target은 보존합니다.

## 초기 Native TSV 원본 다운로드

[Native 전 4쪽 TSV ZIP](https://drive.google.com/file/d/1cLI-GC8HA7lu97GfQV5eLaB2JRlfCtsd/view?usp=drivesdk) — 1,592 bytes, SHA-256 `b60c678372c879ec293f81eda00132bc03408bc136f587e7683c29a4238a3177`. GitHub App의 Gist 쓰기가 403으로 거절되어 Drive에 원본 ZIP을 첨부하고 업로드 성공 및 metadata 크기를 확인했습니다. 연결된 Drive 소유자 접근이며 공유 권한을 확대하지 않았습니다. PR 본문에도 이 접근 범위를 명시합니다. TSV는 Git commit에 포함하지 않습니다.

## 정식 focused 결과

`node scripts/run-rust-test.mjs char_shape_after_empty_keeps_page -- --cargo-profile release-test --target-dir /workspace/rhwp/target/pr-review`: **3 PASS / 0 FAIL**, 225건은 필터 밖 skipped입니다. 정식 suite_025 runner가 두 API와 실제 저장 경계의 검사 세 개를 실행했습니다. production/test candidate `82aeff6091b6827b4f913d4483ba0a354f06f5ca`, 공용 target·locked 실행이며 로그는 `logs/rust-focused-final.log`에 있습니다.


## Merge 후 contributor PR comment 계획

실제 통합 PR의 일반 merge와 devel 반영을 확인한 뒤 아래 내용으로 한국어 존댓말 comment를 게시합니다. 통합 PR 번호·최종 CI run·merge SHA는 실제 완료값으로만 채웁니다. 게시 전 이 기록과 devel의 asset을 대조하고 게시 후 API에서 UTF-8 본문을 다시 확인합니다.

- jun/lidge-jun님의 빈 문단 뒤 두 서식 API의 저장 원점·후속 흐름 복구 기여에 감사하고 author/원 SHA를 보존한 통합 PR을 안내합니다.
- 기존 리뷰 요청대로 최종 C/D 줄 배치, 빈 줄 삭제·재열기, 실제 저장 0 경계 양쪽의 API 편집과 전체 70줄 소속 검사를 강화했음을 설명합니다. 최신 devel guard를 유지하고 전역 typeset 변경은 통합에서 제외했습니다.
- 정식 focused 3 PASS, 수정 전 두 API FAIL / 정상 저장 경계 PASS, Native/fresh WASM 전 4쪽 비교와 최종 Rust/Native Skia/lint·GitHub CI의 실제 결과를 요약합니다.
- deleted p1 및 stored p2의 Native/fresh WASM review와 standalone overlay를 같은 입력·쪽별로 표시합니다. 함초롬바탕 regular/bold 실제 공급 후 전 4쪽 100.00000%, 누락/중복 없음과 직접 판독을 설명합니다. 모든 이미지 URL은 실제 merge SHA에 고정합니다.
- [Visual Sweep 비교 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 링크하고 엄격한 내용 픽셀 일치 18.45444–38.23178%의 glyph/기준선 raster 잔차 및 sample16의 #7445 OPEN을 구분합니다. 관용 수치를 완전한 글꼴 일치로 해석하지 않습니다.
- 함초롬바탕 적용 최종 Native TSV ZIP 다운로드와 연결된 Drive 소유자 접근 범위를 안내합니다. 원 PR 대체 close(merged=false)와 관련 #7467 종료는 실제 확인한 상태만 적습니다.

## 사용자 제공 함초롬바탕 적용 최종 비교

사용자가 첨부한 HCRBatang.ttf(28,412,448 bytes, weight 400)와 HCRBatang-Bold.ttf(30,691,188 bytes, weight 700)를 사용했습니다. FontTools 4.66.1로 각각 Unicode cmap 59,330개와 이름/weight를 확인했습니다. regular SHA-256은 `e75fc81fb6def106ede5917620ba7321eeb59690f52c87ec8c53796058928f15`, bold는 `a909967a50b2b15887ae21cfcc14d26301dad65219b5d6c18b3b02128151d784`입니다. 글꼴 원본은 Git에 포함하지 않았습니다.

최종 PNG의 Native source는 `82aeff6091b6827b4f913d4483ba0a354f06f5ca`, release-test binary SHA-256 `ab0d441cff85b3ecbfc02142700e4eb45b440d84d89d23630ef43b53fd7c8f0d`입니다. fresh 최적화 WASM은 production `282d5d77f6f7167a70047170dbad1a10bcce6ba5`의 기존 빌드 SHA-256 `596c3edea6ba3f9af9e7a5fcf651d50b33d3f3ba02f0fd3a9352d92065d01a80`을 실제 재실행했습니다. 두 source 사이의 추가 변경은 test/fixture와 import 줄바꿈이며 production 동작은 같습니다. 새 PNG가 초기 글꼴 미공급 PNG를 대체합니다.

동일한 canonical HWP/독립 Print PDF에 `--embed-fonts=full --font-path /workspace/rhwp-tools/hamchorom-pr7468`을 적용하여 프로젝트 visual_sweep.py full sweep를 Native/fresh WASM에서 다시 실행했습니다. Native 366.96s, WASM 441.17s. regular는 SVG에 full embed되고 bold는 private FONTCONFIG_FILE로 실제 첨부 font를 공급했습니다. 읽기 전용 Chromium CSS font trace와 후속 assertion으로 Native 4쪽/WASM 4쪽 모두 regular `HCRBatang`, bold `HCRBatang-Bold`의 실제 선택을 확인했습니다. synthetic bold나 다른 face로 대체된 결과가 아닙니다.

full-font SVG raster의 최초 30초 timeout은 완료 측정이 아닙니다. 기존 `RHWP_VISUAL_RASTER_TIMEOUT_MS=120000` 환경 설정을 사용해 완료했으며 source·비교 알고리즘·문서 좌표·SVG/CSS를 수정하지 않았습니다. browser file-URL 정책의 localhost 전송 adapter는 바이트를 그대로 제공하고 font trace는 관측만 합니다.

| 입력·쪽 | Native / fresh WASM 관용 실루엣 | 엄격한 내용 픽셀 일치 | 직접 판독 |
| --- | --- | --- | --- |
| edited p1 | 100.00000% / 100.00000% | 38.23178% | Title/A/B/빈 줄/C/D 순서와 줄 배치 |
| deleted p1 | 100.00000% / 100.00000% | 37.26415% | 빈 줄 삭제 뒤 C/D가 동일하게 올라감 |
| stored p1 | 100.00000% / 100.00000% | 18.45444% | Line 001–041의 소속·순서 |
| stored p2 | 100.00000% / 100.00000% | 20.11124% | Line 042–070의 소속·끝 줄 |

전 4쪽/두 backend의 미달·누락은 없고 면제 없이 90% gate를 통과했습니다. review/standalone overlay 16개 모두 직접 판독했습니다. 실제 font를 공급해도 약 1px의 획/기준선 raster 잔차가 남으며 관용 실루엣 100%를 픽셀 완전 일치로 해석하지 않습니다.

재현: fixture README의 세 입력/PDF에 `venv/bin/python scripts/visual_sweep.py --file-target edited tests/fixtures/pr7468/edited.hwp pdf/pr7468-edited-print-2020.pdf --file-target deleted tests/fixtures/pr7468/deleted.hwp pdf/pr7468-deleted-print-2020.pdf --file-target stored tests/fixtures/pr7468/stored-control.hwp pdf/pr7468-stored-control-print-2020.pdf --rhwp-bin /workspace/rhwp-tools/rhwp-visual-pr7468-82aeff609 --embed-fonts=full --font-path /workspace/rhwp-tools/hamchorom-pr7468 --out output/pr-review/pr7464-pr7468/hamchorom-native`를 사용했습니다. WASM은 같은 명령에 `--wasm-pkg pkg`를 추가하고 출력 경로를 hamchorom-wasm으로 바꿉니다. 각 key의 최종 PNG에 `--silhouette-only --png-pair …/rhwp_png …/pdf_png --key <key>`로 정식 TSV를 산출했습니다. ZIP README에 정확한 명령과 PNG/입력/PDF 해시가 있습니다. 로그·font trace·중간 JSON·TSV는 ignored output에만 둡니다.

[함초롬바탕 적용 최종 Native 전 4쪽 TSV ZIP](https://drive.google.com/file/d/10frfxke2w-TAmQGGJjfxEOYpjpb4RxJ9/view?usp=drivesdk) — 2,843 bytes, SHA-256 `f67237b9df86ebde7a46f555d8d5bec3ea73abda67fd15e8571bb466d91ba430`. 정식 PNG-pair 재사용 명령으로 최종 full sweep PNG에서 산출했으며 새 렌더링으로 표시하지 않습니다. ZIP에는 README와 입력별 TSV 3개만 있고 글꼴 파일은 없습니다. 업로드 성공과 metadata 크기 및 owner-only 권한을 확인했으며 공유 범위를 확대하지 않았습니다.

## 최종 로컬 검사 결과

| 검사 | 결과 | 실측 근거 |
| --- | --- | --- |
| Rust suite manifest prepare | PASS | exit 0 |
| rustfmt all | PASS | exit 0 |
| rustfmt check | PASS | exit 0 |
| 빈 문단 뒤 서식 focused | PASS | Summary [   0.279s] 3 tests run: 3 passed, 225 skipped |
| release-test 전체 nextest | PASS | Summary [4717.635s] 10538 tests run: 10538 passed (43 slow), 50 skipped |
| Native Skia lib | PASS | test result: ok. 3927 passed; 0 failed; 13 ignored; 0 measured; 0 filtered out; finished in 275.12s; test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s; test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| Native Skia 그림 placeholder | PASS | Summary [   3.785s] 2 tests run: 2 passed, 227 skipped |
| Native Skia 직접 PDF | PASS | Summary [   0.319s] 4 tests run: 4 passed, 221 skipped |
| Native Clippy (-D warnings) | PASS | exit 0 |
| WASM lib Clippy (-D warnings) | PASS | exit 0 |
| workspace build | PASS | exit 0 |
| workspace all-targets Clippy (-D warnings) | PASS | exit 0 |
| Rust suite manifest fixed-base check | PASS | exit 0 |

생산 코드·검사 candidate는 `82aeff6091b6827b4f913d4483ba0a354f06f5ca`입니다. Rust 1.93.1, `--locked`, 단일 `/workspace/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=5`, `CARGO_PROFILE_DEV_DEBUG=0`을 사용했습니다. release-test 최적화·nextest test threads는 기존 설정을 유지했습니다. Cargo 명령은 순차로 실행했습니다. Native Skia의 lib 및 두 focused target을 한 Cargo no-run으로 준비하던 중 host의 FreeType/fontconfig 개발용 linker 이름이 없어 regular lib 링크가 실패했습니다. 설치된 실제 runtime DSOs(FreeType .so.6 / fontconfig .so.1)에 대한 두 unversioned alias를 기존 Skia linker 검색 경로 아래에만 제공하고 실제 link/runtime smoke를 통과했습니다. system package·source·RUSTFLAGS를 바꾸지 않았고 공유 target을 삭제하지 않았습니다. 환경 보완 뒤 세 정식 Native Skia 명령의 최종 결과를 위 표에 기록했습니다. 실패한 no-run을 PASS로 세지 않았습니다. 원본 runtime/alias 경로와 SHA-256은 ignored native-system-library-aliases.json에 보존합니다.

초기 jobs=3의 전체 회귀는 compile 단계에서 중단하고 같은 target의 캐시를 보존하여 jobs=5로 완료했습니다. 중단된 compile과 오래된 WASM 실행을 PASS로 세지 않았습니다. 전체 nextest의 실제 compile/test 시간은 그 로그의 Finished/Summary 값이며, coordinator 대기시간을 포함한 외부 wall time과 구분합니다.

최종 체크까지 local candidate의 source·test·fixture는 바뀌지 않았습니다. 초기 시각 빌드는 production `282d5d77f…`입니다. 사용자 제공 글꼴 최종 비교의 Native는 candidate `82aeff609…` binary, WASM은 production `282d5d77f…`의 fresh 최적화 빌드를 실제 재실행한 결과입니다. 사이에는 검사·입력 보강 및 import 줄바꿈만 추가되어 동작은 같으며 출처를 구분했습니다. 문서·이미지 추가 commit을 새 코드 실행으로 표시하지 않습니다.

검증 중 devel은 `74f9b71b15a2db18b17c37ceab6cf2b3c20c03c3`으로 전진했습니다. 새 upstream 표 조판 변경과 이번 생산 변경 파일은 겹치지 않고 코드 merge-tree는 clean입니다. 이번 branch의 code를 억지 merge/rebase하지 않고 최신 base와 최종 문서 head의 merge simulation 및 GitHub current-base merge CI를 독립적으로 확인합니다. 새 upstream 변경을 이번 PR 기능 diff로 집계하지 않습니다.

자체 작성 통합 PR의 author self-review입니다. 원 번호의 archive 네 개를 같은 PR에 보존하며 별도 integration 번호 archive나 기록 PR을 만들지 않습니다. 이 절은 로컬 검사 완료 시점의 기록입니다. 이후 완료한 GitHub CI는 다음 절에 기록하며 실제 merge는 반영 확인 뒤 후속 comment에서 확정합니다.

전체 nextest 안의 svg_snapshot 기준 출력 6개와 프로세스 내 결정성 검사 1개가 모두 PASS했습니다. golden 실패 또는 변경이 없으므로 별도 재생성은 수행하지 않았습니다. Native Skia lib의 workspace 합계는 4,109 PASS / 0 FAIL / 기존 13 ignored입니다.

## 통합 PR 및 GitHub Full CI

[통합 PR #7686](https://github.com/edwardkim/rhwp/pull/7686)은 원 번호별 검토 archive·오늘할일·시각 asset을 같은 branch에 포함합니다.

Full CI candidate `cdaa64680f6faa189aa08d9222216ae9a734d9d9`, 실행 base `74f9b71b15a2db18b17c37ceab6cf2b3c20c03c3`, GitHub 자동 merge ref `8047e559b540d50a863a470d324edac4c7fd7170`, tree `a23a61b252876f056fe31d5a26c9b83a607ae636`입니다. PR event의 실제 base/head와 GitHub merge ref의 부모/tree를 API·Git으로 확인했고 로컬 merge simulation의 tree와 같았습니다. 네 builder/worker의 기본 PR merge checkout 정의 및 성공 step도 확인했습니다.

| 검증 | 실제 run | 결과 |
| --- | --- | --- |
| CI | [37803919567](https://github.com/edwardkim/rhwp/actions/runs/37803919567) / attempt 1 | SUCCESS |
| CodeQL | [37803919737](https://github.com/edwardkim/rhwp/actions/runs/37803919737) / attempt 1 | SUCCESS |
| Render Diff | [37803918893](https://github.com/edwardkim/rhwp/actions/runs/37803918893) / attempt 1 | SUCCESS |
| Adapter inter-diff | [37803919449](https://github.com/edwardkim/rhwp/actions/runs/37803919449) / attempt 1 | SUCCESS |
| Proptest roundtrip | [37803919540](https://github.com/edwardkim/rhwp/actions/runs/37803919540) / attempt 1 | SUCCESS |

CI의 네 default-feature archive builder와 네 Run Archive A/B/C/D step이 실제 실행되어 모두 SUCCESS였습니다. 필수 `Build & Test`의 `Verify archive shard totals`와 worker 결과 집계도 SUCCESS입니다. lint·Native Skia 실제 test step·Frontend package gates를 확인했습니다. CodeQL JavaScript/TypeScript·Python·Rust·Actions의 Perform CodeQL Analysis step 네 개가 모두 SUCCESS이고, 별도 GitHub 보안 집계도 SUCCESS(변경 코드에서 새 경고 없음)였습니다.

WASM Build, release operations, Workflow promotion 및 Frontend unit gate는 정책상 SKIPPED입니다. 이를 새 실행으로 세지 않았습니다. fresh WASM 및 Studio 전체 unit은 위 로컬 실측 결과를 사용합니다. GitHub raw 로그 다운로드는 실행 환경의 log backend 접근에서 HTTP 403이 발생했습니다. 따라서 CI의 개별 테스트 숫자나 raw checkout log 줄을 읽었다고 주장하지 않습니다. 완료 Run/Job/Step API와 count 검증 step의 성공, 실제 source/base 및 merge-ref tree로 근거를 보존합니다. 로컬 10,538 PASS는 실제 읽은 nextest Summary입니다.

이 결과 뒤에는 원 번호의 검토 기록·오늘할일만 single-parent trailing commit으로 추가합니다. source/test/fixture/PNG는 변경하지 않습니다. 최종 trailing head의 preflight·required aggregate와 최신 devel의 clean merge-tree를 다시 확인한 뒤 사용자 요청 범위의 일반 merge를 진행합니다. 실제 merge SHA·issue/original PR 종료는 반영 확인 뒤 후속 comment에서 확정합니다. author self-review 예외를 적용하며 별도 integration 번호 archive나 기록 PR은 만들지 않습니다.

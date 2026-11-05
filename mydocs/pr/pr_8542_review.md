# PR #8542 리뷰 — RowBreak 저장 재시작과 이어받기 원점

## 최종 판정

**메인터너 보정 후 수용 가능 — #8539 해결 범위의 로컬 검증 완료.**

기여 기능은 독립 Print와 최신 devel 비교에서 수용할 근거를 확보했다. 검증한 code/data 후보는
`790ef21567d041fe272253673fa63fb1ce7809df`다. 전체 12,640 PASS·45 skipped와 Native 공식 3종
총 4,064 PASS·12 ignored, 필수 lint·383쪽 편람을 완료했다. 문서 후행은 code/data와 실행 source를 바꾸지 않는다.
원격 push·본문 수정·GitHub review·comment·merge는 수행하지 않았다. 최신 후보의 GitHub CI와 원격
처리 승인은 별도 조건이다. 최종 기록·제출 준비일은 2026-11-06 KST이며 측정과 필수 검사는 11월 5일에 완료했다.

## 접수 정보와 경로

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#8542](https://github.com/edwardkim/rhwp/pull/8542) / johndoekim, 기존 merged PR 21건 |
| 이슈 | [#8539](https://github.com/edwardkim/rhwp/issues/8539), #8198에서 분리한 쪽 경계 차이 |
| 원 head / 제품 수정 | `dc7553366f6c85035d510eaeb50eff2f25d83242` / `1543ae62fe07137da39bbaa88aa82884cc084bf2` |
| 원격 repository·branch | `johndoekim/rhwp` / `fix/issue-8539-rowbreak-stored-restart-cut` |
| 초기 PR base metadata | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`; live devel tip과 구분 |
| 고정한 live devel | `ece9c3383bb2c73e0f055459ede727a9d3101ac6` |
| 병합 실행 객체 | `5df48e77e7fc249feb6eacda74263ed96375cd36`, tree `1e239d728ed6774aabf00e8f3f15623fef335eae`, 충돌 없음 |
| 로컬 접수 기록 | `1f5b3cd68e` |
| 검증 보강 후보 | `790ef21567d041fe272253673fa63fb1ce7809df` |
| 실행한 제품 src tree | `ceaa7c40545ca8036c71c880b8d4a8c447435a4e`, 보강 후보도 동일 |
| 검토 branch | `review/pr-8542-rowbreak-stored-restart` |
| reviewer | `edwardkim`, 2026-11-05T21:35:47+09:00 요청 event와 REST users 재확인 |
| 상태 | Open, non-Draft, devel 대상. head·CI·mergeability는 원격 처리 직전 재조회 |

base route: `maintainer_general.md`.
modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
`rework_and_exceptions.md`(오래된 기준), `post_merge.md`.
loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 기본·보조 문서.
문서·Git·GitHub 운영·개발 환경과 Visual Sweep/출력 backend 정본을 함께 적용했다.
첫 기여자 경로는 비해당이다. 원 기여 커밋과 fork branch는 보존한다.

## 변경과 선행 해결 근거

모델의 0기반 row 9(「5. 손ㆍ외손」)와 row 16(「10. 형제ㆍ자매」)에서 한컴보다 한 줄 일찍
다음 쪽으로 넘어가는 문제다. 선언 행 상자가 현재 물리 공간을 소유할 때 저장 vpos 재시작만으로
자르지 않게 하고, 저장 프레임 중간에서 시작한 이어받기 조각에는 옛 프레임 끝을 강제하지 않는다.

- #8198의 최초 겹침·누락과 남은 쪽 경계 차이를 구분했다. 원본·독립 PDF와
  [메인테이너 의견](https://github.com/edwardkim/rhwp/issues/8198#issuecomment-6579028228)을 대조했다.
- #7870 원 기여 `717679359e6e128f26610727cf7c04d2f8ac6d5f` → 통합
  `bf4fc67e54b1cf8a370de1780cf34898faf19fee`는 중첩 표 atom의 이송 여백을 같은 용량 계산으로 맞췄다.
  [기존 독립 검증](../report/issue7870/71352-p9-review.png)과 해당 커밋을 확인했다.
- #7965 → 통합 #8148, merge `2e087a5b4bc1515ad877e8020b30114639a59392`의 저장 프레임 소유 계약은
  [기존 검토](archives/pr_7965_review.md)와 실제 `SourceFrameQuery`·row scan 경로를 대조했다.
- [HWP 5.0 사양 설명본](../tech/한글문서파일형식_5.0_revision1.3.md)의 HWPUNIT(1/7200인치),
  셀 높이·저장 줄 필드를 확인했다. 재시작의 실제 소유·끝 줄간격은 같은 공개 저장본과 독립 Hwp 2020
  출력 및 반례로 확인했다. 입력 메타데이터를 수정해 조건을 만들지 않았다.

메인테이너 보강은 신규 회귀의 측정 주석, 정답지 쪽수 원장의 신규 3/3 행, 검토 문서와 현재 backend의
대표 PNG다. 제품 로직과 기존 assertion·기대값·허용치를 수정하지 않았다. 독립 PDF가 `samples/`에 있어
자동 정답지 탐색에 의존하지 않고 쪽수 원장에 명시적으로 등록했다.

## 실제 생산·예약·배치 경로

| 경계 | 코드와 실제 소비 | 실행 관측 |
| --- | --- | --- |
| 저장 단위와 선언 상자 | `layout/table_layout.rs::cell_units`, `declared_row_frame_height`; 기존 `stored_restart_end_row_box`와 행 높이 계산 공유 | 해당 칸의 `start == 0`, 선언 상자의 내용 높이·남은 공간·재시작 위치로 판단. 문서 ID나 특정 수치 분기 없음 |
| 재시작과 용량 컷 | `advance_row_cut_inner::trimmed_capacity_fit`; Content/Physical의 기존 trim과 tolerance를 두 판단에서 공유 | row 9 내용 예산 69.027px, 수용 높이 67.200px, 컷 `[2,3,3]`; 끝 줄간격을 같은 계산으로 제외 |
| 물리 fit와 예약 | `layout/table_layout/row_physical_band.rs::advance_row_cut_for_physical_band` → `scan/runner/row_step.rs` → `accepted_partial_row_box`·`split_end_reserved` | 선언 상자 82.32px와 실제 내용+padding을 소비한다. 컷만 줄인 뒤 paint에서 상자를 늘리는 새 보정 없음 |
| 이어받기 원점 | `continuation/fragment/scan.rs` → `row_start_owns_stored_frame` → `scan/source_frame.rs::SourceFrameQuery` | 시작 `[2,3,3]`이 저장 프레임 경계가 아니므로 뒤 행의 옛 프레임 끝을 현재 쪽의 끝으로 인증하지 않음 |
| 다음 끝 컷 | `row_step.rs`의 용량·source tail 소유 판정 → `continuation/fragment/emit.rs::PageItem::PartialTable` | row 16 내용 예산 263.627px, 수용 246.400px, 최종 `[10,8,10]`. 대체 tail 조회도 `owns=false`여서 컷을 덮지 않음 |
| 실제 paint·종료 | PartialTable의 start/end cut·행 상자 → 공통 render tree → Native Skia / WASM Canvas2D | 1·2쪽의 마지막 줄과 2·3쪽 첫 줄, 다음 행, 마지막 「받은 사람」과 표 하단을 직접 판독. 3쪽으로 종료 |

원점 flag는 걸침 블록 뒤의 일반 행에도 영향을 줄 수 있으므로 블록을 통째로 비해당으로 보지 않았다.
`block_step`의 컷·물리 높이 계산은 이 flag를 직접 소비하지 않지만 다음 일반 행의 source-frame 조회는
소비한다. 기존 걸침 경계의 전체 회귀와 최신 공개 코퍼스·정상 대조군으로 보존 여부를 검증한다.
실제 블록 컷 알고리즘·여러 rowspan 종료의 원장은 이번 변경에서 수정하지 않는다.

## 입력 커밋과 독립 시각 검증

입력은 실제 저장된 HWP 5.0.3.4의 18행×3열 표다. 독립 PDF는 같은 국가법령정보센터 공개 서식의
Hwp 2020 출력(3쪽)이며 기존 PR의 파일을 그대로 사용했다. source·PDF·실행 파일·글꼴의 해시,
전체 TSV 원문·383쪽 편람 TSV·명령·PNG는 [검증 원장](assets/pr8542/visual-validation.md)에 보존했다.

| 입력 | SHA-256 |
| --- | --- |
| `samples/issue8539/nhis-dependent-requirements.hwp` | `53d7763a0e5c9860e62c8dc3b20cfb515aac6f6eceeeef01ff0fb1abbdfe8fef` |
| 대응 `nhis-dependent-requirements-2020.pdf` | `a54a96205d1916b44d98a71caf6114d47ddb34ebfa9f34263228f875e90e5da6` |

판정: 커밋 포함과 실제 입력 동일성 충족. 50 MiB 이상 PDF나 새 비공개 원본은 없다.
Native는 Skia의 직접 PNG, WASM은 최적화 package의 실제 Canvas2D API로 출력했다. SVG 래스터를 이 두
backend의 증거로 대신하지 않았다. Source와 동일한 HWPX/2024 재저장본의 해결을 주장하지 않으며,
이번 이슈의 입력·독립 기준은 HWP/Hwp 2020이다.

| 쪽 | devel Native raw | 후보 Native raw | devel Canvas raw | 후보 Canvas raw |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 97.69798% | 100.00000% | 97.74387% | 100.00000% |
| 2 | 57.29471% | 100.00000% | 57.33608% | 100.00000% |
| 3 | 64.01941% | 100.00000% | 64.10226% | 99.99607% |

양쪽 모두 3=3쪽, 미달·누락·2%p 초과 하락 0건이다. 독립 Print와 같은 페이지/칸에서 줄 소속을 직접
확인했다. Native의 일반 모드 gate는 `passed`, Canvas도 `passed`다. raw가 100.00%로 표시돼도 픽셀
완전 일치로 부르지 않는다. 획/안티앨리어싱과 raw·pixel·ink proxy의 구분은 위 원장에 기록했다.
수식이 없는 원문이며, 글자 간격·문장 부호·기준선·괘선과 두 페이지 경계는 별도로 판독했다.

검토자가 같은 원본의 render tree에서 추출한 결과는 devel `[2,2,2]`·`[9,8,9]`(독립 기대 불충족),
후보 `[2,3,3]`·`[10,8,10]`(충족)이다. 원 기여의 첫 회귀가 이 차이를 검출하며, 내용 보존·비겹침
2개가 수정 전에도 통과한다는 구분을 유지한다. 과거 base에서 Rust 실패를 직접 실행했다고 쓰지 않는다.

### 정상 대조와 편람

- 반례 6개 193쪽씩: Native·Canvas PNG가 전쪽 devel과 같고, Native 페이지 컷·렌더 트리도 같다.
  선언 상자가 남은 방보다 큰 행, 상자 밖의 재시작, HWPX·각주·걸침/이어받기 사례를 포함한다.
- 공개 `samples/` 1,494개 구조 비교: 정상 열기 1,490개 중 원 사례만 두 컷이 바뀌고 나머지 1,489개는
  전체 JSON이 같다. 암호 입력 필요 3개와 대형 CellBreak 1개(양쪽 각 90초 timeout)는 정상 통과에 합산하지 않는다.
- 필수 편람 Native 383쪽: 383=383쪽, 최저 raw 99.96465%, #8389 페이지별 하락 0건, `passed`.
  이 결과를 편람의 fresh WASM 전쪽 검증이라고 기록하지 않는다.

## 조판 원칙 준수 검토

| 항목 | 판정과 근거 |
| --- | --- |
| 제출 전 로컬·Open 상태 | 충족 — 고정 code/data 후보의 14개 gate 성공. Open/non-Draft |
| 기존 해결 선행 분석 | 충족 — #7870 원 기여/통합·#7965/#8148·독립 출력과 현재 source의 소유 계약 대조 |
| 일반성과 공통 수정 | 충족 — 저장 행 높이·내용 컷·물리 예산을 소비, 문서별 ID/수치 조건·새 clamp/은폐 없음 |
| 측정과 배치 | 충족 — 공유 trim/행 높이 → 예약/행 상자 → 동일 컷의 PartialTable → 실제 두 backend |
| 최신 base 기능 보존 | 충족 — 현재 devel과 병합, 193쪽씩 PNG 동일·1,489개 정상 구조 동일·편람 및 최종 전체 회귀 |
| 최종 출력 소비 | 충족 — 명시 공급 TTF의 Native PNG·실제 fresh Canvas와 직접 Print 대조 |
| 분할·이어받기 | 충족 — 두 시작/끝 컷·요구/예약/물리 높이·대체 tail 소유·마지막 내용과 다음 행 확인 |
| 줄 소속·점유 | 충족 — 칸별 저장 유닛과 끝 줄간격, 명시 나누기는 기존 guard 보존; 재조판의 source-current guard 유지 |
| 사례·증거 독립성 | 충족 — 미수정 공개 저장본·독립 같은 서식 PDF·반례·현재 base 음성 대조 |
| 기준값 | 충족 — 새 원본의 독립 3쪽을 등록, 기존 baseline/허용치·golden 기대값 변경 없음 |
| 회귀 선행 조건 | 충족 — 관련 전 3쪽의 두 backend raw 90% 이상·직접 판독 뒤 측정 주석/쪽수 등록 |
| 확대 판독 | 충족 — 3쪽의 줄·기준선·문장 부호·표 선·첫/끝 줄·뒤 행 직접 대조; 수식은 없는 원문 |
| 주장·검증 대응 | 충족 — 실제 source/빌드/폰트·TSV·본문/댓글 대상 PNG와 완료 gate를 연결 |

기존 assertion은 쪽·칸 소유와 내용·행 사이 관계를 검사한다. 전체 3쪽은 독립 PDF의 고정형 기대값이다.
절대 px 핀·고정 영역은 추가하지 않았다. 같은 원본을 사용하는 기존 별도 px/golden 검사는 발견하지 않았다.

## 최종 로컬 검증

고정 후보 `790ef21567d041fe272253673fa63fb1ce7809df`, 공용 target
`/home/edward/mygithub/rhwp/target/pr-review`를 사용한다. 새 test 주석/baseline 보강이 있으므로 원 head의
기존 CI 재사용만으로 새 후보를 완료하지 않았고 아래 명령을 순차로 모두 완료했다.

| 실제 단계 | 결과 | 소요(초) |
| --- | --- | ---: |
| `prepare` | PASS | 1.60 |
| `focused-native` | 신규 회귀 3 PASS, 319 filter 제외 | 598.65 |
| `fmt` | PASS | 25.00 |
| `clippy-native` | PASS | 166.53 |
| `clippy-wasm` | PASS | 141.99 |
| `workspace-build` | PASS | 269.17 |
| `workspace-clippy` | PASS | 203.35 |
| `manifest` | PASS, 2,070 sources / 48 integration targets | 0.81 |
| `nextest-all` | 12,640 PASS / 45 skipped, 55 binaries | 2310.19 |
| `native-lib` | 4,040 PASS / 12 ignored (root 3,858 + contracts 15 + chart 165 + crypto 2) | 460.60 |
| `native-placeholder` | 2 PASS / 308 filter 제외 | 467.07 |
| `native-direct-pdf` | 22 PASS / 273 filter 제외 | 25.78 |
| `diff-check` | PASS | 0.08 |
| `document-links` | PASS | 37.50 |

full nextest의 실제 테스트 실행은 930.082초였고, compile을 포함한 명령 전체는 2,310.19초다.
최소 글꼴 환경에서 새 HWP 1개를 전달한 보안 계약 6개와 기존 SVG snapshot 4개도 전체 실행에서 모두
PASS였다. source-frame·exact-fit·저장 재시작·TAC atom·걸침 컷 등 10개 관련 모듈의 46 PASS를 실제
PASS 행과 대조했다. 이 수를 전체 12,640에 다시 합산하지 않는다. Native 4,064는 위 4,040+2+22다.
기존 ignored/filtered와 신규 필수 테스트 실행을 구분했고 skip/allowlist를 추가하지 않았다.

최소 글꼴 검사는 bwrap에서 시스템·사용자·Windows·개인 ttfs 및 검증용 글꼴 cache를 숨기고,
`ttfs/opensource`의 NotoSansKR 2종·NotoSansSymbols2·RHWPPublicBatang·RHWPOldHangul 5개만 노출했다.
Node 24.15.0 실행 디렉터리와 Rust 도구·공용 target을 명시적으로 연결했다. Fontconfig 목록뿐 아니라
`/mnt/c/Windows/Fonts`와 개인 hft 경로의 접근 불가도 확인했다. 시각 측정용 H2 글꼴 공급과 구분한다.
신규 sample 보안 입력은 `RHWP_SECURITY_SWEEP_SAMPLES_JSON=["samples/issue8539/nhis-dependent-requirements.hwp"]`다.

검사 도구는 Rust 1.93.1·nextest 0.9.137을 사용했다. 저장소 권고 nextest 0.9.140에 대한 안내와
사용하지 않은 JUnit profile의 `report-skipped` key 경고가 있었으나 검사 실패·선택 변경은 없었다.
`src/**`의 cfg(test) 변경이 없어 unit-tier 변경 gate는 비해당이다. 기존 SVG snapshot 4개가 모두
통과해 golden은 갱신하지 않았다.

초기 원 head의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41643028246),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41643027931),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41643028219),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/41643028171),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/41643028349)와 정책 집계는 성공 상태를 확인했다.
이는 아직 push하지 않은 새 후보의 GitHub CI가 아니다.

## 원격 처리와 merge 후 comment 계획

[원격 처리안](assets/pr8542/remote-action-plan.md)과 PR 본문·감사 댓글 초안을 준비했다. 원격 범위를 승인받는다. 승인 뒤 원 기여 branch의 head가
변하지 않았는지 확인해 fast-forward push하고, 새 head CI·self-review·실제 최신 base 병합 tree를 확인한다.
merge는 기여 커밋을 보존하는 merge commit과 `--match-head-commit`을 사용한다. fork branch는 자동 삭제하지 않는다.

후속 운영 기록은 [post_merge 7.1](../manual/pr_review/post_merge.md#71-후속-문서-처리-결정)의 maintainer 직접
반영 범위(archive review·오늘할일·asset)만 사용한다. 이번 PR의 고유 diff에는 CI workflow/판정 정책 변경이 없어
옵션 2 강제 대상이 아니다. 실제 merge SHA·운영 기록·devel sync 뒤 승인된 이슈 완료와 감사 댓글을 게시한다.

감사 댓글에는 [Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment),
전 3쪽 수치·두 컷·직접 판독·기여/메인테이너 보강 범위를 적는다. `assets/pr8542/`의 1·2쪽 Native/Canvas
review·overlay 8개를 실제 merge SHA의 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr8542/…`
이미지로 연결한다. asset 존재·응답·해시를 확인한 뒤 `--body-file`로 게시하고 API와 화면을 재확인한다.

원격 조치 직전 head·required check·mergeable을 재조회한다. 완료 후 작업 전용 branch/worktree와 저장소 루트
output 전체·사용 종료한 공용 `target/pr-review/debug`를 정리한다. 사용자/다른 작업의 worktree·소스·글꼴과
contributor fork는 보존하며, 추가 압축·백업은 만들지 않는다.

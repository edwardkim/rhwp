# PR #8420 셀프 리뷰 — PDF 가져오기 오류와 쪽별 진행률

## 최종 판정

**승인 — 승인된 구현 범위의 로컬 검증과 정확한 code head의 GitHub CI를 완료했다.**
셀프 리뷰에서 이번 제출을 차단할 제품 결함은 발견하지 않았다. 초기 표시 지연·일부 글꼴·세선
표현 차이는 메인테이너 결정에 따라 [후속 백로그](../../plans/task_m100_8413_backlog.md)에 남겼다.
이 기술 검토 판정은 GitHub의 타인 승인 리뷰나 작업지시자의 머지 승인이 아니다.
문서 전용 trailing head의 필수 검사와 실제 재사용 판정은 push 후 확인해야 하며, **머지는 별도 승인 대기**다.

## 접수와 검증 신원

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8420](https://github.com/edwardkim/rhwp/pull/8420) / edwardkim / devel |
| code head | `b41883870cd225d67bb47f3b7581bec231718270` |
| 최신 base | `da39c536613a4bd543dc72b63a556183efc0383f` |
| 브랜치 | `task/8413-pdf-import-sample-errors` |
| 관련 이슈 | #8413 곡선 clip, #8414 이미지 예산, #8415 인증, #8416 연산량 예산 — Fixes 연결 |
| 작성 시점 | 2026-11-01 13:02 이후 KST, Open·MERGEABLE·CLEAN, reviewer 미지정 |
| 승인 범위 | 백로그 기록·원격 push·Open PR·self-review, 원본 PDF 4개 공개 커밋 |

본인이 구현한 변경을 검토한 self-review이며 별도 에이전트의 독립 코드 리뷰로 표기하지 않는다.
1,000줄 초과 변경은 구현·로컬 검증·코드 검토·CI를 구분해 수행했다. 관리자 강제 병합은 하지 않는다.

## 변경과 코드 검토

| 경로 | 확인한 계약과 결론 |
| --- | --- |
| backend `image_clip_path.rs` → image 추출 → PDF lowering | 경계 사각형으로 곡선을 대신하지 않는다. 원래 path·winding/even-odd·q/Q·Form·변환을 alpha mask로 소비하고, path/layer/pixel/work 제한을 유지한다. 샘플 이름이나 특정 좌표로 분기하지 않는다. |
| 이미지 decode·mask·PNG 공유 | 동일 자원의 bounded decode/PNG 공유와 실제 mask 크기·표본 검증을 적용한다. Gray/Flate/predictor/JPEG 정상 표본과 손상 입력을 구분하고, 실패 시 부분 결과를 채택하지 않는다. |
| 회전 span → `lowering.rs` char ratio → 공통 IR paint·저장 | 실제 직교 축별 크기를 보존하고 char-shape 키에도 ratio를 포함한다. 반사·shear까지 지원했다고 주장하지 않는다. signed rectangle은 양 끝점으로 영역을 계산한다. |
| `fonts/type3_empty.rs` → IR text paint | 모든 CharProc가 정확한 metrics-only 명령이라는 증거가 있을 때만 비표시한다. 검색 문자열·advance는 유지하며 이름·빈 bbox만으로 숨기지 않는다. paint/unknown/빈 procedure 반례를 검사했다. |
| PDF PNG 출처 → 공통 색 소비 | PDF에서 이미 확정된 색만 재보정을 피한다. 일반 HWP 이미지의 기존 색 처리는 유지한다. 정상 한컴 색 대조군은 이전/후보 Native 출력이 같고 양쪽 raw 100%다. |
| 인증 → bounded parser → CLI/Worker/Studio | R2–R6의 사용자/소유자 인증 후 편집 가져오기를 허용한다. 인증 전 cache와 object/stream 복호화, 한도 전달을 확인했다. 암호 누락·오입력·미지원 방식을 구분하고 Debug·진단·editor reopen 설정에 암호를 남기지 않는다. |
| 요청 정책 → backend counters → WASM reopen | 누적 작업량은 u64로 전달한다. JS 정수·유한성 검사를 유지하며 명시적 낮은 host cap과 constrained 프로필을 덮어쓰지 않는다. 8 GiB 누적 예산은 프로세스 메모리 보장이 아니다. |
| parser 쪽 경계 → Worker → 대화창 | 실제 선택 쪽 시작·완료를 전달하고 현재 원본 쪽/전체 쪽·완료율을 표시한다. 역행·범위 밖 진행 메시지는 거부하며 변환 100%와 편집 문서 준비를 구분한다. |

이번 변경은 PDF 가져오기·공통 이미지 소비 계약이며 HWP 표의 줄 구성·pagination·rowspan·이어받기
알고리즘을 수정하지 않는다. 따라서 표 조각 컷/예약 높이 절차는 비해당이다. 이미지 clip은 실제
추출→IR→Native/Canvas 출력과 저장 재열기를 확인했으며 helper 이름만으로 공유를 판정하지 않았다.

## 기존 해결 이력·리베이스·회귀 검토

- 이전 base `bc9dfdd057258de59afebeb5121c4bee7398e07f`에서 최신 base로 clean rebase했다.
  새로 반영한 것은 #8417의 CFB 테스트 경로 정규화 두 줄·문서다. #8225 → #8233 → #8395와
  [#8417 독립 검토](pr_8417_review.md)를 추적했고, 현재 후보에서 CFB 보존 3건과 해당 suite
  Clippy를 실행해 통과했다. 제품·조판 소비 경로는 바뀌지 않는다.
- 시각 검증 구현 `f17ca45e14`는 리베이스 후 `5ef817b434`와 제품 내용이 같다. 제출 전 변경은
  `lowering.rs`의 source-side 테스트 1건을 통합 검사로 옮긴 것뿐이며 `cfg(test)` 밖 내용을 대조했다.
  기존 전체·시각 실행을 새로운 SHA에서 재실행했다고 표기하지 않는다.
- source-side unit-tier 실패를 허용치 변경으로 우회하지 않았다. 유효한 Gray/Flate PDF를 공개 API로
  가져와 signed crop 축 한도 거부와 실패 후 정상 요청 복구를 검사하는 방식으로 이관했다.
  자동 생성 harness가 오래돼 0건 실행된 첫 시도는 제외하고, 재생성 후 실제 자원 정책 12건 PASS를 확인했다.
- 기존 mask 기대값은 #7974 → #8174/#8360의 이력을 확인했다. 정상 DCT mask를 새로 지원하면서
  한 바이트 가짜 JPEG를 malformed image로 분류했으며 요청 원자성·선택한 정상 쪽 성공은 유지했다.
- 새 합성 관계 검사 5건은 clip 내부/외부·구멍·Form·affine alpha, mask 표본, 회전 ratio·저장,
  검색 문자열, 색 출처와 signed rectangle 포함 관계를 보호한다. 미검증 절대 px 위치를 고정하지 않았다.

## 로컬 검증과 시각 증거

[구현 기록](../../working/task_m100_8413_implementation.md),
[전체 검증](../assets/issue8413/curve-mask-validation-20261101/validation.json),
[제출 직전 검증](../assets/issue8413/curve-mask-validation-20261101/submission-validation.json)에
명령·실행 신원·결과·로그 해시를 기록했다.

- 개인·시스템·Windows·추출 글꼴 경로를 격리한 전체 회귀 12,249 PASS, Native lib 4,041 PASS와
  그림 누락 2·직접 PDF 22·자원 제한 134·새 계약 5건 PASS. 위 수치는 이관 전 측정이며 이후 제품 동작은 동일하다.
- 테스트 이관 후 자원 정책 12건, unit-tier, fmt·Native/WASM Clippy·workspace build/all-target Clippy PASS.
- Studio 2,705 PASS/기존 skip 2, TypeScript PASS, Worker 30 PASS, 실제 브라우저의 암호·쪽 진행률·
  대화창·busy 상태 E2E 4개 모듈 PASS. 8 GiB WASM 잔량과 무효 수치 7개 거부도 실제 실행했다.
- 원본 PDF 4개는 [samples/issue8413](../../../samples/issue8413/README.md)에 공개 승인된 원래 바이트로
  포함했다. 공개 합성 PDF와 생성 코드는 [pdf/issue8413](../../../pdf/issue8413/README.md), 실제 저장·
  재열기한 13쪽 HWP/HWPX는 [계약 fixture](../../../tests/fixtures/issue8413/README.md)에 있다.
- 원본은 PDF 자체가 독립 기준이며 PDFium 96dpi 출력과 비교한다. 이를 한컴 Print로 주장하지 않는다.
  실제 사용하는 글꼴 환경은 manifest에 적고 비교용 추출 글꼴은 배포하지 않는다.

| 원본 | 쪽수 | Native 전쪽 최저 raw | fresh WASM 전쪽 최저 raw |
| --- | ---: | ---: | ---: |
| img_clipping_01.pdf | 32 | 99.89% | 99.89% |
| img_bug_01.pdf | 37 | 99.76% | 99.83% |
| PDF_ENCRYPTION_UNSUPPORTED.pdf | 57 | 91.72% | 91.64% |
| f87pro.pdf | 1 | 99.82% | 99.82% |
| 공개 합성 대조군 | 13 | 98.88% | 98.97% |

[전쪽 TSV·PNG·해시](../assets/issue8413/curve-mask-validation-20261101/README.md)에 누락·90% 미만은 없다.
곡선·mask alpha·회전 문자·색·키보드 세선과 본문 차이를 동일 영역에서 직접 확대 판독했다.
이전 devel은 원본 네 건을 admission/추출 오류로 가져오지 못하므로 이 원본들의 base TSV나
숫자상 개선율을 만들지 않았다. 정상 HWP 색 대조군은 같은 조건에서 무변화이며 기존 clip
5·1·1쪽은 양쪽 100%다. 백로그의 본문 굵기/간격·세선 표현은 점수로 해결됐다고 간주하지 않는다.

실제 Studio 4건과 HWP/HWPX 저장·재열기를 확인했다. 큰 입력은 표준 프로필·제한 시간 300초로
검증했으며 f87pro 초기 표시는 약 140초였다. 이는 후속 성능 과제다. 기본 30초가 모든 입력에
충분하다는 주장이나 서로 다른 단계 시간의 단순 비교를 하지 않는다.
PR 게시 후 본문의 UTF-8·SHA 고정 링크, raw asset 16개 바이트와 원본 PDF 4개의 Git blob을 재확인했다.

## 정확한 code head의 GitHub CI

모든 행의 code head는 `b41883870cd225d67bb47f3b7581bec231718270`이다.

| 검사 | 결과·실행 |
| --- | --- |
| CI: lint·frontend package·Archive A–E 실제 실행·Native Skia·집계 | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41269833397) |
| CodeQL 전체 | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41269833401) |
| Render Diff | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41269833472) |
| Adapter inter-diff | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41269833439) |
| Proptest roundtrip | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41269833478) |
| 최종 CI Impact Policy | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41271434170) |

34개 SUCCESS·5개 조건부 SKIPPED를 확인했다. 별도 WASM Build·release operations·native host
package·frontend unit 전용 job·workflow promotion은 정책 분기상 생략됐으며 실행한 것으로 세지 않는다.
프런트엔드 package와 Render Diff에서 fresh WASM을 실제 빌드·소비했다. CI 재실행·취소·workflow
보정·허용 기준 변경은 없었다. [게시·CI 증적](../assets/issue8413/curve-mask-validation-20261101/pr-8420-code-ci.json)을 연결한다.

## 후행 기록과 남은 절차

이 review와 오늘할일은 위 CI가 성공한 뒤 문서-only single-parent commit으로 추가한다.
최신 devel의 다른 PR 기록을 보존하고 실제 merge tree의 링크·공백·기록 보존을 검사한 뒤 push한다.
새 head의 required checks와 실제 green code candidate 재사용을 확인한다. 머지 승인·머지·이슈 종료는
완료하지 않았으며, 원 contributor에게 게시할 후속 댓글은 본인 PR이므로 비해당이다.

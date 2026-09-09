# PR #6957 self-review

## 통합 merge 확정 기록 (2026-09-09)

- [통합 PR #6957](https://github.com/edwardkim/rhwp/pull/6957)은 2026-09-09 14:41:08 UTC에 일반 merge로 통합됐다. merge SHA는 `d43937e0de7cf465185d23ce6b06fa47e7824e57`, 승인한 PR head는 `4e422a57d6775eb2f11dffb70b37632823659829`다.
- merge 직전 최신 head는 `MERGEABLE / CLEAN`이었다. [Build & Test 및 Rust/Lint/Native Skia](https://github.com/edwardkim/rhwp/actions/runs/34363253946), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34363253722), [CodeQL 언어별 분석](https://github.com/edwardkim/rhwp/actions/runs/34363253980), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34363253961), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34363253986)가 성공했다. GHAS CodeQL check는 `NEUTRAL`, [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/34365098894)는 `SUCCESS`였고 대기·실패 항목은 없었다.
- review·오늘할일·최종 PDF 2개·대표 PNG 5개는 코드 PR head에 이미 포함되어 merge됐다. source PR review 3개는 이번 문서-only 후속 처리에서 archive로 이동한다. 오늘할일에는 새 운영 항목을 반복 추가하지 않고 이동된 링크만 보정한다.
- 리베이스 후 추가 로컬 테스트·PDF 출력은 사용자 지시대로 생략했다. 이번 GitHub Full CI 성공은 이전 로컬 검증과 별도 근거다. devel push의 post-merge CI는 PR head CI와 구분하며 이 기록에서 성공을 선언하지 않는다.
- 후속 문서 PR 완료와 최종 devel sync 뒤 #6872/#6941을 해결 범위 내에서 종료하고 #6949/#6952를 통합 대체로 종료할 계획이다. #6865는 회색조·아이콘·영문 배너 잔여를 기록하고 OPEN을 유지한다. 이 절은 comment 게시 전 확정 기록이며 실제 게시 permalink와 종료 상태는 원 PR/이슈에 남긴다.
- 이번 작업 전용 local·remote branch 정리는 사용자 승인 범위다. 기여자 `planet6897/rhwp`의 source branch, 기본 작업공간과 공유 `target/pr-review`는 보존한다.
- 아래 Open/CI 대기/미게시 표기는 작성 당시 이력이다. 현재 통합 결과는 이 절이 우선하며 최종 판정의 기술적 범위와 잔여는 변경하지 않는다.

## 최종 판정

**최종 판정: 메인터너 보정 후 수용 가능.** #6949/#6952의 원 head를 그대로 승인하는 것이 아니라,
최신 devel 위로 리베이스한 보정 커밋 `fe92170a0cac3e21aca3281b7e4a054052dadf5b`에 대한 판정이다.
작성자 `jangster77`의 self-review이며 reviewer를 지정하지 않았다. 최신 CI 및 작업지시자 시각·merge
승인은 별도 게이트다. 이 기록은 GitHub approve, merge, 원 PR/이슈 close를 수행하지 않는다.

## Metadata (2026-09-09 작성 시점 참고값)

| 항목 | 값 |
| --- | --- |
| PR | [#6957](https://github.com/edwardkim/rhwp/pull/6957) |
| 작성자 | `jangster77` |
| 원 기여자 | `planet6897`; 체리픽 저자 이력 보존 |
| base | `devel`, `d8e4ab727b70b6abfcf11766134e09a9a9bfc982` |
| branch | `review/planet6897-6949-6952-20260909` |
| 생성 시 head | `0bedf37bd3d8cde7c46f71ba4c68322391f9e0f8` |
| 생성 시 규모 | 23 files, +1,676/-28; 이 self-review 후속 문서는 별도 추가 |
| 상태 | Open, Draft 아님, `MERGEABLE`, `BLOCKED` |
| CI | 생성 head의 CI/preflight 진행·대기 중; 전체 성공 아님 |
| Reviewer | 없음 (self PR 정책) |

생성 head의 [CI run](https://github.com/edwardkim/rhwp/actions/runs/34362970483)은 참고 이력이다.
이 문서와 오늘할일을 추가한 후속 head의 CI를 최종 merge 전에 별도로 확인해야 한다.
1,000줄을 넘는 통합 PR이며 규모만으로 admin merge하지 않는다.

## 범위와 메인터너 보정

- #6949 원 head: `8fbfd865b1190fdf95b5a98d80467a53c944ce59`.
  완성된 ROP 시퀀스·목적 영역·클립과 실제 흑백 팔레트를 확인한 뒤 중간 DPA 마스크를 제거한다.
  실물 8x8 하프톤의 1단위 경계 차이만 제한적으로 허용하며 기존 XOR 회귀를 해소했다.
- #6952 원 head: `4529c2a0c0104a2783b44a5b798a7badce628042`.
  각주/미주 USER_CHAR 왕복을 보존하고 #6940의 명시적 빈 접미 처리를 유지했다.
  메인터너 회귀 fixture의 charPrIDRef 오류를 정상 참조 등록으로 수정했다.
- source PR의 발견 사항·이슈별 범위는 [#6949 review](pr_6949_review.md),
  [#6952 review](pr_6952_review.md)를 따른다. 관련 이슈는 #6865, #6872, #6941이다.
- 소스·테스트·증적을 바꾸지 않는 문서 후속 commit으로 이 self-review와 오늘할일을 PR에 포함한다.

## 리베이스와 해시 확인

`upstream/devel`을 fetch하고 위 base로 리베이스했다. 오늘할일을 포함해 충돌은 0건이며 수동 해결은 없었다.
보정 커밋은 `80ceb5ce1149106e389f7325a48545bb20a14031`에서
`fe92170a0cac3e21aca3281b7e4a054052dadf5b`로 바뀌었다.

PR의 소스·테스트 7개, 후보 PDF 2개, 대표 PNG 5개는 리베이스 전 `8be25bae1`와 Git blob 대조에서
동일했다. 파일별 SHA-256과 commit 대응표는 [통합 해시 기록](pr_6949_6952_review_impl.md#리베이스-해시-대응표)에 보존했다.
기존 바이너리도 해시를 재확인했지만 최신 base에서 재빌드한 바이너리는 아니다.

## 완료 검증 및 추가 실행 생략

아래는 리베이스 전 보정본에 대해 완료한 결과이며 최신 base에서 재실행한 결과가 아니다.

- 집중 회귀: 32/32 통과.
- 전체 nextest: 9,377/9,377 통과, 46 skipped.
- Native Skia library: 4,112 passed, 13 ignored. CLI 및 #1144: 각 1/1 통과.
- fmt, native/WASM/workspace-all-target Clippy 3단계, workspace build, manifest/unit-tier 검사 통과.
- WASM web release 패키지 및 wasm-opt 통과.
- WMF 원 문서 15쪽 직접 visual sweep과 2/3/7쪽 대표 PNG 판독 완료.
- 각주 문서 A 36쪽/B 32쪽의 XML·PDF 비교 완료. 최종 바이너리의 HWPX 왕복 결과가 기존 검증본과
  바이트 단위로 같아 후보 PDF와 68쪽 비교 결과를 재사용했다. PDF를 다시 출력하지 않았다.

사용자가 "다른 conflict가 없으면 추가 테스트 없이 PR"을 지시했으므로 리베이스 후 build/회귀/Clippy/
Native Skia/WASM/시각 검증은 추가 실행하지 않았다. 최신 base의 #6773은 표 삭제·model·WASM 코드를
변경했다. 이번 PR 파일의 동일성은 최신 base 전체의 실행 호환성을 보장하지 않으므로 통합 CI는 남은 게이트다.
구체적 명령·실행 시간·과거 실패 및 최종 통과 이력은 [통합 검증 기록](pr_6949_6952_review_impl.md)을 따른다.

## 시각 판정과 잔여 범위

- WMF: 자동 구조 후보 0건, 15쪽 평균 pixel_match 90.16332%, 평균 visual proxy 26.81390%.
  2/3쪽 체크무늬 재출현 해소와 7쪽 마스크 개선을 확인했다. 회색조 차이·누락 아이콘·7쪽 영문 배너 손상은
  잔여 현상이며 완전한 fidelity 통과 또는 #6865 전체 해결로 쓰지 않는다.
- 각주: 68쪽 모두 RGB 차이 32 초과 픽셀 0. A 2쪽에는 임계값 이하 3픽셀 차이가 있으며 B 32쪽은
  픽셀 단위로 같았다. 대표 A 10쪽/B 13쪽의 pixel_match와 ink-union proxy는 100%다.
- 작업지시자 시각 승인은 별도다. 낮은 WMF proxy를 전체 시각 통과로 바꾸지 않는다.

## Merge 후 contributor PR comment 계획

정본은 [Visual Sweep GitHub merge comment](../../manual/verification/visual_sweep_guide.md#github-merge-comment)다.
실제 수치·페이지·잔여 현상은 위 판정과 각 source review의 comment 계획을 그대로 사용한다.
대표 PNG는 `mydocs/pr/assets/pr_6949_6952_maintainer_20260909/`의 다음 5개다.

- `pr6949-original-p002-review.png`
- `pr6949-original-p003-review.png`
- `pr6949-original-p007-review.png`
- `pr6952-note-a-p010-review.png`
- `pr6952-note-b-p013-review.png`

이미지 URL 형식은
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6949_6952_maintainer_20260909/<filename>`이다.
최신 head CI·작업지시자 승인을 충족하고 실제 merge 및 해당 asset의 devel 포함을 확인한 뒤에만
`--body-file`로 게시하고 API로 실제 Markdown을 재조회한다. 현재 comment/merge/close는 수행하지 않았다.
#6865 잔여 및 #6872/#6941 해결 범위를 따로 판단하며 포괄적 자동 close를 요청하지 않는다.

최종 PDF `pdf/pr6952-note-a-roundtrip-2020.pdf`, `pdf/pr6952-note-b-roundtrip-2020.pdf`와
대표 PNG 5개를 포함했다. 원 contributor 첨부 자료는 그 이력으로 보존했고, 새 로그·중간 SVG/JSON·raster·
실패 후보 이미지·generated suite는 추가하지 않았다.

## 다음 단계

처리 순서와 commit 이력은 [implementation 기록](pr_6957_review_impl.md)을 따른다.
최신 PR head CI와 작업지시자 승인을 받은 뒤 merge·원 PR 후속 처리 범위를 결정한다.

---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7263 통합 검토

## 최종 판정

**승인 — 메인터너 보정이 포함된 통합 코드의 검토 범위.** 최신 head의 GitHub required CI와
mergeability, 작업지시자의 merge 승인은 별도 조건이다. 이번 제출로 원 PR 또는 이슈를 종료하지 않았다.
작성자 `jangster77`의 self-review이며 reviewer를 지정하지 않았다.

## 접수와 범위

- [통합 PR #7263](https://github.com/edwardkim/rhwp/pull/7263), base `devel`.
- 코드 보정 commit `43a0fbeec`, 최초 제출 head `334950625261795fa93349ce0af93b29d9b391d8`.
- 고정 base `236a601da803b53429e9090eef652c661dd3bfe2`. 생성 시점 Open/non-draft,
  MERGEABLE/BLOCKED, 596 files(+4161/-375), PNG 528개. 상태는 작성 시점 참고값이다.
- 기본 경로: collaborator_self_merge. 보조: intake_and_review, local_validation,
  visual_fixture_evidence, multi_pr_update_branch, review_only_fast_pass, rework_and_exceptions.
  workflow·선택표와 해당 문서를 읽었다. 대형 PR의 개별 검토·메인터너 보정·시각 확인은 이전 회차에 완료했다.
- 검토 브랜치 `codex/pr7239-7240-review-20260917`. 원 PR별 source·보정·검증은 아래 문서에 보존했다.

| 원 PR | 적용 source | 현재 검토 |
| --- | --- | --- |
| [#7239](pr_7239_review.md) | `f5169220e` | WMF overflow·범위 방어 및 TextOut 보정 승인 |
| [#7240](pr_7240_review.md) | `3276bb635` | CLI setter/사용자 분할 명령 분리·저장 비트 보정 승인 |
| [#7242](pr_7242_review.md) | `28d2c4e0d`부터 `03ba57cb8`까지 7 commits | 저장 표 뒤 흐름·입력 정상화·테두리 보정 승인 |
| [#7243](pr_7243_review.md) | `d465c0a47` | scaffold 및 기존 표·중첩 이어받기 높이 예약 보정 승인 |

#7240은 제출 준비 중 원격 head가 `d2c969f1f`로 바뀐 것을 확인했다. CLI 동작 diff는 주석뿐이며
새 head도 setter/사용자 명령을 분리하는 방향이다. 그러나 새 source 전체를 체리픽·검증한 것으로
기록하지 않는다. 이 PR은 기존 source와 `df1b1de7f` 메인터너 보정의 검증된 결과를 제출한다.
기여자의 새 테스트·카탈로그·문서 변경은 자동으로 포함하지 않았다.

## 조판 원칙과 동작 검증

**충족(명시한 변경 범위).** [#7243 실제 소비 경로·독립 근거](pr_7243_review.md#후속-메인터너-보정--원인과-실제-소비-경로)와
각 원 PR의 계약 검토를 따른다. 부모 예약과 자식 RowCut 실제 물리 높이를 공유하고 비가시 tail의
중복 예약을 제거했다. 문서 ID 분기·전역 여백 판정 변경·출력 clamp로 불일치를 숨기지 않았다.
정상 86712 입력 교체 및 baseline 정정은 독립 한컴 PDF와 원문 식별에 근거했다.
새 실제 HWP/HWPX 검사는 p26 높이·다음 표 간격·p28 본문 수용을 확인하며 수정 전 실패/수정 후 통과했다.
WMF overflow checks 반례, 실제 CLI/WASM 진입점과 HWP/HWPX round-trip도 개별 기록에 연결했다.
키보드 UI·Undo/Redo 자동화는 미실행이며 검증 완료 범위로 확대하지 않는다.

## 최종 실행과 증적

[공통 최종 실행·명령·해시·PNG](pr_7243_review.md#최종-실행증적)를 정본으로 재사용한다.
이번 제출 준비에서 13개 최종 gate의 exit 0, 전체 nextest 및 Skia 최종 로그,
제출 source/test와 verify checkout의 바이트 일치를 다시 확인했다. 검증 뒤 변경은 review 문서다.

- 전체 nextest **10,023 passed / 50 skipped / 0 failed**.
- Native Skia lib **4,112 passed / 13 ignored**, 누락 그림 **2**, 직접 PDF **4 passed**.
- focused 10개 모듈 **50 passed**. fmt·Clippy 3종·workspace build·suite/unit 정책 검사 성공.
- fresh WASM은 `--no-opt` 빌드·브라우저 캡처 성공. wasm-opt 실행 성공을 주장하지 않는다.
- Native/fresh WASM 각 34쪽 compare·standalone overlay·review. 정상 HWP/HWPX 각64쪽,
  128쪽의 전후 SVG에서 p26·28·29만 변경; 정상 대조군 8문서198쪽 SVG 불변.
- p26 높이 61.88→68.613px(독립 저장68.373px/PDF68.245px), 후속 표 상대 간격
  약109.5→116.3px(PDF116.353px). 하단 넘침·겹침·offCanvas 0.
- 남은 공통 원점 약1.915px·글꼴·선 굵기·일부 행 차이와 PDF에도 있는 우측 표 경계는
  해결 완료로 확대하지 않는다. **Refs #5019, #7234**. #7234 전체는 종료하지 않는다.

## 검증 입력 커밋 확인

**충족.** 개별 검토 문서의 모든 로컬 HWP/HWPX/PDF 링크가 최초 제출 commit에 있는지,
작업 파일과 commit의 내용이 일치하는지 제출 문서 검사에서 재확인했다. 기존 입력을 재사용하며
개인 다운로드·임시 출력만을 최종 증거로 사용하지 않는다. 아래는 교체·정상화한 핵심 입력이다.

| 입력 | SHA-256 (최초 제출 commit) |
| --- | --- |
| [samples/86712_regulatory_analysis.hwp](../../../samples/86712_regulatory_analysis.hwp) | `ee82c7755617003cb972ba398da9cffadfed24ac0fa068eee1a5347da7658a88` |
| [samples/issue1891/86712_regulatory_analysis.hwpx](../../../samples/issue1891/86712_regulatory_analysis.hwpx) | `0f4f055c74a3d39f70e417ca6c700880d9645a202798cd0e5c11f6e32c180a19` |
| [pdf/86712_regulatory_analysis-hwp-2024.pdf](../../../pdf/86712_regulatory_analysis-hwp-2024.pdf) | `bc1025b0607bbac01fea960997fa54430fd8dcc2604831b02940e7815cbcf84f` |
| [pdf/86712_regulatory_analysis-hwpx-2024.pdf](../../../pdf/86712_regulatory_analysis-hwpx-2024.pdf) | `5a0b0038d4bc33a391f9f76d2e657d78ce63d1a34c288428ed37d5350cd31955` |
| [samples/stored-table-text-tail/native-8-0.hwpx](../../../samples/stored-table-text-tail/native-8-0.hwpx) | `3aa0379ab1b4d158800d33c73ae26eed909e5f08e3614dca228044780c8c5e64` |
| [pdf/pr7242/native-8-0-2020.pdf](../../../pdf/pr7242/native-8-0-2020.pdf) | `90d6e84f4c24a5d0e79cde09cfd5d9addf1d8e92da322a13cb03f0dab9a2c3c3` |

## Merge 후 contributor PR comment 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과
각 원 PR 검토 문서의 같은 제목 절을 따른다. 실제 merge SHA·최종 CI URL·원 기여/보정 구분·감사를
한국어로 기록한다. 수치·자동 후보와 그 한계는 각 개별 문서에서 가져오며 추정하지 않는다.
#7243 정상 HWP/HWPX p26·28·29의 Native/fresh WASM standalone overlay를 실제 이미지로
삽입하고 compare·review를 함께 연결한다. #7239·#7240·#7242도 개별 계획의 영향 페이지를 빠뜨리지 않는다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/`
아래 실제 보존 경로를 사용한다. asset이 devel에 들어온 뒤 UTF-8 파일과 `--body-file`로 게시하고
한국어·이미지 URL·실제 source/head를 API로 재확인한다. 원 PR/issue의 현재 상태와 전체 해결 범위를
다시 대조한 뒤 후속 처리하며, contributor fork branch는 보존한다.

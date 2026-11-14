# PR #9007 재검토 — 채우기 무늬 인덱스와 숨김 글자 판정

## 최종 판정

**승인.** 기존 세 수정 요청은 해소됐으며, 통합 담당자의 보호 검증 환경도 복원했다. 18개 문서 × Native/fresh WASM 총 3,542쪽의 입력·PDF SHA와 쪽수가 일치하고, 누락·공식 두 자리 보호값 하락은 모두 0이다. 별도 편람 HWPX Native 383쪽 gate도 통과했다. 기여자 제품 코드는 추가 변경하지 않았다. 최신 문서 head의 필수 CI 확인 후 원격 Approval과 원 PR 병합을 진행한다.

## 접수와 변경 범위

- PR: https://github.com/edwardkim/rhwp/pull/9007, 기여자 `gitprogam`, 관련 이슈 #8978.
- 검토 head: `abc4c1e82c34e4a112473c32886703b8356089d9`; 제품 보정 commit: `7de77727cdb31c9439c2582d99a7c4575f096bd6`.
- 최초 base/head 전쪽 비교 devel: `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`. 문서 push 전 호환 확인 base: `a210861488468868b5992231c55b78798e622e98`. 후자의 source를 PR에 merge/rebase하지 않았다.
- 2026-11-14 KST 재조회: Open, non-draft, mergeable, CLEAN. 기여자 원 commit을 변경하지 않았다.
- 기존 인덱스 해석을 공통 enum에 맞춰 무늬 없음 -1, 무늬 0~5로 통일한다. HWPX SLASH/BACK_SLASH와 모델·직렬화·Native/SVG/Canvas·편집 UI 소비를 확인했다.
- 추가 보정은 `shape_background`의 단색 판정을 `solid.hatch().is_none()`로 바꿔 pattern 0 가로 무늬를 불투명 단색 가림으로 취급하지 않게 한다. HWP/HWPX 기준 fresh WASM TSV가 추가됐고, 59043 기존 미달은 #7445 → #7778의 이관 근거와 연결됐다.

## 실행한 검증

| 검증 | 결과 |
| --- | --- |
| `issue_8978_fill_pattern_index` | 최소 글꼴 환경 6개 통과 |
| `hidden_text_contract` | 23개 통과 |
| UI 무늬 sentinel 집중 검사 | 9개 통과 |
| fresh WASM 준비 후 Frontend 전체 | 2,784개 통과, 2개 skip, 실패 0 |
| 정확한 PR head Full CI | [run 43535321673](https://github.com/edwardkim/rhwp/actions/runs/43535321673) 성공; lint·Native Skia·기본 회귀 archive·Frontend package 포함 |
| 관련 원격 검사 | CodeQL·Render Diff·Proptest·adapter 성공 |
| 최신 devel 병합 시뮬레이션 | 원 source 및 검토 기록이 충돌 없이 병합됨. 최종 trailing 문서의 정확한 tree·링크·기록 보존 검사는 push 직전에 재확인한다. |
| 변경 문서 링크·diff 공백 검사 | 통과 |

첫 Frontend 실행은 WASM 패키지 준비 전 모듈 누락으로 실패했으며, fresh WASM 패키지를 공급한 전체 재실행 결과를 위에 기록했다. 전체 Rust 회귀는 제품 코드 변경 없이 해당 exact-head 녹색 Full CI를 재사용했다. 로컬 집중 검사와 시각 검증은 별도로 실행했다.

## 최신 devel 대비 전쪽 비교

[전쪽 정밀 비교와 정수 집계](../assets/pr_9007_recheck_20261114/base-head-comparison.tsv), [입력/PDF 해시](../assets/pr_9007_recheck_20261114/inputs.json), [요약](../assets/pr_9007_recheck_20261114/main-summary.json).
각 backend에서 무늬 HWP 1쪽·HWPX 1쪽·hwpspec 178쪽·59043 37쪽을 base/head 모두 출력했다. 96 DPI, Print, 독립 한컴 PDF, 같은 글꼴 공급, 같은 sweep 측정 경로를 사용했다. 무늬와 hwpspec은 compat 2024, 59043은 compat 2022다.

| 입력 | Native 후보 최저 raw | fresh WASM 후보 최저 raw |
| --- | ---: | ---: |
| 무늬 HWP | 99.966083% | 99.987953% |
| 무늬 HWPX | 99.966083% | 99.987953% |
| hwpspec | 95.205645% | 95.206925% |
| 59043 | 38.340558% | 38.336022% |

두 자리 표시값의 하락과 2%p 초과 하락은 모두 0쪽이다. 정밀 분수에서는 Native hwpspec 23쪽이 99.993960258501% → 99.993885288003%, 59043 9쪽이 99.967274603834% → 99.967157479884%다. 무늬 변경으로 합집합 크기가 달라졌으며 미대응 픽셀은 각각 4개와 78개로 유지됐다. 이를 정밀 하락 0쪽으로 표현하지 않는다. WASM 정밀 하락은 0쪽이다.
59043의 90% 미만 8쪽(12·14·15·16·20·25·28·37)은 같은 base에도 존재한다. #7778에 이미 연결된 기존 미해결 범위이며 이 PR에서 시각 개선 완료로 판정하지 않는다. 기존 유효한 네 회귀 검사의 보호 범위는 유지된다.

무늬 0~5의 방향·격자와 무늬 없음, hwpspec 23쪽 RECORD HEADER의 대각 무늬·본문을 독립 PDF와 직접 대조했다. 두 경로의 무늬 방향은 일치하며 본문 내용은 보존된다. 이것은 모든 보호 문서의 직접 판독 완료를 뜻하지 않는다.

## 보호 검증 복원과 최종 통과

[최종 근거와 재현 조건](../assets/pr9007_protection_20261114/README.md), [3,542행 대조](../assets/pr9007_protection_20261114/comparison.tsv), [문서별 생산 원장](../assets/pr9007_protection_20261114/datasets.json), [필수 편람 gate](../assets/pr9007_protection_20261114/handbook-gate.json).

초기 환경의 Native 78쪽·WASM 87쪽 하락은 [초기 기록](../assets/pr_9007_recheck_20261114/README.md)에 구분했다.
#8896·#8946의 원 해결 → 원 프로그램·명시 환경 → 독립 Print 증거를 추적했다. 문서별 가족·스타일 그룹,
2020/2024 대체 환경, 구형 수식·PUA·삼각형 프로그램, WASM의 연결 그림과 OS Helvetica 전달을 복원했다.
독립 문자 표본의 원 프로그램 해시를 선행 수용본과 대조했으며 목표 PDF에서 글꼴을 추출해 공급하지 않았다.
이 과정에서 제품 코드·정본 TSV·PDF·마스크·허용치를 바꾸지 않았다.

최종 양 경로 각 1,771쪽, 합계 3,542쪽의 보호값 하락은 0이며 최저 raw는 98.09%다.
모든 문서는 전쪽 데이터셋 단위로 선정했고 쪽별 최댓값을 조합하지 않았다. 별도 편람 HWPX Native 383쪽도 passed다.
수식의 분자·분모·괄호와 본문 줄 끝을 같은 영역에서 2배 확대했고, 네모·원형·삼각형·화살표·WMF 글자와
후속 본문도 직접 대조했다. raw 통과를 RGB 완전 동일로 표시하지 않는다. 원본 WMF의 배경 농도와 raster
안티앨리어싱 차이는 보호값과 구분해 기록했다. 59043의 앞서 확인한 기존 미해결 범위를 이번 환경 복원으로 해결했다고 주장하지 않는다.

## 조판 원칙과 범위

무늬 번호·방향은 기존 공통 enum과 HWP/HWPX 계약으로 연결한다. 숨김 글자 질의도 `solid.hatch()`를
소비하므로 가로 무늬를 단색 가림으로 오인하지 않는다. 임의 수치·문서별 조판 분기·clamp를 추가하지 않았다.
이번 추가 작업은 검증 환경 복원과 기록이며 줄 구성·높이·페이지 분할·이어받기 생산/소비 경로를 변경하지 않았다.
따라서 새 pagination 경계나 새 고정 px 회귀를 만들지 않는다. 기존 제품 집중 검사·독립 출력·base/head 비교를 유지했다.

## 생산 정보와 통합 경로

- 원 PR 제품 head: `abc4c1e82c34e4a112473c32886703b8356089d9`.
- 보호 검증 producer: `4d42913568e60565b26d1270739484dd57864714` — 원 head 위의 검토 문서 전용 commit이며 제품 소스는 같다.
- 이번 Native SHA-256: `f5683b5d827e93da553f21f0d6c935e59a99d700675e01377282f95449c2da37`.
- 최적화 fresh WASM SHA-256: `833760f0cf544d9723a220783d573938375c78874a418a8d527cb6c4f260255b`.
- 초기 Native/base 생산 해시는 초기 증적에 보존했다. 과거 출력과 이번 재실행을 혼동하지 않는다.
- 기본 경로: `collaborator_external_pr`의 원 contributor head 직접 기록. 보조 경로: intake, local_validation, visual_fixture_evidence, review_only_fast_pass, post_merge.
- `maintainerCanModify=true`, 원 기여자 `gitprogam`의 commit과 fork branch를 보존한다. 별도 통합 PR이나 기여자 rebase·amend·force-push는 하지 않는다.
- 최신 code head Full CI를 재사용할 수 있는 문서 전용 trailing 범위만 추가한다. push 전 원격 SHA·LFS 범위·최신 devel merge tree·공백·문서 링크·오늘할일 보존을 확인하고, push 뒤 실제 fast-pass와 최신 required aggregate를 확인한다.
- [앞선 재검토 COMMENT](https://github.com/edwardkim/rhwp/pull/9007#pullrequestreview-5901900092) 당시에는 보호 검증 미충족으로 승인하지 않았다. 기존 세 인라인 스레드는 해결됐고, 이번 전쪽 검증으로 남은 보류 조건을 해소했다.
- 사용자 요청 범위에 따라 최종 Approval → 정확한 head 병합 → duration 결과·#8978 종료 확인과 증적 댓글 → devel 동기화·전용 작업공간·유휴 산출물 정리를 수행한다. 병합 SHA와 실제 종료 시각은 실행 전 완료로 기록하지 않는다.

## Merge 후 contributor PR comment 계획

병합 뒤 확정한 merge SHA와 최신 required check를 조회하고, 기여자께 무늬 번호·방향 및 숨김 글자 판정 보정에 감사한다.
원 제품 코드 위의 검토 기록만 추가했으며 보호 검증 미달은 검증 환경 복원으로 해소했음을 구분한다.
문서·오늘할일은 이 PR에 이미 포함되어 있으므로 중복 기록 PR은 만들지 않는다.
#8978의 실제 종료 상태와 duration 메타데이터 갱신 결과를 확인해 댓글에 기록한다.

시각 근거는 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
merge SHA로 고정한 `mydocs/pr/assets/pr9007_protection_20261114/direct-review/fill-patterns-source-head.png`를
본문 이미지로 표시한다. 무늬 HWP/HWPX 각 1쪽의 0~5 방향·무늬 없음과 Native/PDF/WASM 대조이며,
기존 제품 head의 정식 증거를 재사용한 그림이다. Native 최저 raw 99.966083%, WASM 99.987953%를 함께 적는다.
전체 보호 문서 18종·양 경로 3,542쪽은 보호값 하락 0, 최저 98.09%, 별도 편람 gate는 383쪽 통과로 구분한다.
raw 수치는 사람의 판정 정확도가 아닌 내용 실루엣 자동 비교 보조값이며,
수식·기호·줄 끝은 직접 확대 판독했다. 59043의 기존 미해결 8쪽은 이 결과와 섞지 않는다.
게시 전에 merge SHA raw 이미지의 형식·해시를 검증하고 게시 뒤 본문의 한글·링크·이미지 표시를 확인한다.

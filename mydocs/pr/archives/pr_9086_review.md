# PR #9086 검토

## 최종 판정

**머지 보류 — 통합 후보의 최종 검증 진행 중.**

작성: 2026-11-16 18:09 KST. 원 PR의 주장과 적용 경계를 검토하고, 통합 후보의 Native/fresh WASM 전쪽·보호 문서·필수 Rust 게이트 완료 뒤 최종 판정을 갱신한다. 이 문서의 초기 수치를 최종 제품 통과로 사용하지 않는다.

## 접수와 통합 출처

- 원 PR: [#9086](https://github.com/edwardkim/rhwp/pull/9086), 작성자 `planet6897`.
- 원 head: `df9e88aaed2a1ded5b6003988924033df4d9e7e9`. 원격 contributor 브랜치는 변경하지 않았다.
- 관련 이슈: [#9085](https://github.com/edwardkim/rhwp/issues/9085) (`closes`).
- 원 PR reviewer `jangster77` 지정·API 재확인 완료. 통합 PR owner 자동 지정 없음.
- 고정 base: `ae9277dda0ca17be279cba89c4deea74fb75dcae`.
- 검토 브랜치: `review/integration-20261116-planet6897`.
- [고유 체리픽 8개·중복 제외·충돌 검사](../assets/planet6897_20261116/intake-provenance.json).
- 기본 경로: collaborator_external_pr의 devel 기반 통합 검토. 보조: intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge.

## 주장과 검토 범위

fix(composer): 저장 줄 없는 문단 「한 줄로 입력」은 줄을 나누지 않고 자간을 좁힌다 (#9085)

#9080은 #9078 위에 쌓였으며 공유 커밋 두 개는 한 번만 적용했다. 원문 속성·생산자·소비 경로와 비적용 조건은 [선행 분석](../assets/planet6897_20261116/intake-analysis.md)에 연결한다.

## 입력과 독립 기준

기준은 원 PR의 **데스크톱 한컴 2022(12.0.0.4547) Print → Hancom PDF** 첨부본이다. `pdfinfo`는 Hancom PDF 1.3.0.547·PDF1.4를 기록한다. 웹한글기안기 출력물을 기준으로 사용하지 않았다. 유효한 직접 Print 첨부본을 재사용하며 저장 제품 연도를 출력 제품으로 바꾸어 쓰지 않는다. [입력·PDF SHA-1/SHA-256·저장 제품·생성 정보](../assets/planet6897_20261116/claim-input-audit.json).

| 입력 | Print 쪽수 | 초기 통합 Native 전쪽 raw | TSV |
| --- | --- | --- | --- |
| [facility_fee_reduction_criteria.hwpx](../../../samples/issue9085/facility_fee_reduction_criteria.hwpx) | 1 | 99.82 | [원문](../assets/planet6897_20261116/initial-native/facility_fee_reduction_criteria.tsv) |

위 표의 제품 소스는 **보정 전 `c72c16d88efc615c592b8640320899ed31e28c0e`**이고, canonical Visual Sweep·96DPI·실제 Native Skia·Print·2px raw의 일반 Mac 글꼴 환경 결과다. 최신 보정 제품의 재출력은 별도 기록한다.

## 메인터너 보정과 필수 게이트

- 최소 글꼴: CoreText 일반1890 face·격리528 face(OS만), 사용자·공유 비공개1218 파일의 읽기 차단을 확인했다. 개인 글꼴을 삭제·비활성화하지 않았다.
- 초기 원 PR 집중 회귀: 실제8PASS, 선택 밖992개. 전체 회귀 완료를 의미하지 않는다.
- 초기 전체 검사는 임시 공개 fixture 글꼴을 생성할 TMPDIR 설정 누락으로 중단했다. 기대값·제품 회귀로 분류하지 않으며, 기존 프로필의 허용 임시 경로를 지정해 새 소스에서 재실행한다.
- 공통 CJK 결측 전진에서 원 Print1em/현재0.5em 차이를 확인해 제품 소스 `10d0d6ae43273430f74f7055c39b3d74948c1d7f`에 보정했다. [수정 전 독립 원점 대조](../assets/planet6897_20261116/cjk-advance-before.json)·[Windows 원본 글꼴 공급 확인](../assets/planet6897_20261116/windows-probe-font-transfer.json). 비공개 글꼴 파일은 Git에 넣지 않았다.
- 최종 fresh WASM·Native 전쪽·새 관계형 회귀·전체 nextest·Native Skia3종·fmt/Clippy/workspace·실제 출력 소비·반복 저장 검증은 진행 중이다.
- 정본 보호18문서·각1771쪽의 공식 두 자리 값 하락과 누락은 허용하지 않는다. 필요 시 실행한 환경과 생산자, 실제 전쪽 TSV를 연결하고 수용한 상승만 보호 기준에 반영한다.

## 조판 원칙과 직접 판독

사양 속성과 원본 저장값·독립 Print를 기준으로 검토한다. 문서명·쪽수 예외, 임의 px 이동, 실제 결함을 숨기는 검사·기준 완화는 추가하지 않는다. 최종 source에서 대표 raw/PDF/review/overlay를 직접 판독한 뒤 경계·간격·누락·중복 판정을 보완한다. 현재는 최종 수용 판정을 내리지 않았다.

## Merge 후 contributor PR comment 계획

통합 PR의 정확한 head CI·정상 merge·devel의 정식 증적 존재를 확인한 뒤, 이 PR의 입력·주장 페이지에 대응하는 raw/review/overlay를 merge SHA로 고정해 이미지로 게시한다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 실제 수치·검증 범위·필요한 보정 이유를 한국어 존댓말로 적고 API 본문을 재확인한다. 최종 자료가 준비되기 전에는 원 PR을 승인·close하지 않는다.

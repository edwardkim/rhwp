# PR #8605 누적 검토 구현 계획

## 대상과 단계

[원 PR #8605](https://github.com/edwardkim/rhwp/pull/8605)의 원 head는 `5d37286267136c9978d7ac2f804913730ab6becf`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `4f8570309e925468552036f4c20d099011d55a35` | `65120f6a9c231707369f46a93ca248390980181f` | fix(render): WASM Canvas 합성 굵기를 실제 face 윤곽의 fill+stroke로 그린다 (#8430) |
| `3d0b4420dbee220253189fc149c48125cc0e2dc4` | `18da52e820da49317d6b188522fca23c7c507e1f` | docs(#8430): 30396 권고문 Native/fresh WASM 전쪽 TSV·확대 판독·글꼴 예외 증적 |
| `5d37286267136c9978d7ac2f804913730ab6becf` | `96fffe4e4397079546d8284c7bec14e005952e76` | docs(#8430): 최신 devel rebase head 재측정과 글꼴 예외 source SHA 갱신 |

## R42 리베이스 후 적용 상태

새 base `1e16a19aede5ccb2657d35974940eb596286cdf1`, 리베이스 직후 head
`896708a4a02c4138064ab3d426a487bbce52fc02` 기준으로 위 실제 적용 SHA를 갱신했다.
중복 patch 행의 원 기여 commit 참조는 유지한다. [고유 56개 적용표](../assets/planet6897_20261106/contributor-commit-map-r42.tsv)와
[리베이스·충돌 처리 보고서](../assets/planet6897_20261106/rebase-r42-analysis.md)를 따른다.
원 PR head와 입력은 보존됐으며, 이력 갱신을 최종 검증 통과나 승인으로 표시하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.


## R34 실제 검증 완료 범위

원 HFT 대응 OTF의 CFF 굵기 경로를 실제 최적화 WASM에서 확인했다. 원래 TTF 합성 굵기와
새 CFF 경로, Screen·옛한글 대조군은 3개 집중 브라우저 검사에서 통과했고, 기존 CI 집계에
연결한 실제 Canvas 글꼴 검사 16건도 통과했다. 새로운 원 CFF 검사는 소유 fixture를 명시적인
source family에 연결하고 실제 공개 편집 API로 굵기를 설정한다. SVG의 해당 글자 stroke로
합성 굵기 경계에 진입했는지 확인하며, 페이지 테두리의 stroke를 근거로 삼지 않는다.

추가 보호 대조군인 sample16 HWPX는 같은 독립 2024 Print와 Native/fresh WASM 모두 64쪽이
일치한다. 전쪽 최저는 Native 99.99% / WASM 99.99%이고, 기존 기준 64행의 표시값 하락은 0행이다.
지적한 40·51쪽은 양쪽 100.00%다. 이는 원 PR의 다른 문서 주장에 대한 증거와 구분한다.
[전체 TSV·같은 쪽 review/overlay·재현 조건](../assets/planet6897_20261106/protected-r34/sample16-hwpx/README.md).

제품 source는 99f1340628이며 이후 검사 assertion과 문서를 보완한 commit은 실제 제품 실행과
구분한다. 전체 27개 PR 최종 판정은 남은 전체 WASM·Rust·lint 완료 후 갱신한다.


## R35·R36 원본 유형과 실제 CFF 프로그램의 불일치

2022 HWP의 51쪽 하락을 같은 공개 API와 독립 PDF로 재현했다. HWPX 저장 타입만으로
원본을 TTF 프로그램이라고 판단하지 않는다. 기존 `f70a8c937d3`은 HWP5 미정의 비트로
TTF를 확정하지 않는 정상 정책이며 유지했다. 실제 등록된 CFF를 소비하고 원 TTF 별칭의
글리프 소유를 우선 보존하도록 공통 paint 경계를 보정했다.
[원 PR·통합 커밋·원본 바이트·독립 PDF 윤곽·수정 전 실패와 반대 대조군](../assets/planet6897_20261106/cff-program-selection-analysis.md).

R36 제품 `3f2a7c96ee`의 실제 2022 HWP 51쪽은 **100.00%**이며 이전 99.90% 하락을
복구했다. [같은 쪽의 TSV·review/overlay·직접 판독](../assets/planet6897_20261106/protected-r36/sample16-hwp2022/README.md).
실제 HFT/TTF/미확정 CFF 소비, 실제 TTF 별칭, Screen·옛한글을 포함한 집중 브라우저
3건이 통과했다. 원 PR의 30396 권고문과는 구분한 추가 보호 대조군이다.
전체 27개 PR의 전쪽 검증·최종 Rust·lint와 2024 대조군은 실제 완료 뒤 갱신한다.

## R36 세 sample16 보호 입력의 전쪽 확인

2022 HWP·2024 HWP·HWPX의 각 64쪽을 실제 R36 fresh WASM으로 완료했다. 전쪽 최저값은
100.00%·99.99%·99.99%, 기존 기준 192행의 표시값 하락은 0행이다.
[입력별 Native/fresh WASM 전쪽 TSV·해시](../assets/planet6897_20261106/protected-r36/sample16-full-provenance.tsv)를
따른다. 원 PR 주장 문서의 검증과 통합 최종 회귀·lint는 여전히 별도 완료 조건이다.

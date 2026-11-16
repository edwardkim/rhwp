# PR #9022 리뷰 — 칸 안 그림 뒤 글과 행 높이

## 최종 판정

**머지 보류 — 통합 후보 검증 진행 중.** 원 head의 두 그림 호스트 반례를 메인터너 보정했으며
관련 전쪽 Native/fresh WASM, 기존 행 높이·쪽 나눔 계약과 보호 문서 검증을 마쳐야 한다.

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9022), 작성자 `hyunmyungJaneLee`.
- 원 head: `4cf97fe5f40bde52b6e80dfc461b5111b51139b6`.
- base: `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`, milestone `v1.0.0`의 API 확인을 마쳤다.
- #8695·#9008의 저장 줄 없는 칸, 어울림 그림, 결재 도장, 사진 행 높이·분할 범위를 검토한다.

## 메인터너 보정과 현재 근거

원 head는 새 작은 그림 host가 앞 큰 그림 상태를 덮어쓰는 반례를 남겼다.
같은 반례의 base·호환 보정 후보 Native는 모두 60.20161%였으므로 새 회귀로 단정하지 않는다.
`dadfc954fef51162efda8732505ec4d42f32a5ec`는 선행 그림 합집합과 밀린 빈 줄 전진을
측정·배치의 공용 상태에 보존한다. 문서별 특례를 넣지 않았다.
[원인·독립 Print 출처](../assets/all_open_prs_20261116/picture-host-cause.md)를 따른다.

보정 후 Native는 두 host 반례와 원 PR의 그림 칸 5종, 총 6쪽에서 각각 100.00%다.
이는 아직 fresh WASM 또는 사진 행·결재 원본 전부의 수용 결과가 아니다.
원본 HWP/HWPX와 신규 Print PDF의 전체 쪽수·내용 순서·겹침·칸 포함·누락/중복을 확인한다.
TAC float 회피와 #8939 저장 줄 높이 보존의 충돌 해소 내용은
[접수·충돌 분석](../assets/all_open_prs_20261116/intake-analysis.md)에 기록했다.

## 조판 원칙과 검증 입력

소스 모델을 문서명·특정 쪽에 따라 바꾸지 않고 공통 측정·실제 paint가 같은 원점과 전진을
소비하는지 검토한다. 관련 회귀는 최소 글꼴 환경에서, 시각 비교는 원본 글꼴 환경에서 실행한다.
원 PR의 **웹한글기안기 PDF는 공식 대조 자료로 사용하지 않는다.** 실제 한컴 2020·2024
데스크톱 Print를 같은 입력으로 확보하며 제품·engine·SHA·영수증을 구분한다.

## Merge 후 contributor PR comment 계획

최종 수용 후 통합 PR·merge SHA·정확한 CI를 연결하고 원 기여와 호스트 반례 보정 이유를
한국어 존댓말로 설명한다. 이 PR의 각 주장에 대응하는 같은 페이지 raw·review·overlay 이미지를
merge SHA 고정 URL로 본문에 표시하고, 게시 후 API와 화면을 확인한다.
#8695·#9008의 실제 해결 범위를 대조한 뒤 완전 해결된 이슈와 원 PR을 종료한다.


## 독립 Print 보정 경계의 최종 제품 집중 검증

제품 source `167921b3c5`에서 두 그림 호스트 원본과 그림 하나를 제거한 대조군은
Native/fresh WASM 모두 100.00%다. 제목 뒤 사진 행 원본도 전 3쪽 양쪽 100.00%다.
기존·새 그림 호스트 회귀 7건과 사진 행 회귀 3건은 최소 글꼴 환경에서 모두 통과했다.
[호스트 raw](../assets/all_open_prs_20261116/frame-boundary-validation/two-hosts/native/rhwp_001.png)·
[review](../assets/all_open_prs_20261116/frame-boundary-validation/two-hosts/native/review_001.png)·
[overlay](../assets/all_open_prs_20261116/frame-boundary-validation/two-hosts/native/overlay_001.png),
[사진 행 원인과 결과](../assets/all_open_prs_20261116/title-photo-cause.md)를 따른다.
이 집중 결과를 전체 66종·보호 문서·전체 회귀 완료로 확대하지 않는다.


## 통합 후보 최종 소스의 중간 검증 결과

제품 소스 `167921b3c5`와 검증 head `21950b8ab8`에서 최소 글꼴 macOS 전체 회귀는
**13,696 PASS·0 FAIL·45 skip**이다. Native Skia·세 Clippy·workspace build·fmt·
suite 정책·unit tier·문서 테스트를 포함한 [Rust 13단계](../assets/all_open_prs_20261116/rust-gates-final.json)는 통과했다.
통합 검토의 공식 데스크톱 Print 66종·74쪽은
[Native](../assets/all_open_prs_20261116/native-claims-final.json)·
[fresh WASM](../assets/all_open_prs_20261116/wasm-claims-final.json) 각각 전쪽 90% 이상이다.
이 집계는 다른 PR의 입력을 이 PR만의 개선으로 계산한 값이 아니다. PR별 주장·반례는 위 접수 범위와
개별 보정 증거로 판정한다. 정본 보호 전쪽·실제 소비 경로와 최종 base/head 대조가 남아 있어
전체 PR 검토 완료나 머지 승인으로 기록하지 않는다.

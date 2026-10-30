# PR #8276 리뷰 — 고정 격자 타일 API

## 최종 판정

**메인터너 보정 후 수용 가능 — 고정 격자 타일 API 범위의 로컬 검증 완료.**
최신 devel에 대한 보정과 필수 검증을 완료했고 [통합 PR #8394](https://github.com/edwardkim/rhwp/pull/8394)로 merge했다.
merge `7d4741392bb2ffaf308f5ce1cf6ba7f4deeeb1c9`. 원 기여 커밋과 저자를 보존했다.

## 접수와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/8276), 작성자 seo-rii, reviewer edwardkim, non-Draft.
- 원 head: `06bf9c11002879934831f5c2276fe2b267cc8be0`. base: devel.
- 고정 최신 devel: `96ccd557592a711f215e7ad28822f4c7a3f00c15`.
- 통합 제품 소스: `c3f5cbd6d7beecdaa231cd3a40080f2e7f3c8c1e`.
- 512px 격자와 2px guard로 큰 페이지의 부분 영역을 그린다. 전체 페이지 exact crop와 별도 raster 계약이며 기존 API를 대체하지 않는다.
- 원 stack은 #8270 → #8275 → #8276 → #8279이며 최종 head merge로 모든 기여자 commit을 보존했다.
- [수행 계획](seo_rii_20261030_review_impl.md), [공통 근거](../assets/seo_rii_20261030/README.md).

## 메인터너 보정

사용자가 문제를 메인테이너 쪽에서 해결하도록 승인했다. CI workflow·classifier·검사 세 파일의
충돌을 양쪽 Native 검사를 유지하는 방식으로 해결했다. 기존 fork 정책 실패는
`Untrusted head changed the enforcement surface.`이며, 정책을 완화하지 않고 검증된
동일 저장소 통합 PR로 처리할 계획이다. 테스트 첫머리에는 합성 계약과 실제 시각 증거의
구분·직전 commit을 보강했다. 제품 로직과 assertion을 임의로 바꾸지 않았다.

## 검증과 다음 조건

원 head CI와 메인터너 후보의 로컬 실행은 분리한다. 실제 명령·입력·TSV·PNG와 완료 범위는
[공통 근거](../assets/seo_rii_20261030/README.md)에 기록한다. 필수 로컬 검증은 완료했으며 원격 push·통합 PR
생성·통합 승인 범위를 확인한다. 이번 지시는 로컬 보정을 승인한 것이며 아직 새 원격 조치는 수행하지 않았다.


## 완료 결과와 통합 방침

- 전체 회귀 12,198 PASS, Native 집중 81 PASS, Native lib 4,040 PASS, placeholder 2 PASS.
- fmt·4종 Clippy·workspace build·manifest 및 Node/Python CI 계약 각 95건 통과.
- Native/fresh WASM 각각 388쪽의 전후 PNG가 byte-identical. 하락·90% 미만·누락 0쪽,
  편람 각 383쪽 최저 99.96%. 구체적인 원본·PDF·폰트·producer·TSV·확대 판독은 공통 근거에 있다.
- 실제 캐시 왕복은 replay 3→2, hit 0→1. 타일 overlap·allocation 계약 통과.
  편람 63·289쪽 Direct PDF는 기존 거부에서 1쪽 PDF 생성으로 개선됐으며 독립 Print raw 100.00%.
- 문단·줄 구성·페이지 분할은 변경하지 않는다. 지원을 확대한 PDF 경로와 기존 Native/WASM
  출력 보존을 구분했다. 원 기여 commit은 통합 후보의 조상으로 모두 보존했다.

승인 뒤 통합 브랜치를 같은 저장소에 push하고 Open PR로 게시한다. 증적 raw hash 확인,
새 head의 CI·self-review·최신 base 충돌 검사를 거쳐 허가된 범위로 merge한다.
원 PR의 통합 상태와 이력을 확인하고 최종 merge SHA·CI·시각 자료 링크를 포함한 한국어
후속 comment를 준비한다. #536 전체 Native 추적 범위와 편람 전체 Direct PDF 지원을
완료한 것으로 닫지 않는다.

## 최종 CI·self-review·통합 확인

- 제출 head `d5a7d933f83550e89aabc3b08795724873051f34`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41081855120), CodeQL·Render Diff·Proptest·Adapter와 CI 정책이 성공했다.
- self-review에서 LRU 전체 예산·키·무효화, tile origin/guard/offset, PDF 공통 geometry와 admission, CI 기존 검사 보존을 확인했으며 통합을 막는 결함을 발견하지 못했다. 공식 GitHub APPROVE 이벤트는 제출하지 않았다.
- 사용자 승인 범위에서 merge commit `7d4741392bb2ffaf308f5ce1cf6ba7f4deeeb1c9`로 원 기여 이력을 통합했다. 최신 base 충돌 검사를 통과했고 실제 통합 head의 CI를 사용했다.
- 위 원격 승인 전·계획 문단은 검토 당시 기록이다. 최종 상태는 이 절과 첫 판정을 따른다. #536·#7501은 Refs 범위로 유지하며 임의 close하지 않는다.

## Merge 후 contributor PR comment 계획

- 원 기여의 고정 격자 tile API를 인정하고 최신 devel CI 충돌을 메인테이너가 보정한 이유, 통합 PR/merge SHA, 실제 CI·로컬 결과를 게시한다.
- Native/fresh WASM 각 388쪽 전후 byte-identical·편람 최저 각 99.96%, 63/289쪽 Direct PDF raw 100.00%와 지원 범위를 구분한다.
- merge SHA의 편람 p63 Native/fresh WASM review와 overlay를 표시하고 Visual Sweep 정본 링크, 획 굵기·AA 잔차를 함께 기록한다. 원본 fork branch는 삭제하지 않는다.

# PR #8279 리뷰 — Direct PDF 연결선·화살촉 허용

## 최종 판정

**메인터너 보정 후 수용 가능 — #7501 연결선·화살촉 PDF 범위의 로컬 검증 완료.**
최신 devel에 대한 보정과 필수 검증을 완료했다. 원격 push·통합 PR 생성·최종 head CI·merge는
아직 수행하지 않았으며 작업지시자의 해당 원격 범위 승인이 다음 조건이다.

## 접수와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/8279), 작성자 seo-rii, reviewer edwardkim, non-Draft.
- 원 head: `19f332edc9258467cb991f4196c53119f357aeb8`. base: devel.
- 고정 최신 devel: `96ccd557592a711f215e7ad28822f4c7a3f00c15`.
- 통합 제품 소스: `c3f5cbd6d7beecdaa231cd3a40080f2e7f3c8c1e`.
- 검증된 공통 arrow geometry와 Native replay를 Direct PDF에 허용한다. 잘못된 몸통·끝점·접선·f32 outline은 거부한다.
- 원 stack은 #8270 → #8275 → #8276 → #8279이며 최종 head merge로 모든 기여자 commit을 보존했다.
- [수행 계획](seo_rii_20261030_review_impl.md), [공통 근거](assets/seo_rii_20261030/README.md).

## 메인터너 보정

사용자가 문제를 메인테이너 쪽에서 해결하도록 승인했다. CI workflow·classifier·검사 세 파일의
충돌을 양쪽 Native 검사를 유지하는 방식으로 해결했다. 기존 fork 정책 실패는
`Untrusted head changed the enforcement surface.`이며, 정책을 완화하지 않고 검증된
동일 저장소 통합 PR로 처리할 계획이다. 테스트 첫머리에는 합성 계약과 실제 시각 증거의
구분·직전 commit을 보강했다. 제품 로직과 assertion을 임의로 바꾸지 않았다.

## 검증과 다음 조건

원 head CI와 메인터너 후보의 로컬 실행은 분리한다. 실제 명령·입력·TSV·PNG와 완료 범위는
[공통 근거](assets/seo_rii_20261030/README.md)에 기록한다. 필수 로컬 검증은 완료했으며 원격 push·통합 PR
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

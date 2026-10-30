# PR #8279 리뷰 — Direct PDF 연결선·화살촉 허용

## 최종 판정

**머지 보류 — 메인터너 통합 후보의 로컬 검증 진행 중.** baseline 대조, fresh WASM,
전체 회귀·lint를 완료한 뒤 판정한다. 원 PR의 실패한 CI 정책을 우회하거나 수용 완료로 기록하지 않는다.

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
[공통 근거](assets/seo_rii_20261030/README.md)에 기록한다. 필수 검증이 끝나면 원격 push·통합 PR
생성 승인 범위를 확인한다. 이번 지시는 로컬 보정을 승인한 것이며 아직 새 원격 조치는 수행하지 않았다.

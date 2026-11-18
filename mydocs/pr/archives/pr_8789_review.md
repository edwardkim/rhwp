# PR #8789 검토 — HWP3 사적 문장부호 8코드 보존

## 최종 판정

**승인 — 검증한 통합 후보와 명시적으로 승인받은 기존 차이의 예외 범위에 한정한다.** source 개선과 필수 로컬 검증을 완료했다. 2026-11-09 KST 메인테이너는 SO-SUEOP의 기존 조판 차이와 Native 긴 줄표 fallback 간격에 한정한 수용 예외 및 두 PR의 원격 처리를 승인했다. 원문 문자 보존 개선을 수용하며 문서 전체의 표시 문제가 해결됐다고 판정하지 않는다. 새 통합 head CI·self-review 성공이 머지 전 조건이다.

## 접수와 검토 경로

- [원 PR #8789](https://github.com/edwardkim/rhwp/pull/8789), chrisryugj, CONTRIBUTOR, devel 대상 Ready. 첫 외부 기여자 경로는 비해당이다.
- reviewer edwardkim 지정·API 재확인 뒤 상세 검토·fetch를 진행했다. 원 head `3c2c5fdf1bea13912b2eefaf4008ea974a2b4fd9`와 fork branch를 보존했다. 원 head CI 36개는 모두 성공이며 이번 새 보정 head의 CI 성공으로 대체하지 않는다.
- 실제 검증 기준은 최신 devel `b768bee1c4c0023087687f79a0a95f12108743e9`다. 개별/누적 merge가 충돌하지 않았고 2026-11-09 API 재조회에서 devel과 원 head 각각의 SHA가 접수 때와 같았다. GitHub 자동 종결 연결은 없다. 실제 관련 이슈는 [원격 후속 점검](../assets/chrisryugj-20261109/remote-completion.md#관련-이슈-종료-여부-추가-점검)에 별도로 기록했으며 #7807·#7445는 미완료로 OPEN을 유지한다.
- [공통 계획](chrisryugj_20261109_review_plan.md), [승인받은 보정안과 실제 결과](chrisryugj_20261109_review_impl.md), [전쪽 TSV·producer·검증 원장](../assets/chrisryugj-20261109/README.md). 원 기여는 merge commit으로 보존했다.

## 최신 base에서도 남은 원 문제와 원 해결 이력

#5860의 #5869 → #5889/`72674c5653f09cb78b994dc4cd2dfd0a97ae6c8a`, #6380의 #6397/`ce8015138f`·`c78b075aee`, #6057의 #6065 → #6072/`ee7e8a6ed0c4784bea882a30dc94de49c42f6bad`, #8191/`6097253468`과 현재 source를 대조했다. #6057의 반각 PUA 하위 근거는 유효하며 원 글꼴의 1/2 em 책괄호 계약을 유지한다. #7642의 최종 보정 `7f76cee60e38fceb63feacaff706cf984b19c427`은 이 문서의 모든 문자·fidelity 문제가 해결됐다는 뜻이 아니다.

현재 base의 실제 decode probe에서 8코드가 U+003F로 남고 HWP3 parser가 해당 문자를 버린다. 원 기여에서는 독립 한컴 변환본과 같은 Unicode/PUA가 반환된다. 새 문자 의미를 만들어내거나 화면에 맞춰 다른 기호로 치환하는 변경이 아니다.

## 실제 본문·각주·저장 소스 대조

| 대상 | base의 실제 source 개수 | 후보 / 독립 한컴 변환본 |
| --- | ---: | ---: |
| “ / ” | 18 / 18 | 31 / 32 |
| ‧ | 0 | 4 |
| — | 0 | 1 |
| ∣ | 0 | 6 |
| ↳ | 0 | 18 |
| F0854 / F0855 반각 책괄호 | 0 / 0 | 3 / 3 |

다른 encoding의 따옴표 18개가 base에도 존재한다는 사실과 새 8코드의 미지원은 구분한다. Native parser → HWPX 저장, 최적화 fresh WASM → 실제 Chromium의 HWPX 저장 모두 독립 `samples/SO-SUEOP.hwpx`와 일치했다. XML 혼합 내용의 tail·각주까지 한 번씩 세어 검증했다. 최소 글꼴 환경의 집중 검사 3건도 통과했다.

## 시각 검증과 남은 범위

독립 `pdf/SO-SUEOP-hwp-2020.pdf`, compat 2022·96 DPI·print에서 전 46쪽을 양쪽 backend로 비교했다. 최저 raw는 Native 61.69502→61.69802%, fresh WASM 61.97765→61.97235%다. 최대 하락은 각각 0.31024%p·0.27353%p이며 쪽수는 모두 46쪽이다. 기존 90% 미달은 해소되지 않았다.

6·10·13·23·25·29·35·45쪽의 기호·간격을 직접 확대했다. 화살표·선·따옴표·중간점·책괄호의 누락이 복구됐다. Native 긴 줄표의 잉크가 뒤 글자와 겹치는 한계도 관측했다. **기존 독립 한컴 HWPX를 수정 전 base Native renderer로 열어도 같은 문제가 재현**된다. 이 경로는 새 HWP3 decode를 사용하지 않으므로 PR의 잘못된 Unicode 매핑으로 분류하지 않는다.

[표시 차이와 독립 base 대조](../assets/chrisryugj-20261109/so-sueop-visual-hold.md), [긴 줄표](../assets/chrisryugj-20261109/symbols-p23.png), [독립 HWPX의 base renderer](../assets/chrisryugj-20261109/base-independent-dash-control.png), [기존 열 흐름](../assets/chrisryugj-20261109/legacy-p45.png). 실제 glyph cmap과 원/대체 프로그램을 공급해 본 결과를 기록했으며 자동 글꼴 예외를 선언하지 않았다.

## 원격 처리와 댓글 계획

메인테이너가 2026-11-09 KST “기존 차이 한정 예외·2건 원격 처리 승인”으로 결정했다. 범위는 통합 branch push·Open PR → 새 head CI 성공·self-review·merge → 원 PR 2건 완료 반영·감사 댓글·mydocs 운영 기록 devel push·동기화·정리다. 새 회귀·누락·필수 검사 실패는 예외가 아니다. 원 기여 SHA를 보존하고 source 복원 사실과 남은 표시 범위를 원 PR에 분명히 알린다. [원격 처리안](../assets/chrisryugj-20261109/remote-action-plan.md)과 [개별 댓글 초안](../assets/chrisryugj-20261109/merge-comment-drafts.md)을 따른다. 실제 원격 결과는 후속 기록에서 확정한다.

## 공통 검토와 필수 로컬 검사

조판 원칙 준수 검토는 적용 대상이다. 이번 변경은 HWP3 source 문자 해석을 복원하며 새 좌표·높이·분할 예외를 도입하지 않는다. parser → IR의 실제 문자 → Native/WASM HWPX 저장 및 표시를 대조했다. 새로운 px/golden 기대값으로 기존 미달을 고정하지 않았다. 원본 HWP·독립 Print PDF·독립 변환 HWPX는 검토 commit에 이미 추적된 파일을 재사용하고 실제 SHA-256과 전쪽 증거를 [원장](../assets/chrisryugj-20261109/README.md)에 기록했다.

최종 테스트 source `581d22855f60af2de5fed3208949f7dd81258a0a`에서 전체 회귀 13,013 PASS·Native 공식 4,077 PASS·집중 8 PASS와 적용 Rust lint/build/suite 정책이 통과했다. Native 편람 383쪽도 최저 99.96%·기준 대비 회귀 0으로 통과했다. 제품 source는 실제 시각 producer와 같으며 이후 변경은 테스트·문서 보강이다. 기존 원 head의 녹색 CI와 새 통합 후보의 로컬 검증을 구분하고, 원격 승인 후 새 head CI·self-review를 거쳐야 한다.

## 원격 확정

2026-11-09 KST, 통합 [PR #8851](https://github.com/edwardkim/rhwp/pull/8851)의 head `9fa72c60365f6b1f3f168223e7ac56d2fa4b6402`에서 CI·self-review를 완료하고 merge `50b6098853afdb45625e5d505dc3b2070998fc46`으로 통합했다. 원 PR #8789·#8791은 모두 자동 MERGED로 확인했다. [통합 최종 검토](pr_8851_review.md)와 [원격 확정·후속 게이트](../assets/chrisryugj-20261109/remote-completion.md)를 따른다. 원 fork branch는 삭제하지 않는다.

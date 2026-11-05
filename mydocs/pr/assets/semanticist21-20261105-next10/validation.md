# 다음 10건의 누적 검증 기록

**최신 devel #8622 반영 후의 최종 검증을 진행 중이다. PR 제출·수용 완료가 아니다.**

대상은 #8368·#8369·#8385·#8484·#8488·#8490·#8492·#8494·#8500·#8504다. 원 head 10개를 merge 조상으로 보존했으며 contributor branch에 push하지 않았다. 현재 승인 범위는 로컬 보정과 필수 검증이다.

## 완료해 보존한 이전 source

제품 source `6337fe4d102b591b65c61b92a8da8b2df71eb8cb`, base `ed057276c1e0e07e743218d95641e015b46b6715`(중간 d4a72efd7c와 제품 tree 같음).

- 문서별 글꼴 한도 안의 실제 Native/fresh WASM: 각각 46상태·124쪽, 최저 96.49% / 97.10%, 90% 미만·누락·독립 PDF 쪽수 불일치·2%p 초과 하락 없음.
- 최소 글꼴 전체 nextest 12,544 PASS·45 SKIP·exit 0. 실제 Native 공식 3종 4,064 PASS·12 IGNORE, 모든 Rust lint·workspace build, Studio TypeScript·2,720 tests·build 통과.
- 최적화 WASM의 실제 API 9묶음과 한컴 계산 기대값 37개, 실제 Studio Canvas2D 6장면, CanvasKit의 문자 획·공통 테두리/각주 소비 검사를 별도로 실행했다.
- Canonical KoPub/HCR 편람 Native 383쪽 passed·최저 99.96%다. 일반 샘플 글꼴 팩을 공급한 초기 실패와 구분한다.
- 검증 입력 오류를 보정한 보안 suite 6개 및 fixture 45개 × CLI 세 탐지기 검사를 완료했다. `samples/`의 신규/변경 문서는 없다.

[실제 실행 기록](pre8622-validation.md), [전쪽 TSV·입력/PDF 해시](validation/pre8622/summary.json), [확대 판독과 대표 이미지](validation/pre8622/direct-reading.md), [글꼴 공급 보정](font-qualification.md), [원형 점선 독립 측정](circle-print-geometry.md)을 연결한다.

## 새 base의 검증

원격 devel에 [#8622](https://github.com/edwardkim/rhwp/pull/8622), `6e1de406209ba9a58032f6ca6de45b18aa687d1c`가 추가됐다. 이 변경은 제품 코드가 같지 않으므로 위 결과를 새 후보의 결과로 재사용하지 않는다. [최신 base 기능 보존 계획](rebase8622-plan.md)에 따라 별도 base 산출물·통합 후보의 실제 출력과 동작을 대조하고 최종 필수 검증을 완료한다.

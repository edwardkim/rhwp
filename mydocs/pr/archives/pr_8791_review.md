# PR #8791 검토 — HWP3 표의 시작 변 격자와 이웃 탐색

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 head의 셀 누락을 메인테이너 보정 `4fabb47ae28b99d371e2aac3560e2610e327c311`으로 해결했고 필수 로컬 검증을 완료했다. 원 기여의 네 표 개선과 실제 셀 내용 보존을 확인했다. 보정 없는 원 head를 그대로 승인하는 판정은 아니다. 원격 승인·새 통합 head CI·self-review가 머지 전 조건이며, 누적 후보의 #8789 시각 차이 결정도 별도로 필요하다.

## 접수와 검토 경로

- [원 PR #8791](https://github.com/edwardkim/rhwp/pull/8791), chrisryugj, CONTRIBUTOR, devel 대상 Ready. 첫 외부 기여자 경로는 비해당이다.
- reviewer edwardkim 지정·API 재확인 뒤 상세 검토·fetch를 진행했다. 원 head `383f66acca8c966da9a198fd046c3da0ed94c3b6`와 fork branch를 보존했다. 원 head CI 36개는 모두 성공이며 이번 새 보정 head의 CI 성공으로 대체하지 않는다.
- 실제 검증 기준은 최신 devel `b768bee1c4c0023087687f79a0a95f12108743e9`다. 개별/누적 merge가 충돌하지 않았고 2026-11-09 API 재조회에서 devel과 원 head 각각의 SHA가 접수 때와 같았다. GitHub 자동 종결 연결은 없다. 실제 관련 이슈는 [원격 후속 점검](../assets/chrisryugj-20261109/remote-completion.md#관련-이슈-종료-여부-추가-점검)에 별도로 기록했으며 #7807·#7445는 미완료로 OPEN을 유지한다.
- [공통 계획](chrisryugj_20261109_review_plan.md), [승인받은 보정안과 실제 결과](chrisryugj_20261109_review_impl.md), [전쪽 TSV·producer·검증 원장](../assets/chrisryugj-20261109/README.md). 원 기여는 merge commit으로 보존했다.

## 사양·독립 근거·실제 소비 경로

HWP3 10.6의 셀 27-byte 정보에서 위치·크기는 word로 저장되고 sheet serial byte와 구분된다. 원본 cell geometry → 시작 좌표별 격자 → 이웃 좌표에서 row/col/span 계산 → Table IR의 기존 겹침 해소·빈 칸 보정 → layout/저장/paint가 소비한다. 이번 근인은 paint에서 생긴 겹침이 아니라 **parser의 이웃 탐색 결과가 뒤 셀의 내용 소유를 잃게 한 것**이다. 측정 높이나 출력 위치를 clamp하지 않았다.

| 원 대상 | base 행×열 | 기여 / 보정 / 독립 한컴 행×열 | 주소·병합 일치 |
| --- | --- | --- | ---: |
| sample16 접수 | 16×5 | 15×5 | 37/37 |
| sample16 사업책임자 | 23×15 | 23×17 | 234/234 |
| sample16 경력사항 | 15×15 | 15×17 | 69/69 |
| cell-order | 28×12 | 29×12 | 64/64 |

Native·브라우저 WASM에서 실제 저장한 XML을 기존 독립 한컴 변환 XML과 모든 셀 주소·병합 범위로 대조했다. 행열만 맞는 것으로 끝내지 않았다. 보정 후 원 문서의 dump는 원 기여 후보와 동일하다.

## 승인된 메인테이너 보정

원 후보의 2,048개 탐색 제한은 뒤에 실제 이웃이 있는데도 None을 반환했다. 그러면 현재 셀이 표 끝까지 병합되고 기존 겹침 해소가 다음 셀을 버렸다. 실제 Native parser에서 2,049행은 1개, 2,050행은 2개 셀의 내용 누락을 재현했다. [승인·수정 전후 결과](chrisryugj_20261109_review_impl.md).

공통 보정 `4fabb47ae28b99d371e2aac3560e2610e327c311`는 좌표별 반대 축 구간의 합집합을 tree에 색인해 두 축에서 같은 검색을 사용한다. 기존 40 HU 계약은 유지하며 탐색 상한을 단순 제거하거나 높이는 방식은 사용하지 않았다. 구성 O(n log n), 질의 O(log² n), 저장 O(n log n), word 셀 수에서 깊이 최대 16이다.

Native·최적화 WASM의 2·2,048·2,049·2,050행과 전치 열 8개 입력 모두 내용 누락·중복·셀 소속/병합 오염 없이 보존된다. 2,049/2,050행은 4,098/4,100개 셀이 남는다. 합성 parser 계약과 실제 한컴 Print 증거를 구분하며 정상 네 표도 함께 검증했다. 집중 검사 5건이 최소 글꼴 환경에서 통과했다.

## 전쪽 시각·직접 판독

원본 sample16 64쪽·cell-order 1쪽의 Native/fresh WASM 전 65쪽은 모두 raw 90% 이상이다. sample16의 최저는 Native 98.53716→99.57001%, WASM 98.73826→99.62920%, cell-order는 양쪽 100%다. 페이지별 하락은 없다. 세 원 문서 누적 검증 111쪽씩도 별도로 기록했다.

원 네 표의 시작·병합·하단·내용 순서·외곽을 같은 배율로 직접 대조해 새 겹침·누락·중복을 관측하지 않았다. [56쪽 전체](../assets/chrisryugj-20261109/sample16-p056-comparison.png), [cell-order 전체](../assets/chrisryugj-20261109/cell-order-p001-comparison.png), [전쪽 TSV와 판독 기록](../assets/chrisryugj-20261109/README.md). 쪽수·구조·해시만으로 시각 완료를 선언하지 않았다.

편람 Native 383쪽 gate도 최저 99.96%·기준 대비 회귀 0으로 통과했다. 정상 HWPX 경로의 기준이나 다른 PR 기능을 바꾸지 않았다.

## 회귀와 원격 처리

독립 원본의 셀 주소·병합과 대형 입력의 실제 parser 내용 소유를 정식 검사에 보강했다. px 위치·SVG 해시를 새 기대값으로 고정하지 않았고 생성 harness·manifest는 stage하지 않는다. 최종 테스트 source `581d22855f60af2de5fed3208949f7dd81258a0a`에서 전체 회귀 13,013 PASS·Native 공식 4,077 PASS·집중 8 PASS와 모든 적용 Rust lint/build/suite 정책이 통과했다. 제품 source는 실제 Native/fresh WASM producer와 같다. 명령·기존 skip·최소 글꼴 환경은 [검증 원장](../assets/chrisryugj-20261109/README.md)에 기록했다.

후속 원격 수용은 누적 후보에 함께 포함된 #8789의 기존 시각 차이 결정과 함께 제안한다. 원 기여 SHA·fork branch를 보존하며 실제 merge·원 PR 상태·감사 댓글·운영 기록은 승인된 원격 범위에서만 처리한다.

## 조판 원칙과 검증 입력 커밋 확인

적용 대상이다. HWP3 셀의 원본 geometry → 공통 이웃 색인 → row/col/span → Table IR의 기존 겹침 해소 → layout·저장·paint까지 실제 소비 경로를 확인했다. 두 축이 같은 이웃 검색 결과를 소비하며 뒤에서 누락을 숨기는 예외·좌표 clamp를 추가하지 않았다. 이 변경은 parser의 내용 소유 보정이며 새 continuation·pagination·paint 분기를 만들지 않는다. 실제 표 시작·병합·끝과 뒤 내용은 독립 Print 확대 판독으로 확인했다.

수용 근거인 HWP·HWPX·독립 Print PDF는 검토 commit에 이미 추적된 파일을 재사용하며 실제 파일의 SHA-256과 전쪽 수치를 원장에 기록했다. 큰 경계 입력은 테스트 안의 HWP3 사양 기반 생성기로 재현하고, 한컴 생성본의 시각 증거로 표현하지 않는다. 관계형 검사와 원래 표의 독립 XML 계약을 유지했다.

## 원격 확정

2026-11-09 KST, 통합 [PR #8851](https://github.com/edwardkim/rhwp/pull/8851)의 head `9fa72c60365f6b1f3f168223e7ac56d2fa4b6402`에서 CI·self-review를 완료하고 merge `50b6098853afdb45625e5d505dc3b2070998fc46`으로 통합했다. 원 PR #8789·#8791은 모두 자동 MERGED로 확인했다. [통합 최종 검토](pr_8851_review.md)와 [원격 확정·후속 게이트](../assets/chrisryugj-20261109/remote-completion.md)를 따른다. 원 fork branch는 삭제하지 않는다.

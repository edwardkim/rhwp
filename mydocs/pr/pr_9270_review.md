# PR #9270 검토 — feat(clipboard): 수식이 든 셀 문단의 논리 캐럿 복사·붙이기

## 2026-11-21 메인테이너 보완 착수

원 [PR #9270](https://github.com/edwardkim/rhwp/pull/9270)의 head는 `3be4f59be784ca6a174ac88ab0d4224ee3435c0d`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

같은 문자 위치의 인접 수식 사이에서 셀 복사와 붙여넣기의 논리 경계를 확인했다. 새 셀 API 세 개가 공개 선언에서 빠져 있다. 같은 위치의 여러 개체를 논리 순서로 구분하고, 단일 수식 선택과 사이에 붙여넣기 및 타입 소비를 검증한다.

## 메인테이너 보완 검증 중간 결과

같은 논리 캐럿 경계를 셀 HTML 붙여넣기로 실행해 가[수식]X[수식]나를 확인했다. 새 관계 검사가 통과했다. [쪽별 비교 원장](assets/semanticist21-20261121/maintainer-comparison.tsv)

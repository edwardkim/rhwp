Self-review를 완료했습니다. **승인·통합 진행** 판정입니다.

검토 head는 `62a3fe4e2982961f9356a890395dc056eca85570`, 최신 devel은 `1a3ed4a7cd76701085a24ac81d30bc5fb31942c1`입니다. 실제 병합 tree `721ccfdffb245cccd7100e3c5736b735c1784e58`는 로컬에서 추가 검증한 결과와 같습니다. 원 기여 #8982·#8983의 SHA를 모두 ancestor로 보존했습니다.

- 공통 HTML 속성 파서의 이름·따옴표 경계와 기존 숫자 단위·셀 CSS 우선순위, 첫 서식 앞 평문 run, 본문·셀 삽입점 및 반복 저장의 내용·서식 보존을 재검토했습니다. 문서별 수치 보정·backend 예외를 추가하지 않았습니다.
- 로컬 전체 회귀 13,341건·집중 23건·Native 공식 검사 4,073건 및 필수 lint·workspace build를 통과했습니다. Native/fresh 최적화 WASM 각 13쪽 raw 최저 100.00%, 보호 3,542행 공식값 하락 0, 편람 383쪽 gate·Studio 6사례/24개 저장 재열기를 확인했습니다.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/43221597756)의 Archive A–E와 lint·Build & Test, [Adapter](https://github.com/edwardkim/rhwp/actions/runs/43221597713), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/43221598220), 최종 CI Impact Policy가 성공했습니다. 실제 Archive 로그에서 관련 회귀 27건의 PASS도 확인했습니다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43221598571)은 변경 대상인 Rust의 실제 분석이 성공했습니다. 다른 세 언어는 경로 분류에 따른 분석 생략이며, 종합 CodeQL check의 `neutral`·“3 configurations not found” 경고는 그대로 보존합니다. 이를 전체 언어 분석 성공으로 표기하지 않습니다. Native·Frontend의 CI 생략과 미트리거 Render Diff도 직접 수행한 로컬 검증과 구분합니다.
- 최신 devel의 #8956은 표 조회 경로 파싱만 변경합니다. 해당 API 4건은 새 devel에서 통과했고, 후보와의 실제 병합 결과에서는 표 조회·붙여넣기·선행 서식 27건이 모두 통과했습니다. 공유 target의 기준 라이브러리 재사용으로 실패했던 첫 실행은 제외하고 현재 소스 재컴파일 결과로 판정했습니다.

[통합 검토](https://github.com/edwardkim/rhwp/blob/62a3fe4e2982961f9356a890395dc056eca85570/mydocs/pr/bongseop_kim_20261112_review.md)와 [실물·실행 원장](https://github.com/edwardkim/rhwp/blob/62a3fe4e2982961f9356a890395dc056eca85570/mydocs/pr/assets/bongseop-kim-20261112/README.md)을 대조했습니다. 승인 범위의 미해결 필수 항목은 없으며, #7992는 별도 제안으로 유지합니다. 원 commit을 보존하는 merge commit으로 통합합니다.

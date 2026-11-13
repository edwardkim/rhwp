# PR #9042 보완 검증 원장

제품 source `1506a1c34672931c4f5aa5cb6363fba776225949`, 테스트 source
`3eaf74839d8f5e0d9744bb6fe401fc8e3727f1e2`, 기준 devel
`5cbcc9b451cf286e1b7c680426c857dc802e50f3`의 완료 측정이다.
이후 병합된 #9054의 저장·캐럿 기능 통합과 해당 최종 source 재검증은 별도 기록한다.

- 목표 8종 전 16쪽은 Native/fresh 최적화 WASM 모두 raw 100.00%다. 전체 대상 22종 전 40쪽씩은 양쪽 모두 90% 이상이다.
- [전쪽 결과와 명령](completion-page-results.json), [Native base 보존](rebase-preservation-native.json), [보호 3,542쪽](protected/results.json), [편람 383쪽](handbook/gate.json)을 보존한다. 보호 공식값 하락과 기존 source 대비 RGB 차이는 모두 0건이다.
- 공개 CLI·브라우저 무편집 저장 36건, 실제 편집 저장 12건, 두 형식 반복 저장과 내용 보존을 확인했다. [무편집 저장](roundtrip/results.json)·[편집 화면과 저장](live-edit/README.md)을 구분한다. 실제 직렬화 입력은 hash 별로 같은 디렉터리에 커밋했다.
- [로컬 필수 검사](local-gates.json): 전체 회귀 13,464건, Native 공식 4,073건, 집중 28건, fmt·Native/WASM/workspace lint·build·suite check 모두 통과했다. 구성상 제외 45건은 성공 건수에 넣지 않았다.
- [최적화 WASM 빌드](wasm-build.json)는 실제 wrapper와 wasm-opt를 실행하고 Studio 공개 산출물 4개 hash 일치를 확인했다.
- 목표 문서는 양쪽 같은 HCR 프로그램을 공급했다. 초기 Noto 대체 캡처를 완료 근거로 사용하지 않는다. [저장 줄 원인·글꼴 정정](../source-line-cache/README.md)을 따른다.
- 96 DPI·Print·compat 2022(새 인쇄 대조군은 2024)·색상 경계 보정 전 2px raw다. 원본과 BodyX는 384 DPI로 독립 Print/Native/WASM을 실제 재출력해 글자의 하강부·괘선·후속 내용 포함을 직접 판독했다.

원본·독립 PDF·합성/실제 저장 입력 구분은 [상위 원장](../README.md), 중첩 상자는 [독립 한컴 검증](../native-nested-validation/README.md)을 따른다.

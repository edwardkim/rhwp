# PR #9042 최종 보완 검증

상태: **메인테이너 보정 후 수용 가능 — 로컬 필수 검증 완료, 원격 처리 승인 대기.** 예외는 적용하지 않았다.

제품 source `d66d5f05f9efa4d182228494cd970614d479fb7a`, 기준 devel `959300e66c58241a47e6d46030b4b0854e59604b`. 최종 제출 commit은 검토 기록에서 별도로 확인한다. 새 devel의 #9054 저장·캐럿 보정도 포함한다. 이전 `1506a1c346`의 기록으로 이 후보의 검사를 대신하지 않았으며, 아래 결과는 새 source에서 실제 다시 실행했다.

| 범위 | Native | fresh 최적화 WASM |
| --- | --- | --- |
| 목표 8종·16쪽 | 전쪽 raw 100.00% | 전쪽 raw 100.00% |
| 목표·기존·새 base 대조군 22종·40쪽 | 최저 90.01% | 최저 97.08% |
| 실제 BodyX 편집 직후 2쪽 | 전쪽 100.00% | 전쪽 100.00% |
| 보호 18조건·1,771쪽씩 | 공식값 하락 0건 | 공식값 하락 0건 |
| 별도 편람 HWPX 383쪽 gate | 통과 | 비해당: Native gate |

- [대상 전쪽 TSV·명령·입력](completion-page-results.json), [보호 전쪽 TSV](protected/results.json), [별도 편람 gate](handbook/gate.json).
- [Native 통합 전후 PNG 40쌍 동일](latest-base-integration-native.json), [실제로 빌드한 이전 devel의 11종·18쪽 보존](rebase-preservation-native.json). 새 #9054의 비인쇄 기능은 [실제 브라우저·저장 검증](new-base-browser.json)과 집중 검사로 따로 확인했다. 이전 base의 캡처를 새 base 직접 캡처라고 기록하지 않는다.
- [필수 검사](local-gates.json): 최소 글꼴 전체 13,473 PASS·0 FAIL·45 설정상 skip, Native 공식 4,073 PASS, [집중 48건](focused-counts.json), 모든 Rust fmt·Clippy·workspace build·suite 검사 통과. 집중·Native 수는 전체와 중복될 수 있으므로 합산하지 않는다.
- [최적화 WASM](wasm-build.json)은 locked wrapper·wasm-opt를 실제 실행했고 Studio 공개 산출물 4개를 동기화·해시 확인했다. 생성된 타입 선언의 범위 밖 diff는 제출에서 제외한다.
- [무편집 공개 저장](roundtrip/results.json): Native CLI 18건·브라우저 API 18건, 총 36건. HWP/HWPX 두 형식과 원본 2차 반복 저장을 포함한다. 16개 입력·독립 PDF 조합(고유 파일 byte 그룹은 14개)을 양쪽으로 실제 전쪽 캡처했다(32캡처·64쪽). 동일 입력 해시인 별칭만 캡처를 공유한다.
- [편집과 저장](live-edit/README.md): 공개 BodyX 편집과 HWP/HWPX 12건 저장·재열기, 별도 스냅샷 복원 4건 저장이 동일 내용·표 프레임·2쪽을 유지한다. 2개 byte 그룹의 양쪽 전쪽 8쪽도 100.00%다. 실제 저장 파일은 같은 디렉터리에 커밋하고 실행 별칭·해시를 연결했다.
- [검증 입력 30개 보안 탐지 90건](new-input-security.json) 모두 clean, [대상 clipping 16건](clipping.json) 회귀·누락 0건. 가용 입력이 없는 기본 92종을 통과했다고 세지 않았다.
- [SVG 실제 소비 경로](svg/results.json): 원본·편집 모델 전 4쪽은 최저 99.60%다. SVG를 Native/Canvas의 직접 출력 대신 쓰지 않았다.

측정은 96 DPI·Print·compat 2022(새 인쇄 대조군은 2024)·PDFium·색상 경계 보정 전 2px raw다. 목표 문서는 양쪽 같은 HCRDotum/HCRBatang 프로그램을 공급했으며 실제 loaded SHA를 기록했다. 초기 Noto 대체 캡처는 완료 근거로 사용하지 않는다. [공급 정정과 원인](../source-line-cache/README.md).

원본과 BodyX의 제목·마지막 글자·괘선·후속 셀/페이지를 직접 판독했다. [같은 좌표의 실제 384 DPI 확대](live-edit/end-of-body-real384-pdf-native-wasm.png), [좌표·원본 PNG 해시](live-edit/actual384-provenance.json), [실제 WASM 공급과 캡처](live-edit/wasm-capture-provenance.json)를 보존한다. 확대 그림은 같은 crop을 나란히 놓았으며 글자·좌표를 보정하지 않았다. raw 100%는 PDF와 RGB가 같다는 뜻이 아니다.

원본 1쪽 [Native review](representative/native/review_001.png)·[overlay](representative/native/overlay_001.png), [WASM review](representative/wasm/review_001.png)·[overlay](representative/wasm/overlay_001.png)를 보존했다. 전 2쪽 각각의 flagged_pages는 0건이며 pixel·ink/proxy와 raw를 각 metrics.json에 구분했다.

입력 생성·독립 Print 출처는 [상위 원장](../README.md), 실제 저장 중첩 표는 [독립 한컴 검증](../native-nested-validation/README.md)을 따른다. 정식 회귀 첫머리의 `1506a1c346`은 등록 당시의 실제 측정 source이며, 변하지 않은 같은 테스트를 현재 제품에서 9/9 다시 실행했다. 이 원장의 새 source·실제 재측정 결과와 구분한다.

최종 조회에서 추가된 #9052 CLI base `d7b927d073`는 [실제 병합 결과 28건·Native 4쪽 추가 검증](latest-base-merge/README.md)으로 확인했다. 후보 코드와 기존 렌더/WASM/저장 소비자는 유지했으며, 이 추가 검증을 새 base 전체 회귀라고 기록하지 않는다.

승인 후 전진한 devel `9e27947967`의 실제 병합 결과도 [추가 검증](postapproval-base-merge/README.md)에서 확인했다. Rust 집중 49건·호스트 글꼴 40건·Native 6쪽 100.00%이며 후보의 제품·테스트는 바꾸지 않았다.

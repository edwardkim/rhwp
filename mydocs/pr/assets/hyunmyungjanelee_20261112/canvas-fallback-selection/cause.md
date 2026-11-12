# 명시적 글리프 공급 순서 보존

보호 sample16 51쪽의 ○는 기존 독립 Print Type3와 Myeongjo CFF의 윤곽이 일치한다. 그러나 현재 Mac의 Canvas descriptor는 주 face 뒤에 설치 별칭을 확장하여, 명시한 Myeongjo보다 `HYSinMyeongJo-Medium`을 먼저 조회했다. Chrome의 실제 platform font는 custom CFF가 아닌 설치 TTF `H2mjsM`이었다.

원인 분리용 출력에서 미선택 OS 별칭만 제외하면 같은 글꼴 바이트·원본·PDF의 51·61·62·63쪽은 각각 100.00%로 돌아왔다. 이 임시 exporter는 진단용이며 최종 Visual Sweep 결과로 쓰지 않는다. PostScript 이름 치환 실험은 Native 결과까지 바꾸므로 채택하지 않았다.

style resolver의 명시적 glyphFallbacks 표식을 공통 Canvas descriptor가 소비하도록 보정한다. 주 face와 확인된 TTF 별칭은 유지하고, 명시한 글리프 공급자를 일반 OS 별칭보다 앞에 둔다. 환경을 명시하지 않은 일반 설치 글꼴의 순서는 유지한다. 측정과 paint가 같은 descriptor를 사용하며 전진·기준 PDF·판정 기준은 바꾸지 않는다.

새 관계 검사 3건은 사용자 글꼴 설치 없이 descriptor 소유 순서·일반 경로·원 TTF 우선순위를 검증한다. 보정 전 실제 라이브러리에서는 2 PASS / 1 FAIL이었다. `ac8c85ee9dc3ce46f3ea2c23a497e77a61758c34`로 보정한 뒤 최신 `8fe7d1fb8b`의 최소 글꼴 Native 집중 검사는 관련 검사 포함 126 PASS / 0 FAIL이다. 정본 Native 18종·1,771쪽의 공식 표시값 하락은 0쪽이며, fresh WASM 정본 보호는 아직 진행 중이다. 진단 출력과 최종 전쪽 결과를 구분한다.

최신 fresh WASM의 보호 HWP2024 51쪽은 100.00%다. [review](../final-print-8fe7d1fb/protected-focus/sample16-hwp2024/wasm/review_051.png)와 [overlay](../final-print-8fe7d1fb/protected-focus/sample16-hwp2024/wasm/overlay_051.png)를 직접 열어 제목의 ○ 두 개와 뒤 본문의 윤곽·전진 보존을 확인했다. [생산 근거](../final-print-8fe7d1fb/protected-focus/sample16-hwp2024/wasm/producer.json)는 현재 전쪽 실행에서 생성한 PNG를 사용하며, 이전 전쪽 PNG와의 바이트 동일성도 기록한다. 이는 해당 쪽 판독이며 18종 전체 fresh WASM 검증의 완료를 대신하지 않는다.

## 최종 실행과 현재 리베이스의 재사용 범위

위 단계별 실패·중간 수치는 원인 분리 이력이다. 검증 source `30fa7739f0340f2a248b22542c7dd36d9354f056`에서 다음 결과를 확인했다. Native/fresh WASM 정본 각 18종·1771쪽에서 공식 두 자리 표시값 하락은 0쪽이다. 원 PNG의 producer를 유지하여 canonical TSV를 다시 계산했으며, 이전 부분 실행을 전체 완료 근거로 사용하지 않았다.

최소 글꼴 전체 13,365 PASS / 0 FAIL / 45 skipped와 Native 집중 141 PASS 및 필수 16단계 게이트를 완료했다. [전체 게이트](../final-rebase-30fa7739/gates-validation.json), [집중 검사](../final-rebase-30fa7739/focused-validation.json), [Native 전쪽](../final-rebase-30fa7739/native-validation.json), [fresh WASM 전쪽](../final-rebase-30fa7739/wasm-validation.json), [실제 backend 지원 범위](../final-rebase-30fa7739/actual-backends/summary.json)를 구분한다. RawSvg 차트의 강제 CanvasKit 진단은 기존 미지원이므로 지원 통과로 세지 않는다.

현재 `73d06ed0`은 새 base `a85d3265`의 #9017 HTML 붙여넣기를 반영했다. 기존 조판·입력·PDF·캡처 경로가 동일해 이 시각 증적을 원 producer와 함께 재사용한다. [동일성 감사와 추가 검증 경계](../final-rebase-30fa7739/rebase-a85d3265/README.md)를 따르며, 이전 source의 전체 실행 결과를 새 후보의 실행 결과로 고쳐 쓰지 않는다.

현재 제품 source `73d06ed0`에서도 최소 글꼴 Native 집중 164 PASS, 전체 13,380 PASS / 0 FAIL / 45 skipped(실행 404.945초), 최신 WASM·실제 브라우저 6사례/24회 저장/API11건 및 필수 21단계를 모두 통과했다. [현재 실행 원장](../final-rebase-30fa7739/rebase-a85d3265/gates-validation.json)과 [현재 기능 보존](../final-rebase-30fa7739/rebase-a85d3265/incoming-base-validation.json)을 기존 시각 producer의 자료와 구분한다.

# #7642 Stage 68 — 첫 저장 줄 뒤 좌표축 재시작의 앞 간격

Issue: #7642

## 범위와 독립 근거

55개·853쪽의 전쪽 Native/fresh WASM, 확대 판독, 관련 관계형 회귀 전환과
전체 검증 범위를 유지한다. 현재 Stage 67 source의 Native 빌드가 진행 중이며
그 Cargo 소스 스냅샷을 바꾸지 않고 독립 진단을 수행했다.

sample16 6쪽의 원본 첫 문단 141은 저장 vpos 73116HU를 갖지만 Print에서
새 쪽 첫 제목으로 표시된다. 다음 142는 새 축 3152HU에서 시작한다.
현재 첫 제목의 기준선 90.36px와 Print 97.920003px의 차이는 원본 앞 간격
568HU다. 다음 본문도 그 간격만큼 위에 있으며 표 앞의 저장 앵커에서 복원된다.

독립 저장 정보의 식은 다음과 같다.

`첫 문단 앞 간격 568 + 첫 줄 높이 1300 + 줄간격 716 + 다음 앞 간격 568 = 3152HU`

이 관계는 새 쪽에서 첫 줄 앞 간격을 유지한 Print와 일치한다. 이전 축의
73116을 쪽 상대 vpos로 재해석해 원점을 0으로 만드는 것은 잘못이다.
첫 문단의 소비 간격과 다음 문단에서 재시작할 저장 축은 구분해야 한다.

## 독립 진단과 반례

55개 원본과 #7160 대조의 전체 section/paragraph를 읽은 source 후보 조사에서
위 식과 축 재시작이 함께 성립한 쌍은 sample16의 141/142 하나다.
특정 문서/문자열/수치를 구현 조건으로 사용하지 않는다. 현재 일반 저장
첫 줄과 같은 축의 다음 줄, 앞 간격이 다른 저장 좌표, dirty 첫 문단,
합성 첫 줄을 거부하는 진단 4개도 compile/run exit 0으로 확인했다.
진단은 frozen parser metadata 라이브러리와 실제 원본을 사용하며 정식
renderer 회귀나 시각 통과의 증거를 대신하지 않는다.

ignored output의 stage68-first-line-axis-spacing-source-audit.json/tsv,
stage68-first-line-axis-spacing-negative-probe.json,
stage68-page6-baseline-comparison.json에 source·명령·관측값을 기록했다.

## 구현 계획과 소비 경로

권위 있는 한 줄/다음 줄과 해석된 앞 간격 → 새 축 첫 위치의 관계 검증 →
실제 단의 첫 FullParagraph owner가 소비할 확정 간격 →
layout_partial_paragraph의 확정 간격 인수 → 최종 첫 줄 원점으로 이어진다.
format이 이미 예약한 간격과 일치시킨다. 원래 page/lazy vpos 기준을 0으로
변경하지 않으며 이어받기 조각·편집·미주·개체 owner로 이 간격을 흘리지 않는다.
source·스타일의 단위로 비교하며 고정 px 보정이나 glyph 숨김을 추가하지 않는다.

Stage 67 빌드·Native lint가 끝난 뒤 이 변경을 구현하고 sample16의 영향/인접
쪽과 기존 미주/#7160 대조를 함께 검증한다. 문단 간격만으로 수식·내부 기호와
모든 쪽의 시각 일치를 완료했다고 판정하지 않는다. 회귀 기대값은 아직 변경하지 않는다.

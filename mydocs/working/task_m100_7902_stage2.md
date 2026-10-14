# #7902 2단계 — #7877 문서 회귀 보정과 재발 방지

Issue: #7902. 이미 merge된 PR #7877의 원본 19문서·835쪽 범위를 유지한다.
1단계 도구 PR #7903과 기록 PR #7906은 실제 merge됐으며 이 이슈는 OPEN이다.

## 기준과 승인된 진행

- 브랜치: `fix/pr7877-render-regressions-20261014`.
- 기준 `upstream/devel`: `c0d330737fabae77ca2bf1706cba8514d0e8f1df`.
- 공용 Cargo target: `/home/tsjang/rhwp/target/pr-review`, jobs8, Cargo 순차 실행.
- 사용자 승인 범위: 원본 출력 보정, Native/fresh WASM 전쪽 대조, 회귀 보강 및 필수 검증.
  먼저 영향 경계를 확인하고 최종 소스에서 전체 범위를 검증한다. 기존 값·허용치를 완화하지 않는다.
- 원 PR TSV는 이전 결과의 증거로 읽으며 이전 소스를 전쪽 재실행하지 않는다. 새 기준 devel은
  같은 입력·독립 Print·글꼴·DPI·backend로 TSV를 산출한다. 한 쪽·한 경로라도0.01%p 이상
  하락하면 재검토하며 평균 상승으로 상쇄하지 않는다.

## 우선 분석 경로

1. 쪽번호: 원본 `Page Number` 글자모양 → resolved style → 폭 측정과 TextRun →
   정렬 원점. 현재 코드가 font family/size/color만 전달해 bold 등 속성을 누락한다.
   본문과 같은 스타일 변환 결과를 측정·paint에 전달하고 PDF 및 양쪽 backend에서 검증한다.
2. rowbreak15쪽: 그림 선언 상자가 아닌 바깥 host table의 물리 높이·예약 공간·뒤 본문의
   원점을 추적한다. 저장 줄·그림 상자·빈 줄의 실제 점유 계약을 확인한다.
3. overfill: 5쪽의 노트만6쪽으로 이월되고 이후 내용이 한 쪽씩 밀린다. 시작116/끝124에서
   다음 unit의 높이·빈 띠·중첩 예약·컷과 paint 소비를 대조한다. 전체 내용 보존과 종료를 확인한다.

위 내용은 보정 완료 판정이 아닌 기존 진단에서 확인한 분석 시작점이다.

## output 전체 정리

`output/pr-review` 전체를 점검한다. 종료된 #7903/#7906의 중복·중간 산출물은 공개 증적과
필요한 원시 기록의 보존을 확인한 뒤 제거한다. #7902가 실제 사용하는 과거 producer·글꼴·TSV와
미해결 재현 자료는 보존 사유를 기록한다. 공유 target·font raster cache·원본/PDF는 보존한다.
제거 경로·확보 용량·보존 이유와 후속 처리 결과는 이 기록에 이어 적는다.

## 검증 상태

- 전체 output 정리 지침의3개 문서: `git diff --check` 및 파일 단위 Markdown 링크 검사 통과.
- 기준 devel Native Skia 빌드 시작. 아직 결과 미확정.
- 문서별 출력 보정, 새 회귀 및 최종 전체 검증: 미완료.

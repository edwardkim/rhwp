# #9318 병합 전 후보의 완료 결과와 미완료 상태

이 폴더는 이전 base `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`와 제품 후보
`8f9fd0414ffcfb720fa4a601c79c4590480de0e8`의 검증 기록이다.
새 #9318 기반 후보의 검증으로 사용하지 않는다.

- #9318의 [코드 head 전체 CI](source146-pr9318-ci-complete.json)와
  [문서 전용 trailing head CI](source149-pr9318-trailing-ci.json)는 모두 완료됐다.
  [실제 병합](source147-pr9318-merged.json)은 `622af208ee12c4cb16357d465e4802e7fd4726bc`다.
- [보호 문서 집계](source145-qualified-protected-summary.json)의 Native 23조건 2,023쪽은
  정본 대비 하락 0쪽이다. [전쪽 TSV·manifest 해시](preserved-native-files.json)와
  `native/<key>/`에 실행 근거를 보존했다. 화학·outline·sample10은 전체 문서 실행 단위로
  검증된 후속 결과를 적용했다. 원 실패와 글꼴 qualification은 Source49에도 보존했다.
- WASM은 완료 문서 22종 1,260쪽 중 sample16 세 저장본의 18·35쪽이 미달이다.
  `wasm-observed-declines/`에 각 전쪽 TSV와 HWPX의 두 페이지 PNG를 보존했다.
  독립 Print와 직접 본 18쪽에는 도식 채움·테두리 차이가 있다. 원인은 아직 확정하지 않았으며
  #9318의 WMF 공통 경로 변경을 포함해 새 base·후보에서 다시 확인한다.
- Rust·base/candidate 행렬·sample10 WASM의 중단 당시 상태도 보존했다. 일부 검사의 성공을
  전체 통과로 표시하지 않는다. [프로세스 전환 기록](source148-old-head-quiescence.json)은
  소유를 확인한 controller 6개와 자식 등 18개만 종료했고 잔여 실행은 없음을 기록한다.
- [새 base 기능 보존 입력 목록](source144-incoming-9318-input-catalog.json)은 원 PR 5건과
  보조 문서의 총 167쪽이다. 새 base·후보의 측정은 이 폴더에 포함하지 않는다.

정본 보호 TSV·입력·독립 PDF를 변경하거나 하한을 완화하지 않았다. PR 준비까지만 진행한다.

# Stage77: Canvas 장치 표본 표시와 deferred DPR 갱신 보정

Issue: #7445. 부모 HEAD는 `8d9106d2bff13f5389069a927ca073bc284c6c5f`, base는 `def947c2149df5bc812d801e6e79be19a403b4d1`이다. 기존 대상만 사용했다. 이 단계는 로컬 중간 보정이며 PR 준비 완료가 아니다.

## 원인과 공통 소비

로드맵 153쪽의 실제 CanvasView 화면은 raw 85.25831%인 반면 같은 main/front bitmap의 합성은 92.63744%였다. 이미 장치 해상도로 그린 표본을 소수 CSS 크기에 다시 보간하며 옅은 격자선이 변했다. 실제 화면에서 논리 크기를 유지한 필터 개입을 대조했다. 장치 축의 floor/ceil에 대응하는 canvas만 crisp-edges로 표시하고, 실제 해상도 축소·배율 미리보기 및 DOM 그림은 기존 보간을 유지한다. 원본·글꼴·paint 좌표·측정 문턱은 바꾸지 않았다.

실제 DPR 전환에서는 계획이 1.5인데 active 표면이 1에 남았다. `efbdb75484e`(#6042)의 계획간 비교와 `49525b299c4`(#6467)의 deferred 예산, `ffc88e54fd1`(#6987)의 줌 정착 경로를 대조했다. deferred 갱신이 계획만 먼저 교체한 뒤 strict 갱신이 실제 paint를 완료한 것으로 오인했다. 이제 현재 계획뿐 아니라 실제 표면에 기록한 renderScale도 소비한다. 이미 완성된 같은 배율 표면의 재사용과 실패 뒤 active 해제는 유지한다.

## 검증과 자체 오류 정정

- 필터 관계 검사: 실제 1 FAIL → PASS. deferred DPR 관계 검사도 실제 1 FAIL(재래스터 목록 빈 배열) → PASS. cache·scroll·zoom scheduler 묶음 70 PASS다. 문서의 새 px 핀이나 fixture를 추가하지 않았다.
- 최종 TypeScript exit 0. Studio 전체 2,410건 중 **2,408 PASS / 0 FAIL / 2 skip**, 22.406초다.
- 실제 제품 CanvasView의 Canvas2D 54건·지원 CanvasKit 8건, 총 **62건 실패 0 / exit 0**이다. 현재 로컬 글꼴 감지·정상 초기화 경로, 요청 backend·직접 재생·그림 준비와 JS/WASM 응답 hash를 확인했다. 미지원 CanvasKit을 성공으로 분류하지 않았다.
- 독립 Print 31건은 전부 90% 이상, 이전 같은 소비 조건보다 2%p 이상 하락 0건이다. 로드맵 153쪽은 93.18693%로 개선됐다. Native/fresh WASM 엔진은 이 TS 수정에서 변경하지 않았다.
- 기존 4개 입력을 동일 인스턴스에서 DPR 1 → 1.5 → 2 → 배율 1.25/DPR 1.5 → 원상 복원했다. 20개 실제 화면과 동일 PDF의 96/144/192/180 DPI 비교를 산출했다. 원상 복원 트리와 논리 크기·backend·자원 준비를 확인했다. **로드맵 DPR 1.5는 89.22797%로 미완료**다. 다른 19건의 통과로 상쇄하지 않는다.
- 최초 구현에서 진단값 갱신 함수에 DOM 필터 갱신을 넣어 scheduler 전용 검사 4건이 실패했다. DOM 갱신은 실제 배치 경로에 두도록 정정했고 기존 검사를 완화하지 않았다. 최초 실제 캡처의 DOM 그림 준비 실패 1건은 준비 대기를 보완해 재실행했으며 최종 62건에서도 확인했다.
- 첫 추가 CDP 진단은 defaultViewport 초기화 가능성이 있어 무효로 보존했다. 이후 연결은 defaultViewport:null로만 관측했다. 한 복원 검사는 ResizeObserver의 anchor 갱신 전 goto를 호출해 서로 다른 scroll 상태를 비교했다. 레이아웃 완료 뒤 같은 goto를 적용한 재실행에서 기존 복원 assertion을 유지해 통과했다. 실패 기록은 덮어쓰지 않았다.

## 남은 작업

로드맵의 Print 종이는 595×841pt(793.3333×1121.3333 CSSpx)인데 현재 표시 논리 크기는 원본 A4(793.7×1122.5)다. DPR 1.5에서는 두 사각형 차이가 다시 보간돼 격자가 변한다. 공통 인쇄 종이 소비를 추가 추적한다. 기존 sample16 HWP의 18·35쪽도 새 전쪽 비교에서 미달이며 원본·독립 수동 Print와 그림의 패턴·소비 경로를 재검토한다. 새 대상 추가나 검사 제거로 처리하지 않는다.

현재 upstream/devel에 #8181(`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`)이 추가됐으므로 이 단계 변경을 보존한 뒤 리베이스·패키지/기능 보존을 이어간다. 기존 52조건 전쪽 실행은 def947 기준의 중간 증거이며 새 base 통과로 재인용하지 않는다. 전체 회귀·전쪽·실제 소비의 최종 수용 및 PR/CI/후속 처리는 아직 미완료다.

[대표 review·overlay와 기계 증적](../pr/assets/issue7445/completed-20261009/rebase-def947-20261023/canvas-native-samples/README.md). 원시 로그와 임시 진단은 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/canvas-raster-display-final/` 및 앞선 `canvas-raster-display-repair/`에 보존했다.

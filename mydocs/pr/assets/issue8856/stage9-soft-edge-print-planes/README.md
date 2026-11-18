# 2024 Print 효과 그림의 색·마스크 전달 보정

원본 효과 반경을 보존한 공통 그림 준비 결과에서 RGB와 1bit 마스크를 각각 재표본한다.
근거는 `independent-pdf-planes.json`의 실제 독립 PDF 객체·색·마스크 표본이다.
원본·기준 PDF·판정 문턱은 바꾸지 않았다. 원본 모델의 줄·쪽·그림 상자도 유지한다.

반경이 양수인 단독 2024 PNG Print의 확인된 계약만 장치 평면으로 보낸다.
무효과·Screen·JPEG·묶음/회전은 기존 경로를 유지한다. 색·마스크 평면의 소유를
LayerBuilder → ResolvedImagePayload → 공통 print_image_sampling → Native/Canvas로 전달하며,
backend마다 효과를 다시 추측하거나 alpha를 두 번 곱하지 않는다.

## 실제 검증

- 새 검사: 수정 전 1 PASS / 1 FAIL → 최종 2 PASS / 0 FAIL.
- 기존 효과·모델/저장/전달·크기 검사 9건도 실제 통과해 총 11 PASS / 0 FAIL.
- Native·fresh WASM 각 9조건·10쪽을 전체 산출했다. 쪽수 누락·90% 미만은 없다.
- 사진·기존 대조군 6쪽씩은 stage8 PNG와 바이트가 같다. 해시는 `validation.json`에 있다.
- 2024 투명 대조군 4조건은 양 backend 모두 raw 100.00000%다.
  양수 반경·반투명 대조군 중앙 RGB의 최빈값은 (207,223,239)이며 PDF는 (206,222,238)이다.
  이전 (159,191,223)의 큰 전달색 차이는 복구했지만 채널 1단계 차이까지 완전 동일하다고
  주장하지 않는다. raw 100%는 엄격 RGB 일치율이 아니다.
- 효과 없는 반경 0은 기존 RGB를 유지한다. 이 대조군의 기존 (159,191,223)과 PDF 색 차이를
  효과 보정의 해결로 인용하지 않는다. 이번 효과 보정 근거만으로 일반 PNG 정책을 확장하지 않았다.
- Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  루트 pkg·Studio public·새 Chrome 초기 JS/WASM GET 응답 해시가 일치하고, 실제 초기화 후
  캡처한 interview 1쪽 PNG도 같은 fresh WASM sweep과 바이트가 같다.
- Native Clippy·fmt·suite manifest·unit tier 검사를 통과했다. 전체 최종 PR 게이트·보호 TSV·
  다른 잔여 문서의 완료를 대신하지 않는다.

원인·단계 범위는 [stage9 작업 기록](../../../../working/task_m100_8856_stage9.md)에 있다.
같은 쪽의 `*-raw.png`, `*-pdf.png`, `*-review.png`, `*-overlay.png`를 함께 보존했다.
반경 1000의 반투명 면·투명 구멍에서 색, 외곽 감쇠, 구멍 보존을 직접 대조했다.
사진의 stage8 외곽 복원은 유지되며 제목 글리프·간격과 다른 필수 잔여는 계속 보정한다.

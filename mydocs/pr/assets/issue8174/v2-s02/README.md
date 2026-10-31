# #8174 v2-S02 시각 증적

- 제품 source: `5a77054519216f9f2dae6a779e7e334f3894466d`.
- 기준 source: `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`.
- Native Skia와 최적화한 fresh WASM Canvas, Print profile, 96 DPI, PDFium smooth RGB.
- Native·WASM 각각 41조건·41쪽, raw 최저 100%, 90% 미달·누락 쪽 0.
- TSV는 원본 그대로 복사했다. `manifest.json`에 각 SHA-256과 producer hash를 기록했다.
- `*-review.png`는 실제 기준/후보/overlay를 표시한다. source·색 공간·alpha oracle의 경계는
  [입력 설명](../../../../../pdf/issue8174/features/README.md), 명령·직접 판독·성능은
  [S02 기록](../../../../working/task_m100_8174_v2_stage2.md)을 따른다.
- `not_evaluated` 자동 판정과 에이전트의 직접 판독은 GitHub 승인·메인테이너의 최종 시각 승인과 구분한다.

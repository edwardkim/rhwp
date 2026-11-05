# PR #8585 검증 증적

원 기여 source는 `7e7a86fef9`, 고정 devel은 `7fdd5441ff`, 실행 코드 보정은 `a55c53a393`,
최종 로컬 검사 후보는 `c3e28d2c6e`다. `runtime-equivalence.json`은 build source와 validation source의 차이를 설명한다.

- `base-*`, `head-*`, `corrected-*`: 동일 HWP·독립 WebHWP Print 전 1쪽, 96 DPI, Native/fresh optimized WASM의 실제 TSV·PNG·provenance.
- `control-8122-*`: 정상 대조군의 전쪽 비교. 두 backend 모두 base/후보 픽셀이 같다.
- `zoom-*`: 실제 raster를 같은 좌표·배율로 확대한 판독용 그림. 원출력 보정이나 테스트 좌표가 아니다.
- `local-gates-*`: 실제 검사 명령·source·exit·완료 시각. Full nextest 12,426 PASS/45 skip, Native 공식 3종은 각각 2/2/22 PASS다.
- `input-manifest.json`, `merge-preservation.json`: commit 입력 동일성 및 최신 devel 원장 보존 확인.

폰트 프로그램은 포함하지 않았다. 실제 공급한 세 프로그램과 HCR Dotum 대체는 `font-environment.json` 및 provenance에 기록했다.
원 head의 CI와 보정 후보의 로컬 검증은 별개다. 보정 후보의 같은 통합 PR #8601 Full CI와 최종 정책 감사는 `corrected-code-ci.json`에 기록했다.
merge·trailing head 결과는 실제 완료 뒤 구분해 기록한다.

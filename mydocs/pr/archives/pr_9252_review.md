# PR #9252 검토 — Visual Sweep 쪽 점수 재개 조건

**승인.** 검토한 코드 head는 `bcb6d2eb93765139e83fa363daafeff74a3b26b6`이다. 기존 run manifest와 쪽 체크포인트가 모두 같은 디코딩 정보를 소비하는 경로를 확인했다. [CLI와 측정 구현](../../../scripts/visual_sweep.py)에 추가된 값은 Pillow 버전, 로드된 네이티브 모듈 SHA-256과 zlib 버전이다. 구형 결과는 새 출력 경로에서 다시 계산할 수 있다.

- Pillow 11.1.0에서 계산하고 11.3.0에서 재개하면 기존 구현은 쪽 점수를 수용했다. 보정 후 같은 PNG 두 장에서 환경 변경을 거부했고 같은 환경의 TSV는 새 계산과 바이트 단위로 일치했다. 관련 검사 17건이 통과했다.
- 독립 Hancom 2020 Print가 있는 `empty-host-split-table.hwp`는 96 DPI Native Skia와 기준 PDF 모두 2쪽이었다. 새 실행, 동일 조건 재개, 저장된 PNG 쌍 재채점의 TSV SHA-256은 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`이었다. 97 DPI 재개는 거부됐다.
- [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/44753946599)가 성공했다. 비교 수식·렌더러와 CI 실행 정책은 변경하지 않았다. 검토 기록을 추가한 최종 head의 필수 검사 뒤 병합한다.

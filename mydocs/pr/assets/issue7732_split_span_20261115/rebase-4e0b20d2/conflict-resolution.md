# 최신 보호 하한 충돌 해소

사용자의 커밋 후 리베이스 지시에 따라 이전 head `dcab63d173c436f7bb8d7797bb81ef0c3830bf9c`를
고정 base `4e0b20d2922953ee7ab9a1716ac819b921fe484b` 위에 재적용했다.
중첩 표 상승 기록 커밋에서 다음 4개 파일이 충돌했다.

- `mydocs/pr/assets/visual-protected-baseline/final.tsv`
- `mydocs/pr/assets/visual-protected-baseline/manifest.json`
- `mydocs/pr/assets/visual-protected-baseline/README.md`
- `scripts/visual_sweep_handbook.py`

입력·PDF SHA, backend, compat, DPI, 쪽을 모두 대조했다. 총 3,542행의 비교 키·메타데이터는 같았고,
최신 devel은 17행에서 이전 브랜치보다 높고 나머지는 같다. 이전 브랜치가 높은 행은 없다.
따라서 최신 devel의 **표 전체**와 manifest 출처를 그대로 수용했다. 페이지별 최댓값을 조합하지 않았다.
현행 TSV SHA-256은 `43c59cf77a4a86b3744d3df30eb71469de70b584ccc9cb165fa8f7b7884d2e1f`이며 handbook 검사의 해시도 동일하게 유지한다.

이전 브랜치의 6행 상승 영수증은 과거 검증 기록으로 보존하며, 현행 하한에 새로 적용한 행은 0이다.
특히 nested 11쪽의 새 Native/WASM 보호값은 각각 99.67%/99.63%다. 이전 99.09%/99.10%로 낮추지 않는다.

이번 devel에는 조판·글꼴·저장 코드 변경이 포함된다. 기존 검증은 기록된 소스의 증거로 보존하고,
충돌 해소 자체를 최종 head의 기능 보존·전체 회귀 통과로 선언하지 않는다. 새 head의 영향 경로와
보호 하한을 재검증한다. 원본·PDF·글꼴·이전 로그와 컴파일 캐시는 보존했다.

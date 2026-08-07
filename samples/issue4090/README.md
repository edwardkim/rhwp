# Issue #4090 / PR #4094 검증 샘플

## 156492236_규제샌드박스_min.hwpx

- 출처: hwpdocs 코퍼스
  `korea_downloads/중소벤처기업부/156492236_220119_(보도참고자료)_규제샌드박스_시행_3주년(규제자유툭구단).hwpx`
  (대한민국 정책브리핑 korea.kr 공개 보도참고자료, 원본 23.2MB).
- 최소화: BinData 이미지 23개와 Preview 를 1×1 스텁으로 치환해 57KB 로 축소.
  조판 좌표·개체 표시 크기는 전부 `Contents/*.xml` 에 있으므로 레이아웃은 불변 —
  devel 바이너리로 원본 대비 `rhwp info` 쪽수(24)와 `export-render-tree` 전 페이지
  bbox JSON 이 바이트 단위로 동일함을 확인했다. XML(section0/header)은 무수정.
- PR #4094(Square 어울림 표 세로 배제 밴드)의 간판 문서.
  - 수정 전 rhwp **24쪽** → 수정 후 **14쪽** (한글 정답 **17쪽**, 오차 7 → 3).
  - 남는 −3은 밴드 종료 스냅이 fit 판정 경로에 미반영인 축 — #4090 후속 과제.
- 재현: `rhwp info samples/issue4090/156492236_규제샌드박스_min.hwpx` 의 "페이지 수".
  Square 표는 pi=93(밴드 +183.2px)·pi=101(+159.4px) 등 — 상세는 PR #4094 본문.

# Issue #2006 검증 샘플

## 1790387_prep_final_report.hwpx
- 출처: hwpdocs 코퍼스 `prism_downloads/질병관리청/1790387-202500020_D0150004-1-001_HIV
  노출 전 예방요법(PrEP) 수요자 타당도 검증 및 질병부담 연구 최종결과보고서.hwpx`
  (PRISM 정책연구 공개 보고서, 원본 그대로 복사).
- PR #2082(전면 tac 이미지 스택 라인 경계 분할)의 핵심 개선 타깃 문서.
  - 수정 전 rhwp 130쪽 → 수정 후 **141쪽** (한글 2022 편집기 **146쪽**, 잔여 −5).
  - 잔여 −5는 텍스트 줄-채움 누적(#1921 계열) 별건.
- 기준 PDF: `pdf-large/issue2006/1790387_prep_final_report-2022.pdf` (**Git LFS**,
  50.2 MB ≥ 50 MB — `git lfs pull` 필요). 한글 2022 COM, Print 액션 1-up 강제
  출력 146쪽 = 편집기 PageCount 146 정합.
  - 주의: FileSaveAsPdf 경로는 sticky 인쇄 설정(모아찍기)을 따라가므로
    `HPrint.PrintMethod=0` 명시 후 Print 액션으로 출력해야 권위 레이아웃이 나온다.
- 검증: `cargo test --test issue_2006_1790387_prep_pagination_pin` /
  `rhwp dump-pages samples/issue2006/1790387_prep_final_report.hwpx`

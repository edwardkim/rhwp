# Issue #1921 검증 샘플

## 59043_regulatory_analysis.hwp
- 출처: hwpdocs 코퍼스 `opinion_downloads/보건복지부/59043_규제영향분석서.hwp`
  (국민참여입법센터 공개 규제영향분석서, 원본 그대로 복사).
- PR #2092(RowBreak 블록컷 sliver 흡수)의 핵심 개선 타깃 문서.
  - 수정 전 rhwp 48쪽 → 수정 후 **42쪽** (한글 2022 편집기 **37쪽**, 잔여 +5).
  - 잔여 +5는 2단 배치 밀도(부동 표 흐름 패킹) 축 — #1921 후속 과제.
- 기준 PDF: `pdf/issue1921/59043_regulatory_analysis-2022.pdf`
  (한글 2022 COM, Print 액션 1-up 강제 출력 37쪽 = 편집기 PageCount 37 정합).
  - 주의: FileSaveAsPdf 경로는 sticky 인쇄 설정(모아찍기)을 따라가므로
    `HPrint.PrintMethod=0` 명시 후 Print 액션으로 출력해야 권위 레이아웃이 나온다.
- 검증: `cargo test --test issue_1921_59043_pagination_pin` /
  `rhwp dump-pages samples/issue1921/59043_regulatory_analysis.hwp`

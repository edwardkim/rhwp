# 정본 시각 보호 기준

이 자산은 수용된 최종 전쪽 검증을 통합한 품질 하한이다. 현재 개발 후보의 검증 결과나 통과 선언이 아니다.
적용 역할과 판정은 [시각 검증 거버넌스](../../../manual/verification/visual_verification_governance.md#정본-보호-tsv의-공식-표시값-보존)를 따른다.
**통합 담당 collaborator·maintainer만 이 특별 보호 검증을 수행한다.** 일반 사용자·외부 contributor에게
비공개 원본 OTF 보유·설치나 이 기준 전체의 재현을 제출 조건으로 요구하지 않는다.
조판 영향 PR의 `--handbook-gate` 실행·통과는 일반 기여자도 준수하며, 필요한 글꼴은 지원 웹폰트를 사용할 수 있다.
기존 편람 gate도 이 TSV에서 편람 HWPX·기준 PDF·backend가 일치하는 383쪽만 선택한다.
이 연결은 다른 문서 전체의 자동 강제 실행을 추가하지 않는다.

## 정본 파일

- [final.tsv](final.tsv): 입력·기준 PDF·backend·페이지별 공식 두 자리 raw 보호값.
- [manifest.json](manifest.json): 입력/PDF SHA-256·크기·측정 조건·전쪽 수·출처 commit/TSV/열/해시와 중복 제거 내역.
- TSV SHA-256: `ee41455cf7f05585487d92961ccb9fd8160d5d1e15c3567019ea89e2b7bf2aeb`.
- 입력 18개, Native/fresh WASM 각 18개 데이터셋·1,771쪽, 총 36개 데이터셋·3,542행.
- 수용 자료 56개 데이터셋 중 중복 20개를 정리했다. 이전 자료보다 공식 표시값이 낮아진 페이지는 0쪽이다.

## 문서별 범위

HWP와 HWPX는 같은 제목이어도 별도 입력이다. 아래 최저값은 요약이며 판정은 전쪽 각 행으로 한다.
파일명은 탐색용이며 실제 비교는 manifest의 입력/PDF SHA·조건과 TSV의 document_id·backend·page를 사용한다.

| 입력 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| [angle-2400.hwpx](<../../../../mydocs/pr/assets/issue_8371/angle-narrow/angle-2400.hwpx>) | 1 | 100.00% | 100.00% |
| [jongbu_law.hwpx](<../../../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx>) | 6 | 98.86% | 98.19% |
| [2025 행정업무운영 편람(최종).hwp](<../../../../samples/2025 행정업무운영 편람(최종).hwp>) | 383 | 99.96% | 99.96% |
| [2025 행정업무운영 편람(최종).hwpx](<../../../../samples/2025 행정업무운영 편람(최종).hwpx>) | 383 | 99.96% | 99.96% |
| [issue2007_nested_cell_pagination_42065.hwp](<../../../../samples/basic/issue2007_nested_cell_pagination_42065.hwp>) | 17 | 94.96% | 96.32% |
| [hwp3-sample10.hwp](<../../../../samples/hwp3-sample10.hwp>) | 763 | 99.41% | 99.40% |
| [hwp3-sample16-hwp5-2022.hwp](<../../../../samples/hwp3-sample16-hwp5-2022.hwp>) | 64 | 100.00% | 100.00% |
| [hwp3-sample16-hwp5-2024.hwp](<../../../../samples/hwp3-sample16-hwp5-2024.hwp>) | 64 | 99.99% | 99.99% |
| [hwp3-sample16-hwp5.hwpx](<../../../../samples/hwp3-sample16-hwp5.hwpx>) | 64 | 99.99% | 99.99% |
| [20099369_yeongwol_forms.hwp](<../../../../samples/issue5699/20099369_yeongwol_forms.hwp>) | 4 | 99.60% | 99.59% |
| [pic-crop-01.hwp](<../../../../samples/pic-crop-01.hwp>) | 1 | 100.00% | 94.65% |
| [exam-social-insert-delete.hwp](<../../../../samples/pr8040/exam-social-insert-delete.hwp>) | 4 | 99.72% | 98.82% |
| [exam-social-insert.hwp](<../../../../samples/pr8040/exam-social-insert.hwp>) | 4 | 99.72% | 98.82% |
| [re-03-latin-only-hancom.hwp](<../../../../samples/re-03-latin-only-hancom.hwp>) | 1 | 100.00% | 100.00% |
| [shape-group-02.hwp](<../../../../samples/shape-group-02.hwp>) | 1 | 100.00% | 100.00% |
| [issue3307_outline_number.hwpx](<../../../../samples/task3307/issue3307_outline_number.hwpx>) | 9 | 97.89% | 97.81% |
| [body-list-typed-hwp.hwp](<../../../../tests/fixtures/issue_8109_saved_fonts/body-list-typed-hwp.hwp>) | 1 | 100.00% | 100.00% |
| [control-styles-hwp.hwp](<../../../../tests/fixtures/issue_8109_saved_fonts/control-styles-hwp.hwp>) | 1 | 98.93% | 98.79% |

## 출처 통합과 판정

초기 자료는 PR #8389, 최신 수용 자료는 PR #8397의 병합 commit에서 읽었다. 이 번호는 과거 출처이며
새 검증의 선택 기준은 이 정본 TSV와 manifest다. 원래 자료는 삭제하거나 덮어쓰지 않는다.

1. 같은 입력 SHA·독립 Print PDF SHA·compat·DPI·인쇄 프로필·계산 조건·backend가 겹칠 때 최신 수용본을 선택한다.
2. 같은 수용 시점의 중간 표와 최종 재출력이 겹치면 명시된 최종 전쪽 표를 선택한다. 반복된 동등 대조 표는 하나로 정리한다.
   기존 편람·sample16·편집 입력은 최종 전쪽 재검증을 사용하고, 같은 법령 입력의 별도 표와 편람의 SEO 대조 표는 중복으로 기록했다.
   sample10·기존 회귀 입력·각도 대조군·나머지 SEO 입력은 각각 최종 수용 자료를 보존했다.
3. 원본 TSV에 공식 두 자리 값이 있으면 그대로 사용한다. 정밀 최종 재검증 표만 있는 경우 한 번 두 자리로 표시하며,
   기존 TSV 작성기의 `.2f`와 동일함을 확인했다. 원본 TSV SHA·페이지/값 열·문서 필터를 manifest에 남겼다.
4. 각 문서·backend·페이지의 후보 공식 표시값은 정본 이상이어야 한다. 0.01%p 하락도 허용하지 않는다.
   일반 90%·98% 기준과 devel 대비 하락 판정에는 기존 정밀값을 사용한다. 다른 쪽의 개선·평균·CI 성공으로 면제하지 않는다.
5. 이 표는 글꼴 프로그램을 제공하지 않는다. 출처 commit의 condition_evidence와 그 SHA를 통해 수용 당시 조건·생산 기록을 확인한다.
   현재 설치 글꼴이나 임의 대체 글꼴이 원래 조건과 같다고 가정하지 않는다. 통합 담당자가 원본 공급 조건을 확인하고 맞춘다.
6. 갱신 시에는 새로 수용한 최종 전쪽 결과와 입력/PDF·생산·글꼴 증거를 확보한다. 기존 정본 대비 하락 0쪽을 확인하고
   TSV·manifest를 함께 검토·커밋한다. 페이지별 최댓값을 이어 붙이거나 현재 미달 후보로 보호값을 낮추지 않는다.

## 자료 무결성 확인

통합 시 선택한 전 3,542행을 수용 commit의 원본 TSV와 대조한다. 중복 키·쪽 누락·범위 밖 값이 없어야 하며,
입력 18개와 대응 PDF의 SHA·전체 쪽수, 양 backend의 페이지 범위를 확인한다. 선택 전의 중복 표에도 같은 검사를
적용하여 원래 보호값이 낮아지지 않았는지 확인한다. 실행 로그·임시 생성/검사 스크립트는 ignored output에만 보관한다.
이 무결성 확인은 제품의 최신 head 재출력·회귀 검사·시각 판독을 대체하지 않는다.

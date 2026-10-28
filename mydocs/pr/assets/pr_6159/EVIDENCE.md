# planet6897 #6125, #6153, #6155 시각 검토 증적

- 검토 통합 브랜치: `review/planet6897-6125-6153-6155-20260826-r1`
- 검토 대상 HEAD: `a6616441b21c6127999039419625a622f86db982`
- PDF 변환 클라이언트: `hwp2024-mcp-convert`
- 변환 엔진: `--engine 2020`
- Hancom 실행 버전: `12.0.0.4605` (`hwp-managed-direct-dll-host`, 32-bit)
- 생성일: 2026-08-27 KST

## #6125 / #5700

- 원본 HWP의 RHWP 논리 페이지는 156쪽, Hancom PDF는 77쪽이다.
- 쟁점 RHWP 139쪽의 고유 텍스트 `GPS floater`는 Hancom PDF 69쪽에서 확인했다.
- 양면 PDF 한 면과 RHWP 논리 페이지의 페이지 체계가 달라 `visual_sweep.py --page 139` 자동 대조는 실행하지 못했다.
- 원본 PDF, RHWP p139 SVG/render-tree, Hancom PDF p69 및 RHWP p139 래스터를 `manual-compare/`에 보관했다.
- 이 항목은 같은 페이지 번호의 픽셀 지표를 합격/불합격 근거로 사용하지 않는다.

## #6125 / #5701

- 원본 HWP의 RHWP 논리 페이지는 206쪽, Hancom PDF는 103쪽이다.
- 쟁점 RHWP 76쪽의 고유 텍스트 `본건 노래방의 매출액은`은 Hancom PDF 38쪽에서 확인했다.
- 동일 번호 76↔76 자동 sweep은 페이지 내용이 달라 진단 산출물로만 보관한다.
- 대응 Hancom PDF p38 및 RHWP p76 래스터는 `manual-compare/`에 보관했다.
- 이 항목은 같은 페이지 번호의 픽셀 지표를 합격/불합격 근거로 사용하지 않는다.

## #6153 / #6126

- 공개 fixture `3171199_design_capability_criteria.hwp`는 RHWP와 Hancom PDF 모두 7쪽이다.
- 쟁점 3쪽 sweep이 완료됐고, PDF·SVG·render-tree·overlay·review PNG를 보관했다.
- 글꼴/래스터 차이로 pixel/ink proxy는 판정 기준이 아니다. review PNG에서 표 셀 경계와 본문 흐름을 수동 확인했다.

## #6155 / #6128

- 공개 fixture `156653004_privacy_day_ceremony.hwpx`는 RHWP와 Hancom PDF 모두 7쪽이다.
- 쟁점 4쪽 sweep이 완료됐고, PDF·SVG·render-tree·overlay·review PNG를 보관했다.
- 자동 분석의 flagged page는 없었다. 글꼴/래스터 proxy는 참고값이며, review PNG에서 wraparound 이후 본문·표 흐름을 수동 확인했다.

## 보관 및 공개 전환

- 이 디렉터리는 PR 생성 전 로컬 증적 보관소다. private 원본 HWP는 여기서만 유지하며 Git에 추가하지 않는다.
- 통합 PR 번호가 확정되고 코드 CI가 통과하면, 공개 가능한 PDF·PNG·SVG·render-tree·매니페스트·SHA256SUMS만 `mydocs/pr/assets/pr_<번호>/`에 복사하고 개별 PR review 문서에 연결한다.

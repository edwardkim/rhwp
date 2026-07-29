---
kind: guide
status: active
canonical: mydocs/tech/investigations/README.md
last_verified: 2026-07-19
---

# 이슈별 기술 조사

이 디렉터리는 특정 GitHub 이슈에서 나온 가설, 진단, 기준선, 관찰, 실험 결과를 보존한다.
여기 문서는 당시 이슈와 환경을 이해하기 위한 근거이며, 반복 작업의 절차나 장기 계약의 권위 문서는 아니다.

## 사용 방법

- `issue-####/` 단위로 해당 이슈의 조사 문서를 찾는다.
- 확정된 스펙 정정은 `mydocs/tech/hwp_spec_errata.md` 같은 canonical 문서로 승격한다.
- 반복 가능한 증상, 확정 원인, 대응과 검증 방법은 `mydocs/troubleshootings/`에 정리한다.
- 이 문서를 새 구현의 유일한 근거로 사용하지 말고 관련 canonical 문서와 이슈 상태를 함께 확인한다.

## 현재 이슈 묶음

- [Issue #101 부분 표 흐름 조사](issue-101/README.md)
- [Issue #112-115 ThorVG PoC 조사](issue-112/README.md)
- [Issue #124 캔버스·폰트 측정 조사](issue-124/README.md)
- [Issue #139 수식 입력·레이아웃 조사](issue-139/README.md)
- [Issue #257 text-align 시각 비교](issue-257/README.md)
- [Issue #310 LineSeg vpos 조사](issue-310/README.md)
- [Issue #397 증분 레이아웃 조사](issue-397/README.md)
- [Issue #511 Document IR wrap 조사](issue-511/README.md)
- [Issue #516 다층 렌더링 후보 조사](issue-516/README.md)
- [Issue #1151 picture TAC 조사](issue-1151/README.md)
- [Issue #1224 폰트 충실도 측정](issue-1224/README.md)
- [Issue #1236 미주 줄간격 조사](issue-1236/README.md)
- [Issue #1238 미주 간격 조사](issue-1238/README.md)
- [Issue #1239 미주 수식 줄 조사](issue-1239/README.md)
- [Issue #1246 미주 vpos 조사](issue-1246/README.md)
- [Issue #1248 trailing 모델 조사](issue-1248/README.md)
- [Issue #1251 OLE chart 시각 차이 조사](issue-1251/README.md)
- [Issue #1414 Document IR 구조 감사](issue-1414/README.md)
- [Issue #1472 HWP3 variant indent 조사](issue-1472/README.md)
- [Issue #1589 HWPX 페이지 붕괴 조사](issue-1589/README.md)
- [Issue #1584 이후 HWPX 잔여 IR 차이 조사](issue-1584/README.md)
- [Issue #1600 렌더링 -1쪽 갭 조사](issue-1600/README.md)
- [Issue #1658 페이지네이션 조사](issue-1658/README.md)
- [Issue #1370 A3 발산 조사](issue-1370/README.md)
- [Issue #1772 잔여 OVER 조사](issue-1772/README.md)
- [Issue #1773 record-only 인코딩 조사](issue-1773/README.md)
- [Issue #1883 코드 품질 재진단](issue-1883/README.md)
- [Issue #1904 리팩터링 baseline 조사](issue-1904/README.md)
- [Issue #2004 부동개체 계열 조사](issue-2004/README.md)
- [Issue #2023 프론트엔드 조사](issue-2023/README.md)
- [Issue #2124 프론트엔드 기준선 조사](issue-2124/README.md)
- [Issue #2125 font ownership 조사](issue-2125/README.md)
- [Issue #2392 picture props apply 계약 조사](issue-2392/README.md)

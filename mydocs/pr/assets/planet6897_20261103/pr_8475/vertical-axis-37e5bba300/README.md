# PR #8475 세로쓰기 합성 줄 보정의 검증 상태

제품 source `37e5bba3007614d9304c3dbf863503a8c7340405`. 기록 시각 2026-11-05T00:23:33.860898+00:00.

| 입력 | 전체 쪽수 | Native 전쪽 최저 raw | fresh WASM 전쪽 최저 raw |
| --- | ---: | ---: | ---: |
| `samples/issue8470/vertical_fit.hwpx` | 5 | 100.0% | 100.0% |
| `samples/issue8470/vertical_oversize.hwpx` | 5 | 100.0% | 100.0% |
| `samples/issue8470/42565_입법예고문.hwp` | 16 | 94.92448% | 96.96022% |
| `mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_a.hwpx` | 1 | 100.0% | 100.0% |
| `mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_b.hwpx` | 1 | 100.0% | 100.0% |
| `samples/issue6029/3200477_icao_procedure.hwpx` | 1 | 97.97401% | 98.74661% |

두 경로 각각 29쪽을 완료했고 두 문제 입력의 각 5쪽은 모두 100.00%다. 두 문제 쪽만 개선됐으며 나머지 27쪽은 해당 경로의 c183 후보와 PNG 바이트가 동일하다. 실루엣은 96 DPI, Print profile, 색상 경계 보정 전 2px raw다.

[Native 전쪽 TSV·독립 입력/PDF SHA·제품 명령](native/README.md)와 [최적화 fresh WASM 전쪽 TSV·제품 명령](wasm/README.md)을 구분한다. 두 문제 쪽·실물 10/11쪽·저장 줄 정상 대조 1쪽의 review와 별도 overlay도 양쪽 모두 읽고 [직접 판독](../../direct-reading-current.md)에 기록했다.

보정 규칙은 원본 저장 셀 높이·상하 여백을 세로 글 진행 축으로 사용하고, 원본 LineSeg가 없는 내부 합성 줄만 최종 칸 높이에 맞춰 구성하는 것이다. 로드·명시적 재조판·중첩 셀의 프레임과 일반/부분 표의 실제 배치가 공유한다. 유효한 원본 저장 줄과 가로쓰기의 기존 계산은 보존한다.

같은 검출 함수의 [수정 전 2 FAIL·수정 후 2 PASS 및 기존 대조 5 PASS](../vertical-axis-focused-37e5bba300.md)를 확인했다. 설치 OTF가 없는 공개 기본 글꼴 5개 환경을 사용했다.

실물의 기존 저장 세로줄 간격, 정상 대조의 화살표 기호 등에는 이전부터 차이가 남는다. 해당 PNG가 c183와 같음을 확인했으며 이번 합성 줄 누락 복원을 원본 전체의 완전 일치로 확대하지 않는다.

## 아직 남은 통합 완료 조건

- 이 source의 최종 전체 Rust·Native Skia·CI integration 묶음은 실행 중이다. 기존 c183 전체 통과를 이 소스의 통과로 쓰지 않는다.
- [이 source의 필수 Native 편람 383쪽 게이트](handbook-validation.md)는 통과했다. 최저 99.96%, 기준 하락·fallback 0건이다.
- ROOT c183의 기존 PR #8389·#8397 보호 범위 fresh WASM 대조도 진행 중이다. Native는 원 TSV 2,153행 표시값 하락 0행이다.
- ROOT의 고정 측정 종료 뒤 이 보정을 통합 브랜치에 적용하고 source 동일성·최신 base·개별 리뷰·PR 본문을 최종 확인해야 한다.
- 이 기록은 승인·PR 생성·원격 push·comment·close·merge를 뜻하지 않는다.

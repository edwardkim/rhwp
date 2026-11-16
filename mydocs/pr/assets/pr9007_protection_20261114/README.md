# PR #9007 보호 검증 환경 복원과 최종 결과

2026-11-14 KST. 제품 head `abc4c1e82c34e4a112473c32886703b8356089d9`는 변경하지 않았다.
로컬 producer `4d42913568e60565b26d1270739484dd57864714`는 그 위의 검토 문서 전용 commit이다.
Native는 같은 제품 소스에서 다시 빌드했고, 최적화 fresh WASM은 앞선 같은 제품 head의 패키지를 사용했다.
[생산 해시와 최종 집계](summary.json), [문서별 입력·PDF·실제 공급 프로그램](datasets.json).

**18개 입력 × Native/fresh WASM, 3,542쪽 모두 통과.** 입력/PDF SHA·실제 양쪽 쪽수·전쪽 범위를 확인했다.
공식 두 자리 `silhouette_raw_match_percent` 하락 0, 누락 0, 최저 98.09%다.
[페이지별 정본 대조](comparison.tsv)와 `tsv/`의 36개 전쪽 원본을 보존했다.
[별도 편람 HWPX Native 383쪽 gate](handbook-gate.json)도 통과했다.
정본 TSV SHA `ee41455cf7f05585487d92961ccb9fd8160d5d1e15c3567019ea89e2b7bf2aeb`,
비교 PDF·2px raw 규칙·96 DPI·Print profile·compat·판정 허용치는 변경하지 않았다.

## 원인과 복원

[초기 재검토](../pr_9007_recheck_20261114/README.md)의 Native 78쪽·WASM 87쪽 하락은
이번 PR의 신규 결함 수가 아니었다. 아래 실제 공급 조건을 복원했고 제품 보정은 추가하지 않았다.

| 범위 | 확인한 원인과 처리 |
| --- | --- |
| sample16 본문·WMF | 문서별 한컴 2020/2024 대체 환경과 원 가족·스타일 프로그램을 공급했다. Native 적재 한도에 맞는 문서별 목록을 사용하며 WMF에도 필요한 GulimChe 등을 전달했다. 18쪽 99.99%, 35쪽 100.00%로 정본을 충족한다. |
| 구형 분수·산술 | 독립 86문자 표본을 한컴 2020 Print로 출력해 canonical 도구로 77개 프로그램을 복원했다. #8896 수용본의 77개 glyph 프로그램 해시와 모두 일치한다. HyhwpEQ 원 TTF도 양 경로에 명시 공급했다. |
| sample16 네모·본문 간격 | 제공된 Mac HBATANG에는 원 PUA가 없었다. #8946의 선행 해결을 추적해 독립 라벨 92문자 표본을 새로 Print했다. 원 윤곽·전진이 수용본과 92개 모두 일치하며 U+F03C5 공급 후 22·24·25쪽 및 2022의 네모가 복원됐다. |
| sample10 삼각형 | 독립 한양중고딕 보통/굵게 표본의 Type3 프로그램이 #8946 수용본과 각각 바이트 단위로 일치했다. 원 보통 윤곽·전진과 기존 공통 굵기 처리를 사용했다. 81·351쪽 양 경로 100.00%다. |
| sample10 연결 그림 | `samples/oracle.gif`, `samples/rdb02.gif`, `samples/s1.jpg`를 WASM에 명시 공급했다. Native의 파일 자동 탐색과 달리 브라우저에는 별도 전달이 필요했다. |
| 중첩 표 | 수용된 명시 환경·원 가족 그룹, 공개 Noto Serif KR 프로그램, GulimChe의 화살표 및 원 PUA 결측 문자 공급을 복원했다. 17쪽 양 경로 하락 0이다. |
| 영월 서식 | 자동 공개 대체 대신 앞선 동일 PR 실행 영수증의 New Gulim·Gungsuh 원 프로그램을 공급했다. 4쪽 양 경로 100.00%다. |
| SEO 서식 | Native가 사용하는 macOS Helvetica를 WASM에도 전달했다. 원 프로그램 SHA는 #8946 수용본과 같다. WASM 99.25%로 보호 98.79%를 충족한다. |

선행 근거는 [#8896 공급 복원](../pr8896/font-supply/README.md) 및
[#8946 단계 6의 독립 문자 공급](https://github.com/edwardkim/rhwp/blob/9e2794796787b46815462fcecd1536ab74ac8528/mydocs/working/task_m100_8946_stage6.md)이다.
모든 최종 결과는 문서·backend의 **전체 데이터셋**을 선택했다. 서로 다른 실행의 쪽별 최댓값을 조합하지 않았다.
초기 HYHWPEQ 인자 누락 실행과 자동 글꼴 탐색 진단은 최종 결과에 섞지 않았다.

## 독립 프로그램 출처

- 구형 수식: 작업 `375fa7e8-58d8-44b2-bd37-3f729e34268b`, [프로그램 해시](legacy-equation-provenance.json).
- 라벨 92문자 PUA: 작업 `71be6dba-65d5-4d7d-9454-dcb8d9cc3910`, [문자·원 윤곽·전진 해시](pua-provenance.json).
- 한양중고딕 보통/굵게: 작업 `891d4f27-405d-42f7-80b7-f85aaa9d8aa0`, [프로그램 해시](triangle-provenance.json).
- 모두 한컴 2020 `11.0.0.9136`, Print method 0, one-up. 대상 문서 PDF에서 글꼴을 만들지 않았다.
  PUA는 독립 라벨에 따른 cmap·이름·필수 SFNT 메타데이터만 준비하고 원 glyf·hmtx·em·hinting을 유지했다.
  삼각형·구형 수식도 원 윤곽·전진을 이동·확대·보정하지 않았다.
- 원본 상용 글꼴·복원 글꼴·표본 PDF·인증 정보는 로컬 검증에만 사용했다. 저장소에는 프로그램 자체나 윤곽 원문을 넣지 않았다.

## 직접 판독

`direct-review/`는 동일 페이지·영역의 독립 Print / Native / fresh WASM 순서다.
분수의 분자·분모·괄호, 네모·원형·삼각형·화살표, 본문 오른쪽 끝, WMF 글자·후속 본문을 대조했다.
[fraction-detail](direct-review/fraction-detail.png)과 [line-end-detail](direct-review/line-end-detail.png)은
96 DPI raster를 2배 확대한 판독이며 고해상도 재측정으로 표시하지 않는다.
raw 통과를 RGB 완전 동일로 해석하지 않는다. WMF의 배경 농도·안티앨리어싱 등 기존 차이는 남아 있으나
기호 소속·내용·줄 끝·표 외곽 및 후속 본문을 확인했고 보호 기준을 유지했다.

[무늬 비교판](direct-review/fill-patterns-source-head.png)은 앞선 동일 제품 head의 커밋된 세 PNG를 잘라
정리한 것이다. 새로 렌더한 것으로 표시하지 않는다. 무늬 0~5 방향·격자와 무늬 없음의 기존 검증을 재사용한다.

## 재현 조건

정본 `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat <문서 세대>`와 Native `--rhwp-bin`,
WASM `--wasm-pkg`를 사용했다. 입력/PDF·실제 프로그램 SHA와 명시 환경은 `datasets.json`과 `environments/`에 있다.
문서별 `--font-path`는 해당 실제 가족·스타일을 하나의 폴더에 모아 공급하며, sample10 WASM에는 위 세
`--linked-image`를 전달한다. 별도 편람은 `--handbook-gate`로 실행했다.
동일 프로그램이 없는 환경에서 가족 이름만 맞춰 통과를 가정하지 않는다.

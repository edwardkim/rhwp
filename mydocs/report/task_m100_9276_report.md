# #9276 Windows 직접 PDF 내보내기 검증

Issue: #9276

Windows 11 x64에서 조달청 HWPX 세 개를 일반 명령으로 내보냈다. 수정한 제품 커밋은 `9e1d1cd948`, 비교한 devel은 `bd3cd17133`이다. 신고 환경은 24H2, 메인테이너 검증 환경은 26H2 빌드 26300.9974다. GitHub 담당자는 `edwardkim`으로 지정했다. [이슈 #9276](https://github.com/edwardkim/rhwp/issues/9276)

## 실행 시간과 파일 크기

아래 표에서 공식 배포판 두 버전의 수치는 각 한 번의 측정값이다. devel과 수정판은 Rust 1.93.1 MSVC의 같은 `release` 프로필로 빌드한 뒤 예열한 웹폰트 캐시를 사용했다. 각 실행은 새 Windows 프로세스에서 시작했다. 각 원본을 세 번씩 두 빌드에서 번갈아 실행해 경과 시간의 중앙값을 구했다. 이때 같은 호스트에서 WASM 빌드가 병행됐다. [각 실행의 경과·CPU 시간, 압축 글꼴 바이트, 실행 파일과 PDF의 SHA-256](../tech/investigations/issue9276/windows-runs.tsv)

| 원본 | 쪽 | v0.8.7 | v0.9.1 | devel | 수정판 | PDF 바이트, devel → 수정판 |
|---|---:|---:|---:|---:|---:|---:|
| 공고문 | 11 | 0.86초 | 24.09초 | 25.27초 | 3.94초 | 10,933,271 → 449,074 |
| 모집 안내문 | 7 | 1.37초 | 30.34초 | 37.84초 | 3.79초 | 798,387 → 411,951 |
| 추천 서식 | 1 | 0.10초 | 3.09초 | 5.05초 | 4.45초 | 49,636 → 49,636 |

공고문은 공식 v0.9.1에서 14,436,357바이트였다. devel에서 10,615,665바이트였던 압축 글꼴 스트림이 수정 후 131,485바이트가 됐다. 세 원본에서 수정 전후의 글꼴 환경과 실제 공급 목록이 각각 일치했다. 설치되지 않은 글꼴은 내보내기 결과에 그대로 보고된다. [PDF에 포함된 글꼴별 바이트](../tech/investigations/issue9276/pdf-fonts.tsv)

1쪽 추천 서식은 다른 빌드와 시각 검사를 멈춘 Windows에서 네 버전의 실행 순서를 바꿔 각각 네 번 더 측정했다. 중앙값은 v0.8.7 0.10초, v0.9.1 3.15초, devel 3.09초, 수정판 2.76초였다. 수정판에서도 설치 글꼴 확인과 대체 글꼴 준비 비용이 남는다. [단독 실행 16회 측정값](../tech/investigations/issue9276/short-form-times.tsv)

## 두 원인과 보정

1. 공고문에서는 같은 Windows 설치 글꼴이 TrueType인지 4,386번 확인하며 20.72초를 썼고, 모집 안내문은 5,627번에 26.47초가 들었다. 이제 한 카탈로그 안에서 같은 계열·굵기·너비·기울기의 조회 결과를 재사용하고, 이름 조회는 해당 얼굴의 이름 테이블만 읽는다.
2. 일부 한양·굴림 프로그램은 글리프 치환 테이블 GSUB의 주 버전이 0이었다. Skia에 포함된 HarfBuzz가 부분 추출을 마치지 못하면 PDF에 원본 글꼴 전체가 들어갔다. PDF 출력 시 부분 임베딩을 허용한 정적 TrueType 얼굴에서 해당 무효 테이블을 제외해 Skia가 정상적으로 부분 추출하도록 했다. 원본 프로그램의 임베딩 권한, 사용 글리프 ID와 메트릭을 검사했다. [계측·HarfBuzz 대조와 제약](../working/task_m100_9276_implementation.md)

## 출력과 회귀 검증

공고문은 한컴 2020, 모집 안내문과 추천 서식은 한컴 2024에서 원본을 직접 인쇄했다. 두 공고의 발행기관 PDF도 대조했다. 같은 PDFium에서 96 DPI로 그린 Windows 직접 출력 19쪽의 RGB 픽셀과 쪽별 추출 텍스트가 수정 전후 모두 같았다. 공고문의 직위 표에서 열 너비와 행 안의 한글·숫자 위치를 직접 대조했다. 모집 안내문 첫 문단과 추천 서식의 항목명은 한컴 인쇄본과 획 모양·굵기 차이가 있으며, 같은 영역의 Windows 수정 전후 출력은 일치한다. [원본·독립 기준 해시](../tech/investigations/issue9276/inputs.sha256) · [19쪽 측정표와 텍스트 해시](../tech/investigations/issue9276/pages.tsv)

| 원본 | Windows PDF 대 한컴 Print 최저 | fresh WASM 대 한컴 Print 최저 |
|---|---:|---:|
| 공고문 | 98.00% | 98.28% |
| 모집 안내문 | 93.23% | 93.38% |
| 추천 서식 | 93.03% | 93.61% |

### 원본 지정 글꼴을 공급한 대조

위 표는 Windows에서 별도 글꼴 경로를 주지 않고 실행한 기본 출력이다. 같은 문서에 `--font-path`로 휴먼명조, 한컴 윤고딕 240, 함초롬 및 Wingdings 등 로컬 글꼴을 명시해 다시 내보냈다. Windows가 기본 환경에서 Noto Sans KR로 대체하던 휴먼명조를 이 실행에서는 휴먼명조 프로그램으로 선택했다. 한컴 인쇄본·페이지·DPI와 비교 방법은 두 실험 모두 같다. [공급 파일 이름과 해시](../tech/investigations/issue9276/print-fonts.tsv)

| 원본 | Windows 직접 PDF 최저 | Native Skia PNG 최저 | fresh WASM 최저 |
|---|---:|---:|---:|
| 공고문 | 99.77% | 99.76% | 99.60% |
| 모집 안내문 | 99.66% | 99.69% | 99.68% |
| 추천 서식 | 99.70% | 99.97% | 99.97% |

세 문서 19쪽 모두 Windows 직접 PDF의 수정 전후 픽셀과 추출 텍스트가 같았다. 표와 제목·본문을 같은 위치에서 확대해 글꼴 공급에 따른 차이를 확인했다. [전쪽 측정·변경 픽셀](../tech/investigations/issue9276/print-font-pages.tsv) · [모집 안내문](../tech/investigations/issue9276/guide-print-font-control.png) · [추천 서식](../tech/investigations/issue9276/form-print-font-control.png)

fresh WASM은 기준과 후보의 제품 코드를 각각 다시 컴파일·최적화하고 같은 Chrome for Testing 156.0.8078.47에서 전쪽을 그렸다. 두 빌드의 Canvas에서 바뀐 픽셀은 0개다. Windows PNG 첫 대조에서는 공고문 5쪽의 안티앨리어싱 픽셀 4개가 달랐다. 같은 기준 실행 파일에서도 이 4개 값이 달라졌고, 이어서 두 번 실행한 기준판과 수정판의 해당 쪽은 각각 같았다. PDF 내보내기에서는 이 차이가 나타나지 않았다. [공고문 표](../tech/investigations/issue9276/announce-font-detail.png) · [모집 안내문 글자](../tech/investigations/issue9276/guide-font-detail.png) · [추천 서식 항목명](../tech/investigations/issue9276/form-font-detail.png)

공고문 11쪽은 화면용 프로필, 빠른 미리 보기 프로필, 기존 글꼴 환경 재사용 경로에서도 다시 내보냈다. 각 모드의 devel과 수정판을 대조한 33쪽에서 RGB 픽셀과 추출 텍스트가 같았다. [모드별 전쪽 결과](../tech/investigations/issue9276/windows-profiles.tsv)

Windows Native PNG, Windows 직접 PDF를 래스터화한 결과, fresh WASM Canvas에서 각각 측정한 원본별 전쪽 TSV를 보존했다. [Windows Native PNG](../tech/investigations/issue9276/target-pages/windows-native) · [Windows 직접 PDF](../tech/investigations/issue9276/target-pages/windows-pdf) · [fresh WASM Canvas](../tech/investigations/issue9276/target-pages/wasm)

Windows와 Linux의 원형 그라데이션 대조군 6쪽은 각 쪽에서 PDF 방사형 셰이딩을 사용했고 한컴 Print 대비 모두 100.00%였다. Windows에서는 수정 전후 픽셀도 같았다. [그라데이션 전쪽 결과](../tech/investigations/issue9276/gradient.tsv)

최소 글꼴 환경의 전체 회귀 13,948건, Native Skia 공식 검사 4,072건(직접 PDF 37건 포함)이 통과했다. 부분 임베딩이 가능한 글꼴과 금지된 글꼴을 각각 실제 PDF로 내보내 한글·영문 추출, 글리프 경계와 전진폭, 금지 시 전체 프로그램의 바이트를 확인했다. 기여자 공통 보호 문서 4종은 Native와 fresh WASM에서 8개 조합이 통과했다. Native 편람 383쪽 최저는 99.97%였다. 정본 보호 23종·4,046행을 대조해 하락 0건을 확인했다. [검사 결과와 빌드 식별 값](../tech/investigations/issue9276/validation.json) · [보호 문서 쪽별 결과](../tech/investigations/issue9276/protected-pages.tsv) · [입력·인쇄본·실제 공급 글꼴 해시](../tech/investigations/issue9276/protected-conditions.tsv)

동일 devel·WASM·글꼴을 사용해 Windows PDF와 별도로 Linux 브라우저의 인쇄 경로를 대조했다. 같은 버전의 Chrome Headless Shell에서는 보호 문서 시행령 1·54쪽의 실루엣이 각각 99.99%·99.87%였고, 일반 Chrome for Testing에서는 두 쪽 모두 100.00%였다. 정식 비교에는 두 빌드 모두 일반 Chrome for Testing을 사용했다. [브라우저 실행 파일 해시와 대조값](../tech/investigations/issue9276/browser-control.tsv)

## 같은 입력으로 실행하기

Windows PowerShell에서 Native Skia를 활성화한 release CLI로 다음 명령을 실행한다. 각 원본의 쪽수와 결과 바이트는 위 표에서 확인할 수 있다.

```powershell
rhwp export-pdf tests/fixtures/issue9276-windows-direct-pdf/announce.hwpx --backend direct -o announce.pdf --json
```

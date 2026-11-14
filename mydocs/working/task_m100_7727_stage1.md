---
kind: working
status: active
canonical: mydocs/working/task_m100_7727_stage1.md
last_verified: 2026-11-14
---

# #7727·#7729·#7730·#7731·#7732 원본 피델리티 분석과 보정

갱신: 2026-11-14T23:54:10+09:00.

Issue: #7727, #7729, #7730, #7731, #7732.

[분석·검증 증적 목차](../pr/assets/issues7727_7732_20261114/README.md)에서 최신 전쪽 TSV,
원인별 근거, 같은 쪽의 review·overlay를 바로 찾을 수 있다. 이 보고서 앞부분은 현재 판정,
`이전`·`이력`으로 표시한 부분은 해당 보정 단계의 관측이다.

## 목표와 현재 판정

다섯 이슈의 실제 원본을 같은 독립 한컴 Print PDF와 대조하여 **모든 쪽 98% 이상**으로
복구하고, 가능한 차이는 100%까지 개선한다. 98% 이상으로 검증된 범위에는 독립 근거에
맞는 회귀 검사를 추가한다. 미달 쪽을 검사에서 빼거나 기준 PDF·글꼴·일치율 판정을 바꾸지 않는다.

브랜치 `task/7727-related-fidelity`, 고정 base/parent는
`53da4d2da28ae0cacf8feac3a989cc6a69fb2622`다. 현재 구현·검증이 진행 중이며 PR 제출 준비가
완료되지 않았다. 아래 수치는 해당 생산자에 한정한다. 뒤의 변경으로 앞선 전쪽 결과를
새 후보의 결과로 바꾸어 부르지 않는다.

## 최신 번호 문단 보정: 원본 전 389쪽의 98% 목표 충족

parent `fd9207f300` 뒤의 번호 문단 후보는 Native·fresh WASM **각 389쪽**을 모두 다시
출력했다. CBTA 96쪽은 **97.00/96.99% → 99.72/99.74%**로 개선됐다.
원본 389쪽 중 Native는 이 한 쪽만 상승하고 나머지 388쪽의 공식 표시값은 같다.
입력·독립 Print PDF·글꼴 조건·DPI·판정 문턱과 원본 쪽수는 유지했다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 98% 미만 |
| --- | ---: | ---: | ---: | --- |
| CBTA | 204 | 98.11% (2쪽) | 98.08% (2쪽) | 없음 |
| 권고안 | 22 | 98.69% (7쪽) | 98.68% (7쪽) | 없음 |
| 영어 시험지 | 8 | 99.49% (3쪽) | 99.49% (3쪽) | 없음 |
| API | 105 | 99.55% (97쪽) | 99.55% (97쪽) | 없음 |
| 각주 표 | 50 | 98.58% (37쪽) | 98.58% (37쪽) | 없음 |

현재 번호·본문 시작을 반영하는 공통 채움은 두 줄을 유지하면서도 `교육훈련` 뒤로
문자 경계를 옮길 수 있다. 기존 경로는 마커 높이가 이미 맞고 줄 수가 같다는 이유로
이 결과를 버렸다. 같은 수·높이의 줄은 새 문자 경계와 **원본의 세로 슬롯 원점**을 함께
발행하도록 했다. 번호 없는 문단·명시적 개행·다른 물리 프레임·다른 높이는 이 가로 캐시
보정으로 재해석하지 않는다. 기존 HCI Poppy 폭 표와 글꼴 프로그램 선택도 바꾸지 않았다.

설치 글꼴 없이 검출하는 새 관계형 반례는 보정 전 **1 PASS / 1 FAIL**, 보정 후 **2 PASS**다.
정식 suite의 Mac 최소 글꼴 검증은 기존 번호 문단·내어쓰기·HCI Poppy 계약과 이 브랜치의
핵심 검사를 합해 **23 PASS / 0 FAIL**, 4.110초다. 선택 밖 2,778건이며 전체 회귀가 아니다.
CBTA 118·162·166쪽은 각각 양쪽 98.44%·99.94%·99.95%로 자격을 확인했으므로,
독립 PDF 그림 테두리 네 변과 종료 행의 닫는 선을 **±2 CSSpx**로 추가 보호했다.
기존 소유·높이 관계 검사도 유지했다. 이 고정 좌표 검사 역시 위 최소 글꼴 실행에서 통과했다.

[전쪽 TSV·생산자·회귀·독립 좌표](../pr/assets/issues7727_7732_20261114/marker-partition/cause.json),
96쪽 Native [review](../pr/assets/issues7727_7732_20261114/marker-partition/native-p096-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/marker-partition/native-p096-overlay.png),
fresh WASM [review](../pr/assets/issues7727_7732_20261114/marker-partition/wasm-p096-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/marker-partition/wasm-p096-overlay.png)를 함께 확인한다.

98% 목표는 이 원본 범위에서 충족했지만 모든 쪽이 100%라는 뜻은 아니다. CBTA 2쪽의
목차 쪽번호에는 저장된 명시적 RIGHT 탭 전진과 현재 정지점 재계산 차이가 남아 있어 추가
원인을 확인한다. 관측한 탭 확장은 `[1868,0,515,32,32,32,9]`, 현재 정지점은
642.5333 CSSpx이며 실제 PDF 쪽번호가 더 앞에 있다. 이 차이를 글꼴 불가능 예외로
단정하지 않는다. 전체 회귀·필수 lint/build·정본 보호 TSV·편람·실제 backend 검증도
남아 있으므로 PR 제출 준비 완료로 판정하지 않는다.

## 이전 분석: 권고안 4쪽의 배경 채널 보간

parent `6197d8e680` 뒤의 후보는 Native **원본 389쪽과 독립·기존 대조군 12쪽, 총 401쪽**을
모두 출력했다. 권고안 4쪽은 **97.44% → 99.96%**, 전 22쪽 최저는 **98.69% (7쪽)**다.
새 양방향 색상 표본 6쪽과 기존 원형 그러데이션 표본 6쪽은 각각 전쪽 **100.00%**다.
CBTA 전 204쪽은 96쪽 **97.00%**만 목표 미달이며, 영어 시험지 8쪽·API 105쪽·각주 표
50쪽의 최저는 각각 **99.49%·99.55%·98.58%**다. 새 fresh WASM도 원본 389쪽과 대조군 12쪽을 모두 완료했다. 권고안 4쪽은 **99.96%**,
전 22쪽 최저 **98.68% (7쪽)**, 두 대조군 각 6쪽은 **모두 100.00%**다. CBTA는
96쪽 **96.99%**, 나머지 세 원본 최저는 Native와 같은 **99.49%·99.55%·98.58%**다.
아래 완료 행 단계의 WASM 수치는 이전 생산자이므로 이 단계와 구분한다.

권고안의 raw 미일치 4,821픽셀 중 4,748픽셀이 색상 경계 보정에서 해소됐다. 이를 통과
근거로 쓰지 않고 원본 배경과 PDF의 동심원 fill을 대조했다. 공통 `lerp_color`가 최종
RGB 값을 반올림하여 회색 띠를 1단계 어둡게 만드는 것이 실제 원인이었다. 원본 글꼴을
명시한 대조군의 결과가 같으므로 글꼴 교체로 이 차이가 해결된다는 가설은 기각했다.

기존 정상 원형 그러데이션 표본의 크기·셀·내용을 유지하고 RGB 방향과 7/50/255단계만
바꾼 [독립 HWPX](../../tests/fixtures/issue7729/gradient-band-color-probe.hwpx)를 만들었다.
[한컴 2020 Print PDF](../../pdf/issue7729/gradient-band-color-probe-2020.pdf)의 MCP job은
`6c8ae335-f880-41ed-9f83-0687d078a1c4`, 제품은 **11.0.0.9136**, Print method는 **0**이며
6쪽·52,675바이트다. PDF 저장/내보내기나 웹한글기안기 출력이 아니다.

독립 PDF에서 210→40 채널의 1/6 지점은 **182**, 역방향 40→210은 **68**이었다.
따라서 절대값을 일괄 올림하거나 버리는 규칙이 아니라, 시작색에 대한 **부호 있는 변화량의
소수부를 0 방향으로 버린 뒤 더하는 규칙**으로 공통 보간을 수정했다. 띠 수·위치·표 크기와
판정 문턱은 바꾸지 않았다. 중앙의 같은 색 원 두 개는 같은 띠이므로 띠 수를 늘리지 않았다.

새 고정형 RGB 검사 2개는 독립 PDF의 실제 fill을 기대값으로 사용하며, 독립 rustc 실행에서
**보정 전 2 FAIL → 보정 후 2 PASS**다. px 좌표를 고정한 검사가 아니다. 정식 파생 suite의 최소 글꼴 Native Skia 검증도 기존 10개를 포함해 **12 PASS / 0 FAIL**,
3.385초로 완료했다. 선택 밖 1,707건이며 전체 회귀가 아니다. 처음 nextest의 `-j`와
`--test-threads` 중복 지정은 실행 전 오류였고, 빌드 잡은 환경변수 8개로 분리해 재실행했다.

[생산자·MCP 영수증·전쪽 Native TSV](../pr/assets/issues7727_7732_20261114/gradient-channel/cause.json)와
4쪽 [review](../pr/assets/issues7727_7732_20261114/gradient-channel/native-p004-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/gradient-channel/native-p004-overlay.png)를 함께 보존했다.
분석의 다음 대상은 CBTA 96쪽의 제목 줄바꿈이다. `ICAO` 전진은 실제 Native에서 약54.5px이며
독립 PDF의 뒤따르는 한글 시작점은 약2.42px 앞선다. 설치된 HCI Poppy의 보통/굵은 면 폭은
각52.10/54.42px이나, 이것만으로 굵은 면 선택이 원인이라고 확정하지 않는다. CLI와 같은
글꼴 공급·OTF 측정 준비를 적용한 뒤 실제 줄 구성과 글꼴 프로그램을 확인한다.

Native·fresh WASM의 같은 쪽 [review](../pr/assets/issues7727_7732_20261114/gradient-channel/wasm-p004-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/gradient-channel/wasm-p004-overlay.png)를 함께 판독한다.

## CBTA 96쪽 보정 전 원인 분리 기록 (이력)

CLI와 같은 문서 글꼴 환경을 적용하고 `prepare_declared_font_fallbacks`·
`prepare_native_pua_fonts`·`prepare_native_otf_fonts`를 마친 실제 Print layer와 측정 trace를
확인했다. `ICAO`는 HCI Poppy의 굵은 전진으로 **54.4533 CSSpx**이고, 뒤 한글은
휴먼명조의 **1em** 전진이다. 늘어난 단어 공백은 약 **4.1644px**의 양쪽 정렬 배분이다.
이 결과만으로 한글 글리프에 무조건 5%가 더해진다고 해석하면 안 된다.

독립 PDF의 첫 `ICAO` 원점과 뒤 한글 원점 차이는 **52.16 CSSpx**다. PDF의 F4 사전은
Type0/Identity-H이며 `/BaseFont /HCI Poppy`의 공백이 escape되지 않아 MuPDF의 일반
글꼴 추출이 실패한다. 이 오류를 두부 글자나 Type3 폭으로 오해하지 않고 원본 content의
600DPI 변환 행렬·TJ와 한글 시작점을 읽었다. 원본 PDF 바이트는 변경하지 않았다.

현재 저장 줄은 `교육훈` / `련 단계 간 비교`, Print는 `교육훈련` / `단계 간 비교`다.
원 해결 계약 `fa7aefdf10`(#8105)은 번호 문단의 낱말 꼬리를 현재 프레임에서 다시 채우지만,
현재 공통 soft-wrap 투영은 일반적으로 **줄 수 감소**를 수용 조건으로 사용한다.
출력용 진단 코어에서 해당 문단만 정식 `reflow_body_paragraph_range_native`로 다시 채우면
다음 줄 시작이 원본 **34 → 36**으로 옮겨 `교육훈련` 뒤 공백까지 첫 줄이 소유한다.
원본 파일과 제품 코드는 바꾸지 않은 원인 분리용 실행이며 해결 결과가 아니다.
이번에는 두 줄을 유지하면서 경계의 한 글자가 이동하므로, 글꼴 차이와 이 캐시 수용 조건을
분리하여 확인해야 한다. 이는 아직 확정 원인이나 보정 완료가 아니다.

HCI Poppy의 원본 굵은 폭 표는 `17225375ff`의 별도 HFT·Print 근거와 기존 회귀가 보호한다.
이번 PDF의 두 원점만으로 그 표를 일반체 폭으로 일괄 바꾸지 않는다. 같은 프레임·번호·
언어 슬롯의 독립 표본에서 줄 경계와 실제 문자 전진을 확인한 뒤 공통 소비 경로를 보정한다.
[글꼴 준비 후 관측과 PDF 원문 근거](../pr/assets/issues7727_7732_20261114/cbta-heading-investigation.json)를
분석 자료로 보존했다. 실행용 Rust 진단기와 전체 trace는 `output/`에만 둔다.

## 이전 완료 행 보정과 실제 검증 범위

parent `9b07db2002` 뒤의 완료 행 보정으로 Native·fresh WASM **각389쪽**을 모두 다시
측정했다. 원본/PDF·정본 도구·96DPI·compat2022·raw2px·문턱32·정상 설치 글꼴은 유지했다.
Native의 CBTA162쪽은96.35%→99.94%,166쪽은96.76%→99.95%이며 fresh WASM도
각각99.94%·99.95%다. 두 경로 모두 원본 쪽수를 유지했다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 남은98% 미만 |
| --- | ---: | ---: | ---: | --- |
| CBTA | 204 | 97.00% (96쪽) | 96.99% (96쪽) | 96쪽 |
| 권고안 | 22 | 97.44% (4쪽) | 97.44% (4쪽) | 4쪽 |
| 영어 시험지 | 8 | 99.49% (3쪽) | 99.49% (3쪽) | 없음 |
| API | 105 | 99.55% (97쪽) | 99.55% (97쪽) | 없음 |
| 각주 표 | 50 | 98.58% (37쪽) | 98.58% (37쪽) | 없음 |

[생산자·전후 원인·전체TSV 색인](../pr/assets/issues7727_7732_20261114/complete-row-band/cause.json)에
Native 바이너리·WASM JS/바이너리·소스 패치·전체10개TSV의 SHA를 고정했다.
이 단계의 CBTA 전204쪽 Native는 이전 전쪽 생산자 대비 하락0쪽이다. 바뀐 세 쪽은
캡션118쪽과 종료 행162·166쪽이며, 나머지201쪽의 공식 표시값은 동일하다.
162쪽 [review](../pr/assets/issues7727_7732_20261114/complete-row-band/native-p162-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/complete-row-band/native-p162-overlay.png),
166쪽 [review](../pr/assets/issues7727_7732_20261114/complete-row-band/native-p166-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/complete-row-band/native-p166-overlay.png)를 직접 판독했다.

새 관계형 종료 행 검사2개는 같은 원본과 Print 경로에서 보정 전2FAIL→보정 후2PASS다.
정식 파생 suite의 최소 글꼴 NativeSkia 집중 검증은 이전8개도 포함해 **10PASS/0FAIL**,2.834초다.
선택 밖1360건이며 전체 회귀가 아니다. suite 재배정 후 처음 세 target만 선택한 실행은
새2개만 실행했으므로10개 결과로 세지 않는다. 다음 실행의 컴파일 격리 오류는 아래에 구분했다.

## 완료 행 소유와 최소 글꼴 검사의 보정

기존 `1147d7b70a2`가 도입한 refit 뒤의 완전 continuation 경로는 경계를 넘는 병합 칸이
하나라도 있으면 남은 물리 쪽 공간을 끝 행에 먼저 합쳤다. CBTA162쪽 행188은 두 원본 줄
2600HU와 안여백282HU로38.4267px를 소유하지만, 이 단계에서45.2933px로 늘어났다.
166쪽 행252도 같은38.4267px 대신44px를 받았다. 뒤의 실제 병합 잔여 계산은 이미 커진
`current`를 하한으로 쓰므로 이 증가를 되돌릴 수 없었다.

이 선행 확장을 제거하고, 실제 원본 선언·내용·앞 조각이 수용한 행 상자에서 몫을 계산하는
기존 `emit::crossing_span_page_end_band`가 쪽 끝 띠를 한 번만 소유하게 했다. 독립 Print의
닫는 선은 각각724.57064/726.00936CSSpx이며 보정 후 표 끝과 내용 소속을 함께 확인했다.
원본 문서 전체가 아직98% 미만이므로 새 검사는 이 절대 좌표를 고정하지 않는다. 같은 행의
모든 단일 칸에서 원본 마지막 줄 끝+상하 안여백의 최댓값과 실제 마지막 행 높이의 관계를
compact tree의 소수 첫째 자리 직렬화 격자에서 확인한다.

새 검사의 첫 초안은166쪽 col3에도 두 줄이 있다고 잘못 가정했다. 실제 두 줄 소유자는
같은 행의 다른 칸이므로 원본 행 전체의 최댓값으로 바로잡았다. 정정한v2에서 두 실제 결함을
모두 재현하고 보정 후 통과했다. 잘못된 초안 실패는 결함 재현으로 인용하지 않는다.

최소 글꼴 검사에서 새 suite가 공개 `HappinessSansVF.ttf`를 `include_bytes!`로 참조해
기존 격리 규칙의 포괄적인 `ttfs` 읽기 차단에 걸렸다. 이 공개 파일 하나의 읽기만 허용한
새 격리 프로필로 다시 검증했다. 사용자·공유 글꼴 및 font service 차단은 유지했고,
CoreText1875→528개·모든 실제 경로OS 영역·한글 기본 글리프 존재를 다시 확인했다.
이는 공개 내장 fixture의 컴파일 허용이며 개인 글꼴의 설치·등록·탐색 허용이 아니다.

## 영어 공백 보정 단계의 측정 기록 (이력)

영어 시험지의 낱말 공백 기준 em을 보정한 생산자는 parent `d04b21df67`과
소스 패치 SHA-256 `406ae228d79ca7f416b9e6817e0a8eb87daaeaf61b471efa52273fd492ae6761`로
고정했다. Native 바이너리·fresh WASM의 SHA와 측정 요약은
[영어 공백 원인·생산자 기록](../pr/assets/issues7727_7732_20261114/half-space-origin/cause.json)에 있다.
Mac WASM은 locked wrapper의 `--no-opt` 로컬 대체 빌드다.

| 문서 | 최신 측정 범위 | Native 최저 | fresh WASM 최저 | 남은 98% 미만 |
| --- | --- | ---: | ---: | --- |
| 영어 시험지 | 전 8쪽 | 99.49% (3쪽) | 99.49% (3쪽) | 없음 |
| API | 전 105쪽 | 99.55% (97쪽) | 99.55% (97쪽) | 없음 |
| 각주 표 | 전 50쪽 | 98.58% (37쪽) | 98.58% (37쪽) | 없음 |
| 권고안 | 전 22쪽 | 97.44% (4쪽) | 97.44% (4쪽) | 4쪽 |
| CBTA | 96·118·162·166·176·182쪽 | 96.35% (162쪽) | 96.35% (162쪽) | 96·162·166쪽 |

최신 생산자에서는 각 경로 191쪽을 측정했다. CBTA 나머지 198쪽은 이 생산자로 아직 다시
측정하지 않았으므로 전 389쪽 최종 수용으로 보고하지 않는다. 영어 시험지 2쪽은 두 경로 모두
99.96%이며, 전 8쪽 TSV 원문은 [Native](../pr/assets/issues7727_7732_20261114/half-space-origin/exam-native-silhouette.tsv)와
[fresh WASM](../pr/assets/issues7727_7732_20261114/half-space-origin/exam-wasm-silhouette.tsv)에 보존했다.
같은 쪽 [review](../pr/assets/issues7727_7732_20261114/half-space-origin/native-p002-review.png)와
[overlay](../pr/assets/issues7727_7732_20261114/half-space-origin/native-p002-overlay.png)를 직접 판독했다.

최소 글꼴 Native Skia 집중 회귀는 **8 PASS / 0 FAIL**, 2.349초다. 세 파생 suite에서
세 원본 검사 파일만 선택했고 1024개는 선택 밖이다. 새 낱말 원점 고정형 검사도 포함한다.
[격리 정책·실행 요약](../pr/assets/issues7727_7732_20261114/half-space-origin/minimal-font-validation.json)에
기존 7개 검사 실행과 구분하여 기록했다. 전체 회귀·보호 문서·필수 편람·실제 소비 경로는
최종 소스에서 별도로 완료해야 한다.

## 원본과 독립 기준

| 이슈 | 원본 | 독립 PDF | 쪽수·실제 출력 제품 |
| --- | --- | --- | --- |
| #7727 | `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` | `pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf` | 204쪽·한컴 2020 |
| #7729 | `samples/issue6844/30269-anticorruption-recommendation-toc.hwp` | `pdf/30269-anticorruption-recommendation-toc-2020.pdf` | 22쪽·한컴 2022 |
| #7730 | `samples/exam_eng.hwp` | `pdf/exam_eng-hwp-2020.pdf` | 8쪽·한컴 2022 |
| #7731 | `mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp` | `mydocs/pr/assets/issue7731/api-reference/hwpctl_API_v2.4-hwp-2020.pdf` | 105쪽·한컴 2022 |
| #7732 | `mydocs/pr/assets/issue7445/issue1937_rowbreak_footnote_overpagination.hwp` | `pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf` | 50쪽·한컴 2020 |

파일명의 `2020`과 실제 제품 버전을 구분했다. 웹한글기안기 출력물을 기준으로 쓰지 않았다.
입력/PDF SHA-256·쪽수·Creator·Producer는 작업 증적의 `cases.json`에 고정했다.
시각 검증 조건은 정상 설치 글꼴, Native PNG/fresh WASM, 96 DPI, compat2022,
공통 Visual Sweep raw 2px, 픽셀 차이 문턱 32, 페이지 작업자 총 8개다.
현재 정본 도구의 PDFium 변환을 사용하며 기준 PDF 바이트는 수정하지 않았다.
회귀 검사 실행의 최소 글꼴 조건과 실물 시각 검증의 원본 글꼴 조건은 구분한다.

## 이전 생산자의 전쪽 결과 (이력)

`closing-span-clean-native` 및 `closing-span-clean-pkg`는 API 종료 프레임·CBTA 종료 병합 칸
보정까지 포함한다. 이후의 캡션 테두리·NBSP 압축·밑줄 변경은 포함하지 않는다.

| 문서 | Native 전체 최저: base → 보정 | fresh WASM 전체 최저 | 보정 후 98% 미만 |
| --- | --- | --- | --- |
| CBTA 204쪽 | 96.23% → 96.35% | 96.35% | 96·118·162·166쪽 |
| 권고안 22쪽 | 87.21% → 97.44% | 97.44% | 4쪽 |
| 영어 시험지 8쪽 | 97.12% → 97.12% | 97.11% | 2쪽 |
| API 105쪽 | 24.28% → 99.55% | 99.55% | 없음 |
| 각주 표 50쪽 | 98.58% → 98.58% | 98.58% | 없음 |

두 경로 모두 389쪽의 쪽수가 일치했다. Native의 383쪽이 98% 이상이며 6쪽이 남았다.
API는 Native 77쪽이 공식 두 자리 표시값 100.00%다. 이 결과는 전체 nextest·보호 문서·
최종 실제 소비 경로의 통과를 뜻하지 않는다.

## 원인별 보정 규칙

### API 12→13쪽: 표 앞 바깥여백 중복 소비

`fragment/budget.rs`에서 저장 host 종료 뒤 재동기화한 현재 커서를 opening 프레임의
원점으로 비교했다. 이미 소비한 바깥 위 여백 3.773px를 다시 차감해 행 소유가 달라졌다.
불변 `anchor_y`와 host 앞 점유로 실제 opening 프레임을 판별한다.
기존 #8148 계열의 저장 프레임 계약을 추적했으며 명시적 다음 프레임과 페이지 컷은 유지한다.

### 권고안 6·22쪽: 단 시작 전 닫힌 인라인 개체 줄

`stored_column_spacing.rs`가 한 줄에 여러 도형이 있거나 앞 문단이 중립 컨트롤만 가지면
닫힌 줄을 인정하지 못했다. 원본 문단 99의 두 도형은 3370/3338HU이며 줄 높이는 3370HU다.
같은 줄의 개체 높이는 합이 아닌 최댓값으로 닫고 바깥여백을 포함한다.
저장된 앞 500HU 간격을 누락하던 경로를 복구했다. 기존 #7642·#8917 구현을 대조했다.
6쪽 87.21%→99.95%, 22쪽 91.73%→99.60%다.

### API 35→36쪽: 빈 첫 줄과 동일 원점 재시작

첫 빈 문단과 다음 코드 문단이 모두 vpos=0이어도 첫 빈 문단의 실제 줄 높이가 소유된다.
선언 프레임 닫힘으로 검증한 경우 빈 문단의 종료 전진을 보존하고,
`stored_empty_successor_geometry`가 그 전진을 다시 지우지 않도록 했다.
초기 후보는 36쪽을 악화시켜 폐기했으며, 정정한 후보의 35·36쪽은 모두 100.00%다.

### 권고안 4쪽: 저장 오른쪽 탭의 선언 프레임

문단의 명시적 RIGHT 탭이 현재 프레임 바깥에 있고 실제 저장 확장 정보가 있을 때
그 저장 목표를 소비한다. 프레임 안쪽 탭·자동 오른쪽 탭의 계약은 유지한다.
`inline_right_tab.rs`에서 처리했으며 97.34%→97.44%다. 잔여 차이가 있어 완료가 아니다.

### API 65쪽: 폭 0 어울림 표 host의 앞·뒤 점유

독립 글줄이 아닌 빈 host에서 문단 앞 간격과 표 바깥여백을 중복 소비했다.
공통 `stored_empty_square_host_uses_paragraph_origin`으로 측정과 paint가 같은 원점을 쓴다.
뒤 간격도 원본 식 `34574−500+141+22784+141=57140HU`로 실제 종료를 인증한 경우에만
바깥 아래 여백으로 닫는다. 일반 host의 뒤 간격을 일괄 제거하지 않는다.
65쪽은 72.99%→94.53%(앞만 보정)→100.00%(끝 프레임 함께 보정)다.

### API 27·53·56·67쪽: 첫 재시작과 마지막 종료의 이중 인증

`native_text_frame_terminal_successor_gap`은 셀 placeholder 크기만으로 선언 opening을
제외하고, 실제 종료 줄에 뒤 간격을 더했다. 첫 저장 reset의 줄+안여백이 선언 높이를
닫는지 확인하고 마지막 줄+여백이 다음 문단 원점에 정확히 닿는지도 확인한다.
원본의 두 독립 경계를 만족할 때만 다음 프레임의 간격을 중복 소비하지 않는다.
네 쪽 모두 100.00%다. 문서명·쪽 번호 예외를 넣지 않는다.

### API 69쪽: TAC 표 뒤 독립 빈 줄과 fit 예산

표를 소유한 저장 줄 앞의 실제 prefix와 뒤의 독립 빈 줄을 구분하지 않아,
표 fit 계산에 이미 소비한 줄이 다시 포함되고 paint는 공백 host의 뒤 줄을 생략했다.
표 소유 줄의 실제 인덱스로 뒤 시작을 정하고, source prefix가 실제 전진과 일치할 때
소유 표 줄의 높이·간격만 표 예산으로 쓴다. 별도 빈 줄은 자기 위치를 가진다.
초기 시각 통과 뒤에도 fit의 4px 누락을 확인해 추가 보정했다. 최종 69쪽은 100.00%다.

### CBTA 176·182쪽: 이번 행에서 닫히는 병합 칸의 몫 누락

선행 실제 수정 `0024df763a`의 내용·선언·병합 칸 몫의 최댓값 계약을 추적했다.
`crossing_span_page_end_band`의 엄격 부등식이 이번 행에서 끝나는 병합 칸을 제외해
근거 없는 남은 쪽 공간 전체를 끝 행에 배분했다. 182쪽 행222의 17.0933px가 32.44px로
늘어났지만 독립 Print는 약21.43px다.
종료 병합 칸의 실제 내용과 선언 높이 중 큰 점유에서 이미 수용한 앞 행 상자만 뺀다.
줄·안여백의 전체 최소는 기존 공개 측정 API로 구한다. 뒤로 이어지는 칸의 계약은 유지한다.
176쪽 97.45%→99.94%, 182쪽 96.23%→99.94%이며 전204쪽으로 확인했다.

### CBTA 118쪽: 위 캡션 이동에서 소유 테두리 누락

`layout.rs::offset_inline_image_y`가 그림만 옮긴 뒤 반환해, 형제 테두리를 캡션 위에 남겼다.
원본 image y=212.238px, border y=184.918px로 27.32px 분리됐다.
이미지 ID와 `RectangleNode.image_frame_owner`로 같은 소유자의 테두리를 찾아 함께 이동한다.
좌표·색·문서명으로 테두리를 추정하지 않는다.
별도 Native/fresh WASM 집중 결과는 모두 96.85%→98.44%다. review·overlay를 함께 직접 확인했다.
얇은 테두리의 출력 농도 차이는 남아 있어 100% 해결로 쓰지 않는다.
새 관계형 검사 `issue_7727_top_caption_picture_frame`은 실제 보정 전 1 FAIL→보정 후 1 PASS다.
문서 전체가 아직 98% 미만이므로 절대 위치를 고정하는 검사로 등록하지 않았다.
이는 캡션 보정 당시의 판단이다. 현재는 위 최신 전쪽 검증으로 자격을 확인했으며,
같은 검사 파일에 독립 PDF 테두리 네 변의 ±2 CSSpx 고정형 검사를 추가해 통과했다.

## 영어 시험지의 추가 분석

NBSP의 실제 tracking을 소비하면서 압축 슬롯 수에서는 제외한 공통 분모 불일치를 확인했다.
압축 슬롯에 NBSP를 포함한 후보는 Native 전8쪽에서 최저 97.14%다. 아직 목표 미달이다.
앞서 언어 경계를 강제로 이어받긴 별도 후보는 97.12%→96.85%로 악화시켜 되돌렸다.
그 후보의 결과를 최종 증거로 사용하지 않는다.

밑줄 차이는 별도 한컴 2020 인쇄 대조군으로 분리했다. 동일한 12pt 글자·밑줄에
글자 위치 -12/0/+12를 적용했다. MCP job `9e395d33-57c9-4fb5-932e-eea17e70b484`,
실제 버전 11.0.0.9136, Print method 0, 1쪽, 출력 13초다.

| 글자 위치 | 글자 기준선(pt) | 밑줄 y(pt) | 밑줄−글자 기준선(pt) |
| --- | ---: | ---: | ---: |
| -12 | 79.4731 | 82.9490 | 3.4759 |
| 0 | 100.0906 | 102.1280 | 2.0374 |
| +12 | 120.7081 | 121.3070 | 0.5989 |

밑줄은 약19.179pt인 줄 간격만 따라가고 글자 위치 변화는 따라가지 않는다.
아래 밑줄의 공통 replay에서만 글자 위치를 제외하고, 텍스트·장식의 원본 run은 동일하게
보존한다. 최초 후보가 원본 장식 run을 바꾸어 canonical 소유 검증에 거절된 것을 실제 확인했고,
검증을 완화하지 않고 replay 단계로 보정을 옮겼다. 종이 프레임의 밑줄 원점도 같은 줄 기준선이다.
Native 전8쪽 최저97.79%, fresh WASM 전8쪽 최저97.78%다. 개선됐지만 목표 미달이다.

새 밑줄 관계형 검사는 이전 코드에서 1 PASS / 1 FAIL, 보정 후 2 PASS다.
NBSP 끝 전진 관계를 추가한 3개 검사는 보정 후 모두 PASS다. NBSP의 이전 라이브러리 검사는
폐기한 언어 슬롯 후보를 포함한 진단 생산자이므로 base 자체의 결과로 부르지 않는다.
위 밑줄·취소선과 원본 글자 위치 보존을 대조군으로 함께 확인했다.
API의 독립 PDF 표 경계 13·65·69쪽 고정형 검사와 69쪽 독립 빈 줄 소유 관계를 추가했다.
정확한 base Native에서 3 FAIL, 보정 후보에서 3 PASS다. 임시 독립 rustc 실행이며
정식 파생 suite의 최소 글꼴 Native Skia nextest도 실행해 새3개 파일의 **7 PASS / 0 FAIL**을
확인했다(3.353초, 선택 밖1024건). 첫 실행의 임시 폴더 helper 직접 의존성 누락은 표준
라이브러리의 검사 소유 폴더와 Drop 정리로 수정했다. 전체 회귀는 아직 실행하지 않았다.

Mac 격리에서 CoreText1875→528개, system_profiler의 글꼴 파일235개와 CoreText의
모든 실제 경로가 OS 영역에 있음을 확인했다. 기본 한글 프로그램 파일·표시 PNG의 SHA는
일반 환경과 같고, 일반 프로세스의 사용자 글꼴 등록 목록도 전후 동일했다.
[최소 글꼴 집중 검증 기록](../pr/assets/issues7727_7732_20261114/focused-minimal-font-validation.json)을 따른다.

독립 대조군은 [원본 HWPX](../../tests/fixtures/issue7730/underline-position-probe.hwpx),
[한컴2020 Print PDF](../../pdf/issue7730/underline-position-probe-2020.pdf),
[원인·전후 측정·MCP 영수증](../pr/assets/issues7727_7732_20261114/underline-baseline/cause.json)에
같은 SHA로 보존했다. 글꼴 바이너리는 포함하지 않는다.

### 영어 낱말 공백: 언어 슬롯의 상대 크기와 기준 em 분리

원본 영어 시험지 CS13은 1150HU, 한글 장평 95%, 영문 상대 크기 106%다. 반각 공백이
현재 영문 글리프 크기를 기준으로 커지면서, `bike`의 첫 글자가 독립 Print보다 약7.76px
오른쪽에 놓였다. 글리프 크기·장평·자간을 바꾸지 않고 반각 공백의 기준 em만 한글 슬롯의
크기로 환산하여 측정·줄 구성·paint가 공유하도록 했다. `font_space_em`이 명시되거나
앞 언어에서 상속된 전진은 기존 값을 유지한다. 반각 규칙을 선택하지 않은 경로에는 적용하지 않는다.

독립 한컴2020 Print 대조군은 한글12pt·장평95%를 유지하고 영문 상대 크기100/106/120%만
바꿨다. 영문 글리프는 커지지만 낱말 공백은 약5.7pt로 유지됐다. MCP job은
`e2f7be42-a029-49db-920e-ac993c38cd66`, 제품11.0.0.9136, Print method0이다.
[독립 HWPX](../../tests/fixtures/issue7730/relative-size-half-space-probe.hwpx)와
[Print PDF](../../pdf/issue7730/relative-size-half-space-probe-2020.pdf),
[영수증·원인](../pr/assets/issues7727_7732_20261114/half-space-origin/cause.json)을 함께 보존했다.
이 HWPX의 Native 경로는 `layout_half_space=false`라서 전후93.05%이며, 해당 표본까지
시각적으로 해결했다고 주장하지 않는다. 이 표본은 독립적인 공백 전진의 인쇄 근거다.

실제 원본 HWP 전8쪽은 Native·fresh WASM 모두 최저99.49%로 복구됐다. 새 고정형 검사
`english_response_line_keeps_independent_print_word_origins`는 독립 PDF의 낱말 원점
326.759979pt→435.679972 CSSpx와 기준선308.799988 CSSpx를 ±2 CSSpx로 검사한다.
원점은 보정 전443.44px로 허용 범위를 벗어나며 보정 후 통과했다. 추가한 네 번째 검사를
포함하여 같은 파일의4개 검사와 정식 최소 글꼴 집중8개 검사가 실제 통과했다.
이전 독립 rustc 대조군은 폐기한 언어 슬롯 후보를 포함하므로 정확한 base 전체 결과와 구분한다.

## 남은 분석과 완료 조건

- CBTA 96쪽은 위의 번호 문단 보정으로 양쪽 99.72/99.74%다. 가능한 잔여100% 개선을
  계속 확인하며, 현재 관측한 2쪽 목차 탭 전진의 재계산 경계를 원본과 대조한다.
- CBTA162·166쪽은 위의 완료 행 소유 보정으로 두 경로 모두99.94/99.95%다.
  이후 번호 문단 보정까지 포함한 최신 전389쪽 측정에서도 이 수치를 유지했다.
- 권고안 4쪽은 위 채널 보간 후보의 Native에서 99.96%로 개선됐다. 전 22쪽 최저는
  98.69%다. 새 fresh WASM도 4쪽 99.96%, 전 22쪽 최저 98.68%로 완료했고 정식 최소 글꼴 집중
  12개도 통과했다. 최종 전체 회귀·보호 검증은 아직 남아 있다.
- 현재 번호 문단 소스의 Native/fresh WASM 전389쪽 검증은 위와 같이 완료했다.
  추가 구현으로 제품 소스가 달라지면 영향 범위와 생산자를 구분하여 재검증한다.
- 회귀를 최소 글꼴 환경에서 실행하고, 독립 근거로 자격을 얻은 쪽에는 현행 지침에 따른
  정확한 고정형 또는 관계형 검사를 추가한다. 검사의 상단 주석에 종류·측정 근거를 명시한다.
- 전체 nextest·범위별 lint/build·보호 TSV·필수 편람·실제 소비 경로를 최종 head로 확인한다.
  미실행 항목을 이전 head의 통과나 PR CI로 대신하지 않는다.

## 증적 위치

현재 작업 증적은 저장소 기준 `output/pr-review/issues-7727-7732-20261114/`다.
임시 driver·실행 파일·글꼴/SVG 중간 자료는 이 경로에만 두며 Git에 추가하지 않는다.

- `cases.json`: 입력과 기준 PDF의 SHA·쪽수·출처.
- `closing-span-clean-native-state.json`, `closing-span-clean-wasm-state.json`: 동일 source 전쪽 결과.
- `closing-span-clean-native-producer.json`, `closing-span-clean-wasm-producer.json`: 생산자 SHA.
- `logs/closing-span-clean-native-driver.log`, `logs/closing-span-clean-wasm-driver.log`: 전쪽 실행 로그.
- `caption-frame-native-producer.json`, `logs/caption-frame-cbta.log`: 캡션 집중 후보와 측정.
- `logs/caption-frame-before-test.log`, `logs/caption-frame-after-test.log`: 새 검사 전후 실패/통과.
- `caption-frame-direct-review/review/review_118.png`와 `overlay/overlay_118.png`: 같은 쪽 직접 판독 쌍.
- `underline-position-cause.json`, `underline-position-print-{start,status,download}.json`: 독립 밑줄 대조군 영수증.
- `half-space-origin-{native,wasm}-state.json`: 최신191쪽 측정과 남은4쪽.
- `logs/half-space-origin-minimal-font-focused-nextest.log`: 정식 최소 글꼴 집중8개 PASS.
- `cbta-row188-diagnostic.log`: 162쪽 종료 행·걸침 셀의 원본 선언 높이와 저장 줄.

영어 공백의 확정 증거는 정식 `mydocs/pr/assets/issues7727_7732_20261114/half-space-origin/`에
복사했고, 독립 표본은 `tests/fixtures/issue7730/`와 `pdf/issue7730/`에 보존했다.

위 경로는 진행 중인 로컬 증적이다. 최종 제출 때에는 필요한 PNG·TSV·영수증과 독립 표본을
정식 assets/PDF 경로로 선별해 함께 커밋하고 이 문서의 링크와 판정을 갱신한다.

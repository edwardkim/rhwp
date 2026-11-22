# #9276 Windows 직접 PDF 내보내기 보정

기준 소스는 `bd3cd171331e80e79881dc8e9ed37f837ef6be77`이다. Windows x64에서 공고문(11쪽), 모집 안내문(7쪽), 추천 서식(1쪽)을 같은 NTFS 경로와 글꼴 공급 조건으로 내보냈다. 공고문에서는 PDF와 Native PNG가 모두 약 24초 걸렸고, 쪽수 조회와 SVG는 약 4초였다. [이슈와 원본 출처](https://github.com/edwardkim/rhwp/issues/9276)

## Native 글꼴 선택

원본 TTF를 우선하는 `typeface_candidates_for_families_for_source`는 같은 설치 계열·스타일을 그리는 동안 매번 OS에 실제 face를 요청했다. DirectWrite face의 윤곽 형식을 확인하려고 전체 TTF/TTC를 매번 읽었다. 계측용 빌드에서 공고문은 4,386회에 20.720초, 모집 안내문은 5,627회에 26.468초였다. 공고문에서 PDF 문서를 닫는 데 든 시간은 0.745초였다. 각 함수의 측정 시간은 하위 호출을 포함한다.

수정본은 설치 글꼴 카탈로그별로 글꼴 계열명과 굵기·너비·기울기를 키로 써서 TrueType 선택과 미발견 결과를 재사용한다. 사용자 경로의 우선순위, 같은 계열 내 후보 순서, 문자별 실제 글리프 선택은 종전 경로가 결정한다. 현지화된 계열명을 확인할 때는 선택한 face의 이름 테이블을 읽는다. Windows에서는 DirectWrite가 공급하는 원본 파일 스트림과 face 번호를 사용해 TTF와 TTC의 순위를 판정한다. [Skia의 DirectWrite 스트림 구현](https://github.com/rust-skia/skia/blob/m157.2/src/ports/SkTypeface_win_dw.cpp#L691-L718)

별도 Windows 실행 파일에서 문서 열기, 웹폰트 준비, 환경 선택, 환경 적용을 각각 측정했다. 추천 서식은 초기 웹폰트 준비 1.112초, 설치 프로그램을 이용한 치환 판정 1.676초였다. 웹폰트 준비 단계에서는 설치 이름과 PostScript 이름을 한 번 색인한다. 하이픈이 다른 이름은 종전과 같이 서로 다른 실제 PostScript 이름이 하나일 때만 수용한다.

## PDF에 설치 글꼴 전체가 들어간 원인

기준 공고문의 10,933,271바이트 중 10,615,665바이트가 압축된 글꼴 프로그램이었다. 해당 한양 TTF와 굴림 TTC의 GSUB 버전은 `0x00001000`이었다. 같은 글리프로 실험했을 때 HarfBuzz 8.3.0은 부분 추출에 성공했고, Skia에 포함된 14.5.1에서는 실패했다. 후자는 GSUB를 제외하면 성공했다. 굴림 TTC의 네 얼굴, 한양 TTF 및 정상 GSUB를 가진 글꼴을 대조했다. 한글·영문·문장부호 4개 문자열을 가로·세로로 조형한 세 얼굴의 24개 결과에서 글리프 번호, 클러스터, 전진폭과 오프셋은 모두 같았다. [OpenType GSUB 헤더](https://learn.microsoft.com/en-us/typography/opentype/spec/gsub#gsub-header) · [Skia의 부분 추출 실패 시 전체 프로그램 처리](https://github.com/rust-skia/skia/blob/m157.2/src/pdf/SkPDFFont.cpp#L469-L479)

직접 PDF 기록 시 부분 임베딩이 허용된 정적 TrueType 얼굴에 한해 GSUB 주 버전 0을 제외한 SFNT를 만든다. 선택한 얼굴의 윤곽·복합 글리프·폭·문자 매핑·비트맵 테이블을 보존하고 바뀐 컨테이너의 체크섬을 다시 계산한다. 서명 테이블은 새 컨테이너에서 효력이 없어 제외한다. 정상 GSUB, 가변 글꼴, 합성 스타일, 임베딩 제약이 있는 글꼴은 기존 Skia 처리를 사용한다. 출력용 SFNT와 각 입력 테이블의 상한은 32 MiB, 문서별 출력용 얼굴 캐시는 256개·64 MiB다.

## 검증 자료

공고문은 한컴 2020, 모집 안내문과 추천 서식은 한컴 2024에서 인쇄해 각 원본의 전체 쪽을 대조했다. 공고문과 모집 안내문은 조달청에서 함께 배포한 PDF도 추가로 비교했다. 원본 글꼴 프로그램과 Windows 설치 글꼴을 포함하는 PDF는 로컬에서 비교했다. [조달청 공고문 원본](https://www.pps.go.kr/common/fileDown.do?key=202604300039&sn=1) · [모집 안내문 원본](https://www.pps.go.kr/common/fileDown.do?key=202610080012&sn=2) · [추천 서식 원본](https://www.pps.go.kr/common/fileDown.do?key=202610080012&sn=1)

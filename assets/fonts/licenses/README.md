# 배포 폰트의 저작권·라이선스

이 폴더는 공개 GitHub 저장소와 Studio·확장 프로그램의 글꼴 배포에 함께 포함한다.
테스트 목적이나 제품 소스의 MIT 라이선스가 외부 폰트의 재배포·변형 조건을 면제하지 않는다.

`font-notices.json`은 canonical 폰트 전체의 목록이며 VS Code처럼 일부만 포함하는 패키지에도
함께 동봉한다. 실제 패키지에 들어 있는 각 파일을 이 목록과 대조한다. 목록은 실제 파일별 SHA-256, 원본 name table의 저작권 고지, 적용 라이선스 전문을
연결한다. 폰트 바이너리에 있던 저작권 정보도 보존한다. 라이선스 전문을 동봉했다는 사실만으로
원본 대응·수정본 이름 조건까지 충족했다고 판정하지 않는다.

## 라이선스

- Latin Modern Math는 GUST Font License와 그 문서가 참조하는 LPPL 전문을 동봉한다.
- 나머지는 SIL OFL 1.1이다. 각 권리자의 고지·Reserved Font Name 조건과 파생본 OFL 유지
  조건을 따른다. 폰트 자체를 단독 판매하지 않는다.
- 카페24와 해피니스 산스의 PDF는 공식 ZIP에 동봉된 원문 그대로다. 카페24 PDF는 OFL 1.1을
  명시하며, 영어 라이선스 본문은 `SIL-OFL-1.1.txt`에도 보존한다.
- WOFF2는 원본 정보와 기능을 보존하는 단순 압축인지, 글자·동작을 제거한 수정본인지 구별한다.
  수정본이 Reserved Font Name을 쓰려면 권리자의 별도 허용 또는 적절한 이름 변경이 필요하다.

## 출처

공식 다운로드 원문의 URL과 SHA-256은 `font-notices.json`에 있다. 기존 저장소에서 가져온
고지는 해당 원본 경로를 기록한다. 카페24·해피니스 산스는 다음 공식 배포본의 대응 WOFF2와
바이트 일치를 확인했다.

| 공식 배포 ZIP | SHA-256 |
| --- | --- |
| [HappinessSans-Screen.zip](https://thehyundaifont.com/assets/fonts/HappinessSans-Screen.zip) | `1defb001694a8bd3fd5c7a15c45377e82e654569177c6995016ae321223fb041` |
| [Cafe24Ssurround-v2.0.zip](https://img.cafe24.com/csdstatic/freefonts/download/kr/Cafe24Ssurround-v2.0.zip) | `a5f9c0c8f4cc2d3c00220ace363a893618c5cf59ff5aea184aeced3bd3e9af5c` |
| [Cafe24Supermagic-Regular-v1.0.zip](https://img.cafe24.com/csdstatic/freefonts/download/kr/Cafe24Supermagic-Regular-v1.0.zip) | `f646af53800f40355f45f708605c1980b07563118d8b098d9a699f3aa53cf15b` |

공식 OFL 원문: <https://openfontlicense.org/open-font-license-official-text/>.
웹폰트의 수정본·이름 조건: <https://openfontlicense.org/webfonts-and-reserved-font-names/>.

## 수정본 이름과 공개 테스트 자원

- 한컴·한양·Microsoft 등의 상용 글꼴 이름을 다른 프로그램의 내부 family 이름으로 지정하지 않는다.
  문서의 요청 이름은 렌더러의 명시적인 대체 규칙에서 실제 공급 family에 연결한다.
- Adobe Source Han 옛한글 수정본의 family·full name·PostScript/CFF 이름은 `RHWP Old Hangul`이다.
  원본의 저작권·OFL·Reserved Font Name 고지는 그대로 보존한다.
- 수식 테스트 세 파일은 `RHWP Equation Contract`다. Noto 윤곽선을 상용 글꼴 이름으로 등록하지 않는다.
- 나눔 8개, D2Coding 2개, Latin Modern Math는 전체 공식 원본의 정보·동작을 유지하는 WOFF2
  압축만 사용한다. 글자 일부를 없애면서 원본의 예약 이름을 유지하는 배포 방식을 사용하지 않는다.
- Pretendard 9개, Cafe24 2개, Happiness 4개, Spoqa Han Sans 1개 WOFF2는 공식 배포 파일과
  바이트가 일치한다. 고운 글꼴의 OFL에는 Reserved Font Name 선언이 없으며 원본 저작권·OFL을 보존한다.
  Noto 파생본도 원문에 명시된 예약 이름 `Source`를 primary 이름에 사용하지 않는다.
- 목록의 `repositoryOnlyFonts` 18개는 공개 GitHub에 포함되는 테스트·Native 원본이다.
  외부 파생본에는 OFL, 직접 생성한 합성 fixture 7개에는 저장소 MIT 전문이 적용된다.
  제품 폰트 37개와 합쳐 독립 폰트 파일 55개를 모두 SHA와 고지에 연결한다.

## 공식 원본의 WOFF2 재생성

`font-notices.json`의 `officialProgram`은 압축 전 원본 SHA와 파일명을 고정한다. D2Coding의 URL은
공식 ZIP이며 `archiveSha256`과 `archiveMember`도 기록한다. 압축을 해제한 TTF의 SHA와 ZIP의 SHA를
구별한다. 다른 원본으로 바꾸려면 출처·라이선스·시각 검증을 다시 확인한다.

```bash
python3 -m pip install -r scripts/requirements-font-distribution.txt
python3 scripts/restore-official-font-webfonts.py --source-dir /path/to/official-font-programs
```

재생성 도구는 subset·스타일 변경·이름 변경을 하지 않으며 원본의 수정 시각·권리자 고지를 보존한다.
전체 원본 복원으로 문자 coverage는 늘어날 수 있다. 재배포 조건 확인과 실제 문서의 렌더링 검증은
각각 수행한다. SHA·고지 검사 통과를 시각 일치 완료로 보고하지 않는다.

Safari는 Chrome 배포 폴더 전체를 복사하므로 이 `fonts/licenses/`도 전달된다. Xcode의 fonts resource는
폴더 단위로 포함한다. 데스크톱 UI는 자체 폰트 파일을 번들하지 않는다.

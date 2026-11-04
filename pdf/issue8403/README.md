# #8403 공개 합성 PDF

개인 문서의 객체·텍스트·그림을 복제하지 않고 PDF 연산자로 만든 검증 입력이다.
`/mnt/e/tmp/black-image-01.pdf` 원본은 이 디렉터리에 포함하지 않는다.

- `image-patterns.pdf`: 5쪽. 이미지로 채운 타일 패턴, 음수 간격·전단·clip,
  Form의 자원/초기 좌표계, SMask와 외부 alpha, paint 시 CTM 변경 뒤에도 유지되는 위상.
- `text-stroke.pdf`: 7쪽. 채움+획, 획만 표시, ExtGState의 색/alpha/두께,
  round join/miter, 같은 1/1000pt 획 폭으로 표현되는 직교 배율과 빈 dash 배열,
  균일 배율, 폭 0 hairline, 중첩 Form. 각 쪽의 두 번째 문장은
  q/Q가 복원한 일반 채움 대조군이다.
- `text-stroke-edited.pdf`: 같은 연산자에서 `Stroke`를 `ktroke`로 바꾼 독립 편집 기준.
- 나머지 PDF: 간격 0·과도한 반복·재귀·미지원 pattern 종류/paint·특이 행렬,
  점선/비균일 변환/글리프 clip이 결합된 문자 획과 패턴 문자 채움의 거절 대조군.
  성공 출력을 고정하는 fixture가 아니라 오류와 다음 요청의 정상 복구를 검증한다.

글꼴은 저장소의 [Noto Sans KR](../../ttfs/opensource/NotoSansKR-Regular.ttf)를
부분 내장했다. 저작권과 배포 조건은 [SIL OFL 1.1](../../ttfs/opensource/NotoSansKR-OFL.txt)을
따른다. 시스템 글꼴 설치 여부로 회귀를 건너뛰지 않는다.

```bash
venv/bin/python pdf/issue8403/generate.py --out output/issue8403-regenerated
pdftoppm -r 96 -png pdf/issue8403/text-stroke.pdf output/issue8403-reference
```

생성에는 로컬 venv의 PyMuPDF가 필요하다. PDF ID 등 컨테이너 바이트는 재생성 시 달라질 수
있으며, 생성기의 정상 세 문서는 보관본과 독립 렌더링 픽셀이 같음을 확인했다.
문자 기준은 Poppler다. MuPDF는 `0 w / 1 Tr` 글자를 생략하므로 hairline의 기준으로
사용하지 않는다. 패턴 대조군은 MuPDF로 독립 출력했다.

전쪽 Native/fresh WASM TSV와 직접 판독 증거는
[검증 기록](../../mydocs/tech/investigations/issue-8403/validation/README.md)에 있다.

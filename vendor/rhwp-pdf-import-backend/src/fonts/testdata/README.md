# CFF 컨테이너 검사 입력

`minimal-cff.otf`는 `generate_minimal_cff.py`가 fontTools로 생성하는 네 glyph의 원본 합성
폰트다. `.notdef`와 직접 작성한 사각형 A/B/C를 사용하며 외부 폰트 데이터를 복제하지 않는다.
라이선스는 이 crate와 같은 MIT OR Apache-2.0이다.

기존 subset 검사가 CFF의 `OTTO` 컨테이너와 요청한 GID를 보존하는지 확인한다.
폰트 이름·시각 피델리티·실제 문서의 글리프 모양을 검증하는 기준은 아니다.
개발 환경에서 다음 명령으로 재생성한다.

```bash
venv/bin/python vendor/rhwp-pdf-import-backend/src/fonts/testdata/generate_minimal_cff.py
```

fontTools 4.62.1로 독립 재생성한 입력의 SHA-256은
`9fdefe03f069ef9543863b751a466f04c5bc3072c201b9569f7b036ecb372867`이다.

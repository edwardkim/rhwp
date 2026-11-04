# JP2 해상도 검사 입력

`gray-816x1056.jp2`는 `generate_gray_jp2.py`가 직접 생성하는 원본 합성 DeviceGray 이미지다.
각 픽셀은 `(3x + 5y) % 256`이며 크기는 816×1056이다. 라이선스는 이 crate와 같은
MIT OR Apache-2.0이다. 외부 `sample_gray.jp2`가 누락된 배포 소스에서 기존 해상도·표본 수
계약을 실행할 수 있도록 제공한다. 누락된 원본 scan의 복원이나 시각 일치 증거는 아니다.

개발 환경의 Pillow/OpenJPEG로 재생성한다.

```bash
venv/bin/python vendor/rhwp-pdf-import-backend/src/decoders/testdata/generate_gray_jp2.py
```

OpenJPEG 2.5.0을 사용하는 Pillow 10.2.0에서 재생성과 모든 원본 픽셀의 일치를 확인했다. 입력 SHA-256은
`e642396bc461a0be24e11347d3a343b42660261e75393c3c39ab537744c77ccb`이다.

# Native 공급 글꼴 스타일 계약

공개 OFL Noto Sans KR Regular/Bold WOFF2에서 `AW 총투자`만 fontTools subset으로 보존하고 SFNT로 해제했다. 수정본 이름은 `RHWP Supplied Style Contract`다. 원본의 독립 400/700 메트릭과 윤곽은 유지한다. 한컴 배치 fixture나 픽셀 golden이 아니며 실제 Skia 공급 스타일 선택을 검사한다.

재생성에는 fontTools와 Brotli가 필요하다. 저장소 루트에서 다음 명령을 사용한다.

```bash
venv/bin/python scripts/generate-native-style-font-fixtures.py
```

SFNT 수정 시각을 고정했으며, 별도 폴더에서 재생성한 두 파일이 아래 SHA와 바이트 단위로 같은 것을 확인했다.

```json
[
  {
    "source": "assets/fonts/NotoSansKR-Regular.woff2",
    "sourceSha256": "d1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a",
    "output": "tests/fixtures/native_font_styles/RHWPSuppliedStyle-Regular.ttf",
    "outputSha256": "5b172a2f2ca96ac295cb75facd3788db81e1d03b07b8757203eab522d8677cb9",
    "weight": 400
  },
  {
    "source": "assets/fonts/NotoSansKR-Bold.woff2",
    "sourceSha256": "dab3d492daa687386292cff1499225db580f931f800ee0d124eb0636378c6020",
    "output": "tests/fixtures/native_font_styles/RHWPSuppliedStyle-Bold.ttf",
    "outputSha256": "fab15f2e8937d5dacd0503edd3382f94a282f3fc62de4211f6cbe027310b34d2",
    "weight": 700
  }
]
```

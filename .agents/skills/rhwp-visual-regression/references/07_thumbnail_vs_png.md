# 07 — thumbnail vs export-png

두 명령 모두 PNG 비슷한 것을 내지만 **같은 눈이 아니다.**

## thumbnail

HWP **내장** 썸네일(PrvImage)을 추출한다.

```bash
rhwp thumbnail 문서.hwp -o 문서_thumb.png
rhwp thumbnail 문서.hwp --data-uri
```

저장 시점의 미리보기다. `edit fill-fields` 직후 다시 뽑아도, 한컴이
PrvImage 를 갱신하지 않은 파일이면 **빈 서식 그림**이 나온다.
편집 후 눈 검증의 기준이 될 수 없다.

## export-png

현재 IR 을 Skia 로 **재렌더**한다. `native-skia` feature.

```bash
rhwp export-png 문서.hwp -p 0 -o 문서_p0.png
rhwp export-png 문서.hwp --vlm-target claude
```

전후 눈 검증, VLM 입력, 래스터 품질은 이쪽이다.

## 선택 규칙

| 질문 | 명령 |
| --- | --- |
| 저장본에 들어 있는 미리보기 | thumbnail |
| 지금 레이아웃이 어떻게 그려지나 | export-png |
| 문단/표 경계를 겹쳐 보기 | export-svg --debug-overlay |
| px 숫자 | render-diff |

thumbnail 을 채움 확인에 쓰면 F09 함정이다.

# Task M100-949 Stage 34 계획

## 1. 목적

Stage 33에서 `hwpx-h-03.hwp`의 rhwp-studio 페이지네이션은 처음으로 정상화되었다.
따라서 다음 단계는 rhwp 조판 문제가 아니라, 한컴 에디터가 여전히 파일손상으로 판정하는
HWP5 loader contract를 정답 HWP와 비교해 좁히는 것이다.

## 2. 입력

```text
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
samples/hwpx/hancom-hwp/hy-001.hwp
output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage33_lineseg_vpos_preserve/hy-001.hwp
```

## 3. 절차

```text
1. 정답 HWP와 Stage 33 생성 HWP의 BodyText Section0 inventory를 추출한다.
2. shape focus diff를 생성한다.
3. 기존 Stage 29 field decoder를 Stage 33 산출물에 재적용한다.
4. 이미 정답과 일치하는 축과 남은 축을 분리한다.
```

## 4. 판정 기준

```text
닫힌 축:
- CTRL_HEADER payload/hash 일치
- LIST_HEADER payload/hash 일치
- drawText 내부 PARA_HEADER payload/hash 일치
- SHAPE_PICTURE payload/hash 일치
- SHAPE_RECTANGLE payload/hash 일치

남은 축:
- SHAPE_COMPONENT payload/hash 차이
- 특히 offset 36 storage flag와 rendering 뒤 drawing tail
```

## 5. 산출물

```text
output/poc/hwpx2hwp/task949/stage34_hancom_loader_contract/
```


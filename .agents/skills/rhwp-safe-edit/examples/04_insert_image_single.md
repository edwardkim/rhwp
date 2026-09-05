# 04 — 도장·서명 (`edit insert-image`)

층: 1 **만**. `run` 계획서에 `insert_image` action 은 없다.
지어내면 `알 수 없는 action` + exit 2.

권위: [single_edit.md](../references/single_edit.md) §6, #3719 §6-5.

## 0. 단위

`--x --y --width --height` 는 픽셀이 아니다. HWPUNIT (1/7200 inch).
A4 세로 ≈ 59528 × 84188. 쪽 왼쪽 위가 (0,0). `--page` 는 0 기준.

```bash
rhwp info 신청서_filled.hwp --json | jq '{pageCount, format}'
```

쪽 번호가 `pageCount` 이상이면 exit 2. 문서를 읽기 전에 끊길 수 있다.

## 1. 그림 형식

허용: png jpg jpeg bmp tif tiff. 확장자와 내용을 둘 다 본다.
그 밖은 문서를 열기 전에 exit 2.

## 2. 선확인

```bash
rhwp edit insert-image 신청서_filled.hwp --image samples/images/moogung.jpg \
  --page 0 --x 50000 --y 70000 --width 5000 --height 5000 --dry-run --json
```

`binDataId` 는 저장 전에 없다. `overflow` 는 dry-run 에서도 온다.
쪽 밖으로 나가도 자르지 않고 신호만 낸다.

## 3. 실행

```bash
rhwp edit insert-image 신청서_filled.hwp --image samples/images/moogung.jpg \
  --page 0 --x 50000 --y 70000 --width 5000 --height 5000 \
  -o 제출본.hwp --verify --json | jq '{output, overflow, binDataId, verify}'
```

`overflow` 가 비지 않으면 좌표·크기를 줄여 02 절로 돌아간다.
잘린 도장을 제출본으로 넘기지 않는다.

## 4. 계획서에 넣지 않는 이유

`run` 의 action 4종은 fill_fields / replace_text / set_cell / set_checkbox.
도장은 1층으로 분리한다. 사용자가 "한 계획에 도장까지"를 요구하면
"엔진이 받지 않는다. 원자 편집 후 1층으로 얹는다"고 말하고
원자 부분만 `run` 한다.

## 5. 체크리스트

- [ ] 단위가 HWPUNIT 이다
- [ ] `run` steps 에 그림을 넣지 않았다
- [ ] `overflow` 를 읽었다

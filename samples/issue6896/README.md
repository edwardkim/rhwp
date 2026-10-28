# issue 6896 — OLE 미리보기 폴백

## 무엇을 잠그나

OLE 개체의 미리보기 선택이 **EMF 하나에 걸려 실패하지 않는다** — EMF 가 잡히더라도
WMF 폴백을 들고 있어야 한다.

## 개체 안 구조

```text
  BinData/ole5.ole  12,618,756 bytes = [u32 길이 접두] + CFB

    \x01CompObj        90
    \x01Ole            20
    \x02OlePres000     12,076,826     ← 미리보기 (clipFormat 14 = CF_ENHMETAFILE)
    CONTENTS           439,387
```

`OlePres000` 은 헤더 40바이트 뒤부터 **표준 WMF** 이고, 그 WMF 가 EMF 를 ` WMFC`
주석 레코드로 감싼 EMF-in-WMF 형식이다.

```text
  offset 40   01 00 09 00 00 03 …   mtType=1 · mtHeaderSize=9 · mtVersion=0x0300
  offset 58   META_ESCAPE(0x0626) + escape 0x000F + " WMFC"
  offset 102  u32 type=1 · offset 142 에 " EMF"   ← EMF 판정이 여기서 성립한다
```

## 왜 자리표시자만 남았나

`strip_ole_presentation_header` 가 offset 102 의 `EMR_HEADER` 를 잡는데, 그 EMF 는 끝이
잘려 있다.

```text
  rec0 type=1  size=108        (EMR_HEADER, nBytes 선언 6,022,292 · nRecords 7)
  rec1 type=17 size=12
  rec2 type=10 size=16
  rec3 type=9  size=16
  rec4 type=11 size=16
  rec5 type=81 size=6,022,104  (EMR_STRETCHDIBITS)
  rec6 type=0xFFFFFFFF          ← 여기서 깨진다

  걸은 바이트 6,022,272 / 선언 6,022,292 — 남은 20바이트(EMR_EOF)가 전부 0xFF
```

종전 코드는 `preview_emf` 가 `Some` 이면 `preview_wmf` 를 **아예 안 채웠다**. 그래서
EMF 파싱이 실패해도 폴백이 비어 있어 렌더가 자리표시자로 끝났다.

## 정본

`pdf/156564340-ip-dispute-mediation-2020.pdf` (engine 2020 — 저장 제품이
`hancom-office-2010` 이라 규약 §3.5.1 의 2020 버킷).

```text
  4쪽  그림 x=100.6 y=128.3 w=601.9 h=843.7
```

## 고친 자리

`src/parser/ole_container.rs` — `preview_wmf` 를 조건 없이 채운다. 렌더는
`OOXML 차트 → EMF → WMF → 자리표시자` 순으로 내려가므로, 둘 다 들고 있으면 EMF 실패가
자연히 WMF 로 이어진다. `#3363` 의 "EMF 부재 시" 조건은 그때는 맞았지만 **EMF 가 있어도
깨질 수 있다**는 경우를 못 봤다.

## 범위 밖 — 세로 위치

이 fixture 는 **"자리표시자 → 그림"** 까지만 잠근다. 같은 개체의 세로 좌표는
아직 정본과 다르다(rhwp `y=541.5` vs 정본 `y=128.3`). 가로·크기는 맞는다
(`x=100.7 w=602.4 h=844.7` vs `x=100.6 w=601.9 h=843.7`). 세로 축은 호스트 문단의
저장 lineseg 와 `vertRelTo=PARA vertOffset=724` 를 앵커 줄 기준으로 읽는 별개 결함이라
**별도 이슈**로 뗀다 — 여기서 쪽 좌표를 잠그면 안 된다.

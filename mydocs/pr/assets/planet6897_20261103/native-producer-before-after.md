# Native API 산출물의 변경 전후 대조

이 기록은 명시한 리베이스 전 제품의 관측 결과다. 이후 head의 최종 검증 통과를 대신하지 않는다. 원시 JSON·로그는 ignored output에 보관한다.

기준 `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`, 후보 제품 `dc87f4fa2e`.

| 입력 구분 | 기준 HWPX SHA-256 | 후보 HWPX SHA-256 | 바이트 동일 |
| --- | --- | --- | --- |
| 2010 | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | True |
| 2020 | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | True |
| plain | `14676a262b74807f6e0655cc39f34fa79dec29f065ef15bde8473dceb642921e` | `7a54ea971e88cb9e4d946288beb28a131439640f8015dbdbb44a64189b6f9e7d` | False |
| emc | `cd24cd38c78b6d76c3bafb4cc96144bb89a57ef4b130706aaed9806e6aa69d1c` | `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9` | False |

| 무표기판 PDF의 제목→본문 간격(HU) | 5쪽 | 13쪽 |
| --- | ---: | ---: |
| 독립 PDF | 2424 | 2364 |
| 기준 importer | 2808 | 2808 |
| 후보 importer | 2424 | 2364 |

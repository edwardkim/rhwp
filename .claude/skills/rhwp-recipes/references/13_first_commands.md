# 첫 수 상자

이슈: #5331. 라우터 장 `13_first_commands.md`.
정본 디렉터리: `mydocs/manual/recipes/`.
gym 이 아니고 새 CLI 도 없다. 07·08 을 만들지 않는다.

경로 자리 `<file>` 만 치환한다. 새 플래그를 붙이지 않는다.

### 01

```bash
rhwp fields <file> --json
```

### 02

```bash
rhwp export-tables <file> --json
```

### 03

```bash
rhwp edit redact <file> --dry-run
```

### 04

```bash
rhwp info <file> --json
```

### 05

```bash
rhwp fields <file> --json
```

### 06

```bash
rhwp render-diff <file> --via hwpx
```

### 09

```bash
rhwp batch info --json
```

### 10

```bash
rhwp inspect hidden-text <file> --json
```

07·08 의 첫 수는 없다.

# svg2pdf

이 source는 svg2pdf 0.13.0에 rhwp의 출력 보정을 적용한 `rhwp-svg2pdf` 패키지입니다.
공식 upstream 릴리즈와 구분하며, Rust 라이브러리 이름 `svg2pdf`는 유지합니다.
출처·패치 범위는 [RHWP_PATCHES.md](RHWP_PATCHES.md), 원저작자 고지는 [NOTICE](NOTICE)를 따릅니다.
아래 upstream 문서·배지는 원 프로젝트의 참고 자료이며 rhwp 보정 패키지의 게시 상태를 뜻하지 않습니다.

[![Build status](https://github.com/typst/svg2pdf/workflows/Continuous%20integration/badge.svg)](https://github.com/typst/svg2pdf/actions)
[![Current crates.io release](https://img.shields.io/crates/v/svg2pdf)](https://crates.io/crates/svg2pdf)
[![Documentation](https://img.shields.io/badge/docs.rs-svg2pdf-66c2a5?labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/svg2pdf/)

Convert SVG files to PDFs.

This crate allows to convert static (i.e. non-interactive) SVG files to
either standalone PDF files or Form XObjects that can be embedded in another
PDF file and used just like images.

See the [documentation](https://docs.rs/svg2pdf/latest/svg2pdf/) for a more detailed
description, including which features are supported and which are not, as well as for
examples on how to use this library.

## CLI

This crate also contains a command line interface. Install it by running the
command below:

```bash
cargo install svg2pdf-cli
```

You can then convert SVGs to PDFs by running commands like these:

```bash
svg2pdf your.svg
```

## Contributing
We are looking forward to receiving your bugs and feature requests in the Issues
tab. We would also be very happy to accept PRs for bug fixes, features, or
refactorings! We'd be happy to assist you
with your PR's, so feel free to post Work in Progress PRs if marked as such.
Please be kind to the maintainers and other contributors. If you feel that there
are any problems, please feel free to reach out to us privately.

Thanks to each and every prospective contributor for the effort you (plan to)
invest in this project and for adopting it!

## License
`svg2pdf` is licensed under a MIT / Apache 2.0 dual license.

Users and consumers of the library may choose which of those licenses they want
to apply whereas contributors have to accept that their code is in compliance
and distributed under the terms of both of these licenses.

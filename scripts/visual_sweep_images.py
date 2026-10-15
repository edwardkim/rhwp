"""페이지 PNG를 위치·크기 변경 없이 흰 종이 위의 RGB로 읽는다."""
from __future__ import annotations

from pathlib import Path

from PIL import Image


def page_rgb(image: Image.Image) -> Image.Image:
    """숨은 RGB를 잉크로 세지 않고 보이는 alpha까지 흰 배경에 합성한다."""
    if "A" in image.getbands() or "transparency" in image.info:
        rgba = image.convert("RGBA")
        paper = Image.new("RGBA", image.size, (255, 255, 255, 255))
        return Image.alpha_composite(paper, rgba).convert("RGB")
    return image.convert("RGB")


def open_page_rgb(path: Path) -> Image.Image:
    with Image.open(path) as image:
        return page_rgb(image)

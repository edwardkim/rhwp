"""Bounded page-pixel tile planning and host-owned image retention.

The grid matches Native Skia's opt-in tiled contract. Logical request rounding
can add a border pixel; callers must place the returned raster by its metadata
and clip it to the canonical tile, rather than assuming the requested origin.
"""

from collections import OrderedDict
from dataclasses import dataclass
import math
import struct


TILE_SIZE = 512
MAX_EXACT_PIXEL = 16_777_216
MAX_VISIBLE_TILES = 64
MAX_VISIBLE_PAGES = 64
CLIENT_CACHE_BYTES = 80 * 1024 * 1024
CLIENT_CACHE_ENTRIES = 80
NATIVE_CACHE_BYTES = 32 * 1024 * 1024


def finite_number(value):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError("tile geometry must contain finite numbers")
    try:
        value = float(value)
    except OverflowError as error:
        raise ValueError("tile geometry must contain finite numbers") from error
    if not math.isfinite(value):
        raise ValueError("tile geometry must contain finite numbers")
    return value


@dataclass(frozen=True)
class ViewportTile:
    page: int
    x: int
    y: int
    width: int
    height: int
    scale: float
    device_scale: float

    @property
    def region(self):
        return {"x": self.x / self.scale, "y": self.y / self.scale,
                "width": self.width / self.scale, "height": self.height / self.scale}

    def key(self, identity):
        return (*identity, self.page, self.scale, self.device_scale, self.x, self.y, self.width, self.height)


def plan_visible_tiles(pages, visible, device_scale, *, max_tiles=MAX_VISIBLE_TILES):
    """Count before iterating the grid; never allocate a whole-page tile list."""
    finite_number(device_scale)
    if device_scale <= 0 or device_scale > 64:
        raise ValueError("device scale is out of range")
    if isinstance(max_tiles, bool) or not isinstance(max_tiles, int) or not 0 < max_tiles <= MAX_VISIBLE_TILES:
        raise ValueError("tile plan limit is out of range")
    if len(visible) > MAX_VISIBLE_PAGES:
        raise ValueError("too many visible pages")
    planned, seen_pages = [], set()
    for item in visible:
        page = item["page"]
        if isinstance(page, bool) or not isinstance(page, int) or not 0 <= page < len(pages) or page in seen_pages:
            raise ValueError("invalid or duplicate visible page")
        seen_pages.add(page)
        scale = finite_number(item["rasterScale"])
        if not 0 < scale <= 64 or struct.unpack("f", struct.pack("f", scale))[0] <= 0:
            raise ValueError("raster scale is out of range")
        size, region = pages[page], item["pageRegion"]
        page_width, page_height = finite_number(size["width"]), finite_number(size["height"])
        if page_width <= 0 or page_height <= 0:
            raise ValueError("page extent is out of range")
        extent = [page_width * scale, page_height * scale]
        if any(not math.isfinite(v) or not 0 < v <= MAX_EXACT_PIXEL for v in extent):
            raise ValueError("scaled page extent is out of range")
        pixel_width, pixel_height = (math.ceil(v) for v in extent)
        x, y, width, height = (finite_number(region[key]) for key in ("x", "y", "width", "height"))
        right, bottom = x + width, y + height
        if width <= 0 or height <= 0 or right <= x or bottom <= y:
            raise ValueError("viewport region is empty or degenerate")
        endpoints = [x * scale, y * scale, right * scale, bottom * scale]
        if any(not math.isfinite(v) or abs(v) > MAX_EXACT_PIXEL for v in endpoints):
            raise ValueError("scaled viewport extent is out of range")
        left, top = max(0, math.floor(endpoints[0])), max(0, math.floor(endpoints[1]))
        right, bottom = min(pixel_width, math.ceil(endpoints[2])), min(pixel_height, math.ceil(endpoints[3]))
        if right <= left or bottom <= top:
            continue
        first_x, first_y = left // TILE_SIZE, top // TILE_SIZE
        last_x, last_y = (right - 1) // TILE_SIZE, (bottom - 1) // TILE_SIZE
        count = (last_x - first_x + 1) * (last_y - first_y + 1)
        if count > max_tiles - len(planned):
            raise ValueError("viewport exceeds the bounded tile plan")
        for row in range(first_y, last_y + 1):
            for column in range(first_x, last_x + 1):
                px, py = column * TILE_SIZE, row * TILE_SIZE
                planned.append(ViewportTile(page, px, py, min(TILE_SIZE, pixel_width - px),
                                            min(TILE_SIZE, pixel_height - py), scale, device_scale))
    return tuple(planned)


def frame_byte_cost(tile, response):
    """Validate native RGBA metadata before allocating/copying its buffer."""
    fields = ("originX", "originY", "width", "height", "stride", "byteLength")
    if any(isinstance(response.get(key), bool) or not isinstance(response.get(key), int) for key in fields):
        raise ValueError("invalid tile frame metadata")
    x, y, width, height, stride, length = (response[key] for key in fields)
    if not (0 < width <= TILE_SIZE + 2 and 0 < height <= TILE_SIZE + 2
            and stride == width * 4 and length == height * stride):
        raise ValueError("tile frame exceeds its RGBA allocation bound")
    if not (tile.x - 1 <= x <= tile.x and tile.y - 1 <= y <= tile.y
            and tile.x + tile.width <= x + width <= tile.x + tile.width + 1
            and tile.y + tile.height <= y + height <= tile.y + tile.height + 1):
        raise ValueError("tile frame does not cover its canonical grid rectangle")
    return length


class TileImageCache:
    """The sole owner of client images; metadata lists must not retain images."""
    def __init__(self, max_bytes=CLIENT_CACHE_BYTES, max_entries=CLIENT_CACHE_ENTRIES):
        for value in (max_bytes, max_entries):
            if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                raise ValueError("cache limits must be nonnegative integers")
        self.max_bytes, self.max_entries = max_bytes, max_entries
        self.retained_bytes = 0
        self.hits = self.misses = self.evictions = 0
        self._entries = OrderedDict()

    def __contains__(self, key):
        return key in self._entries

    def __len__(self):
        return len(self._entries)

    def get(self, key):
        entry = self._entries.get(key)
        if entry is None:
            self.misses += 1
            return None
        self._entries.move_to_end(key)
        self.hits += 1
        return entry[0]

    def put(self, key, value, byte_cost):
        if isinstance(byte_cost, bool) or not isinstance(byte_cost, int) or byte_cost < 0:
            raise ValueError("invalid image byte cost")
        previous = self._entries.pop(key, None)
        if previous is not None:
            self.retained_bytes -= previous[1]
        if self.max_entries == 0 or self.max_bytes == 0 or byte_cost > self.max_bytes:
            return False
        while self._entries and (len(self._entries) >= self.max_entries or self.retained_bytes + byte_cost > self.max_bytes):
            _, (_, removed_cost) = self._entries.popitem(last=False)
            self.retained_bytes -= removed_cost
            self.evictions += 1
        self._entries[key] = (value, byte_cost)
        self.retained_bytes += byte_cost
        return True

    def clear(self):
        self._entries.clear()
        self.retained_bytes = 0

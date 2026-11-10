#!/usr/bin/env python3
"""Execute production error-limit routing with portable registry/codec boundaries.

This compiles unchanged production function bodies and frame-budget helpers. It
checks the group request emitted by Dart; it does not compile the Windows SDK
bridge or execute D3D/Flutter on Windows.
"""

import argparse
from pathlib import Path
import subprocess


PRELUDE = r'''
#include "FrameLifetime.h"
#include "SurfaceImportTickets.h"
#include "../common/bounded_tile_group.h"
#include <variant>
#include <map>
#include <optional>
#include <stdexcept>
#include <iostream>
using namespace rhwp::d3d;
struct Value;
using Map = std::map<Value, Value>;
struct Value : std::variant<std::monostate, bool, int32_t, int64_t, std::string, Map> {
  using variant::variant;
  Value(const char* text) : variant(std::string(text)) {}
  bool operator<(const Value& other) const {
    return static_cast<const variant&>(*this) < static_cast<const variant&>(other);
  }
};
struct MethodError : std::runtime_error {
  MethodError(const char*, const char* message) : std::runtime_error(message) {}
};
struct Descriptor {
  void* handle = nullptr;
  void (*release_callback)(void*) = nullptr;
  void* release_context = nullptr;
};
using SurfaceImports = SurfaceImportTickets<Descriptor>;
ImmutableFrames& frames() { static ImmutableFrames value; return value; }
uint64_t descriptor_opened_notifications() { return 0; }
'''

BOUNDARIES = r'''
struct State {
  PublicationMode publication_mode = PublicationMode::ImmutableOwnershipTransfer;
  bool close_ack_required = true;
  uint64_t generation = 71;
};
struct Record { std::shared_ptr<State> state = std::make_shared<State>(); };
struct Receipt { PublicationMode mode; bool close_ack; uint64_t generation; };
struct Records {
  std::map<int64_t, Receipt> receipts;
  const Receipt* completed(int64_t id) {
    auto found = receipts.find(id); return found == receipts.end() ? nullptr : &found->second;
  }
};
struct Plugin {
  struct TileGroup { rhwp::tiles::Identity identity; Value terminal; };
  std::optional<TileGroup> group_ = TileGroup{{71, {101, 102}}, Value()};
  std::map<uint64_t, Record> records{{101, Record{}}, {102, Record{}}};
  Records textures_;
  Record* record(uint64_t id) {
    auto found = records.find(id); return found == records.end() ? nullptr : &found->second;
  }
'''

TEST = r'''
};
int checks = 0;
#define CHECK(value) do { ++checks; if (!(value)) throw std::runtime_error("failed: " #value); } while (0)
Map observe(Plugin& plugin, Map arguments) {
  Value value(std::move(arguments)); return std::get<Map>(plugin.error_limits(&value));
}
bool retryable(const Map& value) { return std::get<bool>(value.at(Value("backpressureRetryable"))); }
bool acknowledged(const Map& value) { return std::get<bool>(value.at(Value("closeAcknowledgementRequired"))); }
Map group_args(uint64_t id = 101, uint64_t generation = 71) {
  return {{Value("groupId"), Value(int64_t(id))}, {Value("deviceGeneration"), Value(int64_t(generation))}};
}
int main() {
  try {
    Plugin plugin;
    const Map single{{Value("textureId"), Value(int64_t(101))}};
    CHECK(retryable(observe(plugin, single)));
    std::vector<std::shared_ptr<ImmutableFrames::Reservation>> held;
    for (int index = 0; index < 4; ++index) {
      auto credit = frames().reserve(16 * 1024 * 1024, PublicationMode::ImmutableOwnershipTransfer);
      CHECK(credit); held.push_back(credit);
    }
    CHECK(!frames().reserve(1, PublicationMode::ImmutableOwnershipTransfer));
    auto current = observe(plugin, group_args());
    CHECK(retryable(current));
    CHECK(acknowledged(current));
    CHECK(std::get<std::string>(current.at(Value("publicationMode"))) == "immutableOwnershipTransfer");
    CHECK(std::get<int32_t>(current.at(Value("maxFrames"))) == 4);
    CHECK(std::get<int64_t>(current.at(Value("reservedTargetBytes"))) == 64 * 1024 * 1024);
    held.pop_back(); CHECK(frames().reserve(1, PublicationMode::ImmutableOwnershipTransfer));
    held.clear(); CHECK(frames().stats().outstanding == 0);
    plugin.records.at(101).state->publication_mode = PublicationMode::LegacyImmutable;
    CHECK(!retryable(observe(plugin, group_args())));
    plugin.records.at(101).state->publication_mode = PublicationMode::ImmutableOwnershipTransfer;
    // A caller's desired mode cannot select another or obsolete group's policy.
    auto stale = group_args(101, 72); stale[Value("publicationMode")] = Value("immutableOwnershipTransfer");
    CHECK(!retryable(observe(plugin, stale)));
    auto foreign = group_args(102); foreign[Value("publicationMode")] = Value("immutableOwnershipTransfer");
    CHECK(!retryable(observe(plugin, foreign)));
    auto missing = group_args(); missing.erase(Value("deviceGeneration"));
    CHECK(!retryable(observe(plugin, missing)));
    auto malformed = group_args(); malformed[Value("deviceGeneration")] = Value("71");
    CHECK(!retryable(observe(plugin, malformed)));
    CHECK(!retryable(observe(plugin, {})));
    CHECK(retryable(observe(plugin, {{Value("publicationMode"), Value("immutableOwnershipTransfer")}})));
    plugin.records.erase(101);
    plugin.textures_.receipts.emplace(101, Receipt{PublicationMode::ImmutableOwnershipTransfer, true, 71});
    CHECK(retryable(observe(plugin, group_args())));
    CHECK(acknowledged(observe(plugin, group_args())));
    CHECK(retryable(observe(plugin, single)));
    plugin.textures_.receipts.at(101).generation = 70;
    CHECK(!retryable(observe(plugin, group_args())));
    plugin.textures_.receipts.at(101).generation = 71;
    // Four permanent frames still force fallback. Reporting may not refund them.
    for (int index = 0; index < 4; ++index) {
      auto credit = frames().reserve(16 * 1024 * 1024);
      CHECK(credit && credit->begin_gpu(std::make_shared<int>(index)));
      CHECK(credit->producer_complete() && credit->publish_immutable());
    }
    CHECK(frames().stats().permanent == 4);
    CHECK(!retryable(observe(plugin, group_args())));
    CHECK(!frames().reserve(1, PublicationMode::ImmutableOwnershipTransfer));
    std::cout << "production group error limits: " << checks << " checks passed\n";
    return 0;
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
'''


def function(source, signature):
    start = source.index(signature)
    brace = source.index('{', start)
    depth, end = 1, brace + 1
    while depth:
        if source[end] == '{':
            depth += 1
        elif source[end] == '}':
            depth -= 1
        end += 1
    return source[start:end]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source = Path(__file__).resolve().parents[1]
    text = (source / 'rhwp_d3d_plugin.cpp').read_text()
    helpers = '\n'.join(function(text, signature) for signature in (
        'const Map& map(const Value* value)',
        'PublicationMode publication_mode(const Map& args)',
        'uint64_t integer(const Value& value)',
        'Value unsigned_value(uint64_t value)',
        'Value limits(PublicationMode mode',
    ))
    args.output.mkdir(parents=True, exist_ok=True)
    harness = args.output / 'group_error_limits.cpp'
    harness.write_text(PRELUDE + helpers + BOUNDARIES + function(text, '  Value error_limits(const Value* arguments)') + TEST)
    executable = args.output / 'group_error_limits'
    subprocess.run(['g++', '-std=c++17', '-Wall', '-Wextra', '-Werror', '-pthread',
                    '-I' + str(source), str(harness), '-o', str(executable)], check=True, timeout=60)
    subprocess.run([str(executable)], check=True, timeout=30)


if __name__ == '__main__':
    main()

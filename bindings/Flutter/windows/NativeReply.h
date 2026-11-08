#ifndef RHWP_WINDOWS_NATIVE_REPLY_H_
#define RHWP_WINDOWS_NATIVE_REPLY_H_

#include <charconv>
#include <cmath>
#include <stdexcept>
#include <string>
#include <string_view>

namespace rhwp::d3d {
// Reads bounded trusted native metadata, never HWP/HWPX or channel input JSON.
// Field lookup skips nested objects/escaped strings, rather than matching a
// misleading substring inside a document error message or child object.
class NativeReply {
 public:
  explicit NativeReply(std::string_view text) : text_(text) {
    if (text.size() > 64 * 1024) throw std::runtime_error("native reply exceeds 64 KiB");
  }
  NativeReply field(std::string_view wanted) const {
    size_t offset = 0; space(text_, offset);
    if (offset >= text_.size() || text_[offset++] != '{') throw std::runtime_error("native reply is not an object");
    for (size_t fields = 0; fields < 64; ++fields) {
      space(text_, offset);
      if (offset < text_.size() && text_[offset] == '}') break;
      auto key = token_string(text_, offset);
      space(text_, offset);
      if (offset >= text_.size() || text_[offset++] != ':') throw std::runtime_error("invalid native JSON field");
      space(text_, offset); size_t start = offset; skip(text_, offset, 0);
      if (key == wanted) return NativeReply(text_.substr(start, offset - start));
      space(text_, offset);
      if (offset >= text_.size() || text_[offset] != ',') break;
      ++offset;
    }
    throw std::runtime_error("missing native reply field");
  }
  bool boolean() const {
    if (text_ == "true") return true;
    if (text_ == "false") return false;
    throw std::runtime_error("invalid native boolean");
  }
  uint64_t integer() const {
    uint64_t value = 0;
    const auto result = std::from_chars(text_.data(), text_.data() + text_.size(), value);
    if (result.ec != std::errc() || result.ptr != text_.data() + text_.size()) throw std::runtime_error("invalid native integer");
    return value;
  }
  double number() const {
    double value = 0;
    const auto result = std::from_chars(text_.data(), text_.data() + text_.size(), value);
    if (result.ec != std::errc() || result.ptr != text_.data() + text_.size() || !std::isfinite(value)) throw std::runtime_error("invalid native number");
    return value;
  }
  std::string text() const { size_t offset = 0; return token_string(text_, offset); }
 private:
  std::string_view text_;
  static void space(std::string_view text, size_t& offset) {
    while (offset < text.size() && (text[offset] == ' ' || text[offset] == '\n' || text[offset] == '\r' || text[offset] == '\t')) ++offset;
  }
  static std::string token_string(std::string_view text, size_t& offset) {
    if (offset >= text.size() || text[offset++] != '"') throw std::runtime_error("invalid native JSON string");
    std::string result;
    while (offset < text.size()) {
      char value = text[offset++];
      if (value == '"') return result;
      if (value == '\\') {
        if (offset >= text.size()) break;
        value = text[offset++];
        if (value == 'n') value = '\n'; else if (value == 'r') value = '\r'; else if (value == 't') value = '\t';
        else if (value == 'u') {
          if (text.size() - offset < 4) break;
          if (result.size() > 4090) throw std::runtime_error("native JSON string exceeds its bound");
          result += "\\u"; result.append(text.substr(offset, 4)); offset += 4; continue;
        }
      }
      if (result.size() >= 4096) throw std::runtime_error("native JSON string exceeds its bound");
      result += value;
    }
    throw std::runtime_error("unterminated native JSON string");
  }
  static void skip(std::string_view text, size_t& offset, size_t depth) {
    if (depth > 16 || offset >= text.size()) throw std::runtime_error("invalid/deep native JSON");
    if (text[offset] == '"') { (void)token_string(text, offset); return; }
    if (text[offset] == '[' || text[offset] == '{') {
      const char end = text[offset++] == '[' ? ']' : '}';
      for (size_t elements = 0; elements < 128; ++elements) {
        space(text, offset);
        if (offset < text.size() && text[offset] == end) { ++offset; return; }
        if (end == '}') { (void)token_string(text, offset); space(text, offset); if (offset >= text.size() || text[offset++] != ':') break; space(text, offset); }
        skip(text, offset, depth + 1); space(text, offset);
        if (offset < text.size() && text[offset] == end) { ++offset; return; }
        if (offset >= text.size() || text[offset++] != ',') break;
      }
      throw std::runtime_error("invalid/large native JSON container");
    }
    const auto start = offset;
    while (offset < text.size() && text[offset] != ',' && text[offset] != '}' && text[offset] != ']' &&
           text[offset] != ' ' && text[offset] != '\n' && text[offset] != '\r' && text[offset] != '\t') ++offset;
    if (offset == start) throw std::runtime_error("invalid native JSON scalar");
  }
};
}  // namespace rhwp::d3d
#endif

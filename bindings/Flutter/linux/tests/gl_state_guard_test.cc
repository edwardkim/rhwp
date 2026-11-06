#include "gl_state_guard.h"
#include <gtk/gtk.h>
#include <array>
#include <iostream>
#include <stdexcept>

namespace {
void check(bool condition) { if (!condition) throw std::runtime_error("GL handoff state was not restored"); }
GLint integer(GLenum name) { GLint value; glGetIntegerv(name, &value); return value; }

void deleted_owned_names_are_not_restored() {
  GLuint textures[2], framebuffers[2];
  glGenTextures(2, textures);
  glActiveTexture(GL_TEXTURE3);
  glBindTexture(GL_TEXTURE_2D, textures[0]);
  glActiveTexture(GL_TEXTURE4);
  glBindTexture(GL_TEXTURE_2D, textures[1]);
  glActiveTexture(GL_TEXTURE3);
  glGenFramebuffers(2, framebuffers);
  glBindFramebuffer(GL_DRAW_FRAMEBUFFER, framebuffers[0]);
  glBindFramebuffer(GL_READ_FRAMEBUFFER, framebuffers[1]);
  check(glGetError() == GL_NO_ERROR);
  {
    rhwp_flutter::GlStateGuard outer;
    {
      rhwp_flutter::GlStateGuard inner;
      // The plugin's close/resize path deletes these confirmed owned names.
      glDeleteTextures(1, &textures[0]);
      check(!glIsTexture(textures[0]));
      rhwp_flutter::GlStateGuard::forget_deleted_texture(textures[0]);
      check(integer(GL_DRAW_FRAMEBUFFER_BINDING) == GLint(framebuffers[0]));
      glDeleteFramebuffers(1, &framebuffers[0]);
      check(!glIsFramebuffer(framebuffers[0]));
      rhwp_flutter::GlStateGuard::forget_deleted_framebuffer(framebuffers[0]);
      // Even later allocations must not make the saved deleted name live again.
      GLuint replacement;
      glGenTextures(1, &replacement);
      glBindTexture(GL_TEXTURE_2D, replacement);
      glDeleteTextures(1, &replacement);
      rhwp_flutter::GlStateGuard::forget_deleted_texture(replacement);
    }
    check(integer(GL_DRAW_FRAMEBUFFER_BINDING) == 0);
    check(integer(GL_READ_FRAMEBUFFER_BINDING) == GLint(framebuffers[1]));
    check(integer(GL_TEXTURE_BINDING_2D) == 0);
    check(glGetError() == GL_NO_ERROR);
  }
  // Both nested snapshots forget only their deleted object/namespace. Live
  // caller texture units, read FBO, active unit, and other state survive.
  check(integer(GL_ACTIVE_TEXTURE) == GL_TEXTURE3);
  check(integer(GL_TEXTURE_BINDING_2D) == 0);
  glActiveTexture(GL_TEXTURE4);
  check(integer(GL_TEXTURE_BINDING_2D) == GLint(textures[1]));
  check(integer(GL_DRAW_FRAMEBUFFER_BINDING) == 0);
  check(integer(GL_READ_FRAMEBUFFER_BINDING) == GLint(framebuffers[1]));
  check(glGetError() == GL_NO_ERROR);
  GLuint retired_read;
  glGenFramebuffers(1, &retired_read);
  glBindFramebuffer(GL_DRAW_FRAMEBUFFER, framebuffers[1]);
  glBindFramebuffer(GL_READ_FRAMEBUFFER, retired_read);
  {
    rhwp_flutter::GlStateGuard guard;
    glDeleteFramebuffers(1, &retired_read);
    check(!glIsFramebuffer(retired_read));
    rhwp_flutter::GlStateGuard::forget_deleted_framebuffer(retired_read);
  }
  check(integer(GL_READ_FRAMEBUFFER_BINDING) == 0);
  check(integer(GL_DRAW_FRAMEBUFFER_BINDING) == GLint(framebuffers[1]));
  check(integer(GL_TEXTURE_BINDING_2D) == GLint(textures[1]));
  check(glGetError() == GL_NO_ERROR);
  glBindFramebuffer(GL_FRAMEBUFFER, 0);
  glDeleteFramebuffers(1, &framebuffers[1]);
  glDeleteTextures(1, &textures[1]);
}
}  // namespace

int main(int argc, char** argv) {
  if (!gtk_init_check(&argc, &argv)) { std::cerr << "GTK display unavailable\n"; return 2; }
  auto* window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
  gtk_widget_realize(window);
  GError* error = nullptr;
  auto* context = gdk_window_create_gl_context(gtk_widget_get_window(window), &error);
  if (!context) { std::cerr << (error ? error->message : "GL context failed") << '\n'; return 2; }
  gdk_gl_context_set_required_version(context, 3, 3);
  gdk_gl_context_set_use_es(context, FALSE);
  if (!gdk_gl_context_realize(context, &error)) { std::cerr << error->message << '\n'; return 2; }
  gdk_gl_context_make_current(context);
  try {
    GLuint fbo[2], textures[2], vao, buffer, sampler;
    glGenFramebuffers(2, fbo); glGenTextures(2, textures);
    glGenVertexArrays(1, &vao); glBindVertexArray(vao);
    glGenBuffers(1, &buffer); glBindBuffer(GL_ARRAY_BUFFER, buffer);
    glBufferData(GL_ARRAY_BUFFER, 64, nullptr, GL_STATIC_DRAW);
    glBindBufferBase(GL_UNIFORM_BUFFER, 0, buffer);
    glGenSamplers(1, &sampler);
    glBindFramebuffer(GL_DRAW_FRAMEBUFFER, fbo[0]);
    glBindFramebuffer(GL_READ_FRAMEBUFFER, fbo[1]);
    glActiveTexture(GL_TEXTURE3); glBindTexture(GL_TEXTURE_2D, textures[0]); glBindSampler(3, sampler);
    glViewport(3, 5, 61, 67); glScissor(7, 9, 11, 13);
    glEnable(GL_BLEND); glEnable(GL_SCISSOR_TEST); glEnable(GL_STENCIL_TEST);
    glColorMask(GL_FALSE, GL_TRUE, GL_FALSE, GL_TRUE); glDepthMask(GL_FALSE);
    glBlendFuncSeparate(GL_ONE, GL_DST_COLOR, GL_SRC_ALPHA, GL_ONE_MINUS_DST_ALPHA);
    glStencilMaskSeparate(GL_FRONT, 0x3c); glStencilMaskSeparate(GL_BACK, 0x5a);
    glPixelStorei(GL_UNPACK_ROW_LENGTH, 17); glPixelStorei(GL_PACK_ALIGNMENT, 8);
    check(glGetError() == GL_NO_ERROR);
    {
      rhwp_flutter::GlStateGuard guard;
      glBindFramebuffer(GL_FRAMEBUFFER, 0); glBindVertexArray(0); glBindBuffer(GL_ARRAY_BUFFER, 0);
      glBindBufferBase(GL_UNIFORM_BUFFER, 0, 0);
      glActiveTexture(GL_TEXTURE3); glBindTexture(GL_TEXTURE_2D, textures[1]); glBindSampler(3, 0);
      glActiveTexture(GL_TEXTURE0); glViewport(0, 0, 1, 1); glScissor(0, 0, 2, 2);
      glDisable(GL_BLEND); glDisable(GL_SCISSOR_TEST); glDisable(GL_STENCIL_TEST);
      glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE); glDepthMask(GL_TRUE);
      glBlendFunc(GL_ZERO, GL_ZERO); glStencilMask(0);
      glPixelStorei(GL_UNPACK_ROW_LENGTH, 0); glPixelStorei(GL_PACK_ALIGNMENT, 4);
    }
    check(integer(GL_DRAW_FRAMEBUFFER_BINDING) == GLint(fbo[0]));
    check(integer(GL_READ_FRAMEBUFFER_BINDING) == GLint(fbo[1]));
    check(integer(GL_VERTEX_ARRAY_BINDING) == GLint(vao));
    check(integer(GL_ARRAY_BUFFER_BINDING) == GLint(buffer));
    check(integer(GL_UNIFORM_BUFFER_BINDING) == GLint(buffer));
    GLint uniform; glGetIntegeri_v(GL_UNIFORM_BUFFER_BINDING, 0, &uniform); check(uniform == GLint(buffer));
    check(integer(GL_ACTIVE_TEXTURE) == GL_TEXTURE3);
    check(integer(GL_TEXTURE_BINDING_2D) == GLint(textures[0]));
    check(integer(GL_SAMPLER_BINDING) == GLint(sampler));
    std::array<GLint, 4> viewport{}, scissor{};
    glGetIntegerv(GL_VIEWPORT, viewport.data()); glGetIntegerv(GL_SCISSOR_BOX, scissor.data());
    check(viewport == std::array<GLint, 4>{3, 5, 61, 67});
    check(scissor == std::array<GLint, 4>{7, 9, 11, 13});
    GLboolean mask[4], depth;
    glGetBooleanv(GL_COLOR_WRITEMASK, mask); glGetBooleanv(GL_DEPTH_WRITEMASK, &depth);
    check(!mask[0] && mask[1] && !mask[2] && mask[3] && !depth);
    check(glIsEnabled(GL_BLEND) && glIsEnabled(GL_SCISSOR_TEST) && glIsEnabled(GL_STENCIL_TEST));
    check(integer(GL_BLEND_DST_RGB) == GL_DST_COLOR && integer(GL_BLEND_DST_ALPHA) == GL_ONE_MINUS_DST_ALPHA);
    check(integer(GL_STENCIL_WRITEMASK) == 0x3c && integer(GL_STENCIL_BACK_WRITEMASK) == 0x5a);
    check(integer(GL_UNPACK_ROW_LENGTH) == 17 && integer(GL_PACK_ALIGNMENT) == 8);
    check(glGetError() == GL_NO_ERROR);
    deleted_owned_names_are_not_restored();
    check(glGetError() == GL_NO_ERROR);
    std::cout << "GL state handoff passed: " << glGetString(GL_RENDERER) << '\n';
    glBindFramebuffer(GL_FRAMEBUFFER, 0); glBindVertexArray(0);
    glDeleteFramebuffers(2, fbo); glDeleteTextures(2, textures); glDeleteVertexArrays(1, &vao);
    glDeleteBuffers(1, &buffer); glDeleteSamplers(1, &sampler);
  } catch (const std::exception& failure) { std::cerr << failure.what() << '\n'; return 1; }
  gdk_gl_context_clear_current(); g_object_unref(context); gtk_widget_destroy(window);
  return 0;
}

#include <new>
#include <cerrno>

std::new_handler std::get_new_handler() noexcept {
  return nullptr;
}

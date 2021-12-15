#include <new>
#include <cerrno>

std::new_handler std::get_new_handler() noexcept {
  return nullptr;
}

extern "C" int posix_memalign(void**, size_t, size_t) {
  return ENOMEM;
}


// newlib support
#include <sys/types.h>

extern "C" caddr_t sbrk(int incr) {
  errno = ENOMEM;
  return (caddr_t)-1;
}

extern "C" void _exit(void) {
  while (1) __asm__("hlt");
}

extern "C" int getpid(void) {
  return 1;
}

extern "C" int kill(int pid, int sig) {
  errno = EINVAL;
  return -1;
}
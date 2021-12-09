/*
 * @Date: 2021-12-06 17:44:58
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-09 11:23:48
 * @FilePath: /tCore/sbi/libs/string.c
 */

#include "string.h"

size_t strlen(const char *s) {
  size_t cnt = 0;
  while (*s++ != '\0') cnt++;
  return cnt;
}

void *memset(void *s, char c, size_t n) {
  char *p = s;
  while (n-- > 0) *p++ = c;
  return s;
}

void *memmove(void *dst, const void *src, size_t n) {
  const char *s = src;
  char *d = dst;
  if (s < d && s + n > d) {
    s += n, d += n;
    while (n-- > 0) *--d = *--s;
  } else {
    while (n-- > 0) *d++ = *s++;
  }
  return dst;
}

void *memcpy(void *dst, const void *src, size_t n) {
  const char *s = src;
  char *d = dst;
  while (n-- > 0) {
    *d++ = *s++;
  }
  return dst;
}

int memcmp(const void *v1, const void *v2, size_t n) {
  const char *s1 = (const char *)v1;
  const char *s2 = (const char *)v2;
  while (n-- > 0) {
    if (*s1 != *s2) {
      return (int)((unsigned char)*s1 - (unsigned char)*s2);
    }
    s1++, s2++;
  }
  return 0;
}
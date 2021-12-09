/*
 * @Date: 2021-12-06 17:44:52
 * @Author: Kaifu Tian
 * @LastEditTime: 2021-12-09 11:24:10
 * @FilePath: /tCore/sbi/libs/string.h
 */

#ifndef _SBI_STRING_H
#define _SBI_STRING_H

#include <include/common.h>

size_t strlen(const char *s);
size_t strnlen(const char *s, size_t len);

char *strcpy(char *dst, const char *src);
char *strncpy(char *dst, const char *src, size_t len);
char *strcat(char *dst, const char *src);
char *strdup(const char *src);
char *stradd(const char *src1, const char *src2);

int strcmp(const char *s1, const char *s2);
int strncmp(const char *s1, const char *s2, size_t n);

char *strchr(const char *s, char c);
char *strfind(const char *s, char c);
long strtol(const char *s, char **endptr, int base);

void *memset(void *s, char c, size_t n);
void *memmove(void *dst, const void *src, size_t n);
void *memcpy(void *dst, const void *src, size_t n);
int memcmp(const void *v1, const void *v2, size_t n);

#endif
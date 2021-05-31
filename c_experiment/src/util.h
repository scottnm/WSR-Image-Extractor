#pragma once

/////////////////
// UTILITY MACROS
#define STATEMENT(X) do { (X); } while(0)
#define UNREF(X) STATEMENT((void)(X))

//////////////////
// BUFFER HELPERS
#define ARRAYSIZE(x) (sizeof((x))/sizeof((x)[0]))

//////////////////
// STRING HELPERS
typedef struct str_t {
    char* bytes;
    uint64_t len;
} str_t;

#define cstr(s) { .bytes=(s), .len=ARRAYSIZE(s)-1 }
inline bool cstr_eq(const char* a, const char* b) { return strcmp(a, b) == 0; }
inline bool mem_eq(const void* a, const void* b, size_t len) { return memcmp(a, b, len) == 0; }
inline bool str_eq(str_t a, str_t b) { return a.len == b.len && mem_eq(a.bytes, b.bytes, a.len); }
inline bool str_starts_with(str_t s, str_t prefix)
{
    if (s.len < prefix.len)
    {
        return false;
    }
    str_t truncated_s = {.bytes = s.bytes, .len = prefix.len };
    return str_eq(truncated_s, prefix);
}

str_t make_str(const char* s)
{
    if (s == NULL)
    {
        return (str_t) {0};
    }

    size_t len = strlen(s);
    char* buffer = (char*)malloc(len + 1);
    if (buffer == NULL)
    {
        free(buffer);
        return (str_t) {0};
    }

    memcpy(buffer, s, len);
    buffer[len] = 0;  // null terminate

    return (str_t) { .bytes = buffer, .len = len };
}

//////////////////
// LOGGING HELPERS
#define Log(FMT, ...) printf(FMT "\n", __VA_ARGS__)

///////////////
// SPAN HELPERS
#define DEF_SPAN_T(type, typename) \
    typedef struct typename { \
        type* data; \
        size_t count; \
    } typename 

#define SPAN_ADV(span) \
    do { \
        (span).data += 1; \
        (span).count -= 1; \
    } while (0) \

#define SPAN_FIRST(span, cnt) \
    { \
        .data = (span).data, \
        .count = min((span).count, cnt), \
    }

DEF_SPAN_T(const char*, char_ptr_span_t);
DEF_SPAN_T(uint8_t, byte_span_t);

byte_span_t get_first_split(byte_span_t buffer, uint8_t split_char)
{
    // Skip over any leading split chars
    while (buffer.count > 0 && buffer.data[0] == split_char)
    {
        SPAN_ADV(buffer);
    }

    // The first split of an empty span is another empty span
    if (buffer.count == 0)
    {
        return (byte_span_t){0};
    }

    assert(buffer.data != NULL);

    uint8_t* split_char_ptr = memchr(buffer.data, split_char, buffer.count);
    if (split_char_ptr == NULL)
    {
        // If there is no split character than the first split is the entire span
        return buffer;
    }

    return (byte_span_t) { .data = buffer.data, .count=(split_char_ptr - buffer.data) };
}

byte_span_t get_next_split(const byte_span_t current_split, const byte_span_t full_span, uint8_t split_char)
{
    assert(current_split.count <= full_span.count);
    assert(current_split.data >= full_span.data);
    assert(current_split.data <= full_span.data + full_span.count);

    // Skip over any leading split chars
    size_t current_split_offset = current_split.data - full_span.data;
    byte_span_t remaining_span = {
        .data = current_split.data + current_split.count,
        .count = full_span.count - current_split.count - current_split_offset };
    return get_first_split(remaining_span, split_char);
}

#define split_by(span_t, iter_var_name, buffer, split_char) \
    for ( \
        span_t iter_var_name = get_first_split((buffer), (split_char)); \
        iter_var_name.data != NULL; \
        iter_var_name = get_next_split((iter_var_name), (buffer), (split_char)) \
        ) \

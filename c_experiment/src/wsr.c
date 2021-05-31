#include "pch.h"

typedef struct run_options_t {
    str_t file_path;   
    str_t output_dir;
    bool is_preview;
} run_options_t;

typedef enum parse_args_err_t
{
    PARSE_ARGS_SUCC = 0,
    PARSE_ARGS_MISSING_INPUT_FILE,
    PARSE_ARGS_MISSING_OUTPUT_DIR,
} parse_args_err_t;

parse_args_err_t
get_run_options_from_args(
    int argc,
    char** argv,
    _Out_ run_options_t* out_run_options)
{
    char_ptr_span_t args = {
        .data = argv,
        .count = argc,
    };

    SPAN_ADV(args);
    args = (char_ptr_span_t) SPAN_FIRST(args, 3);
    switch (args.count)
    {
        case 0: return PARSE_ARGS_MISSING_INPUT_FILE;
        case 1: return PARSE_ARGS_MISSING_OUTPUT_DIR;
        default: break;
    }

    bool is_preview = (args.count > 2 && cstr_eq(args.data[2], "--preview"));

    args = (char_ptr_span_t) SPAN_FIRST(args, 2);
    assert(args.count == 2);

    const char* file_path = args.data[0];
    const char* output_dir = args.data[1];

    *out_run_options = (run_options_t) {
        .file_path = make_str(file_path), // could fail to alloc but who cares lol
        .output_dir = make_str(output_dir), // could fail to alloc but who cares lol
        .is_preview = is_preview,
    };

    return PARSE_ARGS_SUCC;
}

void print_usage_and_exit(int argc, char** argv, str_t err_msg)
{
    Log("Invalid usage: %s", err_msg.bytes);
    Log("Usage: wsr_image <path to file> <output directory> [--preview]");
    Log("Arguments:");
    for (int i = 0; i < argc; i += 1)
    {
        Log("    %i: %s", i, argv[i]);
    }

    exit(1);
}

typedef enum scan_phase_t
{
    SCAN_PHASE_LOOK_FOR_JPEG = 0,
    SCAN_PHASE_CHECK_BASE_64,
    SCAN_PHASE_LOOK_FOR_IMAGE_NAME,
    SCAN_PHASE_FIND_IMAGE_DATA_START,
    SCAN_PHASE_READ_JPEG,
} scan_phase_t;

byte_span_t load_wsr_file(str_t file_path)
{
    FILE* wsr_file;
    errno_t fopen_err = fopen_s(&wsr_file, file_path.bytes, "rb");
    if (fopen_err != 0)
    {
        Log("Failed to open WSR file, %s! err=0x%08X", file_path.bytes, fopen_err);
        return (byte_span_t){0};
    }

    errno_t seek_err = fseek(wsr_file, 0, SEEK_END);
    assert(seek_err == 0);

    size_t wsr_file_byte_count = ftell(wsr_file);
    seek_err = fseek(wsr_file, 0, SEEK_SET);
    assert(seek_err == 0);

    byte_span_t wsr_buffer = {0};
    wsr_buffer.data = (uint8_t*)malloc(wsr_file_byte_count);
    if (wsr_buffer.data != NULL)
    {
        wsr_buffer.count = wsr_file_byte_count;
        size_t bytes_read = fread(wsr_buffer.data, 1, wsr_buffer.count, wsr_file);
        if (bytes_read != wsr_buffer.count)
        {
            Log("Failed to read full WSR file! read %zu/%zu bytes", bytes_read, wsr_buffer.count);
            free(wsr_buffer.data);
            wsr_buffer.data = NULL;
        }
    }

    fclose(wsr_file);
    return wsr_buffer;
}

static const str_t CONTENT_TYPE_JPEG_LINE = cstr("Content-Type: image/jpeg");
static const str_t BASE_64_ENCODING_HEADER_LINE = cstr("Content-Transfer-Encoding: base64");
static const str_t IMAGE_NAME_PREFEX = cstr("Content-Location: ");

size_t count_base64_jpgs(byte_span_t wsr_buffer)
{
    scan_phase_t current_scan_phase = SCAN_PHASE_LOOK_FOR_JPEG;

    size_t image_count = 0;
    split_by(byte_span_t, next_line_bytes, wsr_buffer, '\n')
    {
        str_t next_line = {.bytes=(char*)next_line_bytes.data, .len=next_line_bytes.count};
        // DEBUG: Log("LINE: %.*s", (int)next_line.len, next_line.bytes);
        switch (current_scan_phase)
        {
            case SCAN_PHASE_LOOK_FOR_JPEG:
            {
                if (str_starts_with(next_line, CONTENT_TYPE_JPEG_LINE))
                {
                    current_scan_phase = SCAN_PHASE_CHECK_BASE_64;
                }
                break;
            }
            case SCAN_PHASE_CHECK_BASE_64:
            {
                if (str_starts_with(next_line, BASE_64_ENCODING_HEADER_LINE))
                {
                    image_count += 1;
                    current_scan_phase = SCAN_PHASE_LOOK_FOR_JPEG;
                }
                break;
            }
            default:
            {
                Log("Unexpected scan phase value! val=%i", current_scan_phase);
                assert("Unexpected scan phase err!");
                break;
            }
        }
    }

    return image_count;
}

int main(int argc, char** argv)
{
    run_options_t run_options;
    parse_args_err_t parse_err = get_run_options_from_args(argc, argv, &run_options);
    switch (parse_err)
    {
        case PARSE_ARGS_SUCC:
        {
            // SUCCESS! onward
            break;
        }
        case PARSE_ARGS_MISSING_INPUT_FILE:
        {
            print_usage_and_exit(argc, argv, (str_t)cstr("Missing input file parameter"));
            break;
        }
        case PARSE_ARGS_MISSING_OUTPUT_DIR:
        {
            print_usage_and_exit(argc, argv, (str_t)cstr("Missing input file parameter"));
            break;
        }
        default:
        {
            Log("Invalid parse err value! val=%i", parse_err);
            assert("Invalid parse err!");
            break;
        }
    }

    Log("Run options:");
    Log("    file_path: %s", run_options.file_path.bytes);
    Log("    output_dir: %s", run_options.output_dir.bytes);

    byte_span_t wsr_buffer = load_wsr_file(run_options.file_path);
    if (wsr_buffer.data == NULL)
    {
        Log("Failed to load wsr file!");
        return 1;
    }

    size_t base64_jpg_count = count_base64_jpgs(wsr_buffer);
    Log("Found: %zu images!", base64_jpg_count);

    return 0;
}
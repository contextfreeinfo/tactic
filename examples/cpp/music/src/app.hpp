#pragma once

#include <cstdio>
#include <optional>
#include <taca.hpp>
#include <vec.hpp>
#include <vector>

namespace music {

// Which means 12 ticks/second or 720 ticks/minute.
// If each is a 16th note, and a a quarter note is one beat, that's tempo 180.
// And our 36 notes are 3 seconds.
constexpr std::size_t frames_per_tick = 5;

// Rects include notes, highlight bands, background, markers, etc.
// TODO Be more precise with extras?
constexpr std::size_t max_extra_rects = 1000;
constexpr std::size_t max_pitches = 25;
constexpr std::size_t max_ticks = 48;

// Just play and rewind triangles.
constexpr std::size_t max_tris = 2;

// The top of our scale is C7, and our sample is D6.
constexpr auto semitones_default = 11.0f;

enum struct DrawMode {
    Start,
    Add,
    Remove,
};

struct DrawInstance {
    vec::Vec2f offset;
    vec::Vec2f scale;
    float light;
};

struct DrawInfo {
    taca::Buffer index_buffer;
    taca::Buffer vertex_buffer;
    taca::Buffer instance_buffer;
    taca::Buffer vertex_tri_buffer;
    taca::Buffer instance_tri_buffer;
    std::vector<DrawInstance> instance_values;
};

struct PlayInfo {
    std::size_t frames_until_tick;
    bool playing;
    std::size_t tick;
};

struct Note {
    float semitones;
};

struct Tick {
    // Overly flexible for the current app, but meh.
    std::vector<Note> notes;
};

struct Song {
    float ticks_per_second;
    std::vector<Tick> ticks;
};

struct App {
    taca::Sound ding;
    DrawInfo draw_info;
    DrawMode draw_mode;
    PlayInfo play_info;
    bool ready;
    std::size_t rewind_tick;
    Song song;
    taca::WindowState window_state;
};

struct BandInfo {
    vec::Vec2f bands_offset;
    vec::Vec2f bands_scale;
    vec::Vec2f button_back_offset;
    vec::Vec2f button_play_offset;
    vec::Vec2f button_scale;
    std::array<std::optional<std::size_t>, 2> cell_index;
    vec::Vec2f cell_offset;
    vec::Vec2f cell_scale;
    vec::Vec2f cell_start;
    vec::Vec2f pointer;
    vec::Vec2f window_size;
};

template <typename... Args>
auto print(const char* format, Args... args) -> void {
    char buffer[100];
    std::snprintf(buffer, sizeof(buffer), format, args...);
    taca::print(buffer);
}

auto calc_bands(App& app) -> BandInfo {
    using namespace vec;
    auto window_size = Vec2f{
        app.window_state.size.x,
        app.window_state.size.y,
    };
    // auto window_size = std::to_array<float>({1, 1});
    auto pointer = Vec2f{
        app.window_state.pointer.x,
        app.window_state.pointer.y,
    };
    // auto pointer = pointer_px / window_size_px;
    auto grid_count = Vec2f{max_ticks, max_pitches} + 1;
    auto margin_count = Vec2f{0, 2};
    auto margin = window_size / (grid_count + margin_count) * margin_count;
    auto music_size = window_size - margin;
    auto music_pos_frac = (pointer - margin) / music_size;
    auto cell = floor(music_pos_frac * grid_count) - 1;
    auto active = cell[0] >= -1 && cell[1] >= -1;
    auto dim = static_cast<std::size_t>(0);
    auto cell_index = vec::map<std::optional<std::size_t>>(
        cell,
        [&dim, active, grid_count](auto x) {
            return 0 <= x && x < grid_count[dim++] && active
                ? std::make_optional<std::size_t>(x)
                : std::nullopt;
        }
    );
    auto grid_pos_frac = (cell + 1) / grid_count + 0.5 / grid_count;
    auto grid_pos = grid_pos_frac * music_size + margin;
    auto table_size = Vec2f{window_size[0], music_size[1]};
    auto button_scale = margin[1] / window_size;
    auto cell_scale = music_size / grid_count / window_size;
    return {
        .bands_offset = (2 * margin + table_size) / window_size - 1,
        .bands_scale = table_size / window_size,
        .button_back_offset = Vec2f{-1, -1} + button_scale * Vec2f{4, 1},
        .button_play_offset = Vec2f{-1, -1} + button_scale * Vec2f{1.5, 1},
        .button_scale = button_scale,
        .cell_index = cell_index,
        .cell_offset = 2 * (grid_pos / window_size) - 1,
        .cell_scale = cell_scale,
        .cell_start = 2 * margin / window_size - 1 + 3 * cell_scale,
        .pointer = 2 * pointer / window_size - 1,
        .window_size = window_size,
    };
}

auto song_print(const Song& song) -> void {
    for (const auto& tick : song.ticks) {
        print("%s", "---");
        for (const auto& note : tick.notes) {
            print("semitones: %f", note.semitones);
        }
    }
    print("%s\n", "---");
}

} // namespace music

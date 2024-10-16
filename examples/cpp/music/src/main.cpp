#include "app.hpp"
#include "control.hpp"
#include "musicbox-data.hpp"
#include <taca.hpp>

namespace music {

// Init fields to zero.
App app = {};

void start() {
    taca::title_update("Music Box (Taca Demo)");
    taca::print("Hi from C++!");
    app.ding = taca::sound_decode(musicbox_data);
}

// clang-format off
__attribute__((export_name("update")))
// clang-format on
void update(taca::EventKind event) {
    switch (event) {
        case taca::EventKind::Frame: {
            app.window_state = taca::window_state();
            update_control(&app);
            app.was_pressed = app.window_state.press;
            break;
        }
        case taca::EventKind::Key: {
            break;
        }
        case taca::EventKind::TasksDone: {
            taca::print("sounds loaded");
            app.ready = true;
            break;
        }
    }
}

} // namespace music

// Even if I say -Wl,--no-entry, I still get a _start, and the overall size is
// larger, so just use main. Maybe I'm just missing some option.
int main() {
    music::start();
}

mod electron;
mod prompt;

use prompt::App;

fn main() -> anyhow::Result<()> {
    App::default().run()
}

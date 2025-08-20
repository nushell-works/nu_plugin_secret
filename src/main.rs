use nu_plugin::{serve_plugin, MsgPackSerializer};
use nu_plugin_secret::SecretPlugin;

fn main() {
    serve_plugin(&SecretPlugin, MsgPackSerializer {})
}

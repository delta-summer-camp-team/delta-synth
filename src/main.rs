use slint::slint;

slint::slint! {
    import { Button } from "std-widgets.slint";

    export component AppWindow inherits Window {
        width: 1280px;
        height: 720px;
        title: "Minimal Slint App";

        Button {
            text: "Hi";
            // можно добавить position и т.п.
        }
        Button {
            text: "привет";
            // можно добавить position и т.п.
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?; // <- AppWindow теперь сгенерирован из макроса выше
    app.run();
    Ok(())
}


use slint::slint;

slint::slint! {
    import { VerticalBox, HorizontalBox, Button } from "std-widgets.slint";

    export component App inherits Window {
        callback button_clicked(index: int, pressed: bool, value: float);

        VerticalBox {
            spacing: 10px;

            // Кнопка 0
            Button {
                text: "Button 0";
                clicked => {
                    root.button_clicked(0, true, 0.5);
                }
            }

            // Кнопка 1
            Button {
                text: "Button 1";
                clicked => {
                    root.button_clicked(1, true, 1.0);
                }
            }

            // Кнопка 2
            Button {
                text: "Button 2";
                clicked => {
                    root.button_clicked(2, true, 2.0);
                }
            }
        }
    }
}
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    let weak = app.as_weak();

    app.on_button_clicked(move |index, pressed, value| {
        println!(
            "Button {} clicked: pressed = {}, value = {}",
            index, pressed, value
        );




        // Или обновить состояние (пример — установить значение снаружи)
        if let Some(app) = weak.upgrade() {
            // можно app.set_some_property(...) здесь
        }
    });

    app.run()
}
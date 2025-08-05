use slint::slint;


slint::slint! {
    import { HorizontalBox, VerticalBox, Slider, Button } from "std-widgets.slint";

    export component AppWindow inherits Window {
        width: 1280px;
        height: 720px;
        title: "Minimal Slint App";

        VerticalBox {
            spacing: 20px;

            HorizontalBox {
                spacing: 10px;

                label := Text {
                    text: "Button not clicked";
                    font-size: 20px;
                }

                Button {
                    text: "First";
                    clicked => {
                        label.text = "Button clicked";
                    }
                }

                Button { text: "Second";          
            
                        }
                Button { text: "Third"; }
                 Slider {
                  orientation:vertical;
                value: 42;
            }
            }

            HorizontalBox {
                spacing: 10px;

                Button { text: "Right 1"; }
                Button { text: "Right 2"; }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    app.run();
    Ok(())
}

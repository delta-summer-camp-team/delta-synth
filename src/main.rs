use slint::slint;

slint! {
    import{Button}
    export component App inherits Window {
        width: 1280px;
        height: 720px;
        title: "Minimal Slint App";
        }
    }


fn main() -> Result<(), slint::PlatformError> {
  App::new()?.run()
}

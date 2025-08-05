    // fn factorial(n: u32) -> u32 {
    //     let mut result: u32 = 1;

    //     let mut i: u32 = 1;
    //     while i <= n {
    //         result *= i;
    //         i += 1;
    //     }

    //     return result;
    // }

    // fn main() {
    //     let mut numbers = vec![1, 2, 3, 4, 5]; //vec! - макрос, который создает вектор. Мутабельный массив
    //     let i_0 = numbers[0];
    //     for x in numbers.iter_mut() {
    //         *x += 1;
    //         println!("{x}")
    //     }

        

    // }

struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        return Point { x, y, color};
    }
}

fn dis_to_origin(&self) -> f32 {
    (self.x.powi(2) + self.y.powi(2).sqrt())
}

struct Color(u8, u8, u8);
fn main() {
    let p: Point = Point::new(10.0, 10.0, Color(255, 0, 0));

    println!("x = {}, y = {}", p.x, p.y);
    let red = Color(255, 0, 0);
    println!("g = {}", red.1);

    println!("{}", p.dis_to_origin());
}
// fn main() {
// let mut numbers = vec![1,2,3];
// let numbers_ref = &mut numbers;

// numbers_ref.push(5);
// numbers.push(4);

// println!("{:?}", numbers_ref);
// }
// use std::f64::consts::PI;
// #[derive(Debug)]
// struct Point {
//     x: f32,
//     y: f32
// }

// impl Point {
//     fn new(x: f32, y: f32) -> Self {
//         Point {x , y}
//     }

//     fn move_x(&mut self, dx: f32) {
//         self.x += dx;
//     }

//     fn distance_tools(&self, other: &Point) -> f32 {
//         return (self.x - other.x).powi(2) + (self.y - other.y).powi(2).sqrt();
//     }
// }

// enum Shape {
//     Circle(f64),
//     Rectangle{w: f64, h: f64},
// }

// impl Shape {
//     fn area(&self) -> f64 {
//         let area = match self {
//             Shape::Circle(r) => PI *r *r,
//             Shape::Rectangle(w,h) => w * h
//         };
//         return area;
//     }
// }

// fn safe_divide(a: f32, b: f32) -> Result<f32, &'static str> {
//     if b != 0.0 {
//         return Ok(a / b);
//     }
//     return Err("Error")
// }

// fn main() {
//     let mut point = Point::new(10.0, 20.0);
//     let point2 = Point::new(-5.0, 5.0);
    
//     let s1 = Shape::Circle(1.0);
//     println!("{}", s1.area());
//     let s2 = Shape::Rectangle(10.0, 20.0);
//     println!("{}", s2.area());

//     println!("{:?}", point.distance_tools(&point2));

// }

// fn main() {
//     let a = 5.0;
//     let b = 2.0;
//     let result = safe_divide(a, b).unwrap();
//     println("{:?}", safe_divide(a, b))
// }

// fn main() {
//     let a = 5.0;
//     let b = 2.0;
//     let result = safe_divide(a, b).unwrap_or(INFINITY);

//     if let Some(res) = result {
//         println!("{:?}", res);
//     } else {
//         println!("Cant")
//     }

//     println("{:?}", safe_divide(a, b))
// }

// fn main() {
//     let a = 5.0;
//     let b = 2.0;
//     match safe_divide(a, b) {
//         Ok(res) => println!("{res}"),
//         Err(msg) => println!("{msg}"),
//     }

//     if let Some(res) = result {
//         println!("{:?}", res);
//     } else {
//         println!("Cant")
//     }

//     println("{:?}", safe_divide(a, b))
// }
// trait Mess {
//     fn meassure(&self) -> f64;

// }

// impl Mess for Shape {
//     fn meassure(&self) ->f64 {
//         self.area()
//     }
// }
// impl Mess for Point {

//     fn meassure(&self) -> f64 {
//         self.distance_tools(&Point {y: 0.0, x:0.0}) as f64
//     }
// }
// fn main() {
//     let point = Point::new(1.0, 1.0);
//     let shapes_and_points: Vec<&dyn Mess> = vec![
//         &point,
//         &Shape::Circle(5.0),
//         &Shape::Rectangle {w: 10.0, h:100.0},
//     ];

//     for item in shapes_and_points {
//         println!("{}", item.meassure());
//     }
// }

struct Point<T> {
x: T,
y: T,
}

impl<T> Point<T> {
fn distance_tools(&self, other: &Point) -> f32 {
  return (self.x - other.x).powi(2) + (self.y - other.y).powi(2).sqrt();
   }

}
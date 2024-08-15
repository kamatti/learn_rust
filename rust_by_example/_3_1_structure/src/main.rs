#[derive(PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(PartialEq)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

// 演習1
// Rectangleの面積を計算
fn rect_area(rect: Rectangle) -> f32 {
    let Rectangle { top_left: Point { x: x1, y: y1}, bottom_right: Point { x: x2, y: y2}} = rect;

    return (x2 - x1) * (y1 - y2);
}

// 演習2
// pointを左下の座標とする幅と高さがlengthとなるRectangleを生成
fn square(point: Point, length: f32) -> Rectangle {
    let Point { x: left_x, y: bottom_y } = point;
    return Rectangle { top_left: Point { x: left_x, y: bottom_y + length}, bottom_right: Point { x: left_x + length, y: bottom_y} };
}

fn main() {
    // 2*5の長方形
    // 座標は全て自然数
    let a = Rectangle {
        top_left: Point { x: 0.0, y: 5.0},
        bottom_right: Point { x: 2.0, y: 0.0}
    };

    // 2*5の長方形
    // 座標に負の数を含む
    let b = Rectangle {
        top_left: Point { x: -1.0, y: 3.0},
        bottom_right: Point { x: 1.0, y: -2.0}
    };

    // 2.3*4.8の長方形
    let c = Rectangle {
        top_left: Point { x: 0.0, y: 4.8},
        bottom_right: Point { x: 2.3, y: 0.0}
    };

    // rect_areaの動作確認
    std::assert!(rect_area(a) == 10.0);
    std::assert!(rect_area(b) == 10.0);
    std::assert!(rect_area(c) == 11.04);

    println!("passing test of rect_area");

    // 3*3の正方形
    // 座標は全て自然数
    let d = Rectangle {
        top_left: Point { x: 0.0, y: 3.0},
        bottom_right: Point { x: 3.0, y: 0.0}
    };

    // 3*3の正方形
    // 座標に負の数を含む
    let e = Rectangle {
        top_left: Point { x: -1.0, y: 1.0},
        bottom_right: Point { x: 2.0, y: -2.0}
    };

    // 1.5*1.5の正方形
    let f = Rectangle {
        top_left: Point { x: 0.0, y: 1.5},
        bottom_right: Point { x: 1.5, y: 0.0}
    };

    // squareの動作確認
    std::assert!(square(Point{ x: 0.0, y: 0.0 }, 3.0) == d);
    std::assert!(square(Point{ x: -1.0, y: -2.0 }, 3.0) == e);
    std::assert!(square(Point{ x: 0.0, y: 0.0 }, 1.5) == f);
    println!("passing test of square");
}
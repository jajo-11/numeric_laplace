extern crate numeric_laplace;

use numeric_laplace::*;

fn main() {
    let yellow = FixedBox{
        point: Point{x: 5, y: 20 },
        width: 10,
        height: 10,
        potential: 100.0,
    };
    let blue_top = FixedBox {
        point: Point{x: 40, y: 5 },
        width: 5,
        height: 18,
        potential: 0.0,
    };
    let blue_bottom = FixedBox {
        point: Point{x: 40, y: 27 },
        width: 5,
        height: 18,
        potential: 0.0,
    };
    let mut grid = Grid::new(50, 50, vec![yellow, blue_top, blue_bottom],
    Scale {
        nodes_per_unit: 1,
        lowest: 0.0,
        highest: 100.0,
        x_offset: 10,
        y_offset: 25,
        invert_x: false,
        invert_y: true,
    });

    let (iterations, watch_data) = grid.evaluate(0.1, 1.5,Point{x: 42, y: 25});
    println!("Done in {} iterations", iterations);
    grid.to_csv("out.csv").expect("Could not write grid file!");
    watch_data_to_csv(watch_data, "watch.csv").expect("Could not write watch file!");
}

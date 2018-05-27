use super::Grid;
use std::fs::File;
use std::process::Command;
use std::io::Write;

pub fn plot_2d_color_map(grid: Grid) {
    let mut temp = File::create("temp").expect("Could not create temp fileS");
    let mut file_string = String::from(
        "# set terminal pngcairo  transparent enhanced font \"arial,10\" fontscale 1.0 size 600, 400\
            \n# set output 'out.png'
            \nset title \"Electrical Potential Approximated Based On Laplace Equation\"\
            \nset cblabel \"Potential in kV\"\
            \nset datafile separator comma
            \nsplot \"out.csv\" matrix using 1:2:3 with image");
    temp.write_all(file_string.as_bytes());
    Command::new("gnuplot")
        .args(&["-p", "temp"])
        .spawn()
        .expect("Failed to plot Data (Is gnuplot installed?)");
}
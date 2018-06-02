use std::fs::File;
use std::process::Command;
use std::io::Write;

pub fn plot_2d_color_map(file: &str) {
    let mut temp = File::create("temp").expect("Could not create temp fileS");
    let file_string = format!(
        "set terminal pngcairo  transparent enhanced font \"arial,10\" fontscale 1.0 size 800, 600
set output 'out.png'
set title \"Electrical Potential Approximation Based On Laplace Equation\"
set cblabel \"Potential in kV\"
set datafile separator comma
set autoscale xfix
set autoscale yfix
plot \"{}\" matrix nonuniform using 1:2:3 with image", file);
    temp.write_all(file_string.as_bytes()).expect("Could not write to temp file");
    Command::new("gnuplot")
        .args(&["-p", "temp"])
        .status()
        .expect("Failed to plot Data (Is gnuplot installed?)");
}

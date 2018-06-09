use std::fs::File;
use std::process::Command;
use std::io::Write;

/// Writes a temporary gnuplot script file and then calls gnuplot on that file
fn plot(script: &String) {
    let mut temp = File::create("temp").expect("Could not create temp file");
    temp.write_all(script.as_bytes()).expect("Could not write to temp file");
    Command::new("gnuplot")
        .args(&["-p", "temp"])
        .status()
        .expect("Failed to plot Data (Is gnuplot installed?)");
}

pub fn plot_2d_color_map(file: &str) {
    plot(&format!(
        "set terminal pngcairo  transparent enhanced font \"arial,10\" fontscale 1.0 size 800, 600
set output 'out.png'
set title \"Electrical Potential Approximation Based On Laplace Equation\"
set cblabel \"Potential in kV\"
set datafile separator comma
set autoscale xfix
set autoscale yfix
plot \"{}\" matrix nonuniform with image", file));
}

pub fn plot_lines_by_column(file: &str, title: &str, base: f64, step: f64) {
    plot(&format!(
        "set terminal pngcairo  transparent enhanced font \"arial,10\" fontscale 1.0 size 800, 600
set output 'watch.png'
set title \"{}\"
set cblabel \"Potential in kV\"
set datafile separator comma
set autoscale xfix
set autoscale yfix
set xlabel \"Iterations\"
set ylabel \"Potential in kV\"
set key bmargin center horizontal Right noreverse enhanced autotitle box lt black linewidth 1.000\
 dashtype solid
plot for [col=0:*] \"{}\" matrix using 1:0 every :::col::col with lines title sprintf(\"ω=%1.2f\",\
 col*{}+{})", title, file, step, base));
}
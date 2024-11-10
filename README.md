# hopalong

![Example Attractor Image](./Examples/num=200000000_a=-2_b=-0.33_c=0.01.png)

The "Hopalong" attractor<top>*<top>, invented by Barry Martin of Aston University in Birmingham, England, was popularized by A.K. Dewdney in the September 1986 issue of Scientific American. In Germany, it gained further recognition through a translation titled "Hüpfer" in Spektrum der Wissenschaft.  
<sub>*Nicknamed by A.K. Dewdney.</sub>

This Rust program computes and visualizes the “hopalong” attractor by iterating the following recursive functions:

$$
\begin{cases}
x_{n+1} = y_n - \text{sgn}(x_n) \times \sqrt{|b \times x_n - c|} \\
y_{n+1} = a - x_n
\end{cases}
$$

Where:

- The sequence starts from the initial point (x<sub>0</sub> , y<sub>0</sub>) = (0 , 0)
- x<sub>n</sub> and y<sub>n</sub> represent the coordinates at the n-th iteration
- a, b and c are parameters influencing the attracto's dynamics
- sgn is the sign (signum) function

The color scheme is based on the pixel density, i.e. how often a pixel of the image is hit during the iteration.

- Check out my Python version for more detailed information:  
<https://github.com/ratwolfzero/hopalong_python>

- For information on implementing the Signum function in Rust, see:  
- <https://docs.rs/num-traits/latest/num_traits/sign/fn.signum.html>

You can run the program from the command line in a terminal for example ./hopalong a b c num. if you are using MacOs
The number of iterations (num) can be entered as integer or in exponential form such as 1e6  

Example ./hopalomg 2 1 0 1e6

If you are using a mac with apple silicon you should be able to use the executable in the `Binary` folder.  
The binary was Compiled on a Mac Mini with M2 processor.  
The calculated image should be displayed but there will be an error regarding saving the image.

    // Save the image with the generated name
    let save_path = format!("/Users/ralf//Projects/hopalong_pictures/{}", image_name); // Specify your desired save path
    if let Err(e) = image_buffer.save_with_format(&save_path, ImageFormat::Png) {
        eprintln!("Error saving image: {}", e);
    } else {
        println!("Image saved to: {}", save_path);
    }

 

----------------------------------------------------------------------------------------------------------------------------------------------------

## References

Computers in Art, Design and Animation (J. Landsdown and R. A. Earnshaw, eds.), New York: Springer–Verlag, 1989.

Barry Martin, "Graphic Potential of Recursive Functions," in Computers in Art, Design and Animation pp. 109–129.

ISBN-13: 978-1-4612-8868-8,  e-ISBN-13: 978-1-4612-4538-4

----------------------------------------------------------------------------------------------------------------------------------------------------

A.K. Dewdney in Spektrum der Wissenschaft "Computer Kurzweil" 1988, (German version of Scientific American)

ISBN-10: 3922508502, ISBN-13: 978-3922508502

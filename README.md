# hopalong

The "Hopalong" attractor<top>*<top>, invented by Barry Martin of Aston University in Birmingham, England, was popularized by A.K. Dewdney in the September 1986 issue of Scientific American. In Germany, it gained further recognition through a translation titled "Hüpfer" in Spektrum der Wissenschaft.  
<sub>*Nicknamed by A.K. Dewdney.</sub>

This Rust program computes and visualizes the “hopalong” attractor by iterating the following system of recursive functions (1) and (2):

$$
\begin{cases}
x_n+1\space=&y_n-sgn(x_n)\times\sqrt{∣b\times x_n−c∣}&(1) \\
y_n+1\space=&a-x_n&(2)
\end{cases}
$$

Where:

- x<sub>n</sub> and y<sub>n</sub> represent the coordinates at the nth iteration.
- a, b, c are user defined parameters that shape the attractor
- The sequence starts from an initial point (x<sub>0</sub> , y<sub>0</sub>) = (0 , 0)

The color scheme is based on the pixel density, i.e. how often a pixel of the image is hit during the iteration
You can run the program from the command line in a terminal for example ./hopalong a b c num,  if you are using MacOs
The number of iterations (num) can be entered as integer or in exponential form such as 1e6

Example ./hopalomg 2 1 0 1e6  

Also have a look to my Python version where you can also find further explanations about pixel density:  
<https://github.com/ratwolfzero/hopalong_python>

----------------------------------------------------------------------------------------------------------------------------------------------------

## References

Computers in Art, Design and Animation (J. Landsdown and R. A. Earnshaw, eds.), New York: Springer–Verlag, 1989.

Barry Martin, "Graphic Potential of Recursive Functions," in Computers in Art, Design and Animation pp. 109–129.

ISBN-13: 978-1-4612-8868-8,  e-ISBN-13: 978-1-4612-4538-4

----------------------------------------------------------------------------------------------------------------------------------------------------

A.K. Dewdney in Spektrum der Wissenschaft "Computer Kurzweil" 1988, (German version of Scientific American)

ISBN-10: 3922508502, ISBN-13: 978-3922508502

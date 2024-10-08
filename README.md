# hopalong

The "Hopalong *" attractor, invented by Barry Martin at Aston University, was popularized by A.K. Dewdney in the September 1986 issue of Scientific American. In Germany, it gained further recognition through a translation titled "Hüpfer" in Spektrum der Wissenschaft.*Nicknamed by A.K. Dewdney.  

This Rust program calculates and plots the “hopalong” attractor by iterating the following system of recursive equations (1) and (2): 

$$
\begin{align}
x_n+1 & = y_n-sgn(x_n)\times\sqrt{∣b\times x_n−c∣} &(1) \\
y_n+1 & = a-x_n &(2)
\end{align}
$$

The sequence of (x1, y1), (x2, y2), ... (xn, yn) coordinates is specified by an initial point (x0, y0) and three constants a, b, and c.

The color scheme is based on the pixel density, i.e. how often a pixel of the image is hit during the iteration
You can run the program from the command line in a terminal for example ./hopalong a b c num,  if you are using MacOs
The number of iterations (num) can be entered as integer or in exponential form such as 1e6

Example ./hopalomg 2 1 0 1e6  

Also have a look to my Python version:  
<https://github.com/ratwolfzero/hopalong_python>

----------------------------------------------------------------------------------------------------------------------------------------------------

## References

Barry Martin, "Graphic Potential of Recursive Functions," in Computers in Art, Design and Animation (J. Landsdown and R. A. Earnshaw, eds.), New York: Springer–Verlag, 1989 pp. 109–129.

ISBN-13: 978-1-4612-8868-8,  e-ISBN-13: 978-1-4612-4538-4

----------------------------------------------------------------------------------------------------------------------------------------------------

A.K. Dewdney in Spektrum der Wissenschaft "Computer Kurzweil" 1988, (German version of Scientific American)

ISBN-10: 3922508502, ISBN-13: 978-3922508502

----------------------------------------------------------------------------------------------------------------------------------------------------

Maple help:

<https://de.maplesoft.com/support/help/maple/view.aspx?path=MathApps%2FHopalongAttractor>
